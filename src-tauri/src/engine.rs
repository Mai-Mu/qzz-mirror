//! The control panel only launches the bundled executables. It never shells out,
//! kills a shared ADB server, or takes ownership of another app's scrcpy process.
use crate::models::*;
use crate::scrcpy_input::{NativeScrcpyDispatcher, NavigationDispatcher};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    process::Stdio,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader},
    process::{Child, Command},
    sync::{Mutex, OwnedMutexGuard, RwLock},
    time::{sleep, timeout},
};

const ADB_TIMEOUT: Duration = Duration::from_secs(8);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(12);
const MAX_LOGS: usize = 100;

#[derive(Clone)]
pub struct Engine(Arc<Inner>);

/// Native toolbar tracking data for a child owned by this Engine only.
/// A token is renewed on every new scrcpy process, including wireless recovery.
#[derive(Debug, Clone, Copy)]
pub struct ToolbarSession {
    pub process_id: u32,
    pub token: u64,
    pub running: bool,
}

struct Inner {
    runtime_dir: PathBuf,
    data_dir: PathBuf,
    state: RwLock<State>,
    operation: Arc<Mutex<()>>,
    refresh_gate: Mutex<()>,
    busy: Arc<AtomicBool>,
    active: Mutex<Option<ActiveSession>>,
    navigation: Mutex<()>,
    navigation_dispatcher: Arc<dyn NavigationDispatcher>,
    cache: Mutex<HashMap<String, CachedMetadata>>,
    next_token: AtomicU64,
    stop_epoch: AtomicU64,
    background_started: AtomicBool,
    shutting_down: AtomicBool,
}

struct State {
    devices: Vec<Device>,
    saved_devices: Vec<SavedDevice>,
    settings: Settings,
    session: Option<Session>,
    logs: Vec<LogEntry>,
    engine_version: String,
    last_error: Option<String>,
}

struct ActiveSession {
    token: u64,
    child: Arc<Mutex<Child>>,
    session: Session,
    user_stop: Arc<AtomicBool>,
    // Only an actual ADB transport-loss report permits automatic recovery.
    // A normal window close never becomes a reconnection request.
    connection_lost: Arc<AtomicBool>,
    recovery_attempted: bool,
    stop_epoch: u64,
    recovery_serial: Option<String>,
}

