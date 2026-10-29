use qzz_engine_tests::{engine::Engine, models::{NavigationAction,SessionPhase,TransportKind,TransportPreference}};
use qzz_engine_tests::scrcpy_input::NavigationDispatcher;
use std::{fs, path::PathBuf, time::Duration, sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}}};
#[derive(Default)]
struct RecordingDispatcher { calls:Mutex<Vec<(u32,NavigationAction)>>, fail:AtomicBool }
impl NavigationDispatcher for RecordingDispatcher {
 fn dispatch(&self,pid:u32,action:NavigationAction)->Result<(),String> {
  if self.fail.load(Ordering::SeqCst) { return Err("模拟窗口拒绝投递".into()); }
  self.calls.lock().unwrap().push((pid,action)); Ok(())
 }
}
async fn wait_for(engine: &Engine, expected: Option<TransportKind>) {
 for _ in 0..120 {
  let state = engine.snapshot().await;
  if match expected { Some(kind)=>state.session.as_ref().is_some_and(|s| s.transport==kind && s.phase==SessionPhase::Running), None=>state.session.is_none() } { return; }
  tokio::time::sleep(Duration::from_millis(100)).await;
 }
 panic!("unexpected engine state: {}",serde_json::to_string_pretty(&engine.snapshot().await).unwrap());
}
#[tokio::main]
async fn main() {
 let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/mock-test");
 let runtime = base.join("runtime"); let state = base.join("state"); let config = base.join("config");
 fs::create_dir_all(&runtime).unwrap(); fs::create_dir_all(&state).unwrap();
 for old in ["wifi-online","usb-online","close-window","delay-start","mdns-connect","identity-b","identity-missing","adb-calls.log","delay-metadata","metadata-waiting","scrcpy-starts.log","scrcpy-args.json"] { let _=fs::remove_file(state.join(old)); }
 let binary=std::env::current_exe().unwrap().parent().unwrap().join("mock-runtime.exe");
 fs::copy(&binary,runtime.join("adb.exe")).unwrap(); fs::copy(&binary,runtime.join("scrcpy.exe")).unwrap(); fs::write(runtime.join("scrcpy-server"),"").unwrap();
 std::env::set_var("QZZ_MOCK_STATE",&state); fs::write(state.join("usb-online"),"").unwrap();
 let invalid_config=base.join(format!("config-invalid-fps-{}",std::process::id())); fs::create_dir_all(&invalid_config).unwrap();
 fs::write(invalid_config.join("settings.json"),br#"{"settings":{"maxFps":241}}"#).unwrap();
 let invalid=Engine::new(runtime.clone(),invalid_config).unwrap();
 let repaired=invalid.snapshot().await; assert_eq!(repaired.settings.max_fps,None); assert!(repaired.logs.iter().any(|entry| entry.message.contains("默认设置"))); invalid.shutdown().await;
 println!("PASS: loading an out-of-range persisted frame rate restores default settings");
 // The Wi-Fi transport was connected with ADB before the app ever saw it.
 fs::write(state.join("wifi-online"),"").unwrap();
 let existing=Engine::new(runtime.clone(),base.join(format!("config-preexisting-{}",std::process::id()))).unwrap(); existing.spawn_background();
 let discovered=existing.refresh().await.unwrap(); assert!(discovered.saved_devices.is_empty());
 let existing_id=discovered.devices.iter().find(|d| d.id=="android:HARDWARE-123").unwrap().id.clone();
 existing.start_mirror(existing_id,TransportPreference::Usb).await.unwrap();
 let default_args:Vec<String>=serde_json::from_slice(&fs::read(state.join("scrcpy-args.json")).unwrap()).unwrap();
 assert!(default_args.windows(2).any(|pair| pair==["--max-size","0"])); assert!(!default_args.iter().any(|arg| arg=="--max-fps"));
 println!("PASS: actual default Engine launch keeps native resolution and sends no frame-rate cap");
 fs::remove_file(state.join("usb-online")).unwrap(); wait_for(&existing,Some(TransportKind::Wifi)).await;
 assert!(existing.snapshot().await.saved_devices.is_empty()); existing.stop_mirror().await.unwrap(); existing.shutdown().await;
 println!("PASS: an already-online Wi-Fi transport restores USB loss without any saved address");
 fs::write(state.join("usb-online"),"").unwrap();
 let dispatcher=Arc::new(RecordingDispatcher::default());
 let engine=Engine::new_with_navigation_dispatcher(runtime,config,dispatcher.clone()).unwrap(); engine.spawn_background();
 assert!(engine.toolbar_session().await.is_none()); assert!(engine.navigate(1,NavigationAction::Back).await.is_err());
 assert!(dispatcher.calls.lock().unwrap().is_empty());
 println!("PASS: navigation without an owned running session never calls the dispatcher");
 let snapshot=engine.refresh().await.unwrap(); let id=snapshot.devices.iter().find(|d| d.id=="android:HARDWARE-123").unwrap().id.clone();
 let mut settings=snapshot.settings; settings.max_fps=Some(120); engine.save_settings(settings.clone()).await.unwrap();
 engine.start_mirror(id.clone(),TransportPreference::Usb).await.unwrap();
 let custom_args:Vec<String>=serde_json::from_slice(&fs::read(state.join("scrcpy-args.json")).unwrap()).unwrap();
 assert!(custom_args.windows(2).any(|pair| pair==["--max-size","0"])); assert!(custom_args.windows(2).any(|pair| pair==["--max-fps","120"]));
 println!("PASS: actual custom Engine launch sends exactly 120 FPS while preserving native resolution");
 let navigation_session=engine.toolbar_session().await.unwrap(); assert!(navigation_session.running); assert!(navigation_session.process_id>0);
 for action in [NavigationAction::Recents,NavigationAction::Back,NavigationAction::Home] { engine.navigate(navigation_session.token,action).await.unwrap(); }
 let effects=dispatcher.calls.lock().unwrap().clone();
 assert_eq!(effects,vec![(navigation_session.process_id,NavigationAction::Recents),(navigation_session.process_id,NavigationAction::Back),(navigation_session.process_id,NavigationAction::Home)]);
 assert!(!fs::read_to_string(state.join("adb-calls.log")).unwrap().contains("shell input keyevent"));
 assert!(engine.navigate(navigation_session.token+1,NavigationAction::Back).await.is_err());
 assert_eq!(effects,*dispatcher.calls.lock().unwrap());
 println!("PASS: navigation dispatch uses only the owned running child PID, never ADB, and rejects stale tokens");
 dispatcher.fail.store(true,Ordering::SeqCst); assert!(engine.navigate(navigation_session.token,NavigationAction::Home).await.is_err()); dispatcher.fail.store(false,Ordering::SeqCst);
 assert_eq!(effects,*dispatcher.calls.lock().unwrap()); println!("PASS: native dispatcher failure is returned and logged without claiming Android execution");
 engine.stop_mirror().await.unwrap();
 assert_eq!(effects,*dispatcher.calls.lock().unwrap());
 assert!(engine.toolbar_session().await.is_none()); assert!(engine.navigate(navigation_session.token,NavigationAction::Back).await.is_err());
 println!("PASS: stopped session tokens never reach the native dispatcher");
 for fps in [0,241] { let mut invalid=settings.clone(); invalid.max_fps=Some(fps); assert!(engine.save_settings(invalid).await.is_err()); assert_eq!(engine.snapshot().await.settings.max_fps,Some(120)); }
 settings.max_fps=None; engine.save_settings(settings).await.unwrap();
 println!("PASS: save_settings rejects out-of-range frame rates without replacing valid settings");
 engine.configure_wireless(id.clone()).await.unwrap(); println!("PASS: USB-assisted wireless verifies and saves identity");
  fs::write(state.join("identity-b"),"").unwrap();
 assert!(engine.start_mirror(id.clone(),TransportPreference::Wifi).await.is_err()); assert!(engine.snapshot().await.session.is_none());
 fs::remove_file(state.join("identity-b")).unwrap(); println!("PASS: a reassigned Wi-Fi endpoint cannot launch the newly connected authorized device");
 engine.connect_wireless("192.168.1.22:5555".into()).await.unwrap();
 fs::write(state.join("identity-missing"),"").unwrap();
 assert!(engine.start_mirror(id.clone(),TransportPreference::Wifi).await.is_err()); assert!(engine.snapshot().await.session.is_none());
 fs::remove_file(state.join("identity-missing")).unwrap(); println!("PASS: Wi-Fi launch refuses an endpoint without verifiable hardware identity");
 engine.start_mirror(id.clone(),TransportPreference::Usb).await.unwrap(); wait_for(&engine,Some(TransportKind::Usb)).await;
 assert!(engine.configure_wireless(id.clone()).await.is_err()); println!("PASS: active mirror blocks disruptive wireless reconfiguration");
 let usb_token=engine.toolbar_session().await.unwrap().token;
 fs::write(state.join("delay-start"),"").unwrap(); fs::remove_file(state.join("usb-online")).unwrap();
 let mut reconnecting=None;
 for _ in 0..120 { if let Some(session)=engine.toolbar_session().await { if session.token!=usb_token { reconnecting=Some(session); break; } } tokio::time::sleep(Duration::from_millis(25)).await; }
 let reconnecting=reconnecting.expect("recovery child should be tracked before its first frame"); assert!(!reconnecting.running);
 assert!(engine.navigate(reconnecting.token,NavigationAction::Home).await.is_err());
 fs::remove_file(state.join("delay-start")).unwrap(); wait_for(&engine,Some(TransportKind::Wifi)).await; println!("PASS: USB unplug automatically restores the same device over Wi-Fi");
 let wifi_session=engine.toolbar_session().await.unwrap(); assert_ne!(wifi_session.token,usb_token); assert!(wifi_session.running);
 assert!(engine.navigate(usb_token,NavigationAction::Back).await.is_err()); engine.navigate(wifi_session.token,NavigationAction::Back).await.unwrap();
 assert_eq!(dispatcher.calls.lock().unwrap().last().copied(),Some((wifi_session.process_id,NavigationAction::Back)));
 println!("PASS: recovery rejects unready and old tokens; ready navigation uses only the replacement child PID");
 engine.stop_mirror().await.unwrap(); wait_for(&engine,None).await; println!("PASS: stop waits for the owned child and clears the session");
 fs::write(state.join("usb-online"),"").unwrap(); engine.start_mirror(id.clone(),TransportPreference::Usb).await.unwrap(); fs::write(state.join("close-window"),"").unwrap(); wait_for(&engine,None).await; tokio::time::sleep(Duration::from_millis(600)).await; assert!(engine.snapshot().await.session.is_none()); println!("PASS: closing scrcpy normally never triggers wireless recovery");
 fs::remove_file(state.join("close-window")).unwrap();
 let starts_before=fs::read_to_string(state.join("scrcpy-starts.log")).unwrap();
 fs::write(state.join("delay-metadata"),"").unwrap();
 let starter=engine.clone(); let delayed_id=id.clone(); let delayed=tokio::spawn(async move {starter.start_mirror(delayed_id,TransportPreference::Usb).await});
 for _ in 0..80 { if state.join("metadata-waiting").exists() { break; } tokio::time::sleep(Duration::from_millis(25)).await; }
 assert!(state.join("metadata-waiting").exists());
 let stopper=engine.clone(); let stopping=tokio::spawn(async move {stopper.stop_mirror().await});
 tokio::time::sleep(Duration::from_millis(100)).await; fs::remove_file(state.join("delay-metadata")).unwrap();
 assert!(delayed.await.unwrap().is_err()); stopping.await.unwrap().unwrap();
 assert_eq!(starts_before,fs::read_to_string(state.join("scrcpy-starts.log")).unwrap());
 println!("PASS: stop during the final ADB identity check prevents any scrcpy process from spawning");
 fs::write(state.join("delay-start"),"").unwrap();
 let starter=engine.clone(); let task=tokio::spawn(async move {starter.start_mirror(id,TransportPreference::Usb).await});
 tokio::time::sleep(Duration::from_millis(300)).await;
 let starting=engine.toolbar_session().await.expect("starting child is tracked"); assert!(!starting.running); assert!(engine.navigate(starting.token,NavigationAction::Home).await.is_err());
 println!("PASS: navigation rejects a starting session before its first frame");
 engine.stop_mirror().await.unwrap(); let _=task.await.unwrap(); wait_for(&engine,None).await; println!("PASS: stop cancels startup before the first video frame");
  fs::write(state.join("adb-calls.log"),"").unwrap();
 let error=engine.pair_wireless("192.168.1.22:33333".into(),"123456".into(),None).await.unwrap_err(); assert!(error.starts_with("配对成功"));
 let calls=fs::read_to_string(state.join("adb-calls.log")).unwrap(); assert!(!calls.lines().any(|line| line.starts_with("connect "))); println!("PASS: successful pairing without a connect endpoint never connects to the pairing port");
 engine.pair_wireless("192.168.1.22:33333".into(),"123456".into(),Some("192.168.1.22:44444".into())).await.unwrap();
 let calls=fs::read_to_string(state.join("adb-calls.log")).unwrap(); assert!(calls.contains("connect 192.168.1.22:44444")); assert!(!calls.contains("connect 192.168.1.22:33333")); println!("PASS: pairing code goes through stdin and explicit connect endpoint is used");
 fs::write(state.join("mdns-connect"),"").unwrap();
 engine.pair_wireless("192.168.1.22:33333".into(),"123456".into(),None).await.unwrap(); println!("PASS: pairing can discover the same host's separate TLS connect port through mDNS");
 engine.shutdown().await;
}


