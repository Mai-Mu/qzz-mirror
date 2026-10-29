//! Native smoke tests using only windows created by this executable and its child.
//! Never moves the cursor, sends global input, or requests foreground activation.
#![cfg(windows)]

#[path = "../../../src-tauri/src/navigation_bar.rs"]
mod navigation_bar;

use navigation_bar::{NavigationBar, NavigationButton, Target};
use std::{
    ffi::c_void,
    fs,
    io::{BufRead, BufReader, Write},
    mem::{size_of, zeroed},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    ptr::{null, null_mut},
    sync::{mpsc, Arc, Mutex, OnceLock, atomic::{AtomicU64, Ordering}},
    thread,
    time::{Duration, Instant},
};
use std::os::windows::process::CommandExt;
use windows_sys::{
    core::BOOL,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::{Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS}, Gdi::*},
        Storage::Xps::PrintWindow,
        System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::{HiDpi::*, Input::KeyboardAndMouse::IsWindowEnabled, WindowsAndMessaging::*},
    },
};

const HELPER_COMMAND: u32 = WM_APP + 71;
const MOCK_CLASS: &str = "SDL_app_QzzTest";
const COVER_CLASS: &str = "QzzMirrorNavigationTestCover";
const BAR_CLASS: &str = "QzzMirrorNavigationBar";
const WAIT: Duration = Duration::from_secs(5);
const BUTTON_IDS: [i32; 3] = [100, 101, 102];

#[derive(Clone, Copy, Default, Debug)]
struct PaintCounts { parent_paints: u64, button_paints: u64, draw_items: u64, erases: u64, child_positions: u64, enables: u64 }
impl PaintCounts {
    fn difference(self, previous: Self) -> Self { Self {
        parent_paints: self.parent_paints - previous.parent_paints,
        button_paints: self.button_paints - previous.button_paints,
        draw_items: self.draw_items - previous.draw_items,
        erases: self.erases - previous.erases,
        child_positions: self.child_positions - previous.child_positions,
        enables: self.enables - previous.enables,
    } }
    fn json(self) -> String { format!("{{\"parentPaintMessages\":{},\"buttonPaintMessages\":{},\"ownerDrawMessages\":{},\"eraseBackgroundMessages\":{},\"childPositionMessages\":{},\"buttonEnableMessages\":{}}}", self.parent_paints,self.button_paints,self.draw_items,self.erases,self.child_positions,self.enables) }
}

struct PaintProbe {
    bar: usize, buttons: [usize; 3], parent_paints: AtomicU64, button_paints: AtomicU64,
    draw_items: AtomicU64, erases: AtomicU64, child_positions: AtomicU64, enables: AtomicU64,
}
static PAINT_PROBE: OnceLock<PaintProbe> = OnceLock::new();
impl PaintProbe {
    fn snapshot(&self) -> PaintCounts { PaintCounts {
        parent_paints:self.parent_paints.load(Ordering::Relaxed),button_paints:self.button_paints.load(Ordering::Relaxed),
        draw_items:self.draw_items.load(Ordering::Relaxed),erases:self.erases.load(Ordering::Relaxed),
        child_positions:self.child_positions.load(Ordering::Relaxed),enables:self.enables.load(Ordering::Relaxed),
    } }
}
unsafe extern "system" fn paint_probe(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && lparam != 0 {
        if let Some(probe) = PAINT_PROBE.get() {
            let message = &*(lparam as *const CWPSTRUCT);
            let is_bar = message.hwnd as usize == probe.bar;
            let is_button = probe.buttons.contains(&(message.hwnd as usize));
            let counter = match message.message {
                WM_PAINT if is_bar => Some(&probe.parent_paints),
                WM_PAINT if is_button => Some(&probe.button_paints),
                WM_DRAWITEM if is_bar => Some(&probe.draw_items),
                WM_ERASEBKGND if is_bar || is_button => Some(&probe.erases),
                WM_WINDOWPOSCHANGING if is_button => Some(&probe.child_positions),
                WM_ENABLE if is_button => Some(&probe.enables),
                _ => None,
            };
            if let Some(counter) = counter { counter.fetch_add(1, Ordering::Relaxed); }
        }
    }
    CallNextHookEx(null_mut(), code, wparam, lparam)
}
struct PaintHook(HHOOK);
impl Drop for PaintHook { fn drop(&mut self) { unsafe { UnhookWindowsHookEx(self.0); } } }