struct OperationGuard {
    _guard: OwnedMutexGuard<()>,
    busy: Arc<AtomicBool>,
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        self.busy.store(false, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Persisted {
    settings: Settings,
    saved_devices: Vec<SavedDevice>,
}

#[derive(Debug, Clone)]
struct AdbDevice {
    serial: String,
    state: String,
    model: String,
    kind: TransportKind,
}

#[derive(Debug, Clone)]
struct Metadata {
    id: String,
    hardware_identity_verified: bool,
    model: String,
    android_version: Option<String>,
    api_level: Option<u32>,
}

struct CachedMetadata {
    metadata: Metadata,
    fetched: Instant,
}

impl Engine {
    pub fn new(runtime_dir: PathBuf, data_dir: PathBuf) -> Result<Self, String> {
        Self::new_with_navigation_dispatcher(runtime_dir, data_dir, Arc::new(NativeScrcpyDispatcher))
    }

    /// The isolated engine harness supplies a recording dispatcher here; the
    /// application always uses `new` and the native owned-window dispatcher.
    pub fn new_with_navigation_dispatcher(runtime_dir: PathBuf, data_dir: PathBuf, navigation_dispatcher: Arc<dyn NavigationDispatcher>) -> Result<Self, String> {
        let runtime_dir = runtime_dir.canonicalize().map_err(|error| {
            format!("找不到投屏运行组件目录：{}（{error}）", runtime_dir.display())
        })?;
        for executable in ["adb.exe", "scrcpy.exe", "scrcpy-server"] {
            if !runtime_dir.join(executable).is_file() {
                return Err(format!("投屏组件缺失：{executable}。请重新解压完整便携包。"));
            }
        }
        std::fs::create_dir_all(&data_dir)
            .map_err(|error| format!("无法创建配置目录：{error}"))?;
        let config_path = data_dir.join("settings.json");
        let (persisted, config_warning) = match std::fs::read(&config_path) {
            Ok(bytes) => match parse_persisted(&bytes) {
                Ok(value) => (value, None),
                Err(error) => {
                    let backup = data_dir.join(format!("settings.invalid-{}.json", now()));
                    let _ = std::fs::copy(&config_path, backup);
                    (Persisted::default(), Some(format!("配置文件无法读取，已使用默认设置：{error}")))
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (Persisted::default(), None),
            Err(error) => return Err(format!("无法读取本地配置：{error}")),
        };
        let mut logs = vec![LogEntry {
            time: now(),
            level: LogLevel::Info,
            message: "准备就绪。选择设备后，点击「开始投屏」。".into(),
        }];
        if let Some(message) = config_warning {
            logs.push(LogEntry { time: now(), level: LogLevel::Warning, message });
        }
        Ok(Self(Arc::new(Inner {
            runtime_dir,
            data_dir,
            state: RwLock::new(State {
                devices: vec![],
                saved_devices: persisted.saved_devices,
                settings: persisted.settings,
                session: None,
                logs,
                engine_version: "scrcpy".into(),
                last_error: None,
            }),
            operation: Arc::new(Mutex::new(())),
            refresh_gate: Mutex::new(()),
            busy: Arc::new(AtomicBool::new(false)),
            active: Mutex::new(None),
            navigation: Mutex::new(()),
            navigation_dispatcher,
            cache: Mutex::new(HashMap::new()),
            next_token: AtomicU64::new(1),
            stop_epoch: AtomicU64::new(0),
            background_started: AtomicBool::new(false),
            shutting_down: AtomicBool::new(false),
        })))
    }

    pub fn spawn_background(&self) {
        if self.0.background_started.swap(true, Ordering::SeqCst) { return; }
        let engine = self.clone();
        tokio::spawn(async move {
            if let Ok(version) = engine.scrcpy_version().await {
                engine.0.state.write().await.engine_version = version;
            }
            let mut next_refresh = Instant::now();
            while !engine.0.shutting_down.load(Ordering::SeqCst) {
                engine.poll_session().await;
                if Instant::now() >= next_refresh {
                    next_refresh = Instant::now() + Duration::from_secs(3);
                    // Refresh has its own task so a slow/offline ADB client cannot
                    // delay process supervision or a user's stop action.
                    let refresh_engine = engine.clone();
                    tokio::spawn(async move {
                        if refresh_engine.0.busy.load(Ordering::SeqCst) { return; }
                        let Ok(_guard) = refresh_engine.0.operation.clone().try_lock_owned() else { return; };
                        if refresh_engine.0.shutting_down.load(Ordering::SeqCst) { return; }
                        if let Err(error) = refresh_engine.refresh_inner().await {
                            let mut state = refresh_engine.0.state.write().await;
                            if state.last_error.as_ref() != Some(&error) {
                                state.last_error = Some(error.clone());
                                push_log(&mut state, LogLevel::Warning, error);
                            }
                        }
                    });
                }
                sleep(Duration::from_millis(200)).await;
            }
        });
    }

    pub async fn snapshot(&self) -> Snapshot {
        let state = self.0.state.read().await;
        Snapshot {
            devices: state.devices.clone(),
            saved_devices: state.saved_devices.clone(),
            settings: state.settings.clone(),
            session: state.session.clone(),
            logs: state.logs.clone(),
            engine_version: state.engine_version.clone(),
            busy: self.0.busy.load(Ordering::SeqCst),
            last_error: state.last_error.clone(),
        }
    }

    /// No ADB calls or metadata refreshes: suitable for a native window tracker.
    pub async fn toolbar_session(&self) -> Option<ToolbarSession> {
        if self.0.shutting_down.load(Ordering::SeqCst) { return None; }
        let active = self.0.active.lock().await;
        let current = active.as_ref()?;
        if current.user_stop.load(Ordering::SeqCst)
            || current.stop_epoch != self.0.stop_epoch.load(Ordering::SeqCst) { return None; }
        let child = current.child.lock().await;
        Some(ToolbarSession {
            process_id: child.id()?,
            token: current.token,
            running: current.session.phase == SessionPhase::Running,
        })
    }

    pub async fn navigate(&self, token: u64, action: NavigationAction) -> Result<(), String> {
        // Queue only navigation actions, without blocking device refresh/start/stop.
        // Each queued click is validated again immediately before its own dispatch.
        let _navigation = self.0.navigation.lock().await;
        let result = self.navigate_inner(token, action).await;
        if let Err(error) = &result {
            self.log(LogLevel::Warning, format!("导航操作未完成：{error}")).await;
        }
        result
    }

    async fn navigate_inner(&self, token: u64, action: NavigationAction) -> Result<(), String> {
        // Keep the session gate until all four native messages are submitted.
        // The dispatcher is synchronous/non-waiting and has no process/ADB path.
        let active = self.0.active.lock().await;
        let current = active.as_ref().ok_or_else(|| "当前没有正在投屏的设备。".to_string())?;
        self.validate_navigation(current, token)?;
        let mut child = current.child.lock().await;
        if child.try_wait().map_err(|error| format!("无法确认投屏进程状态：{error}"))?.is_some() {
            return Err("投屏窗口已结束，请重新开始投屏后再操作。".into());
        }
        let process_id = child.id().ok_or_else(|| "投屏进程已结束。".to_string())?;
        self.validate_navigation(current, token)?;
        self.0.navigation_dispatcher.dispatch(process_id, action)
    }

    fn validate_navigation(&self, current: &ActiveSession, token: u64) -> Result<(), String> {
        validate_navigation_request(token, current.token, current.session.phase,
            current.user_stop.load(Ordering::SeqCst)
                || current.stop_epoch != self.0.stop_epoch.load(Ordering::SeqCst)
                || self.0.shutting_down.load(Ordering::SeqCst))
    }

    async fn begin_operation(&self) -> Result<OperationGuard, String> {
        if self.0.shutting_down.load(Ordering::SeqCst) {
            return Err("程序正在退出，请稍候。".into());
        }
        self.0.busy.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| "正在处理上一项操作，请稍候再试。".to_string())?;
        // A background refresh does not make a user action fail. Signal busy at
        // once, then take priority over the next background refresh.
        let guard = self.0.operation.clone().lock_owned().await;
        if self.0.shutting_down.load(Ordering::SeqCst) {
            self.0.busy.store(false, Ordering::SeqCst);
            return Err("程序正在退出，请稍候。".into());
        }
        // A stop/recovery guard may have cleared its busy flag while this user
        // action was queued. Acquiring the lock starts our own busy period.
        self.0.busy.store(true, Ordering::SeqCst);
        Ok(OperationGuard { _guard: guard, busy: self.0.busy.clone() })
    }

    async fn finish(&self, result: Result<(), String>, guard: OperationGuard) -> Result<Snapshot, String> {
        {
            let mut state = self.0.state.write().await;
            state.last_error = result.as_ref().err().cloned();
            if let Err(error) = &result { push_log(&mut state, LogLevel::Error, error.clone()); }
        }
        drop(guard);
        result?;
        Ok(self.snapshot().await)
    }

    pub async fn refresh(&self) -> Result<Snapshot, String> {
        let guard = self.begin_operation().await?;
        let result = self.refresh_inner().await;
        self.finish(result, guard).await
    }

    pub async fn select_device(&self, device_id: Option<String>) -> Result<Snapshot, String> {
        let guard = self.begin_operation().await?;
        let result = async {
            let mut state = self.0.state.write().await;
            if let Some(id) = &device_id {
                if !state.devices.iter().any(|device| &device.id == id)
                    && !state.saved_devices.iter().any(|device| &device.id == id) {
                    return Err("设备列表已变化，请刷新后重新选择。".into());
                }
            }
            state.settings.last_device_id = device_id;
            drop(state);
            self.persist().await
        }.await;
        self.finish(result, guard).await
    }

    pub async fn save_settings(&self, settings: Settings) -> Result<Snapshot, String> {
        let guard = self.begin_operation().await?;
        let result = async {
            settings.validate()?;
            self.0.state.write().await.settings = settings;
            self.persist().await
        }.await;
        self.finish(result, guard).await
    }

    pub async fn start_mirror(&self, device_id: String, transport: TransportPreference) -> Result<Snapshot, String> {
        let guard = self.begin_operation().await?;
        let request_epoch = self.0.stop_epoch.load(Ordering::SeqCst);
        let result = async {
            if self.0.active.lock().await.is_some() {
                return Err("已有设备正在投屏，请先停止当前投屏。".into());
            }
            self.refresh_inner().await?;
            let device = self.find_device(&device_id).await?;
            let selected = choose_transport(&device, transport)?.clone();
            {
                let mut state = self.0.state.write().await;
                state.settings.last_device_id = Some(device_id);
                state.settings.preferred_transport = transport;
            }
            self.persist().await?;
            if self.0.stop_epoch.load(Ordering::SeqCst) != request_epoch
                || self.0.shutting_down.load(Ordering::SeqCst) {
                return Err("本次启动已取消。".into());
            }
            self.launch_session(&device, &selected, false, request_epoch).await?;
            self.wait_for_startup().await
        }.await;
        self.finish(result, guard).await
    }

    pub async fn stop_mirror(&self) -> Result<Snapshot, String> {
        // Stop must remain available while startup waits for its first frame.
        // Mark intent before acquiring the operation lock so recovery cannot race it.
        self.0.stop_epoch.fetch_add(1, Ordering::SeqCst);
        if let Some(active) = self.0.active.lock().await.as_ref() {
            active.user_stop.store(true, Ordering::SeqCst);
        }
        // End an already-running process immediately, even while a background
        // device refresh owns the operation lock. Recheck once after the lock.
        let immediate_result = self.stop_inner().await;
        let lock = self.0.operation.clone().lock_owned().await;
        self.0.busy.store(true, Ordering::SeqCst);
        let guard = OperationGuard { _guard: lock, busy: self.0.busy.clone() };
        let result = self.stop_inner().await.and(immediate_result);
        self.finish(result, guard).await
    }

    pub async fn configure_wireless(&self, device_id: String) -> Result<Snapshot, String> {
        let guard = self.begin_operation().await?;
        let result = async {
            if self.0.active.lock().await.is_some() {
                return Err("配置无线会重启设备的调试连接。请先停止投屏，再配置无线。".into());
            }
            self.refresh_inner().await?;
            let device = self.find_device(&device_id).await?;
            let usb = choose_transport(&device, TransportPreference::Usb)?.clone();
            let wifi_ip = self.wifi_ip(&usb.serial).await?
                .ok_or_else(|| "设备尚未连接 Wi-Fi。请让手机与电脑接入同一局域网后重试。".to_string())?;
            let address = format!("{wifi_ip}:5555");
            self.log(LogLevel::Info, format!("正在为 {} 配置无线连接…", device.name)).await;
            self.adb(&["-s", &usb.serial, "tcpip", "5555"], ADB_TIMEOUT).await?;
            sleep(Duration::from_millis(1100)).await;
            self.connect_verified(&address).await?;
            // Verify physical identity before saving or claiming it is safe to unplug.
            let metadata = self.metadata(&address, "", TransportKind::Wifi, true).await?;
            verify_device_identity(&device.id, &metadata)?;
            self.save_address(&metadata.id, &device.name, &address).await?;
            self.refresh_inner().await?;
            self.log(LogLevel::Success, format!("无线连接已验证（{address}），现在可以拔掉数据线。请点击「开始投屏」。")).await;
            Ok(())
        }.await;
        self.finish(result, guard).await
    }

    pub async fn connect_wireless(&self, address: String) -> Result<Snapshot, String> {
        let guard = self.begin_operation().await?;
        let result = async {
            let address = validate_endpoint(&address)?;
            self.connect_and_save(&address).await?;
            self.refresh_inner().await?;
            self.log(LogLevel::Success, format!("已连接无线设备：{address}")).await;
            Ok(())
        }.await;
        self.finish(result, guard).await
    }

    pub async fn pair_wireless(&self, pairing_address: String, code: String, connect_address: Option<String>) -> Result<Snapshot, String> {
        let guard = self.begin_operation().await?;
        let result = async {
            let pairing_address = validate_endpoint(&pairing_address)?;
            let code = code.trim();
            if code.len() != 6 || !code.bytes().all(|value| value.is_ascii_digit()) {
                return Err("请输入手机显示的 6 位配对码。".into());
            }
            // Validate all input before changing pairing state on the device.
            let connect_address = connect_address.filter(|value| !value.trim().is_empty())
                .map(|value| validate_endpoint(&value)).transpose()?;
            let mut command = self.command("adb.exe");
            command.args(["pair", &pairing_address]).stdin(Stdio::piped())
                .stdout(Stdio::piped()).stderr(Stdio::piped());
            let mut child = command.spawn().map_err(|error| format!("无法启动配对：{error}"))?;
            let mut input = child.stdin.take().ok_or_else(|| "无法输入配对码。".to_string())?;
            input.write_all(format!("{code}\n").as_bytes()).await
                .map_err(|error| format!("发送配对码失败：{error}"))?;
            drop(input);
            let output = timeout(Duration::from_secs(30), child.wait_with_output()).await
                .map_err(|_| "配对超时，请重新打开手机的配对码页面。".to_string())?
                .map_err(|error| format!("配对失败：{error}"))?;
            let text = output_text(&output);
            if !output.status.success() || !text.to_ascii_lowercase().contains("successfully paired") {
                return Err(format!("配对失败，请检查地址、端口和配对码。{}", raw_suffix(&text)));
            }
            self.log(LogLevel::Success, "无线调试配对成功。".into()).await;
            let address = match connect_address {
                Some(address) => address,
                None => self.discover_connect_address(&pairing_address).await?.ok_or_else(|| {
                    "配对成功。未发现投屏连接端口，请填写手机「无线调试」主页的 IP 地址和端口，再点击连接；它通常不同于配对码页面的端口。".to_string()
                })?,
            };
            self.connect_and_save(&address).await?;
            self.refresh_inner().await?;
            self.log(LogLevel::Success, format!("已完成配对并连接：{address}")).await;
            Ok(())
        }.await;
        self.finish(result, guard).await
    }

    pub async fn shutdown(&self) {
        self.0.shutting_down.store(true, Ordering::SeqCst);
        self.0.stop_epoch.fetch_add(1, Ordering::SeqCst);
        if let Some(active) = self.0.active.lock().await.as_ref() {
            active.user_stop.store(true, Ordering::SeqCst);
        }
        let _guard = self.0.operation.lock().await;
        let _ = self.stop_inner().await;
    }

    fn command(&self, executable: &str) -> Command {
        let mut command = Command::new(self.0.runtime_dir.join(executable));
        command.current_dir(&self.0.runtime_dir)
            .env("ADB", self.0.runtime_dir.join("adb.exe"))
            .env("SCRCPY_SERVER_PATH", self.0.runtime_dir.join("scrcpy-server"))
            .kill_on_drop(true);
        #[cfg(windows)]
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW, never a flashing console.
        command
    }

    async fn adb(&self, arguments: &[&str], duration: Duration) -> Result<String, String> {
        let mut command = self.command("adb.exe");
        command.args(arguments).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let output = timeout(duration, command.output()).await
            .map_err(|_| "设备响应超时。请检查 USB 授权或局域网连接后重试。".to_string())?
            .map_err(|error| format!("无法运行内置 adb：{error}"))?;
        let text = output_text(&output);
        if !output.status.success() { return Err(translate_error(&text)); }
        Ok(text)
    }

    async fn scrcpy_version(&self) -> Result<String, String> {
        let output = timeout(ADB_TIMEOUT, self.command("scrcpy.exe").arg("--version").output()).await
            .map_err(|_| "读取投屏组件版本超时。".to_string())?
            .map_err(|error| error.to_string())?;
        Ok(String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("scrcpy").split(" <").next().unwrap_or("scrcpy").trim().to_string())
    }

    async fn refresh_inner(&self) -> Result<(), String> {
        let _gate = self.0.refresh_gate.lock().await;
        let text = self.adb(&["devices", "-l"], ADB_TIMEOUT).await?;
        let entries = parse_adb_devices(&text);
        let saved = self.0.state.read().await.saved_devices.clone();
        let mut collected = Vec::new();
        for entry in &entries {
            let metadata = if entry.state == "device" {
                match self.metadata(&entry.serial, &entry.model, entry.kind, false).await {
                    Ok(value) => Some(value),
                    Err(_) => self.0.cache.lock().await.get(&entry.serial).map(|cache| cache.metadata.clone()),
                }
            } else {
                // Never issue a shell command against unauthorized/offline entries.
                self.0.cache.lock().await.get(&entry.serial).map(|cache| cache.metadata.clone())
            };
            collected.push((entry.clone(), metadata));
        }
        let devices = group_devices(collected, &saved);
        let mut state = self.0.state.write().await;
        state.devices = devices;
        if state.settings.last_device_id.is_none() && state.devices.len() == 1 {
            state.settings.last_device_id = state.devices.first().map(|device| device.id.clone());
        }
        Ok(())
    }

    async fn metadata(&self, serial: &str, fallback_model: &str, kind: TransportKind, force: bool) -> Result<Metadata, String> {
        if !force {
            if let Some(cached) = self.0.cache.lock().await.get(serial) {
                if cached.fetched.elapsed() < Duration::from_secs(30) { return Ok(cached.metadata.clone()); }
            }
        }
        let properties = self.adb(&["-s", serial, "shell", "getprop"], ADB_TIMEOUT).await?;
        let property = |name: &str| -> Option<String> {
            let prefix = format!("[{name}]: [");
            properties.lines().find_map(|line| line.trim().strip_prefix(&prefix)
                .and_then(|value| value.strip_suffix(']')).filter(|value| !value.is_empty())
                .map(str::to_string))
        };
        let hardware_serial = property("ro.serialno").or_else(|| property("ro.boot.serialno"))
            .filter(|value| value != "unknown" && value != "null");
        // An endpoint is not physical identity: DHCP may assign it to another
        // device. Never substitute a saved identity when the device cannot prove it.
        let hardware_identity_verified = hardware_serial.is_some();
        let id = hardware_serial.map(|value| format!("android:{value}"))
            .unwrap_or_else(|| fallback_device_id(serial, kind));
        let metadata = Metadata {
            id,
            hardware_identity_verified,
            model: property("ro.product.model").unwrap_or_else(|| fallback_model.replace('_', " ")),
            android_version: property("ro.build.version.release"),
            api_level: property("ro.build.version.sdk").and_then(|value| value.parse().ok()),
        };
        self.0.cache.lock().await.insert(serial.to_string(), CachedMetadata { metadata: metadata.clone(), fetched: Instant::now() });
        Ok(metadata)
    }

    async fn wifi_ip(&self, serial: &str) -> Result<Option<String>, String> {
        // Android's route table excludes mobile data when a Wi-Fi route exists.
        // A Wi-Fi interface can be wlan0, wlan1, or a vendor-specific wifi name.
        let routes = self.adb(&["-s", serial, "shell", "ip", "-4", "route"], ADB_TIMEOUT).await?;
        Ok(parse_wifi_ip(&routes))
    }

    async fn find_device(&self, id: &str) -> Result<Device, String> {
        self.0.state.read().await.devices.iter().find(|device| device.id == id).cloned()
            .ok_or_else(|| "所选设备不在线。请检查连接并刷新设备列表。".into())
    }

    async fn connect_verified(&self, address: &str) -> Result<(), String> {
        let address = validate_endpoint(address)?;
        let response = self.adb(&["connect", &address], CONNECT_TIMEOUT).await?;
        let lower = response.to_ascii_lowercase();
        if lower.contains("failed") || lower.contains("cannot") || lower.contains("unable") {
            return Err(translate_error(&response));
        }
        let model = self.adb(&["-s", &address, "shell", "getprop", "ro.product.model"], ADB_TIMEOUT).await?;
        if model.trim().is_empty() { return Err("无线连接未通过设备响应验证，请确认地址与授权状态。".into()); }
        Ok(())
    }

    async fn connect_and_save(&self, address: &str) -> Result<(), String> {
        self.connect_verified(address).await?;
        let metadata = self.metadata(address, "Android 设备", TransportKind::Wifi, true).await?;
        self.save_address(&metadata.id, &metadata.model, address).await
    }

    async fn save_address(&self, id: &str, name: &str, address: &str) -> Result<(), String> {
        {
            let mut state = self.0.state.write().await;
            // An address can be reassigned by DHCP; its current verified identity wins.
            state.saved_devices.retain(|device| device.id == id || device.address != address);
            if let Some(saved) = state.saved_devices.iter_mut().find(|device| device.id == id) {
                saved.address = address.to_string();
                saved.name = name.to_string();
            } else {
                state.saved_devices.push(SavedDevice { id: id.to_string(), name: name.to_string(), address: address.to_string() });
            }
            state.settings.last_device_id = Some(id.to_string());
        }
        self.persist().await
    }

    async fn discover_connect_address(&self, pairing_address: &str) -> Result<Option<String>, String> {
        let output = match self.adb(&["mdns", "services"], ADB_TIMEOUT).await {
            Ok(output) => output,
            Err(_) => return Ok(None),
        };
        let pair_host = endpoint_host(pairing_address);
        let mut candidates = output.lines().filter(|line| line.contains("_adb-tls-connect._tcp"))
            .filter_map(|line| line.split_whitespace().last())
            .filter_map(|address| validate_endpoint(address).ok())
            .filter(|address| endpoint_host(address) == pair_host)
            .collect::<Vec<_>>();
        candidates.sort();
        candidates.dedup();
        Ok(if candidates.len() == 1 { candidates.pop() } else { None })
    }

    async fn persist(&self) -> Result<(), String> {
        let state = self.0.state.read().await;
        let data = serde_json::to_vec_pretty(&Persisted { settings: state.settings.clone(), saved_devices: state.saved_devices.clone() })
            .map_err(|error| format!("保存设置失败：{error}"))?;
        drop(state);
        let temporary = self.0.data_dir.join("settings.json.tmp");
        let target = self.0.data_dir.join("settings.json");
        std::fs::write(&temporary, data).map_err(|error| format!("保存设置失败：{error}"))?;
        // Keep a recoverable copy and replace the target by rename; never leave a
        // deliberate gap where the original has been deleted but the new file isn't saved.
        if target.is_file() {
            std::fs::copy(&target, self.0.data_dir.join("settings.backup.json"))
                .map_err(|error| format!("备份设置失败：{error}"))?;
        }
        std::fs::rename(temporary, target).map_err(|error| format!("更新设置失败：{error}"))
    }

    async fn log(&self, level: LogLevel, message: String) {
        push_log(&mut *self.0.state.write().await, level, message);
    }

    async fn launch_session(&self, device: &Device, transport: &Transport, recovered: bool, expected_epoch: u64) -> Result<(), String> {
        // A cached IP is only a list/display hint. DHCP may have assigned it to a
        // different authorized phone since the last refresh, so every wireless
        // launch must prove physical identity again without a cache fallback.
        let metadata = self.metadata(&transport.serial, &device.model, transport.kind, true).await
            .map_err(|error| format!("无法确认所选设备的当前状态，已取消启动。请重新连接并选择当前设备。{error}"))?;
        let identity_check = if transport.kind == TransportKind::Wifi {
            verify_device_identity(&device.id, &metadata)
        } else if metadata.id != device.id {
            Err("USB 连接对应的设备已发生变化，已取消启动。请重新选择当前设备后重试。".into())
        } else { Ok(()) };
        if let Err(error) = identity_check {
            let _ = self.refresh_inner().await;
            return Err(error);
        }
        // A Wi-Fi transport may already have been connected outside this app.
        // Remember a freshly verified same-device transport for this USB session
        // without requiring the user to configure or save it a second time.
        let mut recovery_serial = None;
        if transport.kind == TransportKind::Usb {
            for wireless in device.transports.iter().filter(|candidate| candidate.kind == TransportKind::Wifi && candidate.state == "device") {
                if let Ok(identity) = self.metadata(&wireless.serial, &device.model, TransportKind::Wifi, true).await {
                    if verify_device_identity(&device.id, &identity).is_ok() {
                        recovery_serial = Some(wireless.serial.clone());
                        break;
                    }
                }
            }
        }
        let settings = self.0.state.read().await.settings.clone();
        let audio_supported = device.api_level.map(|api| api >= 30).unwrap_or(false);
        let mut command = self.command("scrcpy.exe");
        command.args(build_scrcpy_args(device, transport, &settings)?);
        if settings.audio && !audio_supported {
            self.log(LogLevel::Warning, "该设备不支持声音转发（需要 Android 11 或以上），本次已自动关闭声音。".into()).await;
        }
        // Stop may have arrived while metadata/identity checks were awaiting ADB.
        // Hold the session gate from this final check through registration, so a
        // simultaneous stop either cancels startup or immediately sees its child.
        let mut active = self.0.active.lock().await;
        if self.0.stop_epoch.load(Ordering::SeqCst) != expected_epoch
            || self.0.shutting_down.load(Ordering::SeqCst) {
            return Err("本次启动已取消。".into());
        }
        if active.is_some() { return Err("已有设备正在投屏，请先停止当前投屏。".into()); }
        let mut child = command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().map_err(|error| format!("无法启动投屏组件：{error}"))?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let token = self.0.next_token.fetch_add(1, Ordering::SeqCst);
        let connection_lost = Arc::new(AtomicBool::new(false));
        let session = Session {
            device_id: device.id.clone(), device_name: device.name.clone(),
            serial: transport.serial.clone(), transport: transport.kind,
            phase: if recovered { SessionPhase::Reconnecting } else { SessionPhase::Starting },
            started_at: now(),
        };
        *active = Some(ActiveSession {
            token, child: Arc::new(Mutex::new(child)), session: session.clone(),
            user_stop: Arc::new(AtomicBool::new(false)), connection_lost: connection_lost.clone(),
            recovery_attempted: recovered,
            stop_epoch: expected_epoch,
            recovery_serial,
        });
        {
            let mut state = self.0.state.write().await;
            state.session = Some(session);
            state.last_error = None;
            push_log(&mut state, LogLevel::Info, format!("正在通过 {} 启动 {}…", if transport.kind == TransportKind::Usb { "USB" } else { "Wi-Fi" }, device.name));
        }
        drop(active);
        if let Some(stdout) = stdout { self.read_scrcpy_output(stdout, token, connection_lost.clone()); }
        if let Some(stderr) = stderr { self.read_scrcpy_output(stderr, token, connection_lost); }
        Ok(())
    }

    fn read_scrcpy_output<R: AsyncRead + Unpin + Send + 'static>(&self, reader: R, token: u64, lost: Arc<AtomicBool>) {
        let engine = self.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let line = line.trim();
                if line.is_empty() { continue; }
                let lower = line.to_ascii_lowercase();
                if is_connection_loss(&lower) { lost.store(true, Ordering::SeqCst); }
                let mut active = engine.0.active.lock().await;
                let Some(current) = active.as_mut().filter(|current| current.token == token) else { continue; };
                let is_ready = is_video_ready(line);
                let became_ready = is_ready && current.session.phase != SessionPhase::Running;
                if became_ready { current.session.phase = SessionPhase::Running; }
                drop(active);
                let mut state = engine.0.state.write().await;
                if became_ready {
                    if let Some(session) = state.session.as_mut() { session.phase = SessionPhase::Running; }
                    push_log(&mut state, LogLevel::Success, "投屏已启动，可以使用电脑键鼠操作设备。".into());
                }
                if lower.contains("error:") {
                    let message = translate_error(line);
                    state.last_error = Some(message.clone());
                    push_log(&mut state, LogLevel::Error, message);
                } else if lower.contains("warn:") || lower.contains("warning:") {
                    push_log(&mut state, LogLevel::Warning, format!("组件提示：{}", truncate(line, 500)));
                } else {
                    push_log(&mut state, LogLevel::Info, format!("scrcpy · {}", truncate(line, 500)));
                }
            }
        });
    }

    async fn wait_for_startup(&self) -> Result<(), String> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let state = self.0.state.read().await;
            if state.session.as_ref().is_some_and(|session| session.phase == SessionPhase::Running) { return Ok(()); }
            if state.session.is_none() {
                return Err(state.last_error.clone().unwrap_or_else(|| "投屏已结束。请检查设备状态与诊断信息。".into()));
            }
            drop(state);
            if Instant::now() >= deadline {
                self.log(LogLevel::Warning, "投屏组件仍在等待画面，请检查手机授权与诊断信息。".into()).await;
                return Ok(()); // Remains 'starting'; never falsely reports a running session.
            }
            if let Some(active) = self.0.active.lock().await.as_ref() {
                if active.user_stop.load(Ordering::SeqCst) { return Ok(()); }
            }
            sleep(Duration::from_millis(100)).await;
        }
    }

    async fn stop_inner(&self) -> Result<(), String> {
        let active = self.0.active.lock().await.take();
        self.0.state.write().await.session = None;
        if let Some(active) = active {
            active.user_stop.store(true, Ordering::SeqCst);
            let mut child = active.child.lock().await;
            if child.try_wait().map_err(|error| format!("无法读取投屏进程状态：{error}"))?.is_none() {
                child.kill().await.map_err(|error| format!("停止投屏失败：{error}"))?;
            }
            child.wait().await.map_err(|error| format!("等待投屏结束失败：{error}"))?;
            self.log(LogLevel::Info, "投屏已停止。".into()).await;
        }
        Ok(())
    }

    async fn poll_session(&self) {
        let pending = {
            let active = self.0.active.lock().await;
            active.as_ref().map(|current| (current.token, current.child.clone()))
        };
        let Some((token, child)) = pending else { return; };
        let status = match child.lock().await.try_wait() {
            Ok(Some(status)) => status,
            Ok(None) => return,
            Err(error) => {
                self.log(LogLevel::Error, format!("读取投屏进程状态失败：{error}")).await;
                return;
            }
        };
        // A process exit can be observed just before its buffered stderr is read.
        // Drain that last scheduling turn while the session is still identifiable.
        sleep(Duration::from_millis(100)).await;
        let ended = {
            let mut active = self.0.active.lock().await;
            if active.as_ref().is_none_or(|current| current.token != token) { return; }
            active.take().unwrap()
        };
        self.0.state.write().await.session = None;
        let user_stop = ended.user_stop.load(Ordering::SeqCst);
        let lost = ended.connection_lost.load(Ordering::SeqCst);
        if user_stop || self.0.shutting_down.load(Ordering::SeqCst) { return; }
        // User window closes have no loss flag and therefore never reconnect.
        if !status.success() || lost {
            self.log(LogLevel::Warning, "投屏连接已结束，正在检查设备状态。".into()).await;
        } else {
            self.log(LogLevel::Info, "投屏窗口已关闭。".into()).await;
        }
        if !recovery_allowed(user_stop, ended.session.transport, lost && !status.success(), ended.recovery_attempted, false) {
            if !status.success() {
                let mut state = self.0.state.write().await;
                if state.last_error.is_none() {
                    state.last_error = Some("投屏组件已退出，请查看诊断信息并重试。".into());
                }
            }
            return;
        }
        let saved_address = self.0.state.read().await.saved_devices.iter()
            .find(|saved| saved.id == ended.session.device_id).map(|saved| saved.address.clone());
        let Some(address) = ended.recovery_serial.clone().or(saved_address) else {
            self.log(LogLevel::Warning, "USB 已断开。该设备尚未配置无线连接，请重新插线或配置无线。".into()).await;
            return;
        };
        // Finish any user operation first. Stop/shutdown invalidate this recovery
        // by incrementing stop_epoch, even after the old child has been removed.
        let lock = self.0.operation.clone().lock_owned().await;
        if ended.user_stop.load(Ordering::SeqCst)
            || self.0.shutting_down.load(Ordering::SeqCst)
            || self.0.stop_epoch.load(Ordering::SeqCst) != ended.stop_epoch { return; }
        if self.0.active.lock().await.is_some() { return; }
        self.0.busy.store(true, Ordering::SeqCst);
        let _guard = OperationGuard { _guard: lock, busy: self.0.busy.clone() };
        let recovery = async {
            let adb_state = self.adb(&["devices", "-l"], ADB_TIMEOUT).await?;
            let connections = parse_adb_devices(&adb_state);
            let usb_present = connections.iter().any(|device| device.serial == ended.session.serial);
            if usb_present { return Ok(false); }
            self.log(LogLevel::Info, "USB 已断开，正在尝试恢复为同一设备的无线投屏…".into()).await;
            if !connections.iter().any(|device| device.serial == address && device.state == "device") {
                self.connect_verified(&address).await?;
            }
            let metadata = self.metadata(&address, &ended.session.device_name, TransportKind::Wifi, true).await?;
            verify_device_identity(&ended.session.device_id, &metadata)?;
            self.refresh_inner().await?;
            let device = self.find_device(&ended.session.device_id).await?;
            let transport = device.transports.iter().find(|transport| transport.kind == TransportKind::Wifi && transport.serial == address && transport.state == "device")
                .ok_or_else(|| "无线设备仍未就绪，请检查网络后手动重试。".to_string())?.clone();
            if self.0.stop_epoch.load(Ordering::SeqCst) != ended.stop_epoch
                || self.0.shutting_down.load(Ordering::SeqCst) { return Ok(false); }
            self.launch_session(&device, &transport, true, ended.stop_epoch).await?;
            Ok::<bool, String>(true)
        }.await;
        if let Err(error) = recovery {
            let message = format!("无线恢复失败：{error} 请检查网络后点击「开始投屏」重试。");
            let mut state = self.0.state.write().await;
            state.last_error = Some(message.clone());
            push_log(&mut state, LogLevel::Error, message);
        }
    }
}

