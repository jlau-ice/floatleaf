use crate::{
    platform,
    reading_settings::ReadingSettings,
    runtime,
    storage::{Book, Library, Store},
    text_import::{self, ImportPreview},
};
use std::{path::Path, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub type SharedStore = Mutex<Result<Store, String>>;
pub fn with_store<T>(
    app: &AppHandle,
    f: impl FnOnce(&mut Store) -> Result<T, String>,
) -> Result<T, String> {
    let state = app.state::<SharedStore>();
    let mut store = state.lock().map_err(|_| "存储线程异常，请退出后重试")?;
    match store.as_mut() {
        Ok(store) => f(store),
        Err(error) => Err(error.clone()),
    }
}
fn main_only(window: &WebviewWindow) -> Result<(), String> {
    if window.label() == "main" {
        Ok(())
    } else {
        Err("此操作仅可从设置窗口执行".into())
    }
}
pub fn setup(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let directory = app.path().app_data_dir()?;
    #[cfg(any(debug_assertions, feature = "diagnostics"))]
    let directory = std::env::var_os("FLOATLEAF_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or(directory);
    let store = Store::open(directory);
    app.manage(Mutex::new(store));
    let restored = with_store(app, |store| {
        let library = store.library()?;
        let current = library
            .current_book_id
            .map(|id| Ok::<_, String>((store.book(&id)?, store.content(&id)?)))
            .transpose()?;
        Ok((library.settings, current, store.placement()))
    });
    let shared = app.state::<runtime::Shared>();
    let mut state = shared.lock().unwrap();
    match restored {
        Ok((settings, current, placement)) => {
            state.bindings = settings.shortcuts.parse()?;
            state.placement = placement;
            state.controller.mode = settings.mode;
            state.controller.position_locked = settings.position_locked;
            if let Some((book, text)) = current {
                state
                    .reader
                    .open(book.id, text, book.anchor, settings.interval_seconds);
            }
            state.settings = settings;
        }
        Err(error) => state.error = Some(error),
    }
    Ok(())
}

#[tauri::command]
pub fn choose_txt(window: WebviewWindow) -> Result<Option<String>, String> {
    main_only(&window)?;
    platform::choose_txt(&window)
}
#[tauri::command]
pub async fn inspect_txt(
    window: WebviewWindow,
    path: String,
    encoding: String,
) -> Result<ImportPreview, String> {
    main_only(&window)?;
    tauri::async_runtime::spawn_blocking(move || text_import::preview(Path::new(&path), &encoding))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn library(app: AppHandle, window: WebviewWindow) -> Result<Library, String> {
    main_only(&window)?;
    tauri::async_runtime::spawn_blocking(move || with_store(&app, |s| s.library()))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn data_directory(app: AppHandle, window: WebviewWindow) -> Result<String, String> {
    main_only(&window)?;
    with_store(&app, |s| Ok(s.directory().to_string_lossy().into_owned()))
}

pub fn select_core(app: &AppHandle, id: &str) -> Result<(), String> {
    let (book, text) = with_store(app, |store| {
        let text = store.select(id)?;
        Ok((store.book(id)?, text))
    })?;
    runtime::change(app, move |state| {
        state
            .reader
            .open(book.id, text, book.anchor, state.settings.interval_seconds);
        state.controller.revision += 1;
    });
    let _ = app.emit("library-changed", ());
    Ok(())
}
#[tauri::command]
pub async fn import_txt(
    app: AppHandle,
    window: WebviewWindow,
    path: String,
    encoding: String,
) -> Result<Book, String> {
    main_only(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        flush_progress(&app)?;
        let book = with_store(&app, |s| s.import(Path::new(&path), &encoding))?;
        select_core(&app, &book.id)?;
        Ok(book)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn select_book(app: AppHandle, window: WebviewWindow, id: String) -> Result<(), String> {
    main_only(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        flush_progress(&app)?;
        select_core(&app, &id)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn remove_book(app: AppHandle, window: WebviewWindow, id: String) -> Result<(), String> {
    main_only(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        flush_progress(&app)?;
        let current = runtime::snapshot(&app).reader.book_id.as_deref() == Some(&id);
        with_store(&app, |s| s.remove(&id))?;
        if current {
            runtime::change(&app, |s| {
                s.reader = Default::default();
                s.controller.emergency_hide();
            });
        }
        let _ = app.emit("library-changed", ());
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn book_text(app: AppHandle, id: String) -> Result<String, String> {
    let shared = app.state::<runtime::Shared>();
    let state = shared.lock().unwrap();
    if state.reader.book_id.as_deref() != Some(&id) {
        return Err("当前书籍已改变".into());
    }
    Ok(state.reader.text.to_string())
}
#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    window: WebviewWindow,
    settings: ReadingSettings,
    select_mode: Option<bool>,
) -> Result<(), String> {
    main_only(&window)?;
    settings.validate()?;
    tauri::async_runtime::spawn_blocking(move || {
        let keys = settings.shortcuts.parse()?;
        let old_keys = app.state::<runtime::Shared>().lock().unwrap().bindings;
        let added = crate::shortcuts::add(&app, &keys)?;
        if let Err(error) = with_store(&app, |s| s.set_settings(settings.clone())) {
            let _ = app.global_shortcut().unregister_multiple(added);
            return Err(error);
        }
        let removed: Vec<_> = old_keys.into_iter().filter(|k| !keys.contains(k)).collect();
        let unregister_error = app
            .global_shortcut()
            .unregister_multiple(removed)
            .err()
            .map(|e| format!("旧快捷键释放失败，请退出后重开：{e}"));
        runtime::change(&app, move |state| {
            if state.controller.mode != settings.mode || select_mode.unwrap_or(false) {
                state.controller.set_mode(settings.mode);
            }
            state.controller.set_locked(settings.position_locked);
            state.reader.set_interval(settings.interval_seconds);
            if !state.settings.same_layout(&settings) {
                state.reader.relayout();
            }
            if state.bindings != keys {
                state.controller.emergency_hide();
                state.action_held = [false; 4];
            }
            state.bindings = keys;
            state.settings = settings;
            state.error = unregister_error;
            state.controller.revision += 1;
        });
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn resize_overlay(
    app: AppHandle,
    window: WebviewWindow,
    width: f64,
    lines: u8,
    persist: bool,
) -> Result<(), String> {
    let state = runtime::snapshot(&app);
    if window.label() != "overlay"
        || !state.controller.editing_layout
        || state.controller.position_locked
    {
        return Err("请在编辑布局状态调整尺寸".into());
    }
    let mut settings = state.settings;
    settings.width = width;
    settings.lines = lines;
    settings.validate()?;
    if persist {
        with_store(&app, |s| s.set_settings(settings.clone()))?;
    }
    runtime::change(&app, move |state| {
        state.settings = settings;
        state.reader.relayout();
        state.controller.revision += 1;
    });
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutReport {
    pub book_id: String,
    pub anchor: usize,
    pub version: u64,
    pub end: usize,
    pub previous: Option<usize>,
}
#[tauri::command]
pub fn report_layout(
    app: AppHandle,
    window: WebviewWindow,
    report: LayoutReport,
) -> Result<(), String> {
    if window.label() != "overlay" {
        return Err("只有阅读窗口可提交分页结果".into());
    }
    runtime::change(&app, move |state| {
        match state.reader.accept_layout(
            &report.book_id,
            report.anchor,
            report.version,
            report.end,
            report.previous,
        ) {
            Ok(true) => state.controller.revision += 1,
            Ok(false) => {}
            Err(e) => state.error = Some(e),
        }
    });
    Ok(())
}

pub fn save_progress(app: &AppHandle, flush: bool) -> Result<(), String> {
    let snapshot = runtime::snapshot(app);
    let reader = snapshot.reader;
    with_store(app, |store| {
        store.update_placement(snapshot.placement);
        if let Some(id) = reader.book_id.as_deref() {
            store.progress(id, reader.anchor);
        }
        if flush {
            store.flush()
        } else {
            store.flush_due()
        }
    })
}
pub fn flush_progress(app: &AppHandle) -> Result<(), String> {
    save_progress(app, true)
}