fn install_paint_probe(bar: HWND) -> Result<PaintHook, String> {
    unsafe {
        let mut pid = 0; let thread = GetWindowThreadProcessId(bar, &mut pid);
        require(pid == std::process::id() && bars().contains(&bar), "refusing hook: not own navigation UI thread")?;
        let buttons = BUTTON_IDS.map(|id| GetDlgItem(bar, id) as usize);
        PAINT_PROBE.set(PaintProbe { bar: bar as usize, buttons, parent_paints:AtomicU64::new(0),button_paints:AtomicU64::new(0),
            draw_items:AtomicU64::new(0),erases:AtomicU64::new(0),child_positions:AtomicU64::new(0),enables:AtomicU64::new(0) }).map_err(|_| "paint probe already installed")?;
        // Thread-scoped, same process only. No hook is installed in the helper,
        // the user's running app, or any other existing native window.
        let hook = SetWindowsHookExW(WH_CALLWNDPROC, Some(paint_probe), null_mut(), thread);
        require(!hook.is_null(), "cannot install own-thread paint probe")?;
        Ok(PaintHook(hook))
    }
}

fn wide(value: &str) -> Vec<u16> { value.encode_utf16().chain(Some(0)).collect() }

struct Helper {
    child: Child,
    input: ChildStdin,
    output: mpsc::Receiver<String>,
    window: HWND,
    cover: HWND,
}

impl Helper {
    fn start() -> Result<Self, String> {
        let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
            .arg("--owner-child")
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit())
            .creation_flags(0x08000000) // CREATE_NO_WINDOW: no console window.
            .spawn().map_err(|e| format!("spawn own mock process: {e}"))?;
        let input = child.stdin.take().ok_or("missing helper stdin")?;
        let stdout = child.stdout.take().ok_or("missing helper stdout")?;
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || { for line in BufReader::new(stdout).lines().map_while(Result::ok) { if tx.send(line).is_err() { break; } } });
        let mut helper = Self { child, input, output: rx, window: null_mut(), cover: null_mut() };
        let ready = helper.output.recv_timeout(WAIT).map_err(|e| format!("helper readiness: {e}"))?;
        let words: Vec<_> = ready.split_whitespace().collect();
        if words.len() != 3 || words[0] != "ready" || words[1].parse::<u32>().ok() != Some(helper.child.id()) { return Err(format!("invalid helper handshake: {ready}")); }
        helper.window = words[2].parse::<usize>().map_err(|e| e.to_string())? as HWND;
        Ok(helper)
    }

    fn pid(&self) -> u32 { self.child.id() }

    fn command(&mut self, command: &str) -> Result<(), String> {
        writeln!(self.input, "{command}").and_then(|_| self.input.flush()).map_err(|e| format!("helper write: {e}"))?;
        let reply = self.output.recv_timeout(WAIT).map_err(|e| format!("helper command {command}: {e}"))?;
        if !reply.starts_with("ok ") { return Err(format!("helper rejected {command}: {reply}")); }
        if let Some(value) = reply.strip_prefix("ok create ") { self.window = value.parse::<usize>().map_err(|e| e.to_string())? as HWND; }
        if let Some(value) = reply.strip_prefix("ok cover ") { self.cover = value.parse::<usize>().map_err(|e| e.to_string())? as HWND; }
        if reply == "ok uncover" { self.cover = null_mut(); }
        Ok(())
    }
}

impl Drop for Helper {
    fn drop(&mut self) {
        let _ = writeln!(self.input, "exit"); let _ = self.input.flush();
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if self.child.try_wait().ok().flatten().is_some() { return; }
            thread::sleep(Duration::from_millis(20));
        }
        // This Child was created above; never kill a process found by enumeration.
        let _ = self.child.kill(); let _ = self.child.wait();
    }
}

unsafe extern "system" fn mock_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => MA_NOACTIVATE as LRESULT,
        WM_CLOSE => { DestroyWindow(hwnd); 0 }
        WM_DESTROY => 0, // Keep the process/message queue alive for lifecycle tests.
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn create_mock(class: &[u16]) -> Result<HWND, String> {
    let title = wide("Qzz Mirror navigation smoke — isolated mock");
    let hwnd = CreateWindowExW(WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, class.as_ptr(), title.as_ptr(),
        WS_OVERLAPPEDWINDOW, 30, 30, 300, 300, null_mut(), null_mut(), GetModuleHandleW(null()), null());
    if hwnd.is_null() { return Err("CreateWindowExW(mock) failed".into()); }
    let work = primary_work_area()?;
    let height = (work.bottom - work.top - 180).clamp(140, 340);
    let width = (work.right - work.left - 100).clamp(180, 320);
    SetWindowPos(hwnd, null_mut(), work.left + 35, work.top + 35, width, height,
        SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOOWNERZORDER);
    ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    Ok(hwnd)
}