fn parse_persisted(bytes: &[u8]) -> Result<Persisted, String> {
    let value: Persisted = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    value.settings.validate()?;
    Ok(value)
}

fn validate_navigation_request(token: u64, active_token: u64, phase: SessionPhase, stopping: bool) -> Result<(), String> {
    if token != active_token { return Err("投屏会话已切换，请在当前投屏窗口重新操作。".into()); }
    if stopping { return Err("投屏正在停止，本次导航已取消。".into()); }
    if phase != SessionPhase::Running { return Err("投屏尚未就绪或正在重连，请稍候再操作。".into()); }
    Ok(())
}

fn build_scrcpy_args(device: &Device, transport: &Transport, settings: &Settings) -> Result<Vec<String>, String> {
    settings.validate()?;
    // Keep the Android display at its original size. An absent max-fps option
    // lets scrcpy use the device's maximum instead of imposing a guessed cap.
    let mut args = vec![
        "--serial".into(), transport.serial.clone(),
        "--window-title".into(), format!("Qzz Mirror · {}", device.name),
        "--max-size".into(), "0".into(),
        "--video-bit-rate".into(), "8M".into(),
    ];
    if let Some(fps) = settings.max_fps {
        args.extend(["--max-fps".into(), fps.to_string()]);
    }
    if !settings.audio || !device.api_level.is_some_and(|api| api >= 30) {
        args.push("--no-audio".into());
    }
    if settings.always_on_top { args.push("--always-on-top".into()); }
    Ok(args)
}

