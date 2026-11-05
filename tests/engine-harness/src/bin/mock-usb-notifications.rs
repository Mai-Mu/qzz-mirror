//! Runs only copied mock executables in an isolated per-process directory.
//! It never loads the installed app's runtime, windows, USB device, or ADB server.
use qzz_engine_tests::{engine::Engine, models::{TransportKind, TransportPreference}, usb_notifications::{UsbNotification, UsbNotificationEvent}};
use std::{fs, path::{Path, PathBuf}, time::Duration};
use tokio::sync::broadcast::Receiver;

fn devices(state: &Path, usb: Option<(&str, &str)>, wifi: bool) {
    let mut list = "List of devices attached\n".to_string();
    if let Some((status, transport)) = usb {
        list.push_str(&format!("USB-123 {status} usb:1-2 model:Test_Android transport_id:{transport}\n"));
    }
    if wifi { list.push_str("192.168.1.22:5555 device model:Test_Android transport_id:200\n"); }
    fs::write(state.join("devices-list.txt"), list).unwrap();
}
fn events(receiver: &mut Receiver<UsbNotificationEvent>) -> Vec<UsbNotificationEvent> {
    let mut result = Vec::new();
    while let Ok(event) = receiver.try_recv() { result.push(event); }
    result
}
fn notices(receiver: &mut Receiver<UsbNotificationEvent>) -> Vec<UsbNotification> {
    events(receiver).into_iter().filter_map(|event| match event { UsbNotificationEvent::Show(notice) => Some(notice), _ => None }).collect()
}
async fn attach(engine: &Engine, state: &Path, receiver: &mut Receiver<UsbNotificationEvent>, generation: &str) -> UsbNotification {
    devices(state, None, false); engine.refresh().await.unwrap(); events(receiver);
    devices(state, Some(("device", generation)), false); engine.refresh().await.unwrap();
    let mut found = notices(receiver); assert_eq!(found.len(), 1); found.pop().unwrap()
}
fn starts(state: &Path) -> usize {
    fs::read_to_string(state.join("scrcpy-starts.log")).unwrap_or_default().lines().count()
}