unsafe fn create_cover(class: &[u16], owner: HWND) -> Result<HWND, String> {
    let mut rect: RECT = zeroed();
    require(GetWindowRect(owner, &mut rect) != 0, "own mock owner rectangle missing")?;
    let hwnd = CreateWindowExW(WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, class.as_ptr(),
        wide("Qzz Mirror navigation smoke — own cover").as_ptr(), WS_OVERLAPPEDWINDOW,
        rect.left + 5, rect.top + 15, rect.right - rect.left, rect.bottom - rect.top,
        null_mut(), null_mut(), GetModuleHandleW(null()), null());
    require(!hwnd.is_null(), "CreateWindowExW(own cover) failed")?;
    let above = GetWindow(owner, GW_HWNDPREV);
    // Move only our new cover. Keep it in the ordinary, non-topmost group.
    let insert_after = if above.is_null() || GetWindowLongPtrW(above, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST != 0 { HWND_TOP } else { above };
    if SetWindowPos(hwnd, insert_after, rect.left + 5, rect.top + 15, rect.right - rect.left,
        rect.bottom - rect.top, SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_SHOWWINDOW) == 0 {
        DestroyWindow(hwnd); return Err("position own cover above mock owner failed".into());
    }
    Ok(hwnd)
}

unsafe fn run_helper() -> Result<(), String> {
    SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    let class = wide(MOCK_CLASS);
    let wc = WNDCLASSW { lpfnWndProc: Some(mock_proc), hInstance: GetModuleHandleW(null()),
        lpszClassName: class.as_ptr(), hbrBackground: GetStockObject(BLACK_BRUSH) as _, ..zeroed() };
    if RegisterClassW(&wc) == 0 { return Err("RegisterClassW(mock) failed".into()); }
    let cover_class = wide(COVER_CLASS);
    let cover_wc = WNDCLASSW { lpfnWndProc: Some(mock_proc), hInstance: GetModuleHandleW(null()),
        lpszClassName: cover_class.as_ptr(), hbrBackground: GetStockObject(GRAY_BRUSH) as _, ..zeroed() };
    if RegisterClassW(&cover_wc) == 0 { return Err("RegisterClassW(own cover) failed".into()); }
    let mut hwnd = create_mock(&class)?;
    let mut cover: HWND = null_mut();
    // CreateWindowEx already created this thread's queue before it is published.
    let tid = GetCurrentThreadId();
    println!("ready {} {}", std::process::id(), hwnd as usize); std::io::stdout().flush().map_err(|e| e.to_string())?;
    thread::spawn(move || {
        for line in std::io::stdin().lock().lines().map_while(Result::ok) {
            let pointer = Box::into_raw(Box::new(line));
            if PostThreadMessageW(tid, HELPER_COMMAND, 0, pointer as LPARAM) == 0 { drop(Box::from_raw(pointer)); break; }
        }
        PostThreadMessageW(tid, WM_QUIT, 0, 0);
    });
    let mut msg: MSG = zeroed();
    while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
        if msg.message == HELPER_COMMAND {
            let command = *Box::from_raw(msg.lParam as *mut String);
            let parts: Vec<_> = command.split_whitespace().collect();
            let result: Result<String, String> = match parts.first().copied() {
                Some("move") if parts.len() == 5 => {
                    let values = parts[1..].iter().map(|p| p.parse::<i32>()).collect::<Result<Vec<_>, _>>();
                    match values {
                        Ok(v) if SetWindowPos(hwnd, null_mut(), v[0], v[1], v[2], v[3], SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOOWNERZORDER) != 0 => Ok("move".into()),
                        _ => Err("move failed".into()),
                    }
                }
                Some("hide") => { ShowWindow(hwnd, SW_HIDE); Ok("hide".into()) }
                Some("show") | Some("restore") => { ShowWindow(hwnd, SW_SHOWNOACTIVATE); Ok("show".into()) }
                Some("minimize") => { ShowWindow(hwnd, SW_SHOWMINNOACTIVE); Ok("minimize".into()) }
                Some("destroy") => { if !hwnd.is_null() { DestroyWindow(hwnd); hwnd = null_mut(); } Ok("destroy".into()) }
                Some("create") if hwnd.is_null() => create_mock(&class).map(|created| { hwnd = created; format!("create {}", hwnd as usize) }),
                Some("cover") => {
                    if !cover.is_null() { DestroyWindow(cover); cover = null_mut(); }
                    create_cover(&cover_class, hwnd).map(|created| { cover = created; format!("cover {}", cover as usize) })
                }
                Some("uncover") => { if !cover.is_null() { DestroyWindow(cover); cover = null_mut(); } Ok("uncover".into()) }
                Some("exit") => { if !cover.is_null() { DestroyWindow(cover); } if !hwnd.is_null() { DestroyWindow(hwnd); } println!("ok exit"); std::io::stdout().flush().ok(); break; }
                _ => Err("unknown command".into()),
            };
            match result { Ok(value) => println!("ok {value}"), Err(error) => println!("error {error}") }
            std::io::stdout().flush().map_err(|e| e.to_string())?;
        } else { TranslateMessage(&msg); DispatchMessageW(&msg); }
    }
    Ok(())
}

unsafe fn primary_work_area() -> Result<RECT, String> {
    let monitor = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
    let mut info = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..zeroed() };
    if GetMonitorInfoW(monitor, &mut info) == 0 { return Err("GetMonitorInfoW failed".into()); }
    Ok(info.rcWork)
}