fn parse_adb_devices(text: &str) -> Vec<AdbDevice> {
    text.lines().filter_map(|line| {
        let mut fields = line.split_whitespace();
        let serial = fields.next()?;
        let state = fields.next()?;
        if !matches!(state, "device" | "offline" | "unauthorized" | "recovery" | "sideload" | "bootloader" | "no") { return None; }
        let metadata = fields.collect::<Vec<_>>();
        let model = metadata.iter().find_map(|field| field.strip_prefix("model:")).unwrap_or("Android 设备").replace('_', " ");
        let kind = if metadata.iter().any(|field| field.starts_with("usb:")) {
            TransportKind::Usb
        } else if serial.contains(':') || serial.contains("._adb-tls-connect._tcp") {
            TransportKind::Wifi
        } else { TransportKind::Usb };
        Some(AdbDevice { serial: serial.to_string(), state: state.to_string(), model, kind })
    }).collect()
}

fn fallback_device_id(serial: &str, kind: TransportKind) -> String {
    if kind == TransportKind::Usb { format!("android:{serial}") } else { format!("transport:{serial}") }
}

fn group_devices(entries: Vec<(AdbDevice, Option<Metadata>)>, saved: &[SavedDevice]) -> Vec<Device> {
    let mut devices: Vec<Device> = Vec::new();
    for (entry, metadata) in entries {
        let id = metadata.as_ref().map(|metadata| metadata.id.clone())
            .or_else(|| saved.iter().find(|saved| saved.address == entry.serial).map(|saved| saved.id.clone()))
            .unwrap_or_else(|| fallback_device_id(&entry.serial, entry.kind));
        let transport = Transport { serial: entry.serial.clone(), kind: entry.kind, state: entry.state };
        if let Some(device) = devices.iter_mut().find(|device| device.id == id) {
            device.transports.push(transport);
            if entry.kind == TransportKind::Wifi { device.wifi_address = Some(entry.serial); }
            if device.api_level.is_none() {
                if let Some(metadata) = metadata {
                    device.api_level = metadata.api_level;
                    device.android_version = metadata.android_version;
                    device.audio_supported = metadata.api_level.map(|api| api >= 30);
                    device.model = metadata.model.clone();
                    device.name = metadata.model;
                }
            }
        } else {
            let model = metadata.as_ref().map(|metadata| metadata.model.clone()).filter(|model| !model.is_empty()).unwrap_or(entry.model);
            let api_level = metadata.as_ref().and_then(|metadata| metadata.api_level);
            let wifi_address = if entry.kind == TransportKind::Wifi { Some(entry.serial) }
                else { saved.iter().find(|saved| saved.id == id).map(|saved| saved.address.clone()) };
            devices.push(Device {
                id, name: model.clone(), model,
                android_version: metadata.as_ref().and_then(|metadata| metadata.android_version.clone()),
                api_level, transports: vec![transport], wifi_address,
                audio_supported: api_level.map(|api| api >= 30),
            });
        }
    }
    devices.sort_by(|left, right| left.name.cmp(&right.name).then_with(|| left.id.cmp(&right.id)));
    devices
}

