mod engine;
mod models;
mod navigation_bar;
mod scrcpy_input;

use engine::Engine;
use models::{Settings, Snapshot, TransportPreference};
use std::{path::PathBuf, sync::atomic::{AtomicBool, Ordering}};
use tauri::{Manager, State, menu::{Menu, MenuItem}, tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState}};

#[derive(Default)]
struct ExitState { quitting: AtomicBool }

#[tauri::command]
async fn get_snapshot(engine: State<'_, Engine>) -> Result<Snapshot, String> {
    Ok(engine.snapshot().await)
}

#[tauri::command]
async fn refresh_devices(engine: State<'_, Engine>) -> Result<Snapshot, String> {
    engine.refresh().await
}

#[tauri::command]
async fn save_settings(engine: State<'_, Engine>, settings: Settings) -> Result<Snapshot, String> {
    engine.save_settings(settings).await
}

#[tauri::command]
async fn select_device(engine: State<'_, Engine>, device_id: Option<String>) -> Result<Snapshot, String> {
    engine.select_device(device_id).await
}

#[tauri::command]
async fn start_mirror(engine: State<'_, Engine>, device_id: String, transport: TransportPreference) -> Result<Snapshot, String> {
    engine.start_mirror(device_id, transport).await
}

#[tauri::command]
async fn stop_mirror(engine: State<'_, Engine>) -> Result<Snapshot, String> {
    engine.stop_mirror().await
}

#[tauri::command]
async fn configure_wireless(engine: State<'_, Engine>, device_id: String) -> Result<Snapshot, String> {
    engine.configure_wireless(device_id).await
}

#[tauri::command]
async fn connect_wireless(engine: State<'_, Engine>, address: String) -> Result<Snapshot, String> {
    engine.connect_wireless(address).await
}

#[tauri::command]
async fn pair_wireless(engine: State<'_, Engine>, pairing_address: String, code: String, connect_address: Option<String>) -> Result<Snapshot, String> {
    engine.pair_wireless(pairing_address, code, connect_address).await
}

#[tauri::command]
fn hide_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
fn minimize_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
async fn quit_app(app: tauri::AppHandle, engine: State<'_, Engine>, exit: State<'_, ExitState>) -> Result<(), String> {
    if !exit.quitting.swap(true, Ordering::SeqCst) {
        engine.shutdown().await;
        app.exit(0);
    }
    Ok(())
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn runtime_dir(app: &tauri::AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let executable_dir = std::env::current_exe()?.parent().ok_or("无法确定程序目录")?.to_path_buf();
    let mut candidates = vec![executable_dir.join("runtime")];
    if let Ok(resources) = app.path().resource_dir() { candidates.push(resources.join("runtime")); }
    #[cfg(debug_assertions)]
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("runtime"));
    candidates.into_iter().find(|p| p.join("scrcpy.exe").is_file() && p.join("adb.exe").is_file())
        .ok_or_else(|| "未找到投屏组件，请完整解压便携包，保留程序旁的 runtime 文件夹。".into())
}

fn data_dir(app: &tauri::AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    let preferred = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.work/app-data");
    #[cfg(not(debug_assertions))]
    let preferred = std::env::current_exe()?.parent().ok_or("无法确定程序目录")?.join("data");
    if std::fs::create_dir_all(&preferred).is_ok() { return Ok(preferred); }
    let fallback = app.path().app_data_dir()?;
    std::fs::create_dir_all(&fallback)?;
    Ok(fallback)
}

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show_main(app)))
        .manage(ExitState::default())
        .setup(|app| {
            let engine = Engine::new(runtime_dir(app.handle())?, data_dir(app.handle())?)
                .map_err(std::io::Error::other)?;
            let background = engine.clone();
            tauri::async_runtime::spawn(async move { background.spawn_background(); });
            let navigation_engine = engine.clone();
            let navigation = navigation_bar::NavigationBar::start(move |token, button, complete| {
                let engine = navigation_engine.clone();
                let action = match button {
                    navigation_bar::NavigationButton::Recents => models::NavigationAction::Recents,
                    navigation_bar::NavigationButton::Back => models::NavigationAction::Back,
                    navigation_bar::NavigationButton::Home => models::NavigationAction::Home,
                };
                tauri::async_runtime::spawn(async move { complete(engine.navigate(token, action).await); });
            }).map_err(std::io::Error::other)?;
            let controller = navigation.controller();
            let tracking_engine = engine.clone();
            tauri::async_runtime::spawn(async move {
                while !controller.is_stopped() {
                    controller.set_target(tracking_engine.toolbar_session().await.map(|session| navigation_bar::Target {
                        process_id: session.process_id, token: session.token, running: session.running,
                    }));
                    tokio::time::sleep(std::time::Duration::from_millis(80)).await;
                }
            });
            app.manage(navigation);
            app.manage(engine);
            let show = MenuItem::with_id(app, "show", "打开轻镜", true, None::<&str>)?;
            let stop = MenuItem::with_id(app, "stop", "停止投屏", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &stop, &quit])?;
            let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png"))?;
            TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .tooltip("Qzz Mirror · 轻镜")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main(app),
                    "stop" => {
                        let engine = app.state::<Engine>().inner().clone();
                        tauri::async_runtime::spawn(async move { let _ = engine.stop_mirror().await; });
                    }
                    "quit" => {
                        if !app.state::<ExitState>().quitting.swap(true, Ordering::SeqCst) {
                            let app = app.clone();
                            let engine = app.state::<Engine>().inner().clone();
                            tauri::async_runtime::spawn(async move { engine.shutdown().await; app.exit(0); });
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if matches!(event, TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. }) {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot, refresh_devices, save_settings, select_device,
            start_mirror, stop_mirror, configure_wireless, connect_wireless,
            pair_wireless, hide_window, minimize_window, quit_app,
        ])
        .build(tauri::generate_context!());

    let app = match app {
        Ok(app) => app,
        Err(error) => {
            startup_error(&format!("轻镜暂时无法启动。\n\n{error}\n\n请完整解压便携包，并保留程序旁的 runtime 文件夹。"));
            return;
        }
    };

    app.run(|app, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            if !app.state::<ExitState>().quitting.swap(true, Ordering::SeqCst) {
                api.prevent_exit();
                let handle = app.clone();
                let engine = app.state::<Engine>().inner().clone();
                tauri::async_runtime::spawn(async move { engine.shutdown().await; handle.exit(0); });
            }
        }
    });
}

fn startup_error(message: &str) {
    #[cfg(windows)]
    {
        #[link(name = "user32")]
        extern "system" {
            fn MessageBoxW(window: *mut std::ffi::c_void, text: *const u16, caption: *const u16, flags: u32) -> i32;
        }
        let text: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
        let caption: Vec<u16> = "Qzz Mirror · 轻镜".encode_utf16().chain(Some(0)).collect();
        unsafe { MessageBoxW(std::ptr::null_mut(), text.as_ptr(), caption.as_ptr(), 0x10); }
    }
    #[cfg(not(windows))]
    eprintln!("{message}");
}
