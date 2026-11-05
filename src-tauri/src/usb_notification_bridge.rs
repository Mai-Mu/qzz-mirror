//! Event-driven bridge between successful USB discovery and native Windows toasts.
//! No additional device polling or window-visibility timer is started here.
use crate::{device_toast::{ToastEvent, ToastService}, engine::Engine, usb_notifications::UsbNotificationEvent};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tauri::{AppHandle, Manager};
use tokio::sync::{broadcast, mpsc};

pub struct UsbNotificationBridge {
    service: Arc<ToastService>,
    stopping: Arc<AtomicBool>,
    jobs: Mutex<Vec<tauri::async_runtime::JoinHandle<()>>>,
}

/// Losing focus alone is deliberately insufficient. An unknown window state
/// suppresses notifications rather than treating it as a hidden application.
pub fn is_background(app: &AppHandle) -> bool {
    app.get_webview_window("main").is_some_and(|window| {
        match (window.is_visible(), window.is_minimized()) {
            (Ok(visible), Ok(minimized)) => !visible || minimized,
            _ => false,
        }
    })
}

pub fn sync_background(app: &AppHandle) {
    if let Some(engine) = app.try_state::<Engine>() {
        let background = is_background(app);
        engine.set_notification_background(background);
        if !background { engine.clear_usb_notifications(); }
    }
}

impl UsbNotificationBridge {
    pub fn start(app: AppHandle, engine: Engine) -> Result<Self, String> {
        // Subscribe before the existing background refresh starts so that the
        // initial successful snapshot reliably establishes the baseline.
        let mut discoveries = engine.subscribe_usb_notifications();
        let (events, mut actions) = mpsc::unbounded_channel();
        let service = Arc::new(ToastService::start(app.config().identifier.as_str(), move |event| {
            let _ = events.send(event);
        })?);
        let stopping = Arc::new(AtomicBool::new(false));
        let discovery_engine = engine.clone();
        let discovery_service = service.clone();
        let discovery_app = app.clone();
        let discovery_stopping = stopping.clone();
        let discovery_job = tauri::async_runtime::spawn(async move {
            loop {
                if discovery_stopping.load(Ordering::Acquire) { break; }
                match discoveries.recv().await {
                    Ok(UsbNotificationEvent::Show(notice)) => {
                        if discovery_stopping.load(Ordering::Acquire) { break; }
                        if !is_background(&discovery_app)
                            || !discovery_engine.usb_notification_is_current(&notice.token).await {
                            discovery_engine.dismiss_usb_notification(&notice.token);
                            continue;
                        }
                        if let Err(error) = discovery_service.show(&notice.token, &notice.device_name) {
                            discovery_engine.dismiss_usb_notification(&notice.token);
                            discovery_engine.report_notification_error(error).await;
                        }
                    }
                    Ok(UsbNotificationEvent::Remove { token }) => { let _ = discovery_service.remove(&token); }
                    Ok(UsbNotificationEvent::Clear) => { let _ = discovery_service.clear(); }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // A lost removal must never leave an actionable old toast.
                        discovery_engine.clear_usb_notifications();
                        let _ = discovery_service.clear();
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        let action_stopping = stopping.clone();
        let action_job = tauri::async_runtime::spawn(async move {
            while let Some(action) = actions.recv().await {
                if action_stopping.load(Ordering::Acquire) { break; }
                match action {
                    ToastEvent::Submitted { .. } => engine.report_notification_submitted().await,
                    ToastEvent::OpenPanel { token } => {
                        engine.dismiss_usb_notification(&token);
                        crate::show_main(&app);
                    }
                    ToastEvent::StartMirroring { token } => {
                        // Keep the panel hidden on success. Opening it first
                        // would discard the token before the operation gate.
                        if engine.start_from_usb_notification(token).await.is_err() {
                            crate::show_main(&app);
                        }
                    }
                    ToastEvent::Dismissed { token } => engine.dismiss_usb_notification(&token),
                    ToastEvent::Failed { token, message } => {
                        engine.dismiss_usb_notification(&token);
                        engine.report_notification_error(message).await;
                    }
                }
            }
        });
        Ok(Self { service, stopping, jobs: Mutex::new(vec![discovery_job, action_job]) })
    }

    pub fn shutdown(&self) {
        if self.stopping.swap(true, Ordering::AcqRel) { return; }
        if let Ok(mut jobs) = self.jobs.lock() {
            for job in jobs.drain(..) { job.abort(); }
        }
        self.service.shutdown();
    }
}

impl Drop for UsbNotificationBridge {
    fn drop(&mut self) { self.shutdown(); }
}