#[tokio::main]
async fn main() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("target/usb-notification-test-{}", std::process::id()));
    let runtime = base.join("runtime"); let state = base.join("state");
    fs::create_dir_all(&runtime).unwrap(); fs::create_dir_all(&state).unwrap();
    let binary = std::env::current_exe().unwrap().parent().unwrap().join("mock-runtime.exe");
    for executable in ["adb.exe", "scrcpy.exe"] { fs::copy(&binary, runtime.join(executable)).unwrap(); }
    fs::write(runtime.join("scrcpy-server"), "").unwrap();
    // Mock scrcpy's liveness marker; devices-list controls discovery separately.
    fs::write(state.join("usb-online"), "").unwrap();
    std::env::set_var("QZZ_MOCK_STATE", &state);
    let engine = Engine::new(runtime, base.join("config")).unwrap();
    let mut receiver = engine.subscribe_usb_notifications();
    engine.set_notification_background(true);
    devices(&state, Some(("device", "1")), false); engine.refresh().await.unwrap();
    assert!(notices(&mut receiver).is_empty());
    println!("PASS: initially attached USB devices establish a silent startup baseline");

    let first = attach(&engine, &state, &mut receiver, "2").await;
    assert_eq!(first.usb_serial, "USB-123"); assert_eq!(first.device_id, "android:HARDWARE-123");
    assert_eq!(first.token.len(), 32); assert!(first.token.chars().all(|character| character.is_ascii_hexdigit()));
    assert!(engine.usb_notification_is_current(&first.token).await);
    engine.refresh().await.unwrap(); assert!(notices(&mut receiver).is_empty());
    devices(&state, Some(("device", "2")), true); engine.refresh().await.unwrap();
    assert_eq!(engine.snapshot().await.devices.len(), 1); assert!(notices(&mut receiver).is_empty());
    println!("PASS: one authorized USB attachment produces one opaque action; repeat refresh and Wi-Fi merge never duplicate it");

    engine.set_notification_background(false);
    devices(&state, None, false); engine.refresh().await.unwrap(); events(&mut receiver);
    devices(&state, Some(("device", "3")), false); engine.refresh().await.unwrap();
    assert!(notices(&mut receiver).is_empty());
    engine.set_notification_background(true); engine.refresh().await.unwrap();
    assert!(notices(&mut receiver).is_empty());
    println!("PASS: foreground discovery is consumed and never replayed by subsequently minimizing");

    for (serial, initially_background) in [("USB-NEW-A", false), ("USB-NEW-B", true)] {
        devices(&state, None, false); engine.refresh().await.unwrap(); events(&mut receiver);
        let _ = fs::remove_file(state.join("metadata-waiting"));
        fs::write(state.join("delay-metadata"), "").unwrap();
        engine.set_notification_background(initially_background);
        fs::write(state.join("devices-list.txt"), format!("List of devices attached\n{serial} device usb:1-2 model:Test_Android transport_id:99\n")).unwrap();
        let refresher = engine.clone(); let refreshing = tokio::spawn(async move { refresher.refresh().await });
        for _ in 0..100 {
            if state.join("metadata-waiting").exists() { break; }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(state.join("metadata-waiting").exists());
        engine.set_notification_background(false); engine.set_notification_background(true);
        fs::remove_file(state.join("delay-metadata")).unwrap(); refreshing.await.unwrap().unwrap();
        assert!(notices(&mut receiver).is_empty());
        engine.refresh().await.unwrap(); assert!(notices(&mut receiver).is_empty());
    }
    println!("PASS: entering foreground during a slow refresh suppresses that discovery, including a later return to the tray");

    devices(&state, Some(("unauthorized", "4")), false); engine.refresh().await.unwrap();
    assert!(notices(&mut receiver).is_empty());
    devices(&state, Some(("device", "4")), false); engine.refresh().await.unwrap();
    let authorized = notices(&mut receiver).pop().unwrap();
    devices(&state, Some(("offline", "4")), false); engine.refresh().await.unwrap();
    assert!(!engine.usb_notification_is_current(&authorized.token).await);
    assert!(events(&mut receiver).contains(&UsbNotificationEvent::Remove { token: authorized.token.clone() }));
    assert!(engine.start_from_usb_notification(authorized.token).await.is_err()); assert_eq!(starts(&state), 0);
    println!("PASS: authorization enables a reminder; offline removes the toast and rejects its action without spawning");

    let old = attach(&engine, &state, &mut receiver, "5").await;
    devices(&state, Some(("device", "6")), false);
    assert!(engine.start_from_usb_notification(old.token).await.is_err()); assert_eq!(starts(&state), 0);
    assert!(notices(&mut receiver).is_empty());
    println!("PASS: click-time refresh rejects a same-serial replacement transport, even when no unplug sample was observed");

    let race = attach(&engine, &state, &mut receiver, "7").await;
    fs::write(state.join("replace-devices-after-metadata.txt"), "List of devices attached\nUSB-123 device usb:1-2 model:Test_Android transport_id:8\n").unwrap();
    assert!(engine.start_from_usb_notification(race.token).await.is_err()); assert_eq!(starts(&state), 0);
    println!("PASS: a transport replaced during final fresh metadata is rejected by the adjacent pre-spawn check");

    let unavailable = attach(&engine, &state, &mut receiver, "9").await;
    fs::write(state.join("metadata-error"), "").unwrap();
    assert!(engine.start_from_usb_notification(unavailable.token.clone()).await.is_err());
    fs::remove_file(state.join("metadata-error")).unwrap();
    assert!(!engine.usb_notification_is_current(&unavailable.token).await); assert_eq!(starts(&state), 0);
    println!("PASS: notification launch requires fresh USB identity and never falls back to cached metadata");

    let failed_query = attach(&engine, &state, &mut receiver, "10").await;
    fs::write(state.join("devices-error"), "").unwrap();
    assert!(engine.start_from_usb_notification(failed_query.token.clone()).await.is_err());
    fs::remove_file(state.join("devices-error")).unwrap();
    assert!(!engine.usb_notification_is_current(&failed_query.token).await); assert_eq!(starts(&state), 0);
    println!("PASS: failed click-time ADB query consumes its one-shot action and never guesses the current device");

    let dismissed = attach(&engine, &state, &mut receiver, "11").await;
    engine.dismiss_usb_notification(&dismissed.token);
    assert!(!engine.usb_notification_is_current(&dismissed.token).await);
    assert!(engine.start_from_usb_notification("forged-token".into()).await.is_err()); assert_eq!(starts(&state), 0);
    let before = engine.snapshot().await;
    engine.report_notification_error("模拟系统通知被关闭".into()).await;
    let after = engine.snapshot().await;
    assert_eq!(after.devices.len(), before.devices.len()); assert_eq!(after.last_error, before.last_error);
    assert!(after.logs.last().unwrap().message.contains("模拟系统通知被关闭"));
    println!("PASS: dismissal and forged tokens cannot start; notification errors only append a warning");

    let count_metadata = || fs::read_to_string(state.join("adb-calls.log")).unwrap().lines().filter(|line| line.ends_with("shell getprop")).count();
    let metadata_before = count_metadata();
    for _ in 0..5 { engine.refresh().await.unwrap(); }
    assert_eq!(count_metadata(), metadata_before);
    println!("PASS: notification policy adds no metadata queries to repeated cached discovery refreshes");

    let launch = attach(&engine, &state, &mut receiver, "12").await;
    devices(&state, Some(("device", "12")), true);
    let mut settings = engine.snapshot().await.settings; settings.preferred_transport = TransportPreference::Wifi;
    engine.save_settings(settings).await.unwrap();
    let starter = engine.clone(); let other = engine.clone(); let second_token = launch.token.clone();
    let (first_result, second_result) = tokio::join!(starter.start_from_usb_notification(launch.token), other.start_from_usb_notification(second_token));
    assert_eq!(usize::from(first_result.is_ok()) + usize::from(second_result.is_ok()), 1);
    assert_eq!(starts(&state), 1);
    let snapshot = engine.snapshot().await; let session = snapshot.session.unwrap();
    assert_eq!(session.transport, TransportKind::Usb); assert_eq!(session.serial, "USB-123");
    assert_eq!(snapshot.settings.preferred_transport, TransportPreference::Wifi);
    let args: Vec<String> = serde_json::from_slice(&fs::read(state.join("scrcpy-args.json")).unwrap()).unwrap();
    assert!(args.windows(2).any(|pair| pair == ["--serial", "USB-123"]));
    assert!(events(&mut receiver).contains(&UsbNotificationEvent::Clear));
    println!("PASS: concurrent double activation spawns exactly one USB session, preserves Wi-Fi preference, and clears old notices");

    devices(&state, Some(("device", "13")), false); engine.refresh().await.unwrap();
    assert!(notices(&mut receiver).is_empty());
    assert!(engine.start_from_usb_notification("another-token".into()).await.is_err()); assert_eq!(starts(&state), 1);
    assert_eq!(engine.snapshot().await.session.unwrap().serial, "USB-123");
    engine.stop_mirror().await.unwrap(); engine.refresh().await.unwrap();
    assert!(notices(&mut receiver).is_empty());
    println!("PASS: an active mirror suppresses new attachments, never switches session, and stopping never replays suppressed devices");

    let pending = attach(&engine, &state, &mut receiver, "14").await;
    engine.start_mirror(pending.device_id.clone(), TransportPreference::Usb).await.unwrap();
    assert!(!engine.usb_notification_is_current(&pending.token).await);
    assert!(engine.start_from_usb_notification(pending.token).await.is_err()); assert_eq!(starts(&state), 2);
    engine.stop_mirror().await.unwrap();
    println!("PASS: a normal control-panel launch also revokes older toast capabilities and keeps its current session");

    devices(&state, None, true); engine.refresh().await.unwrap(); events(&mut receiver);
    fs::write(state.join("devices-list.txt"), "List of devices attached\nUSB-123 device usb:1-2 model:Test_Android\n").unwrap();
    engine.refresh().await.unwrap(); assert!(notices(&mut receiver).is_empty());
    assert!(engine.snapshot().await.devices.iter().any(|device| device.transports.iter().any(|transport| transport.kind == TransportKind::Usb)));
    println!("PASS: Wi-Fi-only arrivals and USB without an attachment ID remain discoverable without issuing actionable reminders");

    let pending = attach(&engine, &state, &mut receiver, "16").await;
    fs::write(state.join("delay-start"), "").unwrap();
    fs::write(state.join("delay-metadata"), "").unwrap(); let _ = fs::remove_file(state.join("metadata-waiting"));
    let starter = engine.clone(); let starting_id = pending.device_id;
    let starting = tokio::spawn(async move { starter.start_mirror(starting_id, TransportPreference::Usb).await });
    for _ in 0..100 {
        if state.join("metadata-waiting").exists() { break; }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(state.join("metadata-waiting").exists()); assert!(engine.snapshot().await.session.is_none());
    assert!(!engine.usb_notification_is_current(&pending.token).await);
    fs::remove_file(state.join("delay-metadata")).unwrap();
    for _ in 0..100 {
        if engine.snapshot().await.session.is_some() { break; }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(engine.snapshot().await.session.unwrap().phase, qzz_engine_tests::models::SessionPhase::Starting);
    assert!(!engine.usb_notification_is_current(&pending.token).await);
    assert!(engine.start_from_usb_notification(pending.token).await.is_err());
    assert!(engine.snapshot().await.last_error.as_deref().is_some_and(|error| error.contains("上一项操作")));
    fs::remove_file(state.join("delay-start")).unwrap(); starting.await.unwrap().unwrap();
    assert_eq!(starts(&state), 3); engine.stop_mirror().await.unwrap();
    println!("PASS: pending metadata startup suppresses queued toasts; unready Starting sessions revoke actions before the first frame");

    let pending = attach(&engine, &state, &mut receiver, "15").await;
    engine.clear_usb_notifications(); assert!(!engine.usb_notification_is_current(&pending.token).await);
    engine.shutdown().await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("PASS: explicit notification cleanup revokes capabilities; shutdown leaves no mock child running");
}