fn choose_transport(device: &Device, preference: TransportPreference) -> Result<&Transport, String> {
    let online = |kind| device.transports.iter().find(|transport| transport.kind == kind && transport.state == "device");
    match preference {
        TransportPreference::Auto => online(TransportKind::Usb).or_else(|| online(TransportKind::Wifi)),
        TransportPreference::Usb => online(TransportKind::Usb),
        TransportPreference::Wifi => online(TransportKind::Wifi),
    }.ok_or_else(|| {
        if device.transports.iter().any(|transport| transport.state == "unauthorized") {
            "设备尚未授权。请解锁手机并允许 USB 调试，然后刷新。".into()
        } else if device.transports.iter().any(|transport| transport.state == "offline") {
            "设备处于离线状态，请重新连接 USB 或无线后重试。".into()
        } else {
            match preference {
                TransportPreference::Usb => "所选设备没有可用的 USB 连接，请先插入数据线。",
                TransportPreference::Wifi => "所选设备没有可用的 Wi-Fi 连接，请先配置或连接无线。",
                TransportPreference::Auto => "所选设备没有可用的连接，请检查授权和设备状态。",
            }.into()
        }
    })
}

fn validate_endpoint(address: &str) -> Result<String, String> {
    let address = address.trim();
    let invalid = || "地址格式不正确，请输入 IP 或主机名和端口，例如 192.168.1.20:5555。".to_string();
    if address.is_empty() || address.len() > 300 || address.chars().any(char::is_whitespace) { return Err(invalid()); }
    let (host, port) = address.rsplit_once(':').ok_or_else(invalid)?;
    let port = port.parse::<u16>().ok().filter(|port| *port != 0).ok_or_else(invalid)?;
    if host.starts_with('[') && host.ends_with(']') {
        host[1..host.len() - 1].parse::<Ipv6Addr>().map_err(|_| invalid())?;
    } else {
        if host.is_empty() || host.len() > 253 || host.contains(':') { return Err(invalid()); }
        if host.chars().all(|character| character.is_ascii_digit() || character == '.') {
            host.parse::<Ipv4Addr>().map_err(|_| invalid())?;
        } else if !host.split('.').all(|label| {
            !label.is_empty() && label.len() <= 63 && label.as_bytes()[0].is_ascii_alphanumeric()
                && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
                && label.bytes().all(|character| character.is_ascii_alphanumeric() || character == b'-')
        }) { return Err(invalid()); }
    }
    Ok(format!("{}:{port}", host.to_ascii_lowercase()))
}

