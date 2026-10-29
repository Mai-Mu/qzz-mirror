//! Smoke-test the production navigation dispatcher against the bundled SDL.
//! Creates only its own hidden window; never activates it or injects OS input.
#[cfg(windows)]
mod smoke {
    use qzz_engine_tests::{
        models::NavigationAction,
        scrcpy_input::{NativeScrcpyDispatcher, NavigationDispatcher},
    };
    use serde_json::{json, Value};
    use std::{ffi::{c_char, c_void, CStr}, path::Path, ptr::null_mut,
        sync::atomic::{AtomicUsize, Ordering}, time::{Duration, Instant}};
    use windows_sys::Win32::{
        Foundation::{FreeLibrary, HMODULE, HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::{GetProcAddress, LoadLibraryW}, Threading::GetCurrentProcessId},
        UI::{Input::KeyboardAndMouse::{GetAsyncKeyState, GetCapture, GetKeyboardState},
            WindowsAndMessaging::{CallWindowProcW, GetForegroundWindow, GetWindowLongPtrW,
                SetWindowLongPtrW, GWLP_WNDPROC, WM_CHAR, WM_SYSCHAR}},
    };

    const KEY_DOWN: u32 = 0x300;
    const KEY_UP: u32 = 0x301;
    const TEXT_INPUT: u32 = 0x303;
    const LEFT_ALT: u16 = 0x100;
    const LEFT_ALT_KEY: u32 = (1 << 30) | 226;
    const ACTION_COUNT: usize = 36;
    static ORIGINAL_WNDPROC: AtomicUsize = AtomicUsize::new(0);
    static CHAR_MESSAGES: AtomicUsize = AtomicUsize::new(0);
    static SYSCHAR_MESSAGES: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "system" fn observe_window(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if message == WM_CHAR { CHAR_MESSAGES.fetch_add(1, Ordering::Relaxed); }
        if message == WM_SYSCHAR { SYSCHAR_MESSAGES.fetch_add(1, Ordering::Relaxed); }
        let previous: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT =
            std::mem::transmute(ORIGINAL_WNDPROC.load(Ordering::Relaxed));
        // Observe only our own hidden window, and retain SDL's original behavior.
        CallWindowProcW(Some(previous), window, message, wparam, lparam)
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct KeyboardEvent {
        kind: u32,
        reserved: u32,
        timestamp: u64,
        window_id: u32,
        which: u32,
        scancode: i32,
        key: u32,
        modifiers: u16,
        raw: u16,
        down: bool,
        repeat: bool,
    }

    #[repr(C)]
    union Event {
        kind: u32,
        keyboard: KeyboardEvent,
        padding: [u8; 128],
    }

    type Init = unsafe extern "C" fn(u32) -> bool;
    type CreateWindow = unsafe extern "C" fn(*const c_char, i32, i32, u64) -> *mut c_void;
    type StartTextInput = unsafe extern "C" fn(*mut c_void) -> bool;
    type GetWindowProperties = unsafe extern "C" fn(*mut c_void) -> u32;
    type GetPointerProperty = unsafe extern "C" fn(u32, *const c_char, *mut c_void) -> *mut c_void;
    type PollEvent = unsafe extern "C" fn(*mut Event) -> bool;
    type GetModState = unsafe extern "C" fn() -> u16;
    type GetVersion = unsafe extern "C" fn() -> i32;
    type GetError = unsafe extern "C" fn() -> *const c_char;
    type DestroyWindow = unsafe extern "C" fn(*mut c_void);
    type Quit = unsafe extern "C" fn();

    struct Sdl {
        library: HMODULE,
        window: *mut c_void,
        initialized: bool,
        hwnd: HWND,
        original_wndproc: isize,
        init: Init,
        create_window: CreateWindow,
        start_text_input: StartTextInput,
        get_window_properties: GetWindowProperties,
        get_pointer_property: GetPointerProperty,
        poll_event: PollEvent,
        get_mod_state: GetModState,
        get_version: GetVersion,
        get_error: GetError,
        destroy_window: DestroyWindow,
        quit: Quit,
    }

    unsafe fn symbol<T: Copy>(library: HMODULE, name: &[u8]) -> Result<T, String> {
        let pointer = GetProcAddress(library, name.as_ptr())
            .ok_or_else(|| format!("Missing SDL export {}", String::from_utf8_lossy(name)))?;
        // All callers request the matching function signature, not a data symbol.
        Ok(std::mem::transmute_copy(&pointer))
    }

    impl Sdl {
        unsafe fn load(path: &Path) -> Result<Self, String> {
            use std::os::windows::ffi::OsStrExt;
            let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
            let library = LoadLibraryW(wide.as_ptr());
            if library.is_null() { return Err(format!("LoadLibraryW failed: {}", std::io::Error::last_os_error())); }
            let loaded = (|| Ok(Self {
                library, window: null_mut(), initialized: false, hwnd: null_mut(), original_wndproc: 0,
                init: symbol(library, b"SDL_Init\0")?,
                create_window: symbol(library, b"SDL_CreateWindow\0")?,
                start_text_input: symbol(library, b"SDL_StartTextInput\0")?,
                get_window_properties: symbol(library, b"SDL_GetWindowProperties\0")?,
                get_pointer_property: symbol(library, b"SDL_GetPointerProperty\0")?,
                poll_event: symbol(library, b"SDL_PollEvent\0")?,
                get_mod_state: symbol(library, b"SDL_GetModState\0")?,
                get_version: symbol(library, b"SDL_GetVersion\0")?,
                get_error: symbol(library, b"SDL_GetError\0")?,
                destroy_window: symbol(library, b"SDL_DestroyWindow\0")?,
                quit: symbol(library, b"SDL_Quit\0")?,
            }))();
            if loaded.is_err() { FreeLibrary(library); }
            loaded
        }

        unsafe fn error(&self) -> String {
            let pointer = (self.get_error)();
            if pointer.is_null() { "unknown SDL error".into() }
            else { CStr::from_ptr(pointer).to_string_lossy().into_owned() }
        }
    }

    impl Drop for Sdl {
        fn drop(&mut self) {
            unsafe {
                if self.original_wndproc != 0 && !self.hwnd.is_null() {
                    SetWindowLongPtrW(self.hwnd, GWLP_WNDPROC, self.original_wndproc);
                }
                if !self.window.is_null() { (self.destroy_window)(self.window); }
                if self.initialized { (self.quit)(); }
                FreeLibrary(self.library);
            }
        }
    }

    struct DesktopState {
        foreground: usize,
        capture: usize,
        keyboard: [u8; 256],
        async_modifiers: [bool; 9],
    }

    unsafe fn desktop_state() -> Result<DesktopState, String> {
        let mut keyboard = [0u8; 256];
        if GetKeyboardState(keyboard.as_mut_ptr()) == 0 {
            return Err(format!("GetKeyboardState failed: {}", std::io::Error::last_os_error()));
        }
        // Read only the current-down bit; never synthesize or set keyboard state.
        let keys = [0x10, 0x11, 0x12, 0x5b, 0x5c, 0xa0, 0xa1, 0xa4, 0xa5];
        Ok(DesktopState {
            foreground: GetForegroundWindow() as usize,
            capture: GetCapture() as usize,
            keyboard,
            async_modifiers: keys.map(|key| GetAsyncKeyState(key) as u16 & 0x8000 != 0),
        })
    }

    fn millis(duration: Duration) -> f64 { duration.as_secs_f64() * 1000.0 }

    pub unsafe fn run(repository: &Path) -> Result<Value, String> {
        if std::mem::size_of::<Event>() != 128 || std::mem::size_of::<KeyboardEvent>() != 40 {
            return Err("Unexpected SDL event ABI".into());
        }
        let library_path = repository.join("src-tauri/runtime/SDL3.dll");
        let mut sdl = Sdl::load(&library_path)?;
        let version = (sdl.get_version)();
        if version != 3_004_018 { return Err(format!("Expected SDL 3.4.18, got version {version}")); }
        if !(sdl.init)(0x20) { return Err(sdl.error()); } // SDL_INIT_VIDEO
        sdl.initialized = true;
        sdl.window = (sdl.create_window)(b"Qzz isolated SDL navigation smoke (hidden)\0".as_ptr().cast(), 300, 300, 8);
        if sdl.window.is_null() { return Err(sdl.error()); } // SDL_WINDOW_HIDDEN
        if !(sdl.start_text_input)(sdl.window) { return Err(sdl.error()); }
        sdl.hwnd = (sdl.get_pointer_property)((sdl.get_window_properties)(sdl.window),
            b"SDL.window.win32.hwnd\0".as_ptr().cast(), null_mut());
        if sdl.hwnd.is_null() { return Err("SDL did not expose its own hidden HWND".into()); }
        let previous = GetWindowLongPtrW(sdl.hwnd, GWLP_WNDPROC);
        if previous == 0 { return Err("GetWindowLongPtrW failed for hidden window".into()); }
        ORIGINAL_WNDPROC.store(previous as usize, Ordering::Relaxed);
        if SetWindowLongPtrW(sdl.hwnd, GWLP_WNDPROC, observe_window as *const () as isize) == 0 {
            return Err("SetWindowLongPtrW failed for hidden window".into());
        }
        sdl.original_wndproc = previous;

        let mut event = Event { padding: [0; 128] };
        for _ in 0..4 { while (sdl.poll_event)(&mut event) {} }
        CHAR_MESSAGES.store(0, Ordering::Relaxed);
        SYSCHAR_MESSAGES.store(0, Ordering::Relaxed);
        let before = desktop_state()?;
        let mod_before = (sdl.get_mod_state)();
        let dispatcher = NativeScrcpyDispatcher;
        let process_id = GetCurrentProcessId();
        let actions = [NavigationAction::Recents, NavigationAction::Back, NavigationAction::Home];
        let mut submissions = Vec::new();
        let mut failures = Vec::new();
        let submission_start = Instant::now();
        for index in 0..ACTION_COUNT {
            let action = actions[index % actions.len()];
            let start = Instant::now();
            let result = dispatcher.dispatch(process_id, action);
            let elapsed_ms = millis(start.elapsed());
            if let Err(error) = &result { failures.push(format!("dispatch {index}: {error}")); }
            submissions.push(json!({"index":index,"action":action,"elapsedMs":elapsed_ms,"queued":result.is_ok()}));
        }
        let submission_ms = millis(submission_start.elapsed());
        let consume_start = Instant::now();
        let mut keys = Vec::new();
        let mut text_input_count = 0usize;
        let mut other_event_count = 0usize;
        let deadline = consume_start + Duration::from_secs(2);
        let mut complete_drains = 0usize;
        loop {
            while (sdl.poll_event)(&mut event) {
                match event.kind {
                    KEY_DOWN | KEY_UP => keys.push(event.keyboard),
                    TEXT_INPUT => text_input_count += 1,
                    _ => other_event_count += 1,
                }
                if Instant::now() >= deadline { break; }
            }
            complete_drains += 1;
            // Extra event-pump passes catch any character messages generated
            // after the navigation messages. There are no artificial sleeps.
            if (keys.len() >= ACTION_COUNT * 4 && complete_drains >= 8)
                || Instant::now() >= deadline { break; }
        }
        let consumption_ms = millis(consume_start.elapsed());
        let after = desktop_state()?;
        let mod_after = (sdl.get_mod_state)();
        let char_messages = CHAR_MESSAGES.load(Ordering::Relaxed);
        let syschar_messages = SYSCHAR_MESSAGES.load(Ordering::Relaxed);
        if keys.len() != ACTION_COUNT * 4 {
            failures.push(format!("Expected {} keyboard events, got {}", ACTION_COUNT * 4, keys.len()));
        }
        for (index, key) in keys.iter().enumerate() {
            let step = index % 4;
            let action = actions[(index / 4) % actions.len()];
            let letter = (action.shortcut_virtual_key() as u8).to_ascii_lowercase() as u32;
            let expected_key = if step == 0 || step == 3 { LEFT_ALT_KEY } else { letter };
            let expected_down = step < 2;
            let expected_kind = if expected_down { KEY_DOWN } else { KEY_UP };
            if key.kind != expected_kind || key.down != expected_down || key.key != expected_key
                || key.repeat || (key.modifiers & LEFT_ALT != 0) != (step != 3) {
                failures.push(format!("Unexpected keyboard event at index {index}"));
            }
        }
        let keyboard_changes: Vec<usize> = before.keyboard.iter().zip(after.keyboard.iter())
            .enumerate().filter_map(|(index, (a, b))| (a != b).then_some(index)).collect();
        let checks = json!({
            "allSubmissionsQueued":submissions.iter().all(|entry|entry["queued"] == true),
            "eventOrderAndModifiers":keys.len() == ACTION_COUNT * 4 && failures.is_empty(),
            "noTextInput":text_input_count == 0,
            "noWindowsCharMessages":char_messages == 0,
            "observedWindowsSyscharMessages":syschar_messages > 0,
            "foregroundUnchanged":before.foreground == after.foreground,
            "captureUnchanged":before.capture == after.capture,
            "windowsKeyboardStateUnchanged":keyboard_changes.is_empty(),
            "asyncModifiersUnchanged":before.async_modifiers == after.async_modifiers,
            "sdlModifiersBalanced":mod_before == mod_after && mod_after & LEFT_ALT == 0,
        });
        let passed = checks.as_object().unwrap().values().all(|value|value == true);
        let events: Vec<Value> = keys.iter().map(|key|json!({
            "type":key.kind,"scancode":key.scancode,"key":key.key,"mod":key.modifiers,
            "down":key.down,"repeat":key.repeat,"windowId":key.window_id,
        })).collect();
        Ok(json!({
            "passed":passed,"sdlVersion":"3.4.18","ownedWindowHidden":true,
            "productionDispatcher":true,"actionCount":ACTION_COUNT,"artificialSleeps":0,
            "textInputExplicitlyStarted":true,"ownHiddenWindowSubclassed":true,
            "windowsCharMessages":char_messages,"windowsSyscharMessages":syschar_messages,
            "timingScope":"local enqueue and SDL consumption; not Android end-to-end latency",
            "syscharTextBoundary":"SDL 3.4.18 SDL_windowsevents.c handles WM_CHAR as text but has no WM_SYSCHAR text branch",
            "submissionMs":submission_ms,"consumptionMs":consumption_ms,
            "completeDrainPasses":complete_drains,"textInputCount":text_input_count,
            "otherEventCount":other_event_count,"checks":checks,"failures":failures,
            "changedWindowsVirtualKeys":keyboard_changes,"sdlModifiersBefore":mod_before,
            "sdlModifiersAfter":mod_after,"submissions":submissions,"keyboardEvents":events,
        }))
    }
}

fn main() {
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = repository.join(".work/sdl-navigation-smoke.json");
    #[cfg(windows)]
    let result = unsafe { smoke::run(&repository) };
    #[cfg(not(windows))]
    let result: Result<serde_json::Value, String> = Err("This SDL smoke test requires Windows".into());
    let report = result.unwrap_or_else(|error|serde_json::json!({"passed":false,"error":error}));
    std::fs::create_dir_all(output.parent().unwrap()).expect("create report directory");
    std::fs::write(&output, format!("{}\n", serde_json::to_string_pretty(&report).unwrap())).expect("write SDL smoke report");
    println!("{}", serde_json::to_string(&serde_json::json!({
        "passed":report["passed"],"actionCount":report["actionCount"],
        "submissionMs":report["submissionMs"],"consumptionMs":report["consumptionMs"],
        "checks":report["checks"],"error":report["error"],"report":output,
    })).unwrap());
    if report["passed"] != true { std::process::exit(1); }
}