struct FindWindows { pid: u32, class: String, found: Vec<HWND> }
unsafe extern "system" fn enumerate(hwnd: HWND, param: LPARAM) -> BOOL {
    let context = &mut *(param as *mut FindWindows);
    let mut pid = 0; GetWindowThreadProcessId(hwnd, &mut pid);
    if pid == context.pid {
        let mut text = [0u16; 128];
        let count = GetClassNameW(hwnd, text.as_mut_ptr(), text.len() as i32);
        if count > 0 && String::from_utf16_lossy(&text[..count as usize]) == context.class { context.found.push(hwnd); }
    }
    1
}

fn bars() -> Vec<HWND> {
    let mut context = FindWindows { pid: std::process::id(), class: BAR_CLASS.into(), found: vec![] };
    unsafe { EnumWindows(Some(enumerate), &mut context as *mut _ as LPARAM); }
    context.found
}

fn bar_for(owner: HWND) -> Option<HWND> { bars().into_iter().find(|bar| unsafe { GetWindow(*bar, GW_OWNER) == owner }) }
fn bar_visible(owner: HWND) -> bool { bar_for(owner).is_some_and(|bar| unsafe { IsWindowVisible(bar) != 0 }) }

fn above_in_z_order(upper: HWND, lower: HWND) -> bool {
    if upper.is_null() || lower.is_null() || upper == lower { return false; }
    unsafe {
        let mut cursor = GetWindow(lower, GW_HWNDPREV);
        for _ in 0..4096 {
            if cursor.is_null() { return false; }
            if cursor == upper { return true; }
            cursor = GetWindow(cursor, GW_HWNDPREV);
        }
    }
    false
}

fn wait_for(name: &str, mut condition: impl FnMut() -> bool) -> Result<(), String> {
    let deadline = Instant::now() + WAIT;
    while Instant::now() < deadline { if condition() { return Ok(()); } thread::sleep(Duration::from_millis(20)); }
    Err(format!("timed out: {name}"))
}

unsafe fn frame(hwnd: HWND) -> Result<RECT, String> {
    let mut rect: RECT = zeroed();
    if DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS as u32, &mut rect as *mut _ as *mut c_void, size_of::<RECT>() as u32) < 0 && GetWindowRect(hwnd, &mut rect) == 0 { return Err("cannot read own mock frame".into()); }
    Ok(rect)
}

fn geometry_matches(owner: HWND) -> bool {
    let Some(bar) = bar_for(owner) else { return false; };
    unsafe {
        let Ok(owner_rect) = frame(owner) else { return false; };
        let mut bar_rect: RECT = zeroed();
        if GetWindowRect(bar, &mut bar_rect) == 0 || IsWindowVisible(bar) == 0 { return false; }
        let dpi = GetDpiForWindow(bar).max(96);
        let expected_height = (48 * dpi as i32 + 48) / 96;
        (bar_rect.left - owner_rect.left).abs() <= 1 && (bar_rect.right - owner_rect.right).abs() <= 1
            && (bar_rect.top - owner_rect.bottom).abs() <= 1 && ((bar_rect.bottom - bar_rect.top) - expected_height).abs() <= 1
    }
}

fn require(condition: bool, message: &str) -> Result<(), String> { if condition { Ok(()) } else { Err(message.into()) } }

fn click(bar: HWND, id: i32) -> Result<(), String> {
    unsafe {
        let mut pid = 0; GetWindowThreadProcessId(bar, &mut pid);
        require(pid == std::process::id() && bars().contains(&bar), "refusing click: not our own navigation bar")?;
        let button = GetDlgItem(bar, id);
        require(!button.is_null(), "missing navigation button")?;
        let foreground = GetForegroundWindow();
        let mut result = 0usize;
        let delivered = SendMessageTimeoutW(button, BM_CLICK, 0, 0, SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT, 1000, &mut result);
        require(delivered != 0, "BM_CLICK delivery timed out")?;
        require(GetForegroundWindow() == foreground, "navigation button changed foreground window")?;
    }
    Ok(())
}

unsafe fn owner_focus(owner: HWND) -> Result<HWND, String> {
    let thread = GetWindowThreadProcessId(owner, null_mut());
    require(thread != 0, "own mock owner thread missing")?;
    let mut info = GUITHREADINFO { cbSize: size_of::<GUITHREADINFO>() as u32, ..zeroed() };
    require(GetGUIThreadInfo(thread, &mut info) != 0, "GetGUIThreadInfo(own mock owner) failed")?;
    Ok(info.hwndFocus)
}

