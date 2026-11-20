//! Exercises the production Engine against copied fake executables only.
use qzz_engine_tests::{engine::Engine, models::{SessionPhase, TransportKind, TransportPreference}};
use std::{fs, path::{Path, PathBuf}, time::Duration};

fn starts(state: &Path) -> usize {
    fs::read_to_string(state.join("scrcpy-starts.log")).unwrap_or_default().lines().count()
}
fn calls(state: &Path) -> String { fs::read_to_string(state.join("adb-calls.log")).unwrap_or_default() }
fn clear_calls(state: &Path) { fs::write(state.join("adb-calls.log"), "").unwrap(); }
async fn marker(state: &Path, name: &str) {
    for _ in 0..160 {
        if state.join(name).exists() { return; }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("missing mock marker {name}");
}
async fn wait_session(engine: &Engine, transport: Option<TransportKind>) {
    for _ in 0..160 {
        let snapshot = engine.snapshot().await;
        if match transport { Some(kind) => snapshot.sessions.first().is_some_and(|session|
            session.transport == kind && session.phase == SessionPhase::Running), None => snapshot.sessions.is_empty() } { return; }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("unexpected session: {}", serde_json::to_string(&engine.snapshot().await).unwrap());
}
async fn wait_disabled(engine: &Engine) {
    for _ in 0..100 {
        if !engine.snapshot().await.settings.wifi_enabled { return; }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("Wi-Fi disabling did not reach the immediate gate");
}
#[tokio::main]
async fn main() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("target/wifi-toggle-test-{}", std::process::id()));
    let runtime = base.join("runtime"); let state = base.join("state"); let config = base.join("config");
    for directory in [&runtime, &state, &config] { fs::create_dir_all(directory).unwrap(); }
    let binary = std::env::current_exe().unwrap().parent().unwrap().join("mock-runtime.exe");
    for executable in ["adb.exe", "scrcpy.exe"] { fs::copy(&binary, runtime.join(executable)).unwrap(); }
    fs::write(runtime.join("scrcpy-server"), "").unwrap();
    fs::write(config.join("settings.json"), br#"{"settings":{"audio":false,"preferredTransport":"wifi"},"savedDevices":[{"id":"android:HARDWARE-123","name":"Remembered Android","address":"192.168.1.22:5555"},{"id":"android:OTHER","name":"Saved only","address":"192.168.1.23:5555"}]}"#).unwrap();
    std::env::set_var("QZZ_MOCK_STATE", &state);
    for marker in ["usb-online", "wifi-online"] { fs::write(state.join(marker), "").unwrap(); }
    let engine = Engine::new(runtime.clone(), config.clone()).unwrap(); engine.spawn_background();
    let initial = engine.refresh().await.unwrap();
    assert!(initial.settings.wifi_enabled); assert!(!initial.settings.audio);
    assert_eq!(initial.settings.preferred_transport, TransportPreference::Wifi);
    let id = initial.devices[0].id.clone(); assert_eq!(initial.devices[0].transports.len(), 2);
    println!("PASS: old settings default Wi-Fi on and preserve user choices and remembered endpoints");

    let disabled = engine.set_wifi_enabled(false).await.unwrap();
    assert!(!disabled.settings.wifi_enabled); assert!(disabled.saved_devices.is_empty());
    assert_eq!(disabled.devices.len(), 1); assert_eq!(disabled.devices[0].transports.len(), 1);
    assert_eq!(disabled.devices[0].transports[0].kind, TransportKind::Usb);
    assert!(disabled.devices[0].wifi_address.is_none());
    let persisted: serde_json::Value = serde_json::from_slice(&fs::read(config.join("settings.json")).unwrap()).unwrap();
    assert_eq!(persisted["settings"]["wifiEnabled"], false); assert_eq!(persisted["savedDevices"].as_array().unwrap().len(), 2);
    clear_calls(&state); engine.refresh().await.unwrap();
    assert!(!calls(&state).contains("-s 192.168."));
    let loaded = Engine::new(runtime.clone(), config.clone()).unwrap();
    assert!(!loaded.snapshot().await.settings.wifi_enabled); loaded.shutdown().await;
    assert!(engine.select_device(Some("android:OTHER".into())).await.is_err());
    println!("PASS: off persists, exposes USB only, skips Wi-Fi metadata, hides saved-only selection and keeps saved addresses on disk");

    clear_calls(&state);
    for result in [
        engine.connect_wireless("192.168.1.22:5555".into()).await,
        engine.configure_wireless(id.clone()).await,
        engine.pair_wireless("192.168.1.22:33333".into(), "123456".into(), None).await,
        engine.start_mirror(id.clone(), TransportPreference::Wifi).await,
    ] { assert!(result.unwrap_err().contains("Wi-Fi 已关闭")); }
    assert!(!calls(&state).lines().any(|line| line.starts_with("connect ") || line.starts_with("pair ") || line.contains("tcpip") || line.contains("192.168.")));
    println!("PASS: all public wireless entry points reject disabled mode before wireless ADB or process side effects");

    engine.start_mirror(id.clone(), TransportPreference::Auto).await.unwrap();
    let usb = engine.toolbar_sessions().await.into_iter().next().unwrap();
    assert_eq!(engine.snapshot().await.sessions.first().unwrap().transport, TransportKind::Usb);
    assert!(!calls(&state).contains("-s 192.168."));
    engine.set_wifi_enabled(true).await.unwrap(); engine.set_wifi_enabled(false).await.unwrap();
    let kept = engine.toolbar_sessions().await.into_iter().next().unwrap(); assert_eq!(usb.token, kept.token); assert_eq!(usb.process_id, kept.process_id); assert!(kept.running);
    let before = starts(&state); clear_calls(&state);
    fs::remove_file(state.join("usb-online")).unwrap(); wait_session(&engine, None).await;
    tokio::time::sleep(Duration::from_millis(800)).await;
    assert_eq!(starts(&state), before); assert!(!calls(&state).lines().any(|line| line.starts_with("connect ")));
    println!("PASS: Auto uses USB while off; toggling preserves the running USB child and disables USB-loss wireless recovery");

    let enabled = engine.set_wifi_enabled(true).await.unwrap(); assert_eq!(enabled.saved_devices.len(), 2);
    assert!(enabled.devices.iter().any(|device| device.transports.iter().any(|transport| transport.kind == TransportKind::Wifi)));
    engine.start_mirror(id.clone(), TransportPreference::Wifi).await.unwrap();
    let wifi = engine.toolbar_sessions().await.into_iter().next().unwrap();
    let mut settings = engine.snapshot().await.settings; settings.wifi_enabled = false; settings.max_fps = Some(60);
    let disabled = engine.save_settings(settings).await.unwrap();
    assert!(!disabled.settings.wifi_enabled); assert_eq!(disabled.settings.max_fps, Some(60)); assert!(disabled.sessions.is_empty());
    assert!(engine.toolbar_sessions().await.is_empty());
    assert!(engine.navigate(wifi.token, qzz_engine_tests::models::NavigationAction::Back).await.is_err());
    assert!(!calls(&state).lines().any(|line| line.starts_with("disconnect") || line.contains("kill-server")));
    println!("PASS: re-enabling restores remembered endpoints; save_settings(false) stops only the owned Wi-Fi child without global ADB changes");

    engine.set_wifi_enabled(true).await.unwrap();
    fs::write(state.join("delay-metadata"), "").unwrap(); let _ = fs::remove_file(state.join("metadata-waiting"));
    let before = starts(&state); let starter = engine.clone(); let device = id.clone();
    let starting = tokio::spawn(async move { starter.start_mirror(device, TransportPreference::Wifi).await });
    marker(&state, "metadata-waiting").await;
    let switcher = engine.clone(); let switching = tokio::spawn(async move { switcher.set_wifi_enabled(false).await });
    wait_disabled(&engine).await;
    assert!(engine.snapshot().await.devices.is_empty());
    fs::remove_file(state.join("delay-metadata")).unwrap();
    assert!(starting.await.unwrap().is_err()); switching.await.unwrap().unwrap();
    assert_eq!(starts(&state), before); assert!(engine.snapshot().await.sessions.is_empty());
    println!("PASS: disabling during final Wi-Fi identity verification cancels startup before a child can spawn");

    engine.set_wifi_enabled(true).await.unwrap(); fs::write(state.join("delay-start"), "").unwrap();
    let starter = engine.clone(); let device = id.clone();
    let starting = tokio::spawn(async move { starter.start_mirror(device, TransportPreference::Wifi).await });
    for _ in 0..160 { if !engine.toolbar_sessions().await.is_empty() { break; } tokio::time::sleep(Duration::from_millis(25)).await; }
    assert!(!engine.toolbar_sessions().await.into_iter().next().unwrap().running);
    tokio::time::timeout(Duration::from_secs(3), engine.set_wifi_enabled(false)).await.unwrap().unwrap();
    let _ = starting.await.unwrap(); fs::remove_file(state.join("delay-start")).unwrap();
    assert!(engine.snapshot().await.sessions.is_empty()); assert!(engine.toolbar_sessions().await.is_empty());
    println!("PASS: disabling remains available while Wi-Fi waits for first video and terminates the registered child promptly");

    fs::write(state.join("usb-online"), "").unwrap(); engine.set_wifi_enabled(true).await.unwrap();
    engine.start_mirror(id.clone(), TransportPreference::Usb).await.unwrap(); wait_session(&engine, Some(TransportKind::Usb)).await;
    let before = starts(&state); let _ = fs::remove_file(state.join("metadata-waiting"));
    fs::write(state.join("delay-metadata"), "").unwrap(); fs::remove_file(state.join("usb-online")).unwrap();
    marker(&state, "metadata-waiting").await;
    let switcher = engine.clone(); let switching = tokio::spawn(async move { switcher.set_wifi_enabled(false).await });
    wait_disabled(&engine).await; fs::remove_file(state.join("delay-metadata")).unwrap(); switching.await.unwrap().unwrap();
    assert_eq!(starts(&state), before); assert!(engine.snapshot().await.sessions.is_empty());
    assert!(!engine.snapshot().await.logs.iter().any(|entry| entry.message.contains("无线恢复失败")));
    println!("PASS: disabling during USB-loss recovery invalidates the pending recovery without spawning Wi-Fi or reporting a false recovery failure");

    fs::write(state.join("usb-online"), "").unwrap(); engine.set_wifi_enabled(true).await.unwrap();
    let mut settings = engine.snapshot().await.settings; settings.last_device_id = None; settings.preferred_transport = TransportPreference::Wifi;
    engine.save_settings(settings).await.unwrap();
    fs::write(state.join("devices-list.txt"), "List of devices attached\nUSB-NEW device usb:1-2 model:Test_Android transport_id:99\n192.168.1.22:5555 device model:Test_Android transport_id:200\n").unwrap();
    let _ = fs::remove_file(state.join("metadata-waiting")); fs::write(state.join("delay-metadata"), "").unwrap();
    let starter = engine.clone(); let device = id.clone();
    let starting = tokio::spawn(async move { starter.start_mirror(device, TransportPreference::Usb).await });
    marker(&state, "metadata-waiting").await;
    let switcher = engine.clone(); let switching = tokio::spawn(async move { switcher.set_wifi_enabled(false).await });
    wait_disabled(&engine).await; fs::remove_file(state.join("delay-metadata")).unwrap();
    starting.await.unwrap().unwrap(); switching.await.unwrap().unwrap();
    let snapshot = engine.snapshot().await;
    assert_eq!(snapshot.settings.preferred_transport, TransportPreference::Usb);
    assert_eq!(snapshot.settings.last_device_id.as_deref(), Some(id.as_str()));
    assert_eq!(snapshot.sessions.first().unwrap().transport, TransportKind::Usb);
    println!("PASS: a Wi-Fi toggle queued during USB discovery preserves the eventual USB session and its freshly committed device preferences");
    engine.shutdown().await;
}