fn endpoint_host(address: &str) -> &str { address.rsplit_once(':').map(|(host, _)| host).unwrap_or(address) }

fn parse_wifi_ip(routes: &str) -> Option<String> {
    routes.lines().find_map(|line| {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        let interface = fields.windows(2).find(|pair| pair[0] == "dev").map(|pair| pair[1])?;
        if !(interface.starts_with("wlan") || interface.starts_with("wifi") || interface.starts_with("swlan")) { return None; }
        let source = fields.windows(2).find(|pair| pair[0] == "src").map(|pair| pair[1])?;
        let address = source.parse::<Ipv4Addr>().ok()?;
        if address.is_loopback() || address.is_unspecified() || address.is_link_local() { return None; }
        Some(address.to_string())
    })
}

fn recovery_allowed(user_stop: bool, kind: TransportKind, connection_lost: bool, already_attempted: bool, usb_present: bool) -> bool {
    !user_stop && kind == TransportKind::Usb && connection_lost && !already_attempted && !usb_present
}

fn verify_device_identity(expected_id: &str, metadata: &Metadata) -> Result<(), String> {
    if !metadata.hardware_identity_verified {
        return Err("设备未提供可核验的唯一标识，无法确认无线地址仍属于所选设备。请重新连接并选择当前设备；若仍无法确认，请使用 USB 投屏。".into());
    }
    if metadata.id != expected_id {
        return Err("无线地址对应的设备已发生变化，已取消启动。请重新连接并选择当前实际设备后重试。".into());
    }
    Ok(())
}