fn click_mouse_messages(bar: HWND, id: i32, owner: HWND) -> Result<(), String> {
    unsafe {
        let mut pid = 0; GetWindowThreadProcessId(bar, &mut pid);
        require(pid == std::process::id() && bars().contains(&bar), "refusing mouse messages: not our own bar")?;
        require(GetWindow(bar, GW_OWNER) == owner, "refusing mouse messages: wrong mock owner")?;
        let button = GetDlgItem(bar, id);
        require(!button.is_null(), "missing own button for mouse-message smoke")?;
        let mut client: RECT = zeroed();
        require(GetClientRect(button, &mut client) != 0, "cannot read own button client rectangle")?;
        let x = (client.right - client.left) / 2; let y = (client.bottom - client.top) / 2;
        let coordinates = ((y as u32 & 0xffff) << 16 | (x as u32 & 0xffff)) as LPARAM;
        let foreground = GetForegroundWindow(); let focus = owner_focus(owner)?;
        let mut result = 0usize;
        let activation = (HTCLIENT | (WM_LBUTTONDOWN << 16)) as LPARAM;
        require(SendMessageTimeoutW(button, WM_MOUSEACTIVATE, bar as WPARAM, activation,
            SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT, 1000, &mut result) != 0 && result == MA_NOACTIVATE as usize,
            "own button WM_MOUSEACTIVATE must return MA_NOACTIVATE")?;
        require(SendMessageTimeoutW(button, WM_LBUTTONDOWN, 1, coordinates,
            SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT, 1000, &mut result) != 0, "own button mouse-down timed out")?;
        require(SendMessageTimeoutW(button, WM_LBUTTONUP, 0, coordinates,
            SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT, 1000, &mut result) != 0, "own button mouse-up timed out")?;
        require(GetForegroundWindow() == foreground, "mouse-message path changed foreground window")?;
        require(owner_focus(owner)? == focus, "mouse-message path changed own mock owner focus")?;
    }
    Ok(())
}

fn measure_pure_moves(owner: &mut Helper) -> Result<PaintCounts, String> {
    let bar = bar_for(owner.window).ok_or("bar missing for paint probe")?;
    let mut rect: RECT = unsafe { zeroed() }; let mut cursor = POINT { x: 0, y: 0 };
    unsafe { require(GetWindowRect(owner.window, &mut rect) != 0, "own mock rectangle missing")?; require(GetCursorPos(&mut cursor) != 0, "cannot read cursor for exclusion region")?; }
    let work = unsafe { primary_work_area()? }; let width = rect.right - rect.left; let height = rect.bottom - rect.top;
    let bar_height = unsafe { (48 * GetDpiForWindow(bar).max(96) as i32 + 48) / 96 };
    let candidates = [
        (work.left + 25, work.top + 25),
        ((work.right - width - 125).max(work.left + 25), work.top + 25),
        (work.left + 25, (work.bottom - height - bar_height - 25).max(work.top + 25)),
        ((work.right - width - 125).max(work.left + 25), (work.bottom - height - bar_height - 25).max(work.top + 25)),
    ];
    let (x, y) = candidates.into_iter().find(|(x,y)| {
        // Read the cursor only; choose our mock path elsewhere so a legitimate
        // hover-state change does not get counted as unnecessary movement paint.
        !(cursor.x >= *x - 20 && cursor.x <= *x + width + 120 && cursor.y >= *y + height - 25 && cursor.y <= *y + height + bar_height + 25)
    }).ok_or("no own-window movement path clear of the real cursor")?;
    owner.command(&format!("move {x} {y} {width} {height}"))?;
    wait_for("paint-probe initial position", || geometry_matches(owner.window))?;
    let _hook = install_paint_probe(bar)?;
    unsafe { RedrawWindow(bar, null(), null_mut(), RDW_INVALIDATE | RDW_ALLCHILDREN | RDW_UPDATENOW | RDW_NOERASE); }
    thread::sleep(Duration::from_millis(160));
    let initial = PAINT_PROBE.get().ok_or("paint probe missing")?.snapshot();
    for step in 1..=20 {
        let dx = (step % 6) * 16;
        owner.command(&format!("move {} {y} {width} {height}", x + dx))?;
        wait_for("pure-move geometry", || geometry_matches(owner.window))?;
    }
    thread::sleep(Duration::from_millis(100));
    let counts = PAINT_PROBE.get().unwrap().snapshot().difference(initial);
    println!("PURE_MOVE_METRICS {}", counts.json());
    require(counts.child_positions == 0, "pure movement relaid out child buttons")?;
    require(counts.enables == 0, "pure movement toggled child enabled state")?;
    require(counts.draw_items == 0 && counts.button_paints == 0, "pure movement repainted navigation buttons")?;
    require(counts.erases == 0, "pure movement requested repeated background erase")?;
    Ok(counts)
}

#[derive(Default)]
struct Report { checks: Vec<String>, screenshot: Option<PathBuf>, warnings: Vec<String>, paint_metrics: Option<PaintCounts>, resize_glyphs: Vec<[u64;3]> }

