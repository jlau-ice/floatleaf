mod commands;
mod controller;
mod geometry;
mod platform;
mod reader;
mod reading_settings;
mod runtime;
mod shortcuts;
mod storage;
mod text_import;

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
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            runtime::open_settings(app)
        }))
        .manage(runtime::Shared::default())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(shortcuts::handle)
                .build(),
        )
        .setup(|app| runtime::setup(app.handle()))
        .on_window_event(|window, event| {
            if window.label() == "overlay" {
                match event {
                    tauri::WindowEvent::Moved(_) => geometry::capture(window.app_handle()),
                    tauri::WindowEvent::ScaleFactorChanged { .. } => {
                        geometry::recover(window.app_handle(), false)
                    }
                    _ => {}
                }
            }
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
            drag_overlay,
            commands::choose_txt,
            commands::inspect_txt,
            commands::library,
            commands::data_directory,
            commands::import_txt,
            commands::select_book,
            commands::remove_book,
            commands::book_text,
            commands::save_settings,
            commands::resize_overlay,
            commands::report_layout
        ])
        .build(tauri::generate_context!())
        .expect("failed to build FloatLeaf")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if matches!(event, tauri::RunEvent::Reopen { .. }) {
                runtime::open_settings(app);
            }
            if matches!(
                event,
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
            ) && let Err(error) = commands::flush_progress(app)
            {
                eprintln!("退出保存失败：{error}");
            }
        });
}
