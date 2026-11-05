//! Running-process-only USB notifications. No protocol or COM-server registration is
//! created here. Only current in-memory tokens can request a mirror session.
//! Both portable and installed runs register their current-user notification identity.

use std::time::Duration;

const TOAST_LIFETIME: Duration = Duration::from_secs(5 * 60);
const TOAST_GROUP: &str = "usb-device";
const MAX_TOASTS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastEvent {
    Submitted { token: String },
    OpenPanel { token: String },
    StartMirroring { token: String },
    Dismissed { token: String },
    Failed { token: String, message: String },
}

fn valid_token(token: &str) -> bool {
    !token.is_empty()
        && token.len() <= 128
        && token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn activation_event(arguments: &str, expected_token: &str) -> Option<ToastEvent> {
    if !valid_token(expected_token) {
        return None;
    }
    let (action, token) = arguments.split_once(':')?;
    if token != expected_token {
        return None;
    }
    match action {
        "open" => Some(ToastEvent::OpenPanel { token: token.into() }),
        "start" => Some(ToastEvent::StartMirroring { token: token.into() }),
        _ => None,
    }
}

fn xml_text(value: &str) -> String {
    let mut escaped = String::new();
    // A device-reported model is untrusted text. Bound its visual size, strip
    // invalid XML control characters and escape it before adding it to XML.
    for ch in value.chars().filter(|ch| {
        matches!(*ch, '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
    }).take(160) {
        escaped.push_str(match ch {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '\"' => "&quot;",
            '\'' => "&apos;",
            _ => { escaped.push(ch); continue; }
        });
    }
    if escaped.trim().is_empty() { "Android 设备".into() } else { escaped }
}

fn toast_xml(token: &str, device_name: &str) -> Result<String, String> {
    if !valid_token(token) {
        return Err("通知标识无效".into());
    }
    Ok(format!(
        "<toast launch=\"open:{token}\" activationType=\"foreground\" duration=\"short\">\
         <visual><binding template=\"ToastGeneric\">\
         <text>检测到 USB 设备</text><text>{}，是否开始投屏？</text>\
         </binding></visual><actions>\
         <action content=\"开始投屏\" arguments=\"start:{token}\" activationType=\"foreground\"/>\
         <action content=\"暂不\" arguments=\"dismiss\" activationType=\"system\"/>\
         </actions><audio silent=\"true\"/></toast>",
        xml_text(device_name)
    ))
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::{
        collections::HashMap,
        sync::{
            atomic::{AtomicBool, Ordering},
            mpsc::{self, Receiver, RecvTimeoutError, Sender},
            Arc, Mutex,
        },
        thread::{self, JoinHandle},
        time::{Instant, SystemTime, UNIX_EPOCH},
    };
    use windows::{
        core::{IInspectable, Interface, HSTRING, PCWSTR},
        Data::Xml::Dom::XmlDocument,
        Foundation::{DateTime, IReference, PropertyValue, TypedEventHandler},
        UI::Notifications::{
            NotificationSetting, ToastActivatedEventArgs, ToastDismissalReason,
            ToastDismissedEventArgs, ToastFailedEventArgs, ToastNotification,
            ToastNotificationManager, ToastNotifier,
        },
        Win32::{
            Foundation::ERROR_FILE_NOT_FOUND,
            System::{
                Registry::{RegCloseKey, RegCreateKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_CREATED_NEW_KEY, REG_OPEN_CREATE_OPTIONS, REG_SZ},
                WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
            },
            UI::Shell::SetCurrentProcessExplicitAppUserModelID,
        },
    };

    type Callback = Arc<dyn Fn(ToastEvent) + Send + Sync>;
    type Gate = Arc<AtomicBool>;

    fn wide(value: &str) -> Vec<u16> { value.encode_utf16().chain(Some(0)).collect() }

    fn valid_app_id(identifier: &str) -> bool {
        !identifier.is_empty() && identifier.len() <= 128
            && identifier.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    }

    struct IdentityKey(HKEY);
    impl Drop for IdentityKey { fn drop(&mut self) { let _ = unsafe { RegCloseKey(self.0) }; } }

    impl IdentityKey {
        fn open(identifier: &str) -> Result<(Self, bool), String> {
            if !valid_app_id(identifier) { return Err("通知应用标识无效".into()); }
            let subkey = wide(&format!("Software\\Classes\\AppUserModelId\\{identifier}"));
            let mut key = HKEY::default();
            let mut disposition = REG_CREATED_NEW_KEY;
            unsafe {
                RegCreateKeyExW(HKEY_CURRENT_USER, PCWSTR(subkey.as_ptr()), None, PCWSTR::null(), REG_OPEN_CREATE_OPTIONS(0), KEY_QUERY_VALUE | KEY_SET_VALUE, None, &mut key, Some(&mut disposition))
            }.ok().map_err(|error| format!("无法注册通知应用标识：{error}"))?;
            Ok((Self(key), disposition == REG_CREATED_NEW_KEY))
        }

        fn value_exists(&self, name: &str) -> Result<bool, String> {
            let name = wide(name);
            let result = unsafe { RegQueryValueExW(self.0, PCWSTR(name.as_ptr()), None, None, None, None) };
            if result == ERROR_FILE_NOT_FOUND { return Ok(false); }
            result.ok().map_err(|error| format!("无法读取通知应用信息：{error}"))?;
            Ok(true)
        }

        fn set_string(&self, name: &str, value: &str) -> Result<(), String> {
            let name = wide(name);
            let bytes: Vec<u8> = wide(value).into_iter().flat_map(u16::to_le_bytes).collect();
            unsafe { RegSetValueExW(self.0, PCWSTR(name.as_ptr()), None, REG_SZ, Some(&bytes)) }
                .ok().map_err(|error| format!("无法保存通知应用信息：{error}"))
        }
    }

    fn ensure_notification_identity(identifier: &str) -> Result<(), String> {
        // A Start-menu shortcut may point at a different installed version while
        // this run is portable. Register only this app's current-user identity;
        // preserve any existing branded values and never alter notification policy.
        let (key, _) = IdentityKey::open(identifier)?;
        if !key.value_exists("DisplayName")? { key.set_string("DisplayName", "Qzz Mirror · 轻镜")?; }
        if !key.value_exists("IconUri")? {
            let cache = std::env::var_os("LOCALAPPDATA").filter(|value| !value.is_empty())
                .map(std::path::PathBuf::from).ok_or("无法确定通知图标缓存目录")?
                .join("com.qzz.mirror").join("notifications");
            std::fs::create_dir_all(&cache).map_err(|error| format!("无法创建通知图标目录：{error}"))?;
            let icon = cache.join("icon.png");
            let icon_bytes = include_bytes!("../icons/icon.png");
            if std::fs::read(&icon).ok().as_deref() != Some(icon_bytes.as_slice()) {
                std::fs::write(&icon, icon_bytes).map_err(|error| format!("无法保存通知图标：{error}"))?;
            }
            key.set_string("IconUri", &icon.to_string_lossy())?;
        }
        let identifier = wide(identifier);
        unsafe { SetCurrentProcessExplicitAppUserModelID(PCWSTR(identifier.as_ptr())) }
            .map_err(|error| format!("无法设置通知进程标识：{error}"))
    }

    fn check_notification_setting(app_id: &str, result: windows::core::Result<NotificationSetting>) -> Result<(), String> {
        match result {
            Ok(NotificationSetting::Enabled) => Ok(()),
            Ok(_) => Err("Windows 已关闭轻镜的系统通知".into()),
            // A newly registered unpackaged app can have no notification record
            // yet. Treat this one status as unknown and let Show (and Windows'
            // own permission checks) decide, instead of blocking the first toast.
            Err(error) if error.code().0 == 0x8007_0490_u32 as i32 => Ok(()),
            Err(error) => Err(format!("无法读取 Windows 通知状态（{app_id}）：{error}")),
        }
    }

    struct Shared {
        running: AtomicBool,
        gates: Mutex<HashMap<String, Gate>>,
        sender: Sender<Command>,
        callback: Callback,
    }

    enum Command {
        Show { token: String, name: String, gate: Gate },
        Remove(String),
        Clear,
        Retire { token: String, gate: Gate },
        Shutdown,
    }

    /// Calls enqueue work only. Windows events invoke `callback` from a WinRT
    /// worker; the application must queue its own UI/engine work from there.
    pub struct ToastService {
        shared: Arc<Shared>,
        worker: Mutex<Option<JoinHandle<()>>>,
    }

    impl ToastService {
        pub fn start(
            identifier: &str,
            callback: impl Fn(ToastEvent) + Send + Sync + 'static,
        ) -> Result<Self, String> {
            if !valid_app_id(identifier) {
                return Err("通知应用标识无效".into());
            }
            let (sender, receiver) = mpsc::channel();
            let shared = Arc::new(Shared {
                running: AtomicBool::new(true),
                gates: Mutex::new(HashMap::new()),
                sender,
                callback: Arc::new(callback),
            });
            let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
            let app_id = identifier.to_owned();
            let worker_shared = shared.clone();
            let worker = thread::Builder::new().name("qzz-usb-toast".into()).spawn(move || {
                let _apartment = match Apartment::new() {
                    Ok(value) => value,
                    Err(error) => { let _ = ready_sender.send(Err(error)); return; }
                };
                if let Err(error) = ensure_notification_identity(&app_id) {
                    let _ = ready_sender.send(Err(error));
                    return;
                }
                let notifier = match ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(&app_id)) {
                    Ok(value) => value,
                    Err(error) => { let _ = ready_sender.send(Err(format!("无法初始化系统通知：{error}"))); return; }
                };
                // Only remove this feature's group, preserving any future unrelated notifications.
                remove_group(&app_id);
                if ready_sender.send(Ok(())).is_err() { return; }
                run_worker(&app_id, notifier, &worker_shared, receiver);
            }).map_err(|error| format!("无法创建通知线程：{error}"))?;
            match ready_receiver.recv() {
                Ok(Ok(())) => Ok(Self { shared, worker: Mutex::new(Some(worker)) }),
                result => {
                    shared.running.store(false, Ordering::Release);
                    let _ = worker.join();
                    Err(match result { Ok(Err(error)) => error, _ => "系统通知线程未能初始化".into() })
                }
            }
        }

        pub fn show(&self, token: &str, device_name: &str) -> Result<(), String> {
            if !valid_token(token) { return Err("通知标识无效".into()); }
            let mut gates = self.shared.gates.lock().map_err(|_| "通知状态不可用")?;
            if !self.shared.running.load(Ordering::Acquire) { return Err("通知服务已关闭".into()); }
            if gates.contains_key(token) { return Err("通知标识已存在".into()); }
            if gates.len() >= MAX_TOASTS { return Err("待处理设备通知已达到上限".into()); }
            let gate = Arc::new(AtomicBool::new(true));
            gates.insert(token.into(), gate.clone());
            if self.shared.sender.send(Command::Show { token: token.into(), name: device_name.into(), gate }).is_err() {
                gates.remove(token);
                return Err("通知服务不可用".into());
            }
            Ok(())
        }

        pub fn remove(&self, token: &str) -> Result<(), String> {
            let mut gates = self.shared.gates.lock().map_err(|_| "通知状态不可用")?;
            if let Some(gate) = gates.remove(token) { gate.store(false, Ordering::Release); }
            self.shared.sender.send(Command::Remove(token.into())).map_err(|_| "通知服务不可用".into())
        }

        pub fn clear(&self) -> Result<(), String> {
            let mut gates = self.shared.gates.lock().map_err(|_| "通知状态不可用")?;
            for (_, gate) in gates.drain() { gate.store(false, Ordering::Release); }
            self.shared.sender.send(Command::Clear).map_err(|_| "通知服务不可用".into())
        }

        pub fn shutdown(&self) {
            if !self.shared.running.swap(false, Ordering::AcqRel) { return; }
            if let Ok(mut gates) = self.shared.gates.lock() {
                for (_, gate) in gates.drain() { gate.store(false, Ordering::Release); }
                let _ = self.shared.sender.send(Command::Shutdown);
            } else { let _ = self.shared.sender.send(Command::Shutdown); }
            if let Ok(mut worker) = self.worker.lock() {
                if let Some(worker) = worker.take() { let _ = worker.join(); }
            }
        }
    }

    impl Drop for ToastService { fn drop(&mut self) { self.shutdown(); } }

    struct Apartment;
    impl Apartment {
        fn new() -> Result<Self, String> {
            unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(|error| format!("无法初始化通知组件：{error}"))?;
            Ok(Self)
        }
    }
    impl Drop for Apartment { fn drop(&mut self) { unsafe { RoUninitialize() }; } }

    struct ActiveToast {
        toast: ToastNotification,
        activated: i64,
        dismissed: i64,
        failed: i64,
        gate: Gate,
        tag: String,
        expires: Instant,
    }

    impl Drop for ActiveToast {
        fn drop(&mut self) {
            self.gate.store(false, Ordering::Release);
            let _ = self.toast.RemoveActivated(self.activated);
            let _ = self.toast.RemoveDismissed(self.dismissed);
            let _ = self.toast.RemoveFailed(self.failed);
        }
    }

    fn notify(shared: &Shared, event: ToastEvent) {
        if shared.running.load(Ordering::Acquire) {
            // Never unwind through a Windows event handler if the host callback fails.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (shared.callback)(event)));
        }
    }

    fn consume(shared: &Shared, token: &str, gate: &Gate, event: ToastEvent) {
        if shared.running.load(Ordering::Acquire) && gate.swap(false, Ordering::AcqRel) {
            notify(shared, event);
            let _ = shared.sender.send(Command::Retire { token: token.into(), gate: gate.clone() });
        }
    }

    fn prepare_toast(shared: &Arc<Shared>, token: &str, name: &str, gate: Gate, tag: String) -> Result<ActiveToast, String> {
        let xml = XmlDocument::new().map_err(|error| format!("无法创建通知内容：{error}"))?;
        xml.LoadXml(&HSTRING::from(toast_xml(token, name)?)).map_err(|error| format!("通知内容无效：{error}"))?;
        let toast = ToastNotification::CreateToastNotification(&xml).map_err(|error| format!("无法创建系统通知：{error}"))?;
        toast.SetTag(&HSTRING::from(&tag)).map_err(|error| error.to_string())?;
        toast.SetGroup(&HSTRING::from(TOAST_GROUP)).map_err(|error| error.to_string())?;
        let unix_ticks = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() / 100;
        let expires = DateTime { UniversalTime: (unix_ticks + 116_444_736_000_000_000 + TOAST_LIFETIME.as_nanos() / 100) as i64 };
        let reference: IReference<DateTime> = PropertyValue::CreateDateTime(expires).and_then(|value| value.cast()).map_err(|error| error.to_string())?;
        toast.SetExpirationTime(&reference).map_err(|error| error.to_string())?;
        // Keep every handler token until explicit removal, failure or expiration.
        // A banner timeout is NOT removal: its Action Center entry is still clickable.
        let activation_shared = shared.clone();
        let activation_gate = gate.clone();
        let activation_token = token.to_owned();
        let activated = toast.Activated(&TypedEventHandler::<ToastNotification, IInspectable>::new(move |_, args| {
            if let Some(args) = args.as_ref().and_then(|value| value.cast::<ToastActivatedEventArgs>().ok()) {
                if let Ok(arguments) = args.Arguments() {
                    if let Some(event) = activation_event(&arguments.to_string(), &activation_token) {
                        consume(&activation_shared, &activation_token, &activation_gate, event);
                    }
                }
            }
            Ok(())
        })).map_err(|error| error.to_string())?;
        let dismissal_shared = shared.clone();
        let dismissal_gate = gate.clone();
        let dismissal_token = token.to_owned();
        let dismissed = match toast.Dismissed(&TypedEventHandler::<ToastNotification, ToastDismissedEventArgs>::new(move |_, args| {
            if let Some(args) = args.as_ref() {
                if args.Reason().ok() != Some(ToastDismissalReason::TimedOut) {
                    consume(&dismissal_shared, &dismissal_token, &dismissal_gate, ToastEvent::Dismissed { token: dismissal_token.clone() });
                }
            }
            Ok(())
        })) {
            Ok(value) => value,
            Err(error) => { let _ = toast.RemoveActivated(activated); return Err(error.to_string()); }
        };
        let failure_shared = shared.clone();
        let failure_gate = gate.clone();
        let failure_token = token.to_owned();
        let failed = match toast.Failed(&TypedEventHandler::<ToastNotification, ToastFailedEventArgs>::new(move |_, args| {
            let message = args.as_ref().and_then(|value| value.ErrorCode().ok()).map(|code| format!("系统通知发送失败（0x{:08X}）", code.0 as u32)).unwrap_or_else(|| "系统通知发送失败".into());
            consume(&failure_shared, &failure_token, &failure_gate, ToastEvent::Failed { token: failure_token.clone(), message });
            Ok(())
        })) {
            Ok(value) => value,
            Err(error) => { let _ = toast.RemoveActivated(activated); let _ = toast.RemoveDismissed(dismissed); return Err(error.to_string()); }
        };
        Ok(ActiveToast { toast, activated, dismissed, failed, gate, tag, expires: Instant::now() + TOAST_LIFETIME })
    }

    fn remove_group(app_id: &str) {
        if let Ok(history) = ToastNotificationManager::History() {
            let _ = history.RemoveGroupWithId(&HSTRING::from(TOAST_GROUP), &HSTRING::from(app_id));
        }
    }

    fn remove_active(app_id: &str, notifier: &ToastNotifier, shared: &Shared, active: &mut HashMap<String, ActiveToast>, token: &str) {
        if let Some(entry) = active.remove(token) {
            entry.gate.store(false, Ordering::Release);
            if let Ok(mut gates) = shared.gates.lock() {
                if gates.get(token).is_some_and(|gate| Arc::ptr_eq(gate, &entry.gate)) { gates.remove(token); }
            }
            let _ = notifier.Hide(&entry.toast);
            if let Ok(history) = ToastNotificationManager::History() {
                let _ = history.RemoveGroupedTagWithId(&HSTRING::from(&entry.tag), &HSTRING::from(TOAST_GROUP), &HSTRING::from(app_id));
            }
        }
    }

    fn run_worker(app_id: &str, notifier: ToastNotifier, shared: &Arc<Shared>, receiver: Receiver<Command>) {
        let mut active: HashMap<String, ActiveToast> = HashMap::new();
        let mut next_tag = 0_u64;
        loop {
            let command = match active.values().map(|entry| entry.expires).min() {
                Some(deadline) => match receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                    Ok(command) => Some(command),
                    Err(RecvTimeoutError::Timeout) => None,
                    Err(RecvTimeoutError::Disconnected) => break,
                },
                None => match receiver.recv() { Ok(command) => Some(command), Err(_) => break },
            };
            let expired: Vec<String> = active.iter().filter(|(_, entry)| entry.expires <= Instant::now()).map(|(token, _)| token.clone()).collect();
            for token in expired {
                remove_active(app_id, &notifier, shared, &mut active, &token);
                notify(shared, ToastEvent::Dismissed { token });
            }
            match command {
                Some(Command::Show { token, name, gate }) => {
                    if !shared.running.load(Ordering::Acquire) || !gate.load(Ordering::Acquire) { continue; }
                    if active.len() >= MAX_TOASTS {
                        if let Some(oldest) = active.iter().min_by_key(|(_, entry)| entry.expires).map(|(token, _)| token.clone()) {
                            remove_active(app_id, &notifier, shared, &mut active, &oldest);
                            notify(shared, ToastEvent::Dismissed { token: oldest });
                        }
                    }
                    next_tag = next_tag.wrapping_add(1);
                    let result = (|| {
                        check_notification_setting(app_id, notifier.Setting())?;
                        let entry = prepare_toast(shared, &token, &name, gate.clone(), format!("{next_tag:016x}"))?;
                        // Keep the handlers alive before calling Show; failure may be asynchronous.
                        active.insert(token.clone(), entry);
                        if !shared.running.load(Ordering::Acquire) || !gate.load(Ordering::Acquire) {
                            remove_active(app_id, &notifier, shared, &mut active, &token);
                            return Ok(());
                        }
                        notifier.Show(&active[&token].toast).map_err(|error| format!("无法发送系统通知：{error}"))?;
                        // This is native API submission, not proof that Windows
                        // displayed a banner (Do Not Disturb may suppress it).
                        if gate.load(Ordering::Acquire) {
                            notify(shared, ToastEvent::Submitted { token: token.clone() });
                        }
                        Ok(())
                    })();
                    if let Err(message) = result {
                        // Failed can already have run while Show was returning an error.
                        let should_report = gate.swap(false, Ordering::AcqRel);
                        remove_active(app_id, &notifier, shared, &mut active, &token);
                        if let Ok(mut gates) = shared.gates.lock() {
                            if gates.get(&token).is_some_and(|value| Arc::ptr_eq(value, &gate)) { gates.remove(&token); }
                        }
                        if should_report { notify(shared, ToastEvent::Failed { token, message }); }
                    }
                }
                Some(Command::Remove(token)) => remove_active(app_id, &notifier, shared, &mut active, &token),
                Some(Command::Retire { token, gate }) => {
                    if active.get(&token).is_some_and(|entry| Arc::ptr_eq(&entry.gate, &gate)) {
                        remove_active(app_id, &notifier, shared, &mut active, &token);
                    }
                }
                Some(Command::Clear) => {
                    let tokens: Vec<_> = active.keys().cloned().collect();
                    for token in tokens { remove_active(app_id, &notifier, shared, &mut active, &token); }
                    remove_group(app_id);
                }
                Some(Command::Shutdown) => break,
                None => {}
            }
        }
        let tokens: Vec<_> = active.keys().cloned().collect();
        for token in tokens { remove_active(app_id, &notifier, shared, &mut active, &token); }
        remove_group(app_id);
    }

    #[cfg(test)]
    mod event_tests {
        use super::*;

        fn state() -> (Arc<Shared>, Receiver<Command>, Arc<Mutex<Vec<ToastEvent>>>) {
            let (sender, receiver) = mpsc::channel();
            let events = Arc::new(Mutex::new(Vec::new()));
            let event_sink = events.clone();
            let shared = Arc::new(Shared {
                running: AtomicBool::new(true),
                gates: Mutex::new(HashMap::new()),
                sender,
                callback: Arc::new(move |event| event_sink.lock().unwrap().push(event)),
            });
            (shared, receiver, events)
        }

        #[test]
        fn native_toast_preparation_sets_expiration_and_registers_handlers_without_showing() {
            let _apartment = Apartment::new().unwrap();
            let (shared, _receiver, _events) = state();
            let entry = prepare_toast(
                &shared,
                "native_preparation_test",
                "测试设备",
                Arc::new(AtomicBool::new(true)),
                "native-test".into(),
            ).expect("the entire native preparation chain must succeed before Show can run");
            assert_eq!(entry.toast.Tag().unwrap().to_string(), "native-test");
            assert_eq!(entry.toast.Group().unwrap().to_string(), TOAST_GROUP);
            assert!(entry.toast.ExpirationTime().unwrap().Value().unwrap().UniversalTime > 0);
        }

        #[test]
        fn unknown_first_notification_record_does_not_override_real_disabled_settings() {
            use windows::core::{Error, HRESULT};
            assert!(check_notification_setting("app", Ok(NotificationSetting::Enabled)).is_ok());
            assert!(check_notification_setting("app", Err(Error::from_hresult(HRESULT(0x8007_0490_u32 as i32)))).is_ok());
            for setting in [NotificationSetting::DisabledForApplication, NotificationSetting::DisabledForUser, NotificationSetting::DisabledByGroupPolicy, NotificationSetting::DisabledByManifest] {
                assert!(check_notification_setting("app", Ok(setting)).is_err(), "{setting:?}");
            }
            assert!(check_notification_setting("app", Err(Error::from_hresult(HRESULT(0x8007_0005_u32 as i32)))).is_err());
        }

        /// Opt-in desktop diagnostic. Does not register an identity or launch a
        /// device. Showing is separately gated, and only its own toast is hidden.
        #[test]
        #[ignore = "manual native notification probe; set QZZ_TOAST_DIAGNOSTIC_SHOW=1 to display one toast"]
        fn native_notification_delivery_diagnostic() {
            let _apartment = Apartment::new().unwrap();
            let app_id = std::env::var("QZZ_TOAST_DIAGNOSTIC_APPID").unwrap_or_else(|_| "com.qzz.mirror".into());
            let show = std::env::var("QZZ_TOAST_DIAGNOSTIC_SHOW").as_deref() == Ok("1");
            struct DiagnosticRegistration { subkey: Vec<u16>, settings_key: Vec<u16> }
            impl Drop for DiagnosticRegistration {
                fn drop(&mut self) {
                    let result = unsafe { windows::Win32::System::Registry::RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR(self.subkey.as_ptr())) };
                    println!("diagnostic_identity_cleanup={result:?}");
                    let result = unsafe { windows::Win32::System::Registry::RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR(self.settings_key.as_ptr())) };
                    println!("diagnostic_settings_cleanup={result:?}");
                }
            }
            let _registration = if std::env::var("QZZ_TOAST_DIAGNOSTIC_REGISTER").as_deref() == Ok("1") {
                assert!(app_id.starts_with("com.qzz.mirror.toast-diagnostic-"), "only a unique diagnostic identity may be registered by this probe");
                let settings_key = wide(&format!("Software\\Microsoft\\Windows\\CurrentVersion\\Notifications\\Settings\\{app_id}"));
                let mut existing = HKEY::default();
                let opened = unsafe { windows::Win32::System::Registry::RegOpenKeyExW(HKEY_CURRENT_USER, PCWSTR(settings_key.as_ptr()), None, KEY_QUERY_VALUE, &mut existing) };
                if opened.is_ok() { let _ = unsafe { RegCloseKey(existing) }; }
                assert_eq!(opened, ERROR_FILE_NOT_FOUND, "refuse to alter existing notification settings for a diagnostic identity");
                let (key, created) = IdentityKey::open(&app_id).unwrap();
                assert!(created, "refuse to alter any existing notification identity");
                let registration = DiagnosticRegistration { subkey: wide(&format!("Software\\Classes\\AppUserModelId\\{app_id}")), settings_key };
                key.set_string("DisplayName", "轻镜通知诊断").unwrap();
                let icon = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../src-tauri/icons/icon.png").canonicalize().unwrap();
                key.set_string("IconUri", &icon.to_string_lossy()).unwrap();
                Some(registration)
            } else { None };
            if std::env::var("QZZ_TOAST_DIAGNOSTIC_PROCESS_IDENTITY").as_deref() == Ok("1") {
                let wide_id: Vec<u16> = app_id.encode_utf16().chain(Some(0)).collect();
                let result = unsafe { windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(windows::core::PCWSTR(wide_id.as_ptr())) };
                println!("SetCurrentProcessExplicitAppUserModelID={result:?}");
            }
            let seconds = std::env::var("QZZ_TOAST_DIAGNOSTIC_SECONDS").ok().and_then(|value| value.parse::<u64>().ok()).unwrap_or(15).min(120);
            let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(&app_id)).expect("CreateToastNotifierWithId");
            println!("identity={app_id:?}; setting={:?}; show={show}", notifier.Setting());
            let history = ToastNotificationManager::History().expect("ToastNotificationManager::History");
            println!("history_before={:?}", history.GetHistoryWithId(&HSTRING::from(&app_id)).and_then(|items| items.Size()));
            let (shared, _receiver, events) = state();
            let tag = format!("d{:015x}", std::process::id());
            let entry = prepare_toast(&shared, "delivery_diagnostic", "轻镜通知诊断（不会启动投屏）", Arc::new(AtomicBool::new(true)), tag.clone()).expect("prepare_toast");
            println!("prepare_toast=OK; tag={tag}");
            if show {
                let shown = notifier.Show(&entry.toast);
                println!("Show={shown:?}");
                for _ in 0..seconds {
                    std::thread::sleep(Duration::from_secs(1));
                    let recorded = events.lock().unwrap().clone();
                    if !recorded.is_empty() { println!("callback={recorded:?}"); break; }
                }
                println!("history_after={:?}", history.GetHistoryWithId(&HSTRING::from(&app_id)).and_then(|items| items.Size()));
                println!("setting_after={:?}", notifier.Setting());
                println!("events={:?}", events.lock().unwrap());
                let _ = notifier.Hide(&entry.toast);
                let _ = history.RemoveGroupedTagWithId(&HSTRING::from(tag), &HSTRING::from(TOAST_GROUP), &HSTRING::from(app_id));
                shown.expect("Show must not fail");
                assert!(!events.lock().unwrap().iter().any(|event| matches!(event, ToastEvent::Failed { .. })), "native toast Failed callback");
            }
        }

        #[test]
        fn a_notification_activation_is_consumed_once_and_queues_retirement() {
            let (shared, receiver, events) = state();
            let gate = Arc::new(AtomicBool::new(true));
            let event = ToastEvent::StartMirroring { token: "nonce".into() };
            consume(&shared, "nonce", &gate, event.clone());
            consume(&shared, "nonce", &gate, event.clone());
            assert_eq!(*events.lock().unwrap(), vec![event]);
            match receiver.try_recv().unwrap() {
                Command::Retire { token, gate: retired_gate } => {
                    assert_eq!(token, "nonce");
                    assert!(Arc::ptr_eq(&gate, &retired_gate));
                }
                _ => panic!("activation must only queue retirement"),
            }
            assert!(receiver.try_recv().is_err());
        }

        #[test]
        fn removed_notifications_and_a_stopped_service_ignore_late_events() {
            let (shared, receiver, events) = state();
            let gate = Arc::new(AtomicBool::new(false));
            let event = ToastEvent::StartMirroring { token: "nonce".into() };
            consume(&shared, "nonce", &gate, event.clone());
            shared.running.store(false, Ordering::Release);
            gate.store(true, Ordering::Release);
            consume(&shared, "nonce", &gate, event);
            assert!(events.lock().unwrap().is_empty());
            assert!(receiver.try_recv().is_err());
        }
    }
}