fn is_connection_loss(lower: &str) -> bool {
    lower.contains("device disconnected") || lower.contains("device was disconnected")
        || lower.contains("device connection closed") || lower.contains("device not found")
        || lower.contains("device offline") || lower.contains("disconnected from device")
        || lower.contains("connection reset by peer") || lower.contains("demuxer") && lower.contains("connection error")
}

fn is_video_ready(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    let message = lower.strip_prefix("info:").unwrap_or(&lower).trim_start();
    let Some(mut suffix) = message.strip_prefix("video texture").or_else(|| message.strip_prefix("texture")) else {
        return false;
    };
    suffix = suffix.trim_start();
    // scrcpy 5 logs hardware-decoded frames as "Texture (D3D11VA): WxH".
    if let Some(annotation) = suffix.strip_prefix('(') {
        let Some((_, rest)) = annotation.split_once(')') else { return false; };
        suffix = rest.trim_start();
    }
    let Some(dimensions) = suffix.strip_prefix(':').and_then(|value| value.split_whitespace().next()) else {
        return false;
    };
    let Some((width, height)) = dimensions.split_once('x') else { return false; };
    width.parse::<u32>().is_ok_and(|value| value > 0)
        && height.parse::<u32>().is_ok_and(|value| value > 0)
}

fn translate_error(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let message = if lower.contains("unauthorized") {
        "设备尚未授权，请解锁手机并允许 USB 调试。"
    } else if lower.contains("offline") {
        "设备处于离线状态，请重新连接后重试。"
    } else if lower.contains("more than one") || lower.contains("multiple adb") {
        "检测到多条连接，请重新选择设备与连接方式。"
    } else if lower.contains("no devices") || lower.contains("device not found") || lower.contains("no adb device") {
        "未找到所选设备，请检查 USB 或无线连接。"
    } else if lower.contains("refused") || lower.contains("10061") {
        "设备拒绝无线连接，请检查无线调试是否开启以及连接端口是否正确。"
    } else if lower.contains("timed out") || lower.contains("10060") || lower.contains("no route") || lower.contains("unreachable") {
        "无法访问设备，请确认手机与电脑在同一局域网，并检查地址与端口。"
    } else if lower.contains("failed to authenticate") || lower.contains("authentication") {
        "无线调试授权失效，请在手机上重新配对。"
    } else if is_connection_loss(&lower) {
        "设备连接已断开，请检查数据线或局域网。"
    } else if lower.contains("encoder") || lower.contains("codec") {
        "设备编码器启动失败，请尝试降低帧率后重新投屏。"
    } else if lower.contains("audio") {
        "声音转发失败，请关闭声音后重新投屏。"
    } else {
        "操作未完成，请检查设备连接和下面的组件信息。"
    };
    format!("{message}{}", raw_suffix(text))
}

fn output_text(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!("{}\n{}", stdout.trim(), stderr.trim()).trim().to_string()
}

fn raw_suffix(text: &str) -> String {
    if text.trim().is_empty() { String::new() } else { format!("\n组件信息：{}", truncate(text.trim(), 700)) }
}

fn truncate(text: &str, length: usize) -> String {
    let mut characters = text.chars();
    let value = characters.by_ref().take(length).collect::<String>();
    if characters.next().is_some() { format!("{value}…") } else { value }
}

fn push_log(state: &mut State, level: LogLevel, message: String) {
    state.logs.push(LogEntry { time: now(), level, message });
    if state.logs.len() > MAX_LOGS { state.logs.drain(..state.logs.len() - MAX_LOGS); }
}

fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64 }

#[cfg(test)]
mod tests {
    use super::*;

    fn adb(serial: &str, kind: TransportKind, model: &str) -> AdbDevice {
        AdbDevice { serial: serial.into(), state: "device".into(), model: model.into(), kind }
    }
    fn metadata(id: &str, model: &str) -> Metadata {
        Metadata { id: format!("android:{id}"), hardware_identity_verified: true, model: model.into(), android_version: Some("9".into()), api_level: Some(28) }
    }

    fn launch_fixture() -> (Device, Transport) {
        let device = group_devices(vec![
            (adb("USB-A", TransportKind::Usb, "Android"), Some(metadata("A", "Android"))),
        ], &[]).remove(0);
        let transport = device.transports[0].clone();
        (device, transport)
    }

    #[test]
    fn navigation_actions_serialize_and_map_to_scrcpy_shortcuts() {
        for (action, json, key) in [
            (NavigationAction::Recents, "\"recents\"", b'S' as u16),
            (NavigationAction::Back, "\"back\"", b'B' as u16),
            (NavigationAction::Home, "\"home\"", b'H' as u16),
        ] {
            assert_eq!(serde_json::to_string(&action).unwrap(), json);
            assert_eq!(serde_json::from_str::<NavigationAction>(json).unwrap(), action);
            assert_eq!(action.shortcut_virtual_key(), key);
        }
        assert!(serde_json::from_str::<NavigationAction>("\"4;reboot\"").is_err());
    }

    #[test]
    fn navigation_only_accepts_the_current_running_session() {
        assert!(validate_navigation_request(7, 7, SessionPhase::Running, false).is_ok());
        assert!(validate_navigation_request(6, 7, SessionPhase::Running, false).is_err());
        assert!(validate_navigation_request(7, 7, SessionPhase::Starting, false).is_err());
        assert!(validate_navigation_request(7, 7, SessionPhase::Reconnecting, false).is_err());
        assert!(validate_navigation_request(7, 7, SessionPhase::Running, true).is_err());
    }

    #[test]
    fn default_arguments_preserve_native_resolution_without_frame_rate_cap() {
        let (device, transport) = launch_fixture();
        let args = build_scrcpy_args(&device, &transport, &Settings::default()).unwrap();
        assert!(args.windows(2).any(|pair| pair == ["--max-size", "0"]));
        assert!(args.windows(2).any(|pair| pair == ["--video-bit-rate", "8M"]));
        assert!(args.windows(2).any(|pair| pair == ["--serial", "USB-A"]));
        assert!(!args.iter().any(|argument| argument == "--max-fps"));
    }

