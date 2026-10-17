use std::ffi::c_void;
use tauri::WebviewWindow;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

unsafe extern "C" {
    fn fl_show(window: *mut c_void);
    fn fl_key_down(code: u16) -> bool;
    fn fl_watch_session(window: *mut c_void, hide: extern "C" fn(), displays: extern "C" fn());
}

fn raw_window(window: &WebviewWindow) -> Result<*mut c_void, String> {
    #[cfg(target_os = "macos")]
    {
        window.ns_window().map_err(|e| e.to_string())
    }
    #[cfg(windows)]
    {
        window.hwnd().map(|h| h.0).map_err(|e| e.to_string())
    }
}

/// Must run on the event thread. Never calls activate/set_focus.
pub fn show(window: &WebviewWindow) -> Result<(), String> {
    unsafe {
        fl_show(raw_window(window)?);
    }
    Ok(())
}

/// Must run on the event thread, like show. Windows visibility is managed natively.
pub fn hide(window: &WebviewWindow) -> Result<(), String> {
    #[cfg(windows)]
    {
        unsafe extern "C" {
            fn fl_hide(window: *mut c_void);
        }
        // Tauri hide can skip SW_HIDE because native show bypasses Tao's cache.
        unsafe { fl_hide(raw_window(window)?) };
        Ok(())
    }
    #[cfg(target_os = "macos")]
    window.hide().map_err(|e| e.to_string())
}

pub fn watch_session(
    window: &WebviewWindow,
    hide: extern "C" fn(),
    displays: extern "C" fn(),
) -> Result<(), String> {
    unsafe {
        fl_watch_session(raw_window(window)?, hide, displays);
    }
    Ok(())
}

fn down(code: u16) -> bool {
    unsafe { fl_key_down(code) }
}

pub fn choose_txt(window: &WebviewWindow) -> Result<Option<String>, String> {
    unsafe extern "C" {
        fn fl_choose_txt(window: *mut c_void) -> *mut std::ffi::c_char;
        fn fl_free_string(value: *mut std::ffi::c_char);
    }
    unsafe {
        let value = fl_choose_txt(raw_window(window)?);
        if value.is_null() {
            return Ok(None);
        }
        let path = std::ffi::CStr::from_ptr(value)
            .to_string_lossy()
            .into_owned();
        fl_free_string(value);
        Ok(Some(path))
    }
}

#[cfg(target_os = "macos")]
fn main_code(code: Code) -> Option<u16> {
    Some(match code {
        Code::Space => 49,
        Code::Escape => 53,
        Code::ArrowLeft => 123,
        Code::ArrowRight => 124,
        Code::ArrowDown => 125,
        Code::ArrowUp => 126,
        Code::KeyA => 0,
        Code::KeyB => 11,
        Code::KeyC => 8,
        Code::KeyD => 2,
        Code::KeyE => 14,
        Code::KeyF => 3,
        Code::KeyG => 5,
        Code::KeyH => 4,
        Code::KeyI => 34,
        Code::KeyJ => 38,
        Code::KeyK => 40,
        Code::KeyL => 37,
        Code::KeyM => 46,
        Code::KeyN => 45,
        Code::KeyO => 31,
        Code::KeyP => 35,
        Code::KeyQ => 12,
        Code::KeyR => 15,
        Code::KeyS => 1,
        Code::KeyT => 17,
        Code::KeyU => 32,
        Code::KeyV => 9,
        Code::KeyW => 13,
        Code::KeyX => 7,
        Code::KeyY => 16,
        Code::KeyZ => 6,
        Code::F1 => 122,
        Code::F2 => 120,
        Code::F3 => 99,
        Code::F4 => 118,
        Code::F5 => 96,
        Code::F6 => 97,
        Code::F7 => 98,
        Code::F8 => 100,
        Code::F9 => 101,
        Code::F10 => 109,
        Code::F11 => 103,
        Code::F12 => 111,
        _ => return None,
    })
}