#[cfg(windows)]
pub use platform::ToastService;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_identifiers_that_could_escape_or_target_another_device() {
        for value in ["", "usb:5555", "abc:def", "x y", "\"/><action", "设备", "../x", &"a".repeat(129)] {
            assert!(!valid_token(value), "{value:?}");
            assert!(toast_xml(value, "device").is_err());
        }
        assert!(valid_token("qzz_token-123"));
    }

    #[test]
    fn activation_requires_exact_current_opaque_token_and_known_action() {
        assert_eq!(activation_event("start:opaque_1", "opaque_1"), Some(ToastEvent::StartMirroring { token: "opaque_1".into() }));
        assert_eq!(activation_event("open:opaque_1", "opaque_1"), Some(ToastEvent::OpenPanel { token: "opaque_1".into() }));
        for value in ["", "start:other", "start:opaque_1:extra", "dismiss:opaque_1", "START:opaque_1", "start:opaque_1 ", "open:opaque_1&serial=device"] {
            assert_eq!(activation_event(value, "opaque_1"), None, "{value:?}");
        }
    }

    #[test]
    fn xml_escapes_device_text_and_keeps_actions_separate() {
        let xml = toast_xml("nonce_42", "A<&\"'\u{1}>设备").unwrap();
        assert!(xml.contains("A&lt;&amp;&quot;&apos;&gt;设备，是否开始投屏？"));
        assert!(!xml.contains('\u{1}'));
        assert!(xml.contains("launch=\"open:nonce_42\""));
        assert!(xml.contains("arguments=\"start:nonce_42\""));
        assert!(xml.contains("arguments=\"dismiss\" activationType=\"system\""));
        assert!(!xml.contains("protocol"));
        assert!(!xml.contains("serial"));
    }

    #[test]
    fn device_text_is_bounded_and_has_fallback() {
        assert_eq!(xml_text("\u{1} \u{2}"), "Android 设备");
        assert_eq!(xml_text(&"界".repeat(1000)).chars().count(), 160);
    }

    #[cfg(windows)]
    #[test]
    fn windows_xml_parser_accepts_toast_generic_payload_without_showing_it() {
        use windows::{core::HSTRING, Data::Xml::Dom::XmlDocument, Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED}};
        unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.unwrap();
        let result = (|| -> windows::core::Result<()> {
            let document = XmlDocument::new()?;
            document.LoadXml(&HSTRING::from(toast_xml("nonce_42", "设备<&\"'\u{1}>").unwrap()))?;
            let actions = document.GetElementsByTagName(&HSTRING::from("action"))?;
            assert_eq!(actions.Length()?, 2);
            let text = document.GetElementsByTagName(&HSTRING::from("text"))?;
            assert_eq!(text.Length()?, 2);
            Ok(())
        })();
        unsafe { RoUninitialize() };
        result.unwrap();
    }
}
