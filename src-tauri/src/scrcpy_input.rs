//! Deliver navigation through the owned, unmodified scrcpy client's shortcut
//! handler. This never injects global input or starts an ADB/input process.
use crate::models::NavigationAction;

pub trait NavigationDispatcher: Send + Sync {
    /// Success means the shortcut was queued, not Android execution confirmed.
    fn dispatch(&self, process_id: u32, action: NavigationAction) -> Result<(), String>;
}

#[derive(Default)]
pub struct NativeScrcpyDispatcher;

impl NavigationDispatcher for NativeScrcpyDispatcher {
    fn dispatch(&self, process_id: u32, action: NavigationAction) -> Result<(), String> {
        #[cfg(windows)]
        { platform::dispatch(process_id, action) }
        #[cfg(not(windows))]
        { let _ = (process_id, action); Err("导航快捷键仅支持 Windows。".into()) }
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::ptr::null_mut;
    use windows_sys::Win32::{
        Foundation::{GetLastError, HWND, LPARAM, WPARAM},
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    };

    fn held_keys(action: NavigationAction) -> [i32; 6] {
        [VK_SHIFT as i32, VK_CONTROL as i32, VK_MENU as i32,
            VK_LWIN as i32, VK_RWIN as i32, action.shortcut_virtual_key() as i32]
    }

    unsafe fn validate_window(window: HWND, process_id: u32) -> bool {
        let mut actual = 0;
        if process_id == 0 || window.is_null() || IsWindow(window) == 0
            || GetWindowThreadProcessId(window, &mut actual) == 0 || actual != process_id {
            return false;
        }
        let mut class = [0u16; 128];
        let length = GetClassNameW(window, class.as_mut_ptr(), class.len() as i32);
        length > 0 && String::from_utf16_lossy(&class[..length as usize]).starts_with("SDL_app")
    }

    struct Search { process_id: u32, window: HWND }
    unsafe extern "system" fn enumerate(window: HWND, parameter: LPARAM) -> windows_sys::core::BOOL {
        let search = &mut *(parameter as *mut Search);
        if GetWindow(window, GW_OWNER).is_null() && validate_window(window, search.process_id) {
            search.window = window;
            return 0;
        }
        1
    }

    fn key_parameter(scancode: u32, down: bool, alt_context: bool) -> LPARAM {
        let mut value = 1u32 | (scancode << 16);
        if alt_context { value |= 1 << 29; }
        if !down { value |= (1 << 30) | (1 << 31); }
        value as LPARAM
    }

    fn submit_messages(
        messages: [(u32, WPARAM, LPARAM); 4],
        mut valid: impl FnMut() -> bool,
        mut post: impl FnMut(u32, WPARAM, LPARAM) -> Result<(), u32>,
    ) -> Result<(), u32> {
        let mut started = false;
        for (message, key, parameter) in messages {
            let result = if valid() { post(message, key, parameter) } else { Err(0) };
            if let Err(error) = result {
                // The closures stay bound to the original window/PID; never
                // discover or clean up a replacement session after a failure.
                if started && valid() {
                    let (message, key, parameter) = messages[2];
                    let _ = post(message, key, parameter);
                    let (message, key, parameter) = messages[3];
                    let _ = post(message, key, parameter);
                }
                return Err(error);
            }
            started = true;
        }
        Ok(())
    }

    pub(super) fn dispatch(process_id: u32, action: NavigationAction) -> Result<(), String> {
        unsafe {
            if held_keys(action).into_iter().any(|key| GetAsyncKeyState(key) as u16 & 0x8000 != 0) {
                return Err("请先松开 Shift、Ctrl、Alt、Windows 键及当前导航快捷键，再点击导航。".into());
            }
            let mut search = Search { process_id, window: null_mut() };
            EnumWindows(Some(enumerate), &mut search as *mut _ as LPARAM);
            let window = search.window;
            if !validate_window(window, process_id) {
                return Err("没有找到当前投屏进程的窗口，请稍候重试。".into());
            }
            let letter = action.shortcut_virtual_key() as WPARAM;
            let thread_id = GetWindowThreadProcessId(window, null_mut());
            let scan = MapVirtualKeyExW(letter as u32, MAPVK_VK_TO_VSC, GetKeyboardLayout(thread_id));
            if scan == 0 { return Err("无法确定当前键盘布局的导航快捷键。".into()); }
            // Use the left Alt scancode explicitly. RAlt/AltGr is not scrcpy's
            // default MOD key. SYSKEY prevents TranslateMessage from generating
            // ordinary WM_CHAR input that could leak into the Android editor.
            let messages = [
                (WM_SYSKEYDOWN, VK_MENU as WPARAM, key_parameter(0x38, true, false)),
                (WM_SYSKEYDOWN, letter, key_parameter(scan, true, true)),
                (WM_SYSKEYUP, letter, key_parameter(scan, false, true)),
                (WM_SYSKEYUP, VK_MENU as WPARAM, key_parameter(0x38, false, true)),
            ];
            submit_messages(messages, || validate_window(window, process_id), |message, key, parameter| {
                if PostMessageW(window, message, key, parameter) != 0 { Ok(()) } else { Err(GetLastError()) }
            }).map_err(|error| format!("导航快捷键未能完整投递（Windows 错误 {error}），请重试。"))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn partial_post_failure_balances_keys_only_on_the_original_valid_target() {
            let messages = [(WM_SYSKEYDOWN,18,1),(WM_SYSKEYDOWN,72,2),(WM_SYSKEYUP,72,3),(WM_SYSKEYUP,18,4)];
            let mut calls=Vec::new();
            let result=submit_messages(messages,||true,|message,key,parameter| {
                calls.push((message,key,parameter));
                if calls.len()==2 { Err(1816) } else { Ok(()) }
            });
            assert_eq!(result,Err(1816));
            assert_eq!(calls,vec![messages[0],messages[1],messages[2],messages[3]]);
            let mut first=true;
            let mut calls=Vec::new();
            let result=submit_messages(messages,|| { let valid=first;first=false;valid },|message,key,parameter| {calls.push((message,key,parameter));Ok(())});
            assert_eq!(result,Err(0));
            assert_eq!(calls,vec![messages[0]]);
        }

        #[test]
        fn every_sequence_has_left_alt_context_balanced_up_and_no_repeat() {
            let down=key_parameter(0x23,true,true) as u32;
            let up=key_parameter(0x23,false,true) as u32;
            assert_eq!(down & (1<<29),1<<29);
            assert_eq!(down & (1<<30),0);
            assert_eq!(up & ((1<<30)|(1<<31)),(1<<30)|(1<<31));
            assert_eq!(held_keys(NavigationAction::Home),[16,17,18,91,92,72]);
        }
    }
}
