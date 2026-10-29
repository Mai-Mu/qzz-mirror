//! A small, non-activating companion for the scrcpy window we own. No hooks or
//! window procedures are injected into scrcpy, and its input handling is intact.
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub process_id: u32,
    pub token: u64,
    pub running: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationButton { Recents, Back, Home }

pub type Completion = Box<dyn FnOnce(Result<(), String>) + Send>;
type Action = Arc<dyn Fn(u64, NavigationButton, Completion) + Send + Sync>;

#[cfg(windows)]
#[allow(unused_imports)] // Public controller type is also consumed by the isolated harness.
pub use platform::{NavigationBar, NavigationController};

#[cfg(windows)]
mod platform {
    use super::*;
    use std::result::Result::Ok;
    use std::{
        cell::{Cell, RefCell}, ffi::c_void, mem::{size_of, zeroed},
        ptr::{null, null_mut}, sync::{atomic::{AtomicBool, Ordering}, mpsc, Mutex},
        thread::{self, JoinHandle}, time::{Duration, Instant},
    };
    use windows_sys::{core::w, Win32::{
        Foundation::*, Graphics::{Dwm::*, Gdi::*, GdiPlus::*},
        System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::{Accessibility::*, Controls::*, HiDpi::*, Input::KeyboardAndMouse::*, Shell::{SetWindowSubclass, RemoveWindowSubclass, DefSubclassProc}, WindowsAndMessaging::*},
    }};

    const CONTROLLER_CLASS: *const u16 = w!("QzzMirrorNavigationController");
    const BAR_CLASS: *const u16 = w!("QzzMirrorNavigationBar");
    const SYNC: u32 = WM_APP + 10;
    const PRESS: u32 = WM_APP + 11;
    const RELEASE: u32 = WM_APP + 12;
    const TICK: usize = 1;
    const FIRST_BUTTON: usize = 100;
    const HEIGHT: i32 = 48;
    const LABELS: [*const u16; 3] = [w!("最近任务"), w!("返回"), w!("主页")];
    thread_local! { static CONTROLLER: Cell<HWND> = const { Cell::new(null_mut()) }; }

    #[derive(Default)]
    struct Feedback { generation: u64, busy: bool, error_at: Option<Instant> }

    struct Shared {
        target: Mutex<Option<Target>>,
        feedback: Mutex<Feedback>,
        stopped: AtomicBool,
    }

    #[derive(Clone)]
    pub struct NavigationController { shared: Arc<Shared> }

    impl NavigationController {
        pub fn set_target(&self, target: Option<Target>) {
            if let Ok(mut current) = self.shared.target.lock() { *current = target; }
        }
        pub fn is_stopped(&self) -> bool { self.shared.stopped.load(Ordering::Acquire) }
    }

    pub struct NavigationBar {
        controller: NavigationController,
        thread_id: u32,
        thread: Option<JoinHandle<()>>,
    }

    impl NavigationBar {
        pub fn start(callback: impl Fn(u64, NavigationButton, Completion) + Send + Sync + 'static) -> Result<Self, String> {
            let shared = Arc::new(Shared {
                target: Mutex::new(None), feedback: Mutex::new(Feedback::default()),
                stopped: AtomicBool::new(false),
            });
            let controller = NavigationController { shared: shared.clone() };
            let (ready_tx, ready_rx) = mpsc::sync_channel(1);
            let callback: Action = Arc::new(callback);
            let thread = thread::Builder::new().name("mirror-navigation".into()).spawn(move || {
                let result = unsafe { run(shared.clone(), callback, &ready_tx) };
                shared.stopped.store(true, Ordering::Release);
                if let Err(error) = result { let _ = ready_tx.send(Err(error)); }
            }).map_err(|e| format!("无法创建导航栏：{e}"))?;
            match ready_rx.recv_timeout(Duration::from_secs(5)) {
                Ok(Ok(thread_id)) => Ok(Self { controller, thread_id, thread: Some(thread) }),
                result => {
                    controller.shared.stopped.store(true, Ordering::Release);
                    // A failed initialization has already returned from the thread.
                    if thread.is_finished() { let _ = thread.join(); }
                    Err(match result { Ok(Err(error)) => error, _ => "导航栏初始化超时。".into() })
                }
            }
        }
        pub fn controller(&self) -> NavigationController { self.controller.clone() }
    }

