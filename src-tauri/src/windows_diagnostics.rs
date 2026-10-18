use serde_json::{Value, json};
use std::ffi::c_void;
use tauri::WebviewWindow;

#[repr(C)]
struct ProcessInfo {
    pid: u32,
    integrity_rid: u32,
    query_error: u32,
    token_error: u32,
    image_error: u32,
    executable: [u16; 260],
}

impl ProcessInfo {
    fn json(&self) -> Value {
        let end = self.executable.iter().position(|c| *c == 0).unwrap_or(260);
        json!({
            "pid": self.pid,
            "executable": String::from_utf16_lossy(&self.executable[..end]),
            "integrityRid": (self.query_error == 0 && self.token_error == 0).then_some(self.integrity_rid),
            "queryError": self.query_error, "tokenError": self.token_error, "imageError": self.image_error,
        })
    }
}

#[repr(C)]
struct WindowContext {
    foreground_window: usize,
    application: ProcessInfo,
    foreground: ProcessInfo,
    foreground_style: u32,
    overlay_style: u32,
    foreground_rect: [i32; 4],
    overlay_rect: [i32; 4],
    foreground_rect_valid: i32,
    overlay_rect_valid: i32,
    overlay_visible: i32,
    foreground_above_overlay: i32,
    overlay_cloaked: u32,
    show_error: u32,
    cloak_result: i32,
}

pub fn window_context(window: &WebviewWindow) -> Value {
    unsafe extern "C" {
        fn fl_window_context(window: *mut c_void, context: *mut WindowContext);
    }
    let raw = match window.hwnd() {
        Ok(raw) => raw.0,
        Err(error) => return json!({"error": error.to_string()}),
    };
    // The C structure contains only integers and integer arrays; zero is valid.
    let mut context: WindowContext = unsafe { std::mem::zeroed() };
    unsafe { fl_window_context(raw, &mut context) };
    json!({
        "application": context.application.json(),
        "foreground": context.foreground.json(),
        "foregroundWindow": context.foreground_window,
        "foregroundTopmost": context.foreground_style & 0x8 != 0,
        "foregroundRect": (context.foreground_rect_valid != 0).then_some(context.foreground_rect),
        "overlayVisible": context.overlay_visible != 0,
        "overlayTopmost": context.overlay_style & 0x8 != 0,
        "overlayNoActivate": context.overlay_style & 0x08000000 != 0,
        "overlayClickthrough": context.overlay_style & 0x20 != 0,
        "overlayRect": (context.overlay_rect_valid != 0).then_some(context.overlay_rect),
        "overlayCloaked": (context.cloak_result >= 0).then_some(context.overlay_cloaked != 0),
        "foregroundAboveOverlay": (context.foreground_above_overlay >= 0).then_some(context.foreground_above_overlay == 1),
        "lastShowError": context.show_error,
    })
}

// A denied token query is unknown, not proof that the foreground is elevated.
pub fn higher_integrity_foreground(context: &Value) -> Option<String> {
    let application = context["application"]["integrityRid"].as_u64()?;
    let foreground = context["foreground"]["integrityRid"].as_u64()?;
    if foreground <= application {
        return None;
    }
    context["foreground"]["executable"]
        .as_str()
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warn_only_for_known_higher_foreground_integrity() {
        let mut context = json!({
            "application": {"integrityRid": 8192},
            "foreground": {"integrityRid": 12288, "executable": "example.exe"},
        });
        assert_eq!(
            higher_integrity_foreground(&context).as_deref(),
            Some("example.exe")
        );
        context["foreground"]["integrityRid"] = Value::Null;
        assert_eq!(higher_integrity_foreground(&context), None);
        context["foreground"]["integrityRid"] = json!(8192);
        assert_eq!(higher_integrity_foreground(&context), None);
        context["foreground"]["integrityRid"] = json!(12288);
        context["application"]["integrityRid"] = Value::Null;
        assert_eq!(higher_integrity_foreground(&context), None);
    }
}
