use crate::{
    controller::{Controller, DisplayMode},
    platform,
};
use serde::Serialize;
use std::{
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

static APP: OnceLock<AppHandle> = OnceLock::new();
static WATCHER: OnceLock<std::thread::Thread> = OnceLock::new();
const DISPLAY: &str = "Control+Alt+Space";
const EMERGENCY: &str = "Control+Alt+Escape";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    #[serde(flatten)]
    pub controller: Controller,
    pub shortcuts: Vec<String>,
    pub error: Option<String>,
}

pub struct Runtime {
    pub controller: Controller,
    pub display: Shortcut,
    pub error: Option<String>,
    last_tick: Instant,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            controller: Controller::default(),
            display: DISPLAY.parse().unwrap(),
            error: None,
            last_tick: Instant::now(),
        }
    }
}

pub type Shared = Mutex<Runtime>;

pub fn snapshot(app: &AppHandle) -> Snapshot {
    let shared = app.state::<Shared>();
    let state = shared.lock().unwrap();
    Snapshot {
        controller: state.controller.clone(),
        shortcuts: vec![DISPLAY.into(), EMERGENCY.into()],
        error: state.error.clone(),
    }
}

// State transitions and native calls are serialized on the event thread. Never
// hold the mutex while calling a window API, since it may pump native events.
pub fn change(app: &AppHandle, update: impl FnOnce(&mut Runtime) + Send + 'static) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let before = snapshot(&handle);
        {
            let shared = handle.state::<Shared>();
            update(&mut shared.lock().unwrap());
        }
        let after = snapshot(&handle);
        if !before.controller.needs_key_watch()
            && after.controller.needs_key_watch()
            && let Some(watcher) = WATCHER.get()
        {
            watcher.unpark();
        }
        if before.controller.revision != after.controller.revision || before.error != after.error {
            apply(&handle, &after);
        }
    });
}

fn apply(app: &AppHandle, state: &Snapshot) {
    let Some(window) = app.get_webview_window("overlay") else {
        return;
    };
    let apply_window = || -> Result<(), String> {
        window
            .set_ignore_cursor_events(!state.controller.editing_layout)
            .map_err(|e| e.to_string())?;
        // Even edit mode never accepts keyboard focus. Dragging is explicitly initiated.
        window.set_focusable(false).map_err(|e| e.to_string())?;
        if state.controller.visible {
            platform::show(&window)?;
        } else {
            window.hide().map_err(|e| e.to_string())?;
        }
        Ok(())
    };
    if let Err(error) = apply_window() {
        let _ = window.hide();
        let shared = app.state::<Shared>();
        let mut state = shared.lock().unwrap();
        state.controller.emergency_hide();
        state.error = Some(format!("阅读窗口操作失败：{error}"));
    }
    let _ = app.emit("reader-state", snapshot(app));
    #[cfg(debug_assertions)]
    write_probe(app);
}

extern "C" fn session_interrupted() {
    if let Some(app) = APP.get() {
        change(app, |s| s.controller.emergency_hide());
    }
}

pub fn setup(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let _ = APP.set(app.clone());
    let window = app.get_webview_window("overlay").ok_or("overlay missing")?;
    window.set_ignore_cursor_events(true)?;
    platform::watch_session(&window, session_interrupted)?;
    for (key, emergency) in [(DISPLAY, false), (EMERGENCY, true)] {
        let shortcut: Shortcut = key.parse()?;
        if !platform::supported(&shortcut) {
            return Err("unsupported shortcut".into());
        }
        if let Err(error) = app
            .global_shortcut()
            .on_shortcut(shortcut, move |app, _, event| {
                change(app, move |state| {
                    if emergency {
                        if event.state == ShortcutState::Pressed {
                            state.controller.emergency_hide();
                        }
                    } else {
                        let (all, main, any) = platform::chord(&state.display);
                        state.controller.key_sample(all, main, any);
                        // A late Released cannot end a new physical press.
                        if event.state == ShortcutState::Pressed {
                            state.controller.press(all);
                        }
                    }
                });
            })
        {
            app.state::<Shared>().lock().unwrap().error = Some(format!(
                "快捷键 {key} 注册失败：{error}。托盘仍可打开设置或隐藏。"
            ));
        }
    }
    start_watch(app.clone());
    setup_tray(app)?;
    Ok(())
}