    impl Drop for NavigationBar {
        fn drop(&mut self) {
            self.controller.shared.stopped.store(true, Ordering::Release);
            unsafe { PostThreadMessageW(self.thread_id, WM_QUIT, 0, 0); }
            if let Some(thread) = self.thread.take() { let _ = thread.join(); }
        }
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    struct Bounds { left: i32, top: i32, right: i32, bottom: i32 }
    impl Bounds {
        fn width(self) -> i32 { self.right - self.left }
        fn height(self) -> i32 { self.bottom - self.top }
        fn contains(self, point: POINT) -> bool {
            point.x >= self.left && point.x < self.right && point.y >= self.top && point.y < self.bottom
        }
    }
    impl From<RECT> for Bounds {
        fn from(r: RECT) -> Self { Self { left: r.left, top: r.top, right: r.right, bottom: r.bottom } }
    }

    // Keep normal windows untouched. When there is no room outside the window,
    // reveal a temporary overlay from the visible bottom edge instead.
    fn placement(frame: Bounds, client: Bounds, work: Bounds, monitor: Bounds, height: i32, zoomed: bool) -> (Bounds, bool) {
        let fullscreen = client.left <= monitor.left && client.top <= monitor.top
            && client.right >= monitor.right && client.bottom >= monitor.bottom;
        if !zoomed && !fullscreen && frame.bottom + height <= work.bottom {
            return (Bounds { left: frame.left, top: frame.bottom, right: frame.right, bottom: frame.bottom + height }, false);
        }
        let visible = if fullscreen { monitor } else { work };
        let bottom = client.bottom.min(visible.bottom);
        (Bounds {
            left: client.left.max(visible.left), right: client.right.min(visible.right),
            top: (bottom - height).max(client.top.max(visible.top)), bottom,
        }, true)
    }

    // Painting can reenter while sync holds Ui's mutable borrow (for example
    // EnableWindow or a child resize can synchronously send WM_DRAWITEM). Keep
    // the small, immutable painting state separate so a valid paint is never
    // silently skipped because geometry/action state is already borrowed.
    #[derive(Clone, Copy)]
    struct PaintSnapshot { dpi: u32, hover: Option<usize>, error: bool }

    struct WindowState { ui: RefCell<Ui>, paint: Cell<PaintSnapshot> }

    struct PaintBuffer { target: HDC, dc: HDC, bitmap: HBITMAP, previous: HGDIOBJ, width: i32, height: i32 }

    impl PaintBuffer {
        unsafe fn new(target: HDC, width: i32, height: i32) -> Option<Self> {
            if width <= 0 || height <= 0 { return None; }
            let dc = CreateCompatibleDC(target);
            if dc.is_null() { return None; }
            let bitmap = CreateCompatibleBitmap(target, width, height);
            if bitmap.is_null() { DeleteDC(dc); return None; }
            let previous = SelectObject(dc, bitmap);
            if previous.is_null() || previous as isize == -1 {
                DeleteObject(bitmap); DeleteDC(dc); return None;
            }
            Some(Self { target, dc, bitmap, previous, width, height })
        }
        unsafe fn present(&self, x: i32, y: i32) {
            BitBlt(self.target, x, y, self.width, self.height, self.dc, 0, 0, SRCCOPY);
        }
    }

    impl Drop for PaintBuffer {
        fn drop(&mut self) { unsafe { SelectObject(self.dc, self.previous); DeleteObject(self.bitmap); DeleteDC(self.dc); } }
    }

    struct Ui {
        shared: Arc<Shared>, action: Action,
        state_ptr: *const c_void,
        owner: HWND, bar: HWND, buttons: [HWND; 3], tooltip: HWND,
        target: Option<Target>, hook: HWINEVENTHOOK,
        previous_bounds: Option<Bounds>, dpi: u32,
        hover: Option<usize>, pressed: Option<(usize, u64)>, reveal_until: Option<Instant>,
        last_visual: (bool, bool),
    }