#[cfg(windows)]
fn main_code(code: Code) -> Option<u16> {
    Some(match code {
        Code::Space => 0x20,
        Code::Escape => 0x1B,
        Code::ArrowLeft => 0x25,
        Code::ArrowRight => 0x27,
        Code::ArrowDown => 0x28,
        Code::ArrowUp => 0x26,
        Code::KeyA => 0x41,
        Code::KeyB => 0x42,
        Code::KeyC => 0x43,
        Code::KeyD => 0x44,
        Code::KeyE => 0x45,
        Code::KeyF => 0x46,
        Code::KeyG => 0x47,
        Code::KeyH => 0x48,
        Code::KeyI => 0x49,
        Code::KeyJ => 0x4A,
        Code::KeyK => 0x4B,
        Code::KeyL => 0x4C,
        Code::KeyM => 0x4D,
        Code::KeyN => 0x4E,
        Code::KeyO => 0x4F,
        Code::KeyP => 0x50,
        Code::KeyQ => 0x51,
        Code::KeyR => 0x52,
        Code::KeyS => 0x53,
        Code::KeyT => 0x54,
        Code::KeyU => 0x55,
        Code::KeyV => 0x56,
        Code::KeyW => 0x57,
        Code::KeyX => 0x58,
        Code::KeyY => 0x59,
        Code::KeyZ => 0x5A,
        Code::F1 => 0x70,
        Code::F2 => 0x71,
        Code::F3 => 0x72,
        Code::F4 => 0x73,
        Code::F5 => 0x74,
        Code::F6 => 0x75,
        Code::F7 => 0x76,
        Code::F8 => 0x77,
        Code::F9 => 0x78,
        Code::F10 => 0x79,
        Code::F11 => 0x7A,
        Code::F12 => 0x7B,
        _ => return None,
    })
}

pub fn supported(shortcut: &Shortcut) -> bool {
    main_code(shortcut.key).is_some()
}

/// Samples only configured chord members. No global input log or event tap.
pub fn chord(shortcut: &Shortcut) -> (bool, bool, bool) {
    let main = main_code(shortcut.key).is_some_and(down);
    #[cfg(target_os = "macos")]
    let groups = [
        (Modifiers::CONTROL, [59, 62]),
        (Modifiers::ALT, [58, 61]),
        (Modifiers::SHIFT, [56, 60]),
        (Modifiers::SUPER, [55, 54]),
    ];
    #[cfg(windows)]
    let groups = [
        (Modifiers::CONTROL, [0xA2, 0xA3]),
        (Modifiers::ALT, [0xA4, 0xA5]),
        (Modifiers::SHIFT, [0xA0, 0xA1]),
        (Modifiers::SUPER, [0x5B, 0x5C]),
    ];
    let mut all = main;
    let mut any = main;
    for (modifier, keys) in groups {
        if shortcut.mods.contains(modifier) {
            let pressed = keys.into_iter().any(down);
            all &= pressed;
            any |= pressed;
        }
    }
    (all, main, any)
}

#[cfg(any(debug_assertions, feature = "diagnostics"))]
pub fn probe(window: &WebviewWindow) -> serde_json::Value {
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn fl_automation_allowed() -> bool;
            fn fl_window_clickthrough(window: *mut c_void) -> bool;
            fn fl_window_key(window: *mut c_void) -> bool;
            fn fl_window_opaque(window: *mut c_void) -> bool;
            fn fl_window_level(window: *mut c_void) -> std::ffi::c_long;
            fn fl_frontmost_pid() -> i32;
            fn fl_web_pid(window: *mut c_void) -> i32;
        }
        let raw = raw_window(window).unwrap();
        unsafe {
            serde_json::json!({
                "webPid": fl_web_pid(raw), "automationAllowed": fl_automation_allowed(), "clickthrough": fl_window_clickthrough(raw),
            "keyWindow": fl_window_key(raw), "opaque": fl_window_opaque(raw), "level": fl_window_level(raw), "frontmostPid": fl_frontmost_pid(),
            })
        }
    }
    #[cfg(windows)]
    {
        serde_json::json!({ "focused": window.is_focused().ok() })
    }
}

#[cfg(all(any(debug_assertions, feature = "diagnostics"), target_os = "macos"))]
pub fn capture(window: &WebviewWindow, path: &std::path::Path) {
    unsafe extern "C" {
        fn fl_snapshot(window: *mut c_void, path: *const std::ffi::c_char);
    }
    if let Ok(path) = std::ffi::CString::new(path.to_string_lossy().as_bytes()) {
        unsafe {
            fl_snapshot(raw_window(window).unwrap(), path.as_ptr());
        }
    }
}

#[cfg(target_os = "macos")]
pub fn check_hotkey(app: &tauri::AppHandle, key: Shortcut) -> Result<(), String> {
    unsafe extern "C" {
        fn fl_check_hotkey(code: u16, modifiers: u32) -> i32;
    }
    let code = main_code(key.key).ok_or("不支持的按键")?;
    let mut mods = 0;
    for (flag, mask) in [
        (Modifiers::SUPER, 256),
        (Modifiers::SHIFT, 512),
        (Modifiers::ALT, 2048),
        (Modifiers::CONTROL, 4096),
    ] {
        if key.mods.contains(flag) {
            mods |= mask;
        }
    }
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = tx.send(unsafe { fl_check_hotkey(code, mods) });
    })
    .map_err(|e| e.to_string())?;
    let result = rx.recv().map_err(|e| e.to_string())?;
    if result == 0 {
        Ok(())
    } else {
        Err(format!("系统独占快捷键检查失败（{result}）"))
    }
}
