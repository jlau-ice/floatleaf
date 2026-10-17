mod controller;
mod platform;
mod runtime;

use tauri::Manager;

#[tauri::command]
fn reader_state(app: tauri::AppHandle) -> runtime::Snapshot {
    runtime::snapshot(&app)
}

#[tauri::command]
fn reader_action(app: tauri::AppHandle, action: String) -> Result<(), String> {
    runtime::action(&app, action)
}

#[tauri::command]
fn drag_overlay(app: tauri::AppHandle) -> Result<(), String> {
    let state = runtime::snapshot(&app);
    if !state.controller.editing_layout || state.controller.position_locked {
        return Err("当前不是布局编辑状态".into());
    }
    app.get_webview_window("overlay")
        .ok_or("阅读窗口不存在")?
        .start_dragging()
        .map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .manage(runtime::Shared::default())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| runtime::setup(app.handle()))
        .on_window_event(|window, event| {
            if window.label() == "main"
                && let tauri::WindowEvent::CloseRequested { api, .. } = event
            {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            reader_state,
            reader_action,
            drag_overlay
        ])
        .run(tauri::generate_context!())
        .expect("failed to run FloatLeaf");
}