    impl Ui {
        fn new(shared: Arc<Shared>, action: Action) -> Self {
            Self {
                shared, action, state_ptr: null(), owner: null_mut(), bar: null_mut(),
                buttons: [null_mut(); 3], tooltip: null_mut(), target: None, hook: null_mut(),
                previous_bounds: None, dpi: 96, hover: None, pressed: None,
                reveal_until: None, last_visual: (false, false),
            }
        }

        unsafe fn detach(&mut self) {
            if !self.hook.is_null() { UnhookWinEvent(self.hook); self.hook = null_mut(); }
            if IsWindow(self.bar) != 0 { DestroyWindow(self.bar); }
            self.owner = null_mut(); self.bar = null_mut(); self.buttons = [null_mut(); 3];
            self.tooltip = null_mut(); self.target = None; self.previous_bounds = None;
            self.hover = None; self.pressed = None; self.reveal_until = None;
            if let Ok(mut feedback) = self.shared.feedback.lock() {
                feedback.generation = feedback.generation.wrapping_add(1);
                feedback.busy = false; feedback.error_at = None;
            }
        }

        unsafe fn sync(&mut self) {
            let target = self.shared.target.lock().ok().and_then(|value| *value);
            let Some(target) = target else { self.detach(); return; };
            let same_session = self.target.is_some_and(|current| current.token == target.token && current.process_id == target.process_id);
            if !same_session || !belongs_to(self.owner, target.process_id) || IsWindow(self.bar) == 0 {
                self.detach();
                self.owner = find_owner(target.process_id);
                if self.owner.is_null() { return; }
                self.target = Some(target);
                if !self.create_bar() { self.detach(); return; }
                self.hook = SetWinEventHook(EVENT_OBJECT_DESTROY, EVENT_OBJECT_LOCATIONCHANGE,
                    null_mut(), Some(owner_event), target.process_id, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS);
            }
            self.target = Some(target);
            let mut cloaked: u32 = 0;
            DwmGetWindowAttribute(self.owner, DWMWA_CLOAKED as _, &mut cloaked as *mut _ as _, size_of::<u32>() as _);
            if IsWindowVisible(self.owner) == 0 || IsIconic(self.owner) != 0 || cloaked != 0 {
                ShowWindow(self.bar, SW_HIDE); self.reveal_until = None; return;
            }
            let mut frame: RECT = zeroed();
            if DwmGetWindowAttribute(self.owner, DWMWA_EXTENDED_FRAME_BOUNDS as _, &mut frame as *mut _ as _, size_of::<RECT>() as _) < 0 {
                if GetWindowRect(self.owner, &mut frame) == 0 { ShowWindow(self.bar, SW_HIDE); return; }
            }
            let mut client: RECT = zeroed();
            if GetClientRect(self.owner, &mut client) == 0 { return; }
            let mut origin = POINT { x: 0, y: 0 };
            if ClientToScreen(self.owner, &mut origin) == 0 { return; }
            client.left += origin.x; client.right += origin.x; client.top += origin.y; client.bottom += origin.y;
            let mut monitor: MONITORINFO = zeroed(); monitor.cbSize = size_of::<MONITORINFO>() as _;
            if GetMonitorInfoW(MonitorFromWindow(self.owner, MONITOR_DEFAULTTONEAREST), &mut monitor) == 0 { return; }
            let dpi = GetDpiForWindow(self.owner).max(96);
            let height = (HEIGHT * dpi as i32 + 48) / 96;
            let (bounds, overlay) = placement(frame.into(), client.into(), monitor.rcWork.into(), monitor.rcMonitor.into(), height, IsZoomed(self.owner) != 0);
            if bounds.width() < 3 || bounds.height() < 1 { ShowWindow(self.bar, SW_HIDE); return; }
            let mut cursor: POINT = zeroed();
            let cursor_known = GetCursorPos(&mut cursor) != 0;
            let root = if cursor_known { GetAncestor(WindowFromPoint(cursor), GA_ROOT) } else { null_mut() };
            let hot_edge = cursor_known && root == self.owner && bounds.contains(cursor)
                && cursor.y >= bounds.bottom - ((6 * dpi as i32 + 48) / 96);
            let on_bar = cursor_known && root == self.bar && bounds.contains(cursor);
            let capture = GetCapture();
            let pressing = self.pressed.is_some() && self.buttons.contains(&capture);
            if hot_edge || on_bar || pressing { self.reveal_until = Some(Instant::now() + Duration::from_millis(350)); }
            let show = !overlay || self.reveal_until.is_some_and(|until| Instant::now() < until);
            let position_changed = self.previous_bounds.is_none_or(|previous|
                previous.left != bounds.left || previous.top != bounds.top);
            let size_changed = self.dpi != dpi || self.previous_bounds.is_none_or(|previous|
                previous.width() != bounds.width() || previous.height() != bounds.height());
            let hover = if on_bar { self.buttons.iter().position(|&button| {
                let mut r = zeroed(); GetWindowRect(button, &mut r) != 0 && Bounds::from(r).contains(cursor)
            }) } else { None };
            let feedback = self.shared.feedback.lock().ok();
            let visual = (target.running && feedback.as_ref().is_none_or(|f| !f.busy),
                feedback.as_ref().and_then(|f| f.error_at).is_some_and(|at| at.elapsed() < Duration::from_millis(800)));
            drop(feedback);
            let previous_hover = self.hover;
            let error_changed = self.last_visual.1 != visual.1;
            self.dpi = dpi; self.last_visual = visual; self.hover = hover;
            let state = &*(self.state_ptr as *const WindowState);
            state.paint.set(PaintSnapshot { dpi, hover, error: visual.1 });

            if position_changed || size_changed {
                let mut flags = SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOOWNERZORDER;
                if !size_changed { flags |= SWP_NOSIZE; }
                if !position_changed { flags |= SWP_NOMOVE; }
                if SetWindowPos(self.bar, null_mut(), bounds.left, bounds.top, bounds.width(), bounds.height(), flags) != 0 {
                    self.previous_bounds = Some(bounds);
                }
                // A move retains the existing composed pixels. Repositioning
                // or invalidating each child on every drag step causes flicker.
                if size_changed {
                    self.layout_buttons(bounds.width(), bounds.height());
                    InvalidateRect(self.bar, null(), 0);
                }
            }
            for (index, &button) in self.buttons.iter().enumerate() {
                let enabled_changed = (IsWindowEnabled(button) != 0) != visual.0;
                if enabled_changed { EnableWindow(button, visual.0 as _); }
                let hover_changed = previous_hover != hover && (previous_hover == Some(index) || hover == Some(index));
                if size_changed || enabled_changed || error_changed || hover_changed { InvalidateRect(button, null(), 0); }
            }
            if show && IsWindowVisible(self.bar) == 0 { ShowWindow(self.bar, SW_SHOWNOACTIVATE); }
            else if !show && IsWindowVisible(self.bar) != 0 { ShowWindow(self.bar, SW_HIDE); }
            if show { self.sync_z_order(); }
        }

