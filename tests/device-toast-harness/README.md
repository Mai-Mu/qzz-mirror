# Windows notification regression harness

`cargo test --manifest-path tests/device-toast-harness/Cargo.toml --offline`
imports the production notification implementation. Normal tests cover the full
WinRT toast preparation chain (including expiration boxing and event handlers),
XML escaping, exact activation tokens, one-shot event gates, and Windows setting
handling. They do not show notifications, register an identity, or launch devices.

The ignored `native_notification_delivery_diagnostic` test is an opt-in desktop
probe. Without environment variables it only reads the current `com.qzz.mirror`
notifier setting/history and prepares a toast without displaying it.

For an isolated native delivery and manual button check, use PowerShell:

```powershell
$env:QZZ_TOAST_DIAGNOSTIC_APPID = 'com.qzz.mirror.toast-diagnostic-' + [guid]::NewGuid().ToString('N')
$env:QZZ_TOAST_DIAGNOSTIC_REGISTER = '1'
$env:QZZ_TOAST_DIAGNOSTIC_PROCESS_IDENTITY = '1'
$env:QZZ_TOAST_DIAGNOSTIC_SHOW = '1'
$env:QZZ_TOAST_DIAGNOSTIC_SECONDS = '60'
cargo test --manifest-path tests/device-toast-harness/Cargo.toml --offline native_notification_delivery_diagnostic -- --ignored --nocapture
```

Registration is refused unless the ID has the diagnostic prefix and its key is
new. The test displays at most one toast; actions only log `ToastEvent`, with no
engine, ADB, app launch, or mirroring. It prints `Setting`, native preparation,
`Show`, callback events, and history results. On completion it hides/removes only
its own toast and deletes its own newly created registration and notification
settings keys. The probe refuses an ID with pre-existing notification settings. It does not
alter the production AUMID, Start menu shortcut, notification permissions, or
Do Not Disturb settings. The test intentionally fails if a native `Failed` event
is received. Successful `Show` alone does not prove a banner was visible or that
a human clicked an action.
