# Isolated Windows navigation smoke harness

Run `node scripts/test-navigation.mjs` from the project root after the production
`src-tauri/src/navigation_bar.rs` module is present. The standalone crate imports
that exact module; it does not contain a second toolbar implementation.

The harness creates only its own `SDL_app_QzzTest` helper windows/processes. It
never moves the real cursor, calls `SendInput`/`SetForegroundWindow`, manipulates
an existing user window, sends ADB commands, or captures the desktop. Helper
windows use `WS_EX_NOACTIVATE`; movement, hide/show and minimize/restore preserve
activation. All clicks are synchronous messages addressed to verified buttons
in the harness's own toolbar process.

Checks cover attachment, 48 DIP geometry, move/resize, native accessibility
names, `WM_MOUSEACTIVATE`, unchanged foreground around `BM_CLICK`, callback
order/token, disabled non-running targets, PID replacement, owner destruction,
and worker shutdown. Optional `PrintWindow` captures only the verified toolbar.
Results are written to `.work/navigation-smoke.json` and the capture to
`.work/navigation-bar-smoke.bmp` (PNG when Pillow is available).
The runner also executes the production module's pure geometry unit tests for
bottom-edge, maximized, full-screen and negative-monitor-coordinate placement.
The three buttons are additionally tested with `WM_LBUTTONDOWN`/`WM_LBUTTONUP`
at their client-coordinate centers, checking `MA_NOACTIVATE`, unchanged
foreground and the mock owner's `GetGUIThreadInfo().hwndFocus` before/after.
An independent, nonactivating cover window created by the helper sits above the
mock owner before attachment; the new toolbar must remain below that cover and
above its owner without changing the foreground window. The test only changes
the z-order of its own mock cover.

A `WH_CALLWNDPROC` hook is installed only on the harness's own navigation UI
thread. After an explicit warm paint, 20 position-only moves must produce zero
button layout, enable, owner-draw, button-paint and background-erase messages.
The path is chosen away from the cursor without moving it, so ordinary hover
changes are excluded. Message counters are included in `pureMoveMessages` in
the JSON report. Three subsequent resizes are captured with `PrintWindow`; all
three cells must contain bright glyph pixels and the bar must retain a dark
background. The last resized capture is also available as
`.work/navigation-bar-resize.png` when Pillow is installed.

These checks catch unnecessary repaint/layout during movement and incomplete
resize rendering. They do not measure every DWM-presented frame or prove that
no user can perceive flicker on every GPU, monitor, scaling or desktop setup.

`BM_CLICK` does not reproduce the complete physical mouse/activation sequence.
Overlay hover, full-screen edge reveal, mixed-DPI monitor transitions, real
pointer focus behavior and actual Android navigation require separate checks.
No real cursor or user window should be used to extend this harness.