        unsafe fn create_bar(&mut self) -> bool {
            let instance = GetModuleHandleW(null());
            let topmost = (GetWindowLongPtrW(self.owner, GWL_EXSTYLE) as u32) & WS_EX_TOPMOST;
            self.bar = CreateWindowExW(WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | topmost,
                BAR_CLASS, w!("安卓导航"), WS_POPUP | WS_CLIPCHILDREN,
                0, 0, 1, 1, self.owner, null_mut(), instance, self.state_ptr);
            if self.bar.is_null() { return false; }
            let corners: u32 = DWMWCP_DONOTROUND as _;
            DwmSetWindowAttribute(self.bar, DWMWA_WINDOW_CORNER_PREFERENCE as _, &corners as *const _ as _, size_of::<u32>() as _);
            for (index, label) in LABELS.into_iter().enumerate() {
                let button = CreateWindowExW(WS_EX_NOACTIVATE, w!("BUTTON"), label,
                    WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32, 0, 0, 1, 1,
                    self.bar, (FIRST_BUTTON + index) as HMENU, instance, null());
                if button.is_null() { return false; }
                self.buttons[index] = button;
                if SetWindowSubclass(button, Some(button_proc), 1, index) == 0 { return false; }
            }
            self.tooltip = CreateWindowExW(WS_EX_NOACTIVATE | WS_EX_TOPMOST, TOOLTIPS_CLASSW, null(),
                WS_POPUP | TTS_ALWAYSTIP | TTS_NOPREFIX, 0, 0, 0, 0, self.bar, null_mut(), instance, null());
            if !self.tooltip.is_null() {
                for (index, &button) in self.buttons.iter().enumerate() {
                    let mut info: TTTOOLINFOW = zeroed();
                    info.cbSize = size_of::<TTTOOLINFOW>() as _;
                    info.uFlags = TTF_IDISHWND | TTF_SUBCLASS;
                    info.hwnd = self.bar; info.uId = button as usize; info.lpszText = LABELS[index] as *mut _;
                    SendMessageW(self.tooltip, TTM_ADDTOOLW, 0, &info as *const _ as _);
                }
            }
            true
        }

