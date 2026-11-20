use qzz_engine_tests::{
    engine::{Engine, ToolbarSession},
    models::{LogLevel, NavigationAction, Session, SessionPhase, TransportKind, TransportPreference},
    scrcpy_input::NavigationDispatcher,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

const TABLET_A: &str = "android:HARDWARE-123";
const TABLET_B: &str = "android:HARDWARE-456";

#[derive(Default)]
struct RecordingDispatcher(Mutex<Vec<(u32, NavigationAction)>>);

impl NavigationDispatcher for RecordingDispatcher {
    fn dispatch(&self, pid: u32, action: NavigationAction) -> Result<(), String> {
        self.0.lock().unwrap().push((pid, action));
        Ok(())
    }
}

async fn wait_for(engine: &Engine, predicate: impl Fn(&[Session]) -> bool) {
    wait_until(async || predicate(&engine.snapshot().await.sessions)).await;
}

async fn wait_until(predicate: impl AsyncFn() -> bool) {
    tokio::time::timeout(Duration::from_secs(8), async {
        while !predicate().await {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("expected mock state did not appear");
}

async fn other_toolbar(engine: &Engine, previous: &ToolbarSession) -> ToolbarSession {
    engine.toolbar_sessions().await
        .into_iter()
        .find(|session| session.token != previous.token)
        .expect("the other tablet has its own toolbar session")
}

async fn verify_encoder_diagnostics(engine: &Engine, state: &Path) {
    let delayed = state.join("delay-start-192.168.1.22_5555");
    let before_frame = state.join("capture-before-192.168.1.22_5555");
    fs::write(&delayed, "").unwrap();
    fs::write(&before_frame, "").unwrap();
    let starter = engine.clone();
    let starting_a = tokio::spawn(async move {
        starter.start_mirror(TABLET_A.into(), TransportPreference::Wifi).await
    });
    wait_until(async || {
        engine.snapshot().await.logs.iter().any(|entry|
            entry.message.contains("Capture/encoding error"))
    }).await;
    let pending = engine.snapshot().await;
    assert_eq!(pending.sessions.len(), 1);
    assert_eq!(pending.sessions[0].phase, SessionPhase::Starting);
    assert!(pending.last_error.is_none(), "{:?}", pending.last_error);
    assert!(pending.logs.iter().any(|entry|
        entry.level == LogLevel::Warning && entry.message.contains("0xfffffff4")));
    fs::remove_file(&before_frame).unwrap();
    fs::remove_file(&delayed).unwrap();
    let ready = starting_a.await.unwrap().unwrap();
    assert_eq!(ready.sessions[0].phase, SessionPhase::Running);
    assert!(ready.last_error.is_none());
    println!("PASS: an encoder retry before the first frame is recorded without a failure banner");

    let ready_toolbar = engine.toolbar_sessions().await.into_iter().next().unwrap();
    fs::write(state.join("capture-after-192.168.1.22_5555"), "").unwrap();
    wait_until(async || {
        engine.snapshot().await.logs.iter().filter(|entry|
            entry.message.contains("Capture/encoding error")).count() == 2
    }).await;
    let still_running = engine.snapshot().await;
    assert_eq!(still_running.sessions[0].phase, SessionPhase::Running);
    assert!(still_running.last_error.is_none(), "{:?}", still_running.last_error);
    assert_eq!(still_running.logs.iter().filter(|entry|
        entry.level == LogLevel::Warning && entry.message.contains("0xfffffff4")).count(), 2);
    let toolbar = engine.toolbar_sessions().await.into_iter().next().unwrap();
    assert_eq!(toolbar.token, ready_toolbar.token);
    assert_eq!(toolbar.process_id, ready_toolbar.process_id);
    println!("PASS: an encoder retry after the first frame can keep the same running window without another Texture line");

    let wifi_online = state.join("wifi-online");
    fs::remove_file(&wifi_online).unwrap();
    wait_until(async || {
        let snapshot = engine.snapshot().await;
        snapshot.sessions.is_empty() && snapshot.last_error_device_id.as_deref() == Some(TABLET_A)
    }).await;
    let disconnected = engine.snapshot().await;
    let alert = disconnected.last_error.expect("actual Wi-Fi loss still needs an alert");
    assert!(!alert.contains("0xfffffff4"), "{alert}");
    fs::write(&wifi_online, "").unwrap();
    println!("PASS: a later Wi-Fi disconnect surfaces an alert without reusing the previous codec diagnostic");

    fs::write(&delayed, "").unwrap();
    let starter = engine.clone();
    let starting_a = tokio::spawn(async move {
        starter.start_mirror(TABLET_A.into(), TransportPreference::Wifi).await
    });
    wait_for(engine, |sessions| sessions.iter().any(|session|
        session.device_id == TABLET_A && session.phase == SessionPhase::Starting)).await;

    let fatal = state.join("capture-fatal-USB-456");
    fs::write(&fatal, "").unwrap();
    let failure = engine.start_mirror(TABLET_B.into(), TransportPreference::Usb).await.unwrap_err();
    assert!(failure.contains("0xfffffff4"), "{failure}");
    wait_until(async || {
        let snapshot = engine.snapshot().await;
        snapshot.last_error_device_id.as_deref() == Some(TABLET_B)
            && snapshot.sessions.iter().all(|session| session.device_id != TABLET_B)
    }).await;
    let failed = engine.snapshot().await;
    assert!(failed.last_error.as_deref().is_some_and(|message| message.contains("0xfffffff4")));
    assert!(failed.devices.iter().any(|device| device.id == TABLET_B
        && device.transports.iter().any(|transport|
            transport.kind == TransportKind::Usb && transport.state == "device")));
    fs::remove_file(&fatal).unwrap();
    println!("PASS: fatal capture startup over USB remains actionable even when the USB device stays online");

    fs::remove_file(&delayed).unwrap();
    let ready = starting_a.await.unwrap().unwrap();
    assert_eq!(ready.sessions.len(), 1);
    assert_eq!(ready.sessions[0].device_id, TABLET_A);
    assert_eq!(ready.sessions[0].phase, SessionPhase::Running);
    assert_eq!(ready.last_error_device_id.as_deref(), Some(TABLET_B));
    assert_eq!(ready.last_error, failed.last_error);
    engine.stop_mirror(TABLET_A.into()).await.unwrap();
    println!("PASS: one device's first frame leaves another device's confirmed error visible");
}

#[tokio::main]
async fn main() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("target/multi-device-test-{}", std::process::id()));
    let runtime = base.join("runtime");
    let state = base.join("state");
    for directory in [&runtime, &state] {
        fs::create_dir_all(directory).unwrap();
    }
    let binary = std::env::current_exe().unwrap().parent().unwrap().join("mock-runtime.exe");
    for executable in ["adb.exe", "scrcpy.exe"] {
        fs::copy(&binary, runtime.join(executable)).unwrap();
    }
    fs::write(runtime.join("scrcpy-server"), "").unwrap();
    std::env::set_var("QZZ_MOCK_STATE", &state);
    for marker in ["usb-online", "wifi-online", "usb-456-online", "wifi-456-online"] {
        fs::write(state.join(marker), "").unwrap();
    }

    let dispatcher = Arc::new(RecordingDispatcher::default());
    let engine = Engine::new_with_navigation_dispatcher(runtime, base.join("config"), dispatcher.clone()).unwrap();
    engine.spawn_background();
    let discovered = engine.refresh().await.unwrap();
    assert_eq!(discovered.devices.len(), 2);
    assert!(discovered.devices.iter().all(|device| device.transports.len() == 2));

    engine.start_mirror(TABLET_A.into(), TransportPreference::Usb).await.unwrap();
    let a = engine.toolbar_sessions().await.into_iter().next().unwrap();
    engine.select_device(Some(TABLET_B.into())).await.unwrap();
    engine.start_mirror(TABLET_B.into(), TransportPreference::Usb).await.unwrap();
    let b = other_toolbar(&engine, &a).await;
    let snapshot = engine.snapshot().await;
    assert_eq!(snapshot.sessions.len(), 2);
    assert!(snapshot.sessions.iter().all(|session| session.phase == SessionPhase::Running));
    assert_ne!(a.process_id, b.process_id);
    assert!(engine.start_mirror(TABLET_A.into(), TransportPreference::Wifi).await.is_err());
    assert_eq!(fs::read_to_string(state.join("scrcpy-starts.log")).unwrap().lines().count(), 2);
    println!("PASS: two devices run simultaneously, and a second transport cannot duplicate a tablet");

    engine.navigate(a.token, NavigationAction::Back).await.unwrap();
    engine.navigate(b.token, NavigationAction::Home).await.unwrap();
    assert_eq!(*dispatcher.0.lock().unwrap(), [
        (a.process_id, NavigationAction::Back),
        (b.process_id, NavigationAction::Home),
    ]);

    fs::write(state.join("close-window-USB-456"), "").unwrap();
    wait_for(&engine, |sessions| sessions.len() == 1 && sessions[0].device_id == TABLET_A).await;
    assert!(engine.navigate(b.token, NavigationAction::Home).await.is_err());
    engine.navigate(a.token, NavigationAction::Back).await.unwrap();
    fs::remove_file(state.join("close-window-USB-456")).unwrap();
    println!("PASS: navigation routes to the correct child and closing one window invalidates only its token");

    engine.start_mirror(TABLET_B.into(), TransportPreference::Usb).await.unwrap();
    let b = other_toolbar(&engine, &a).await;
    fs::remove_file(state.join("usb-online")).unwrap();
    wait_for(&engine, |sessions| sessions.len() == 2 && sessions.iter().any(|session|
        session.device_id == TABLET_A && session.transport == TransportKind::Wifi
            && session.phase == SessionPhase::Running)).await;
    let recovered_a = other_toolbar(&engine, &b).await;
    assert_ne!(recovered_a.token, a.token);
    assert!(engine.navigate(a.token, NavigationAction::Back).await.is_err());
    engine.navigate(recovered_a.token, NavigationAction::Back).await.unwrap();
    engine.navigate(b.token, NavigationAction::Home).await.unwrap();
    assert_eq!(dispatcher.0.lock().unwrap().last().unwrap().0, b.process_id);
    println!("PASS: loss of A's USB replaces its window and token while B continues with the same child");

    engine.set_wifi_enabled(false).await.unwrap();
    wait_for(&engine, |sessions| sessions.len() == 1 && sessions[0].device_id == TABLET_B).await;
    assert!(engine.navigate(recovered_a.token, NavigationAction::Back).await.is_err());
    engine.navigate(b.token, NavigationAction::Home).await.unwrap();
    engine.set_wifi_enabled(true).await.unwrap();
    engine.stop_mirror(TABLET_B.into()).await.unwrap();
    engine.start_mirror(TABLET_A.into(), TransportPreference::Wifi).await.unwrap();
    engine.start_mirror(TABLET_B.into(), TransportPreference::Wifi).await.unwrap();
    assert_eq!(engine.snapshot().await.sessions.len(), 2);
    assert!(engine.set_wifi_enabled(false).await.unwrap().sessions.is_empty());
    assert!(engine.toolbar_sessions().await.is_empty());
    println!("PASS: Wi-Fi off preserves a USB stream and stops every owned wireless stream");

    fs::write(state.join("usb-online"), "").unwrap();
    engine.refresh().await.unwrap();
    engine.start_mirror(TABLET_A.into(), TransportPreference::Usb).await.unwrap();
    fs::write(state.join("delay-start-USB-456"), "").unwrap();
    let starter = engine.clone();
    let start_b = tokio::spawn(async move {
        starter.start_mirror(TABLET_B.into(), TransportPreference::Usb).await
    });
    wait_for(&engine, |sessions| sessions.iter().any(|session|
        session.device_id == TABLET_B && session.phase == SessionPhase::Starting)).await;
    let remaining = tokio::time::timeout(Duration::from_secs(3), engine.stop_mirror(TABLET_A.into()))
        .await.expect("stopping another device must not wait for the first frame").unwrap();
    assert_eq!(remaining.sessions.len(), 1);
    assert_eq!(remaining.sessions[0].device_id, TABLET_B);
    assert_eq!(remaining.sessions[0].phase, SessionPhase::Starting);
    fs::remove_file(state.join("delay-start-USB-456")).unwrap();
    start_b.await.unwrap().unwrap();
    println!("PASS: stopping A does not cancel or wait for B's startup");

    engine.set_wifi_enabled(true).await.unwrap();
    engine.stop_mirror(TABLET_B.into()).await.unwrap();
    engine.start_mirror(TABLET_B.into(), TransportPreference::Usb).await.unwrap();
    engine.start_mirror(TABLET_A.into(), TransportPreference::Usb).await.unwrap();
    assert_eq!(engine.snapshot().await.sessions.len(), 2);
    fs::remove_file(state.join("usb-online")).unwrap();
    fs::remove_file(state.join("usb-456-online")).unwrap();
    wait_for(&engine, |sessions| sessions.len() == 2 && sessions.iter().all(|session|
        session.transport == TransportKind::Wifi && session.phase == SessionPhase::Running)).await;
    let recovered = engine.snapshot().await.sessions;
    assert!(recovered.iter().any(|session| session.device_id == TABLET_A && session.serial == "192.168.1.22:5555"));
    assert!(recovered.iter().any(|session| session.device_id == TABLET_B && session.serial == "192.168.1.33:5555"));
    println!("PASS: two simultaneous USB losses each recover to their own Wi-Fi endpoint");

    assert!(engine.stop_all_mirrors().await.unwrap().sessions.is_empty());
    fs::write(state.join("usb-online"), "").unwrap();
    fs::write(state.join("usb-456-online"), "").unwrap();
    let starts_before = fs::read_to_string(state.join("scrcpy-starts.log")).unwrap();
    fs::write(state.join("delay-devices"), "").unwrap();
    wait_until(async || state.join("devices-waiting").exists()).await;
    let starter = engine.clone();
    let queued_start = tokio::spawn(async move {
        starter.start_mirror(TABLET_A.into(), TransportPreference::Usb).await
    });
    wait_until(async || engine.snapshot().await.busy).await;
    assert!(engine.stop_all_mirrors().await.unwrap().sessions.is_empty());
    fs::remove_file(state.join("delay-devices")).unwrap();
    assert!(queued_start.await.unwrap().is_err());
    assert_eq!(fs::read_to_string(state.join("scrcpy-starts.log")).unwrap(), starts_before);
    println!("PASS: tray stop cancels a launch queued behind a background discovery");

    verify_encoder_diagnostics(&engine, &state).await;

    engine.start_mirror(TABLET_A.into(), TransportPreference::Usb).await.unwrap();
    engine.start_mirror(TABLET_B.into(), TransportPreference::Usb).await.unwrap();
    assert_eq!(engine.snapshot().await.sessions.len(), 2);
    engine.shutdown().await;
    assert!(engine.snapshot().await.sessions.is_empty());
    assert!(engine.toolbar_sessions().await.is_empty());
    println!("PASS: tray stop and application shutdown close all owned sessions");
}