fn start_watch(app: AppHandle) {
    std::thread::spawn(move || {
        let _ = WATCHER.set(std::thread::current());
        let pending = std::sync::Arc::new(AtomicBool::new(false));
        loop {
            let active = app
                .state::<Shared>()
                .lock()
                .unwrap()
                .controller
                .needs_key_watch();
            std::thread::park_timeout(Duration::from_millis(if active { 12 } else { 150 }));
            if pending.swap(true, Ordering::SeqCst) {
                continue;
            }
            let done = pending.clone();
            change(&app, move |state| {
                let now = Instant::now();
                if now.duration_since(state.last_tick) > Duration::from_secs(2) {
                    state.controller.emergency_hide();
                }
                state.last_tick = now;
                if state.controller.needs_key_watch() {
                    let (all, main, any) = platform::chord(&state.display);
                    state.controller.key_sample(all, main, any);
                }
                done.store(false, Ordering::SeqCst);
            });
            #[cfg(debug_assertions)]
            probe_command(&app);
        }
    });
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    use tauri::{
        menu::{Menu, MenuItem},
        tray::TrayIconBuilder,
    };
    let settings = MenuItem::with_id(app, "settings", "打开浮页设置", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "紧急隐藏", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出浮页", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings, &hide, &quit])?;
    let mut tray = TrayIconBuilder::with_id("floatleaf")
        .tooltip("浮页 FloatLeaf")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "hide" => change(app, |s| s.controller.emergency_hide()),
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

pub fn action(app: &AppHandle, action: String) -> Result<(), String> {
    if ![
        "hide", "hold", "toggle", "pinned", "unlock", "lock", "edit", "finish",
    ]
    .contains(&action.as_str())
    {
        return Err("未知操作".into());
    }
    if action == "edit" && snapshot(app).controller.position_locked {
        return Err("请先解除位置锁定。".into());
    }
    change(app, move |state| match action.as_str() {
        "hide" => state.controller.emergency_hide(),
        "hold" => state.controller.set_mode(DisplayMode::Hold),
        "toggle" => state.controller.set_mode(DisplayMode::Toggle),
        "pinned" => state.controller.set_mode(DisplayMode::Pinned),
        "unlock" => state.controller.set_locked(false),
        "lock" => state.controller.set_locked(true),
        "edit" => {
            let _ = state.controller.set_editing(true);
        }
        "finish" => {
            let _ = state.controller.set_editing(false);
        }
        _ => {}
    });
    Ok(())
}

#[cfg(debug_assertions)]
fn write_probe(app: &AppHandle) {
    if let Ok(directory) = std::env::var("FLOATLEAF_PROBE_DIR")
        && let Some(window) = app.get_webview_window("overlay")
    {
        let value = serde_json::json!({ "state": snapshot(app), "native": platform::probe(&window),
                "visible": window.is_visible().ok(), "position": window.outer_position().ok(), "size": window.outer_size().ok() });
        let dir = std::path::Path::new(&directory);
        let _ = std::fs::create_dir_all(dir);
        if std::fs::write(dir.join("status.tmp"), value.to_string()).is_ok() {
            let _ = std::fs::rename(dir.join("status.tmp"), dir.join("status.json"));
        }
    }
}

#[cfg(debug_assertions)]
fn probe_command(app: &AppHandle) {
    if let Ok(directory) = std::env::var("FLOATLEAF_PROBE_DIR") {
        // Sample on a later event-loop turn so queued native cursor/focus changes
        // have settled. This filesystem probe is absent from release builds.
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || write_probe(&handle));
        let path = std::path::Path::new(&directory).join("command.txt");
        if let Ok(command) = std::fs::read_to_string(&path) {
            let _ = std::fs::remove_file(path);
            #[cfg(target_os = "macos")]
            if command.trim() == "capture" {
                let handle = app.clone();
                let _ = app.run_on_main_thread(move || {
                    for label in ["main", "overlay"] {
                        if let Some(window) = handle.get_webview_window(label) {
                            platform::capture(
                                &window,
                                &std::path::Path::new(&directory).join(format!("{label}.png")),
                            );
                        }
                    }
                });
                return;
            }
            if command.trim() == "exit" {
                app.exit(0);
                return;
            }
            let _ = action(app, command.trim().into());
            let app = app.clone();
            let handle = app.clone();
            let _ = app.run_on_main_thread(move || write_probe(&handle));
        }
    }
}