        unsafe fn layout_buttons(&self, width: i32, height: i32) {
            let cell = (width / 3).min((88 * self.dpi as i32 + 48) / 96).max(1);
            let left = (width - cell * 3) / 2;
            for (index, &button) in self.buttons.iter().enumerate() {
                SetWindowPos(button, null_mut(), left + index as i32 * cell, 1, cell, height - 1,
                    SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOREDRAW);
            }
        }

        unsafe fn sync_z_order(&self) {
            // Showing a new popup must not lift it above an unrelated foreground
            // app when its scrcpy owner is behind that app. Keep the pair in the
            // owner's existing Z-order group; never promote the owner itself.
            let above_owner = GetWindow(self.owner, GW_HWNDPREV);
            if above_owner == self.bar { return; }
            let same_group = !above_owner.is_null()
                && (GetWindowLongPtrW(above_owner, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST)
                    == (GetWindowLongPtrW(self.owner, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST);
            let after = if same_group { above_owner } else { HWND_TOP };
            SetWindowPos(self.bar, after, 0, 0, 0, 0,
                SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOMOVE | SWP_NOSIZE);
        }

        fn press(&mut self, index: usize) {
            self.pressed = self.target.filter(|target| target.running).map(|target| (index, target.token));
        }

        fn release(&mut self, index: usize, inside: bool) {
            let pressed = self.pressed.take();
            if !inside { return; }
            let Some(target) = self.target.filter(|target| target.running) else { return; };
            if pressed != Some((index, target.token)) { return; }
            let button = match index { 0 => NavigationButton::Recents, 1 => NavigationButton::Back, 2 => NavigationButton::Home, _ => return };
            let generation = {
                let Ok(mut feedback) = self.shared.feedback.lock() else { return; };
                if feedback.busy { return; }
                feedback.generation = feedback.generation.wrapping_add(1);
                feedback.busy = true; feedback.error_at = None;
                feedback.generation
            };
            let shared = self.shared.clone();
            (self.action)(target.token, button, Box::new(move |result| {
                if let Ok(mut feedback) = shared.feedback.lock() {
                    if feedback.generation == generation {
                        feedback.busy = false;
                        feedback.error_at = result.err().map(|_| Instant::now());
                    }
                }
            }));
        }

        unsafe fn draw_button(item: &DRAWITEMSTRUCT, paint: PaintSnapshot) {
            let index = (item.CtlID as usize).saturating_sub(FIRST_BUTTON);
            if index > 2 { return; }
            let width = item.rcItem.right - item.rcItem.left;
            let height = item.rcItem.bottom - item.rcItem.top;
            let buffer = PaintBuffer::new(item.hDC, width, height);
            let dc = buffer.as_ref().map_or(item.hDC, |buffer| buffer.dc);
            let rect = if buffer.is_some() { RECT { left: 0, top: 0, right: width, bottom: height } } else { item.rcItem };
            // Background and glyph become visible together, including during
            // synchronous owner-draw messages raised by layout/EnableWindow.
            fill(dc, &rect, 0x181818);
            let scale = paint.dpi as f32 / 96.0;
            let cx = (rect.left + rect.right) as f32 / 2.0;
            let cy = (rect.top + rect.bottom) as f32 / 2.0;
            let mut graphics = null_mut();
            if GdipCreateFromHDC(dc, &mut graphics) != 0 {
                if let Some(buffer) = buffer.as_ref() { buffer.present(item.rcItem.left, item.rcItem.top); }
                return;
            }
            GdipSetSmoothingMode(graphics, SmoothingModeAntiAlias);
            GdipSetPixelOffsetMode(graphics, PixelOffsetModeHalf);
            let enabled = IsWindowEnabled(item.hwndItem) != 0;
            if enabled && (paint.hover == Some(index) || item.itemState & ODS_SELECTED != 0) {
                let mut brush = null_mut();
                GdipCreateSolidFill(if item.itemState & ODS_SELECTED != 0 { 0xff414141 } else { 0xff2c2c2c }, &mut brush);
                if !brush.is_null() {
                    let diameter = 36.0 * scale;
                    GdipFillEllipse(graphics, brush as _, cx - diameter / 2.0, cy - diameter / 2.0, diameter, diameter);
                    GdipDeleteBrush(brush as _);
                }
            }
            let color = if !enabled { 0xff707070 } else if paint.error { 0xfff3a09d } else { 0xffededed };
            let mut pen = null_mut();
            GdipCreatePen1(color, 1.8 * scale, UnitPixel, &mut pen);
            if !pen.is_null() {
                match index {
                    0 => { let side = 14.0 * scale; GdipDrawRectangle(graphics, pen, cx - side / 2.0, cy - side / 2.0, side, side); }
                    1 => {
                        let points = [PointF { X: cx - 6.0 * scale, Y: cy }, PointF { X: cx + 5.5 * scale, Y: cy - 7.5 * scale }, PointF { X: cx + 5.5 * scale, Y: cy + 7.5 * scale }];
                        GdipDrawPolygon(graphics, pen, points.as_ptr(), points.len() as _);
                    }
                    _ => { let diameter = 17.0 * scale; GdipDrawEllipse(graphics, pen, cx - diameter / 2.0, cy - diameter / 2.0, diameter, diameter); }
                }
                GdipDeletePen(pen);
            }
            GdipDeleteGraphics(graphics);
            if let Some(buffer) = buffer.as_ref() { buffer.present(item.rcItem.left, item.rcItem.top); }
        }
    }

    unsafe fn fill(dc: HDC, rect: &RECT, color: u32) {
        let brush = CreateSolidBrush(color);
        if !brush.is_null() { FillRect(dc, rect, brush); DeleteObject(brush); }
    }

    unsafe fn paint_bar(dc: HDC, rect: RECT) {
        let width = rect.right - rect.left; let height = rect.bottom - rect.top;
        let buffer = PaintBuffer::new(dc, width, height);
        let target = buffer.as_ref().map_or(dc, |buffer| buffer.dc);
        let mut local = if buffer.is_some() { RECT { left: 0, top: 0, right: width, bottom: height } } else { rect };
        fill(target, &local, 0x181818);
        local.bottom = local.top + 1; fill(target, &local, 0x303030);
        if let Some(buffer) = buffer.as_ref() { buffer.present(rect.left, rect.top); }
    }

    unsafe fn belongs_to(window: HWND, process_id: u32) -> bool {
        let mut actual = 0;
        !window.is_null() && IsWindow(window) != 0
            && GetWindowThreadProcessId(window, &mut actual) != 0 && actual == process_id
    }

    struct Search { process_id: u32, found: HWND }
    unsafe extern "system" fn enumerate(window: HWND, parameter: LPARAM) -> windows_sys::core::BOOL {
        let search = &mut *(parameter as *mut Search);
        if !belongs_to(window, search.process_id) || IsWindowVisible(window) == 0 || !GetWindow(window, GW_OWNER).is_null() { return 1; }
        let mut class = [0u16; 128];
        let count = GetClassNameW(window, class.as_mut_ptr(), class.len() as _) as usize;
        if String::from_utf16_lossy(&class[..count]).starts_with("SDL_app") {
            search.found = window; return 0;
        }
        1
    }
    unsafe fn find_owner(process_id: u32) -> HWND {
        let mut search = Search { process_id, found: null_mut() };
        EnumWindows(Some(enumerate), &mut search as *mut _ as _);
        search.found
    }

    unsafe extern "system" fn owner_event(_: HWINEVENTHOOK, _: u32, _: HWND, object: i32, _: i32, _: u32, _: u32) {
        if object == OBJID_WINDOW {
            CONTROLLER.with(|window| { if !window.get().is_null() { PostMessageW(window.get(), SYNC, 0, 0); } });
        }
    }

    unsafe extern "system" fn button_proc(window: HWND, message: u32, w: WPARAM, l: LPARAM, _: usize, index: usize) -> LRESULT {
        match message {
            WM_ERASEBKGND => 1, // The buffered owner-draw pass paints the entire button.
            WM_MOUSEACTIVATE => MA_NOACTIVATE as _,
            WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
                if IsWindowEnabled(window) != 0 {
                    SetCapture(window);
                    SendMessageW(window, BM_SETSTATE, 1, 0);
                    SendMessageW(GetParent(window), PRESS, index, 0);
                }
                0
            }
            WM_LBUTTONUP => {
                let was_pressed = SendMessageW(window, BM_GETSTATE, 0, 0) as u32 & BST_PUSHED != 0;
                let cursor = POINT { x: l as u16 as i16 as i32, y: (l >> 16) as u16 as i16 as i32 };
                let mut rect: RECT = zeroed();
                let inside = GetClientRect(window, &mut rect) != 0 && Bounds::from(rect).contains(cursor);
                SendMessageW(window, BM_SETSTATE, 0, 0);
                if was_pressed { SendMessageW(GetParent(window), RELEASE, index, inside as _); }
                if GetCapture() == window { ReleaseCapture(); }
                0
            }
            BM_CLICK => {
                // Also support accessibility Invoke without the stock BUTTON's
                // SetFocus/activation path or synthetic cursor movement.
                if IsWindowEnabled(window) != 0 {
                    SendMessageW(GetParent(window), PRESS, index, 0);
                    SendMessageW(GetParent(window), RELEASE, index, 1);
                }
                0
            }
            WM_CAPTURECHANGED => { SendMessageW(window, BM_SETSTATE, 0, 0); 0 }
            WM_NCDESTROY => { RemoveWindowSubclass(window, Some(button_proc), 1); DefSubclassProc(window, message, w, l) }
            _ => DefSubclassProc(window, message, w, l),
        }
    }

