#![allow(dead_code, unused_imports)]

// Compiles the production service and covers XML, WinRT preparation, activation
// gates, and notification-setting handling. Ordinary tests do not show a toast
// or change registry values. The ignored native diagnostic has explicit opt-ins
// for a temporary test-only AUMID and one displayed toast; it never starts a device.
#[path = "../../../src-tauri/src/device_toast.rs"]
mod device_toast;