impl Report {
    fn passed(&mut self, name: &str) { println!("PASS {name}"); self.checks.push(name.into()); }
    fn write(&self, directory: &Path, error: Option<&str>) {
        fn quote(value: &str) -> String { format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\"").replace('\r', "\\r").replace('\n', "\\n")) }
        let checks = self.checks.iter().map(|v| quote(v)).collect::<Vec<_>>().join(",");
        let warnings = self.warnings.iter().map(|v| quote(v)).collect::<Vec<_>>().join(",");
        let screenshot = self.screenshot.as_ref().map(|p| quote(&p.to_string_lossy())).unwrap_or_else(|| "null".into());
        let error = error.map(quote).unwrap_or_else(|| "null".into());
        let paint_metrics = self.paint_metrics.map(PaintCounts::json).unwrap_or_else(|| "null".into());
        let resize_glyphs = self.resize_glyphs.iter().map(|pixels| format!("[{},{},{}]",pixels[0],pixels[1],pixels[2])).collect::<Vec<_>>().join(",");
        let content = format!("{{\"passed\":{},\"checks\":[{checks}],\"warnings\":[{warnings}],\"screenshot\":{screenshot},\"pureMoveMessages\":{paint_metrics},\"resizeGlyphBrightPixels\":[{resize_glyphs}],\"error\":{error},\"scope\":\"own mock windows only; no SendInput, cursor movement, foreground requests, ADB or desktop capture\"}}", self.checks.len());
        let _ = fs::write(directory.join("navigation-smoke.json"), content);
    }
}

unsafe fn capture_bar(bar: HWND, output: &Path) -> Result<(), String> {
    let mut pid = 0; GetWindowThreadProcessId(bar, &mut pid);
    require(pid == std::process::id() && bars().contains(&bar), "refusing capture: foreign HWND")?;
    require(BUTTON_IDS.iter().all(|id| !GetDlgItem(bar, *id).is_null()), "refusing capture: missing expected test buttons")?;
    let mut rect: RECT = zeroed(); GetWindowRect(bar, &mut rect);
    let width = rect.right - rect.left; let height = rect.bottom - rect.top;
    require(width > 0 && height > 0 && width <= 8192 && height <= 1024, "invalid own bar capture dimensions")?;
    let source = GetDC(bar); require(!source.is_null(), "GetDC(own bar) failed")?;
    let memory = CreateCompatibleDC(source);
    if memory.is_null() { ReleaseDC(bar, source); return Err("CreateCompatibleDC failed".into()); }
    let info = BITMAPINFO { bmiHeader: BITMAPINFOHEADER { biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width, biHeight: -height, biPlanes: 1, biBitCount: 32, biCompression: BI_RGB, ..zeroed() }, ..zeroed() };
    let mut pixels: *mut c_void = null_mut();
    let bitmap = CreateDIBSection(memory, &info, DIB_RGB_COLORS, &mut pixels, null_mut(), 0);
    if bitmap.is_null() { DeleteDC(memory); ReleaseDC(bar, source); return Err("CreateDIBSection failed".into()); }
    let previous = SelectObject(memory, bitmap);
    let painted = PrintWindow(bar, memory, 0);
    let result = if painted == 0 { Err("PrintWindow(own bar) failed".into()) } else {
        let bytes = std::slice::from_raw_parts(pixels as *const u8, width as usize * height as usize * 4);
        let mut bmp = Vec::with_capacity(54 + bytes.len());
        bmp.extend_from_slice(b"BM"); bmp.extend_from_slice(&(54u32 + bytes.len() as u32).to_le_bytes());
        bmp.extend_from_slice(&[0; 4]); bmp.extend_from_slice(&54u32.to_le_bytes()); bmp.extend_from_slice(&40u32.to_le_bytes());
        bmp.extend_from_slice(&width.to_le_bytes()); bmp.extend_from_slice(&(-height).to_le_bytes());
        bmp.extend_from_slice(&1u16.to_le_bytes()); bmp.extend_from_slice(&32u16.to_le_bytes()); bmp.extend_from_slice(&0u32.to_le_bytes());
        bmp.extend_from_slice(&(bytes.len() as u32).to_le_bytes()); bmp.extend_from_slice(&[0; 16]); bmp.extend_from_slice(bytes);
        fs::write(output, bmp).map_err(|e| e.to_string())
    };
    SelectObject(memory, previous); DeleteObject(bitmap); DeleteDC(memory); ReleaseDC(bar, source);
    result
}

fn validate_capture(path: &Path, dpi: u32) -> Result<[u64;3], String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    require(bytes.len() >= 54 && &bytes[..2] == b"BM", "invalid own-bar bitmap")?;
    let width = i32::from_le_bytes(bytes[18..22].try_into().unwrap());
    let height = i32::from_le_bytes(bytes[22..26].try_into().unwrap()).abs();
    require(width > 0 && height > 0 && bytes.len() >= 54 + width as usize * height as usize * 4, "incomplete own-bar bitmap")?;
    let cell = (width / 3).min((88 * dpi as i32 + 48) / 96).max(1); let left = (width - cell * 3) / 2;
    let mut glyphs = [0u64;3]; let mut dark = 0u64;
    for y in 0..height { for x in 0..width {
        let offset = 54 + (y * width + x) as usize * 4;
        let rgb = &bytes[offset..offset+3];
        if rgb.iter().all(|channel| *channel <= 80) { dark += 1; }
        if rgb.iter().all(|channel| *channel >= 180) && x >= left && x < left + cell * 3 {
            glyphs[((x-left)/cell) as usize] += 1;
        }
    } }
    require(glyphs.iter().all(|count| *count >= 12), "resize capture is missing one or more bright navigation glyphs")?;
    require(dark * 100 >= (width * height) as u64 * 80, "resize capture exposed a light/unpainted background")?;
    Ok(glyphs)
}