    unsafe extern "system" fn window_proc(window: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
        if message == WM_NCCREATE {
            let create = &*(l as *const CREATESTRUCTW);
            SetWindowLongPtrW(window, GWLP_USERDATA, create.lpCreateParams as _);
        }
        match message {
            WM_MOUSEACTIVATE => return MA_NOACTIVATE as _,
            WM_ERASEBKGND => return 1,
            WM_PAINT => {
                let mut paint = zeroed(); let dc = BeginPaint(window, &mut paint);
                let mut rect: RECT = zeroed(); GetClientRect(window, &mut rect);
                paint_bar(dc, rect);
                EndPaint(window, &paint); return 0;
            }
            _ => {}
        }
        let pointer = GetWindowLongPtrW(window, GWLP_USERDATA) as *const WindowState;
        if !pointer.is_null() {
            let state = &*pointer;
            match message {
                WM_TIMER | SYNC | WM_DPICHANGED => {
                    if let Ok(mut ui) = state.ui.try_borrow_mut() {
                        if ui.shared.stopped.load(Ordering::Acquire) { PostQuitMessage(0); }
                        else { ui.sync(); }
                    }
                    return 0;
                }
                WM_DRAWITEM if l != 0 => {
                    Ui::draw_button(&*(l as *const DRAWITEMSTRUCT), state.paint.get());
                    return 1;
                }
                PRESS => { if let Ok(mut ui) = state.ui.try_borrow_mut() { ui.press(w); } return 0; }
                RELEASE => { if let Ok(mut ui) = state.ui.try_borrow_mut() { ui.release(w, l != 0); } return 0; }
                _ => {}
            }
        }
        DefWindowProcW(window, message, w, l)
    }