    #[test]
    fn custom_frame_rates_are_exact_and_do_not_change_resolution() {
        let (device, transport) = launch_fixture();
        for fps in [1, 30, 120, 240] {
            let settings = Settings { max_fps: Some(fps), ..Settings::default() };
            let args = build_scrcpy_args(&device, &transport, &settings).unwrap();
            assert!(args.windows(2).any(|pair| pair[0] == "--max-fps" && pair[1] == fps.to_string()));
            assert!(args.windows(2).any(|pair| pair == ["--max-size", "0"]));
        }
    }

    #[test]
    fn old_preset_settings_migrate_without_losing_user_choices_or_saved_devices() {
        let persisted = parse_persisted(br#"{
            "settings": {"preset":"smooth","audio":false,"alwaysOnTop":true,"preferredTransport":"wifi","lastDeviceId":"android:A"},
            "savedDevices": [{"id":"android:A","name":"My Android","address":"192.168.1.2:5555"}]
        }"#).unwrap();
        assert_eq!(persisted.settings.max_fps, None);
        assert!(!persisted.settings.audio);
        assert!(persisted.settings.always_on_top);
        assert_eq!(persisted.settings.preferred_transport, TransportPreference::Wifi);
        assert_eq!(persisted.settings.last_device_id.as_deref(), Some("android:A"));
        assert_eq!(persisted.saved_devices[0].address, "192.168.1.2:5555");
        let migrated = serde_json::to_value(&persisted).unwrap();
        assert!(migrated["settings"].get("preset").is_none());
        assert!(migrated["settings"]["maxFps"].is_null());
    }

    #[test]
    fn invalid_frame_rates_are_rejected_in_memory_arguments_and_persisted_settings() {
        let (device, transport) = launch_fixture();
        for fps in [0, 241, u16::MAX] {
            let settings = Settings { max_fps: Some(fps), ..Settings::default() };
            assert!(settings.validate().is_err());
            assert!(build_scrcpy_args(&device, &transport, &settings).is_err());
        }
        for value in ["0", "241", "65536", "-1", "29.5", "\"60\""] {
            let json = format!("{{\"settings\":{{\"maxFps\":{value}}}}}");
            assert!(parse_persisted(json.as_bytes()).is_err(), "{value}");
        }
        assert!(parse_persisted(br#"{"settings":{"maxFps":null}}"#).is_ok());
        assert!(parse_persisted(br#"{"settings":{"maxFps":240}}"#).is_ok());
    }

    #[test]
    fn parses_usb_wifi_and_unavailable_devices_without_daemon_noise() {
        let devices = parse_adb_devices("* daemon started successfully\r\nList of devices attached\r\nABC device usb:1-2 product:p model:Pixel_8 device:p transport_id:1\r\n192.168.1.7:5555 device product:p model:Pixel_8 transport_id:2\r\nXYZ unauthorized usb:2\r\n10.0.0.2:5555 offline\r\n");
        assert_eq!(devices.len(), 4);
        assert_eq!(devices[0].kind, TransportKind::Usb);
        assert_eq!(devices[0].model, "Pixel 8");
        assert_eq!(devices[1].kind, TransportKind::Wifi);
        assert_eq!(devices[2].state, "unauthorized");
        assert_eq!(devices[3].state, "offline");
    }

    #[test]
    fn deduplicates_by_hardware_serial_never_by_model() {
        let devices = group_devices(vec![
            (adb("USB-A", TransportKind::Usb, "Same model"), Some(metadata("A", "Same model"))),
            (adb("10.0.0.1:5555", TransportKind::Wifi, "Same model"), Some(metadata("A", "Same model"))),
            (adb("USB-B", TransportKind::Usb, "Same model"), Some(metadata("B", "Same model"))),
        ], &[]);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices.iter().find(|device| device.id == "android:A").unwrap().transports.len(), 2);
        assert_eq!(devices[0].audio_supported, Some(false));
    }

    #[test]
    fn preference_only_uses_transports_of_the_selected_device() {
        let devices = group_devices(vec![
            (adb("10.0.0.1:5555", TransportKind::Wifi, "A"), Some(metadata("A", "A"))),
            (adb("USB-B", TransportKind::Usb, "B"), Some(metadata("B", "B"))),
        ], &[]);
        let selected = devices.iter().find(|device| device.id == "android:A").unwrap();
        assert_eq!(choose_transport(selected, TransportPreference::Auto).unwrap().serial, "10.0.0.1:5555");
        assert!(choose_transport(selected, TransportPreference::Usb).is_err());
    }

    #[test]
    fn wireless_launch_rejects_reassigned_addresses_and_unproven_identity() {
        assert!(verify_device_identity("android:A", &metadata("A", "Same model")).is_ok());
        assert!(verify_device_identity("android:A", &metadata("B", "Same model")).is_err());
        let mut unproven = metadata("A", "Same model");
        unproven.hardware_identity_verified = false;
        assert!(verify_device_identity("android:A", &unproven).is_err());
    }

    #[test]
    fn endpoint_validation_accepts_ports_and_rejects_shell_or_invalid_addresses() {
        for address in ["192.168.1.2:5555", "phone.local:37123", "[fe80::1]:5555", "localhost:1"] {
            assert!(validate_endpoint(address).is_ok(), "{address}");
        }
        for address in ["192.168.1.2", "192.168.1.2:0", "a:65536", "999.1.1.1:5555", "x;calc:5555", "x:1 --help", "-x:5555", "a:b:5555", "a:5555\ncalc"] {
            assert!(validate_endpoint(address).is_err(), "{address}");
        }
    }

    #[test]
    fn close_stop_and_wifi_disconnect_do_not_trigger_recovery() {
        assert!(recovery_allowed(false, TransportKind::Usb, true, false, false));
        assert!(!recovery_allowed(true, TransportKind::Usb, true, false, false));
        assert!(!recovery_allowed(false, TransportKind::Usb, false, false, false));
        assert!(!recovery_allowed(false, TransportKind::Wifi, true, false, false));
        assert!(!recovery_allowed(false, TransportKind::Usb, true, true, false));
        assert!(!recovery_allowed(false, TransportKind::Usb, true, false, true));
    }

    #[test]
    fn disconnect_detection_uses_transport_errors_not_audio_warnings() {
        assert!(is_connection_loss("warn: device disconnected"));
        assert!(is_connection_loss("error: device offline"));
        assert!(!is_connection_loss("warn: audio disabled: before android 11"));
        assert!(!is_connection_loss("warn: demuxer 'audio': stream explicitly disabled by the device"));
        assert!(!is_connection_loss("debug: user requested to quit"));
    }

    #[test]
    fn first_video_frame_supports_scrcpy5_hardware_decoder_annotations() {
        for line in [
            "INFO: Texture (D3D11VA): 480x800",
            "INFO: Texture: 480x800",
            "INFO: Video texture: 1920x1080",
            "INFO: Video texture (VAAPI): 1080x1920",
            "Texture (software): 480x800",
        ] {
            assert!(is_video_ready(line), "{line}");
        }
        for line in [
            "INFO: Renderer: direct3d11",
            "INFO: Video codec: h264",
            "ERROR: Texture: 480x800",
            "INFO: Texture: creation failed",
            "INFO: Texture (D3D11VA): 0x800",
            "INFO: Texture (D3D11VA: 480x800",
        ] {
            assert!(!is_video_ready(line), "{line}");
        }
    }

    #[test]
    fn wifi_route_excludes_mobile_and_loopback_addresses() {
        let routes = "default via 10.1.1.1 dev rmnet_data0 src 10.1.1.9\n192.0.2.0/24 dev wlan0 proto kernel scope link src 192.0.2.10\n";
        assert_eq!(parse_wifi_ip(routes).as_deref(), Some("192.0.2.10"));
        assert!(parse_wifi_ip("127.0.0.0/8 dev wlan0 src 127.0.0.1").is_none());
    }
}