fn smoke(report: &mut Report, output: &Path) -> Result<(), String> {
    unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2); }
    let calls = Arc::new(Mutex::new(Vec::<(u64, u8)>::new()));
    let recorded = calls.clone();
    let bar = NavigationBar::start(move |token, button, completion| {
        let key = match button { NavigationButton::Recents => 0, NavigationButton::Back => 1, NavigationButton::Home => 2 };
        recorded.lock().unwrap().push((token, key)); completion(Ok(()));
    })?;
    let controller = bar.controller();
    let mut owner = Helper::start()?;
    let foreground_before_cover = unsafe { GetForegroundWindow() };
    owner.command("cover")?;
    require(above_in_z_order(owner.cover, owner.window), "own cover did not start above own mock owner")?;
    require(unsafe { GetForegroundWindow() } == foreground_before_cover, "creating own cover changed foreground")?;
    controller.set_target(Some(Target { process_id: owner.pid(), token: 101, running: true }));
    wait_for("own mock attachment", || bar_visible(owner.window))?;
    wait_for("new popup stays below own covering window", || bar_for(owner.window).is_some_and(|navigation|
        above_in_z_order(owner.cover, navigation) && above_in_z_order(navigation, owner.window)))?;
    require(unsafe { GetForegroundWindow() } == foreground_before_cover, "attaching below own cover changed foreground")?;
    report.passed("new bar stays below own covering window and above owner without activation");
    owner.command("uncover")?;
    let navigation = bar_for(owner.window).ok_or("bar missing")?;
    unsafe {
        let styles = GetWindowLongPtrW(navigation, GWL_EXSTYLE) as u32;
        require(styles & WS_EX_NOACTIVATE != 0 && styles & WS_EX_TOOLWINDOW != 0, "bar missing NOACTIVATE/TOOLWINDOW styles")?;
        require(GetWindowLongPtrW(navigation, GWL_STYLE) as u32 & WS_POPUP != 0, "bar must be an owned popup")?;
        for id in BUTTON_IDS {
            let button = GetDlgItem(navigation, id); require(!button.is_null(), "expected three native child buttons")?;
            let mut label = [0u16; 80]; require(GetWindowTextW(button, label.as_mut_ptr(), label.len() as i32) > 0, "native button missing accessibility name")?;
        }
        let before = GetForegroundWindow(); let mut result = 0usize;
        require(SendMessageTimeoutW(navigation, WM_MOUSEACTIVATE, 0, 0, SMTO_ABORTIFHUNG, 1000, &mut result) != 0 && result == MA_NOACTIVATE as usize, "WM_MOUSEACTIVATE must return MA_NOACTIVATE")?;
        require(GetForegroundWindow() == before, "mouse activation smoke changed foreground")?;
    }
    report.passed("owned popup, noactivate/toolwindow styles, native labelled buttons");
    wait_for("initial bottom attachment", || geometry_matches(owner.window))?;
    report.passed("normal-window bottom geometry and 48 DIP height");
    report.paint_metrics = Some(measure_pure_moves(&mut owner)?);
    report.passed("20 pure moves keep child layout, button paint, enable and erase messages at zero");

    let work = unsafe { primary_work_area()? };
    let available_height = work.bottom - work.top;
    let h = (available_height - 200).clamp(140, 300);
    for (dx, dy, width, height) in [(60, 48, 300, h), (95, 74, 350, h + 25), (45, 40, 275, h - 20)] {
        owner.command(&format!("move {} {} {width} {height}", work.left + dx, work.top + dy))?;
        wait_for("follow own mock move/resize", || geometry_matches(owner.window))?;
        let navigation = bar_for(owner.window).ok_or("resized bar missing")?;
        let bitmap = output.join("navigation-bar-resize.bmp");
        unsafe { capture_bar(navigation, &bitmap)?; }
        report.resize_glyphs.push(validate_capture(&bitmap, unsafe { GetDpiForWindow(navigation).max(96) })?);
    }
    report.passed("follows own mock movement and resizing");
    report.passed("three resize captures retain all glyphs and a complete dark background");

    let navigation = bar_for(owner.window).ok_or("bar missing after movement")?;
    for (index, id) in BUTTON_IDS.into_iter().enumerate() {
        click(navigation, id)?;
        wait_for("navigation callback", || calls.lock().unwrap().len() == index + 1)?;
    }
    require(*calls.lock().unwrap() == vec![(101, 0), (101, 1), (101, 2)], "incorrect navigation callback order/token")?;
    report.passed("BM_CLICK dispatches Recents/Back/Home once without changing foreground");

    for (index, id) in BUTTON_IDS.into_iter().enumerate() {
        click_mouse_messages(navigation, id, owner.window)?;
        wait_for("client-coordinate mouse callback", || calls.lock().unwrap().len() == index + 4)?;
    }
    require(*calls.lock().unwrap() == vec![(101, 0), (101, 1), (101, 2), (101, 0), (101, 1), (101, 2)],
        "mouse-message path produced duplicate, missing or incorrect actions")?;
    report.passed("client-center mouse down/up preserves foreground and owner GUI-thread focus");

    let capture = output.join("navigation-bar-smoke.bmp");
    match unsafe { capture_bar(navigation, &capture) } {
        Ok(()) => { report.screenshot = Some(capture); report.passed("PrintWindow captures only the verified own navigation bar"); }
        Err(error) => { report.warnings.push(format!("Optional PrintWindow capture: {error}")); }
    }

    owner.command("hide")?; wait_for("hidden owner hides bar", || !bar_visible(owner.window))?;
    owner.command("show")?; wait_for("shown owner restores bar", || bar_visible(owner.window))?;
    report.passed("explicit owner hide/show visibility");
    owner.command("minimize")?;
    wait_for("own owner minimized", || unsafe { IsIconic(owner.window) != 0 })?;
    wait_for("minimized owner hides bar", || !bar_visible(owner.window))?;
    owner.command("restore")?;
    wait_for("own owner restored", || unsafe { IsIconic(owner.window) == 0 } && bar_visible(owner.window))?;
    report.passed("nonactivating mock minimize/restore visibility");

    controller.set_target(Some(Target { process_id: owner.pid(), token: 101, running: false }));
    wait_for("non-running session disables buttons", || bar_for(owner.window).is_some_and(|hwnd| BUTTON_IDS.iter().all(|id| unsafe { IsWindowEnabled(GetDlgItem(hwnd, *id)) == 0 })))?;
    let count = calls.lock().unwrap().len();
    let navigation = bar_for(owner.window).ok_or("non-running bar missing")?;
    for id in BUTTON_IDS { click(navigation, id)?; }
    thread::sleep(Duration::from_millis(150));
    require(calls.lock().unwrap().len() == count, "disabled navigation dispatched an action")?;
    controller.set_target(Some(Target { process_id: owner.pid(), token: 202, running: true }));
    wait_for("new token enables buttons", || bar_for(owner.window).is_some_and(|hwnd| BUTTON_IDS.iter().all(|id| unsafe { IsWindowEnabled(GetDlgItem(hwnd, *id)) != 0 })))?;
    click(bar_for(owner.window).ok_or("new token bar missing")?, 101)?;
    wait_for("new token callback", || calls.lock().unwrap().len() == count + 1)?;
    require(calls.lock().unwrap().last().copied() == Some((202, 1)), "token replacement dispatched stale token")?;
    report.passed("running=false rejects clicks; replacement token is used");

    let mut replacement = Helper::start()?;
    controller.set_target(Some(Target { process_id: replacement.pid(), token: 303, running: true }));
    wait_for("new process replaces old owner", || bar_visible(replacement.window) && bar_for(owner.window).is_none() && bars().len() == 1)?;
    click(bar_for(replacement.window).ok_or("replacement bar missing")?, 102)?;
    wait_for("replacement callback", || calls.lock().unwrap().last().copied() == Some((303, 2)))?;
    report.passed("new PID/session gets one bar; old owner is detached");

    replacement.command("destroy")?;
    wait_for("destroyed mock owner removes popup", || bars().is_empty())?;
    report.passed("owner destruction removes the owned navigation popup");
    controller.set_target(None); drop(bar);
    wait_for("navigation worker shutdown", || controller.is_stopped())?;
    require(bars().is_empty(), "navigation popup leaked after shutdown")?;
    report.passed("target clearing and Drop stop the worker without ghost windows");
    report.warnings.push("No real cursor movement or foreground requests were used. Real pointer activation, overlay hover, multi-monitor DPI transitions and actual Android input still require their separate checks.".into());
    Ok(())
}

fn main() {
    if std::env::args().any(|arg| arg == "--owner-child") {
        if let Err(error) = unsafe { run_helper() } { eprintln!("mock owner failed: {error}"); std::process::exit(2); }
        return;
    }
    let output = std::env::var_os("QZZ_NAV_OUTPUT_DIR").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.work"));
    if let Err(error) = fs::create_dir_all(&output) { eprintln!("cannot create report directory: {error}"); std::process::exit(1); }
    let mut report = Report::default();
    let result = smoke(&mut report, &output);
    report.write(&output, result.as_ref().err().map(String::as_str));
    if let Err(error) = result { eprintln!("FAIL {error}"); std::process::exit(1); }
    println!("{} navigation smoke checks passed", report.checks.len());
}