    unsafe fn run(shared: Arc<Shared>, action: Action, ready: &mpsc::SyncSender<Result<u32, String>>) -> Result<(), String> {
        SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let mut gdiplus_token = 0;
        let input = GdiplusStartupInput { GdiplusVersion: 1, ..zeroed() };
        if GdiplusStartup(&mut gdiplus_token, &input, null_mut()) != 0 { return Err("无法初始化导航图形。".into()); }
        let controls = INITCOMMONCONTROLSEX { dwSize: size_of::<INITCOMMONCONTROLSEX>() as _, dwICC: ICC_WIN95_CLASSES };
        InitCommonControlsEx(&controls);
        let instance = GetModuleHandleW(null());
        for class_name in [CONTROLLER_CLASS, BAR_CLASS] {
            let class = WNDCLASSW { lpfnWndProc: Some(window_proc), hInstance: instance,
                hCursor: LoadCursorW(null_mut(), IDC_ARROW), lpszClassName: class_name, ..zeroed() };
            if RegisterClassW(&class) == 0 && GetLastError() != ERROR_CLASS_ALREADY_EXISTS {
                GdiplusShutdown(gdiplus_token); return Err("无法注册导航窗口。".into());
            }
        }
        let state = Box::new(WindowState {
            ui: RefCell::new(Ui::new(shared.clone(), action)),
            paint: Cell::new(PaintSnapshot { dpi: 96, hover: None, error: false }),
        });
        let pointer = &*state as *const WindowState as *const c_void;
        state.ui.borrow_mut().state_ptr = pointer;
        let controller = CreateWindowExW(0, CONTROLLER_CLASS, w!("Qzz Mirror Navigation Controller"), 0,
            0, 0, 0, 0, HWND_MESSAGE, null_mut(), instance, pointer);
        if controller.is_null() { GdiplusShutdown(gdiplus_token); return Err("无法创建导航窗口。".into()); }
        CONTROLLER.with(|slot| slot.set(controller));
        if SetTimer(controller, TICK, 40, None) == 0 {
            DestroyWindow(controller); GdiplusShutdown(gdiplus_token); return Err("无法启动导航栏跟随。".into());
        }
        let _ = ready.send(Ok(GetCurrentThreadId()));
        let mut message = zeroed();
        while !shared.stopped.load(Ordering::Acquire) && GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            TranslateMessage(&message); DispatchMessageW(&message);
        }
        KillTimer(controller, TICK);
        state.ui.borrow_mut().detach();
        CONTROLLER.with(|slot| slot.set(null_mut()));
        DestroyWindow(controller);
        GdiplusShutdown(gdiplus_token);
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        fn r(l: i32, t: i32, right: i32, bottom: i32) -> Bounds { Bounds { left: l, top: t, right, bottom } }
        #[test]
        fn normal_window_uses_space_below_without_moving_the_owner() {
            assert_eq!(placement(r(100,100,500,700), r(108,130,492,692), r(0,0,1920,1040), r(0,0,1920,1080),48,false), (r(100,700,500,748),false));
        }
        #[test]
        fn bottom_edge_and_zoomed_windows_use_visible_client_overlay() {
            assert_eq!(placement(r(100,100,500,1030),r(108,130,492,1022),r(0,0,1920,1040),r(0,0,1920,1080),48,false), (r(108,974,492,1022),true));
            assert!(placement(r(-8,-8,1928,1048),r(0,30,1920,1040),r(0,0,1920,1040),r(0,0,1920,1080),48,true).1);
        }
        #[test]
        fn fullscreen_and_negative_monitor_coordinates_preserve_visible_bottom() {
            assert_eq!(placement(r(-1920,0,0,1080),r(-1920,0,0,1080),r(-1920,0,0,1040),r(-1920,0,0,1080),72,false), (r(-1920,1008,0,1080),true));
            assert_eq!(placement(r(-2000,0,-1500,1200),r(-1992,30,-1508,1192),r(-1920,0,0,1040),r(-1920,0,0,1080),96,false), (r(-1920,944,-1508,1040),true));
        }
    }
}
