use crate::{
    commands,
    controller::{Controller, DisplayMode},
    geometry::{self, Placement},
    platform,
    reader::Reader,
    reading_settings::ReadingSettings,
    shortcuts,
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
use tauri_plugin_global_shortcut::Shortcut;

static APP: OnceLock<AppHandle> = OnceLock::new();
static WATCHER: OnceLock<std::thread::Thread> = OnceLock::new();

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    #[serde(flatten)]
    pub controller: Controller,
    pub placement: Option<Placement>,
    pub error: Option<String>,
    pub reader: Reader,
    pub settings: ReadingSettings,
}

pub struct Runtime {
    pub controller: Controller,
    pub bindings: [Shortcut; 5],
    pub action_held: [bool; 4],
    pub placement: Option<Placement>,
    pub error: Option<String>,
    pub reader: Reader,
    pub settings: ReadingSettings,
    last_tick: Instant,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            controller: Controller::default(),
            bindings: shortcuts::Bindings::default().parse().unwrap(),
            action_held: [false; 4],
            placement: None,
            error: None,
            reader: Reader::default(),
            settings: ReadingSettings::default(),
            last_tick: Instant::now(),
        }
    }
}

impl Runtime {
    fn needs_key_watch(&self) -> bool {
        self.controller.needs_key_watch() || self.action_held.iter().any(|held| *held)
    }
    fn advance_clock(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.last_tick += Duration::from_millis(elapsed.as_millis() as u64);
        if elapsed > Duration::from_secs(2) {
            self.controller.emergency_hide();
            return;
        }
        let seconds = self.reader.remaining_ms / 1000;
        let advanced = self.reader.tick(
            elapsed.as_millis() as u64,
            self.controller.visible,
            self.controller.editing_layout,
        );
        if advanced || seconds != self.reader.remaining_ms / 1000 {
            self.controller.revision += 1;
        }
    }
}

pub type Shared = Mutex<Runtime>;

pub fn snapshot(app: &AppHandle) -> Snapshot {
    let shared = app.state::<Shared>();
    let state = shared.lock().unwrap();
    Snapshot {
        controller: state.controller.clone(),
        placement: state.placement.clone(),
        error: state.error.clone(),
        reader: state.reader.clone(),
        settings: state.settings.clone(),
    }
}

// State transitions and native calls are serialized on the event thread. Never
// hold the mutex while calling a window API, since it may pump native events.
pub fn change(app: &AppHandle, update: impl FnOnce(&mut Runtime) + Send + 'static) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let before = snapshot(&handle);
        let wake = {
            let shared = handle.state::<Shared>();
            let mut state = shared.lock().unwrap();
            let was_watching = state.needs_key_watch();
            // Charge elapsed time to the previous visibility state before changing it.
            state.advance_clock();
            update(&mut state);
            !was_watching && state.needs_key_watch()
        };
        let after = snapshot(&handle);
        if wake && let Some(watcher) = WATCHER.get() {
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
        geometry::apply(app, &window)?;
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
    #[cfg(any(debug_assertions, feature = "diagnostics"))]
    write_probe(app);
}

extern "C" fn session_interrupted() {
    if let Some(app) = APP.get() {
        change(app, |s| s.controller.emergency_hide());
    }
}

pub fn setup(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let _ = APP.set(app.clone());
    commands::setup(app)?;
    let window = app.get_webview_window("overlay").ok_or("overlay missing")?;
    window.set_ignore_cursor_events(true)?;
    platform::watch_session(&window, session_interrupted, displays_changed)?;
    shortcuts::setup(app);
    apply(app, &snapshot(app));
    start_watch(app.clone());
    setup_tray(app)?;
    Ok(())
}

extern "C" fn displays_changed() {
    if let Some(app) = APP.get() {
        geometry::recover(app, false);
    }
}

fn start_watch(app: AppHandle) {
    std::thread::spawn(move || {
        let _ = WATCHER.set(std::thread::current());
        let pending = std::sync::Arc::new(AtomicBool::new(false));
        loop {
            let active = app.state::<Shared>().lock().unwrap().needs_key_watch();
            std::thread::park_timeout(Duration::from_millis(if active { 12 } else { 150 }));
            if pending.swap(true, Ordering::SeqCst) {
                continue;
            }
            let done = pending.clone();
            change(&app, move |state| {
                if state.controller.needs_key_watch() {
                    let (all, main, any) = platform::chord(&state.bindings[0]);
                    state.controller.key_sample(all, main, any);
                }
                for (index, key) in state.bindings[1..].iter().enumerate() {
                    if state.action_held[index] && !platform::chord(key).1 {
                        state.action_held[index] = false;
                    }
                }
                done.store(false, Ordering::SeqCst);
            });
            if let Err(error) = commands::save_progress(&app, false) {
                change(&app, move |state| {
                    if state.error.as_deref() != Some(&error) {
                        state.error = Some(error);
                    }
                });
            }
            #[cfg(any(debug_assertions, feature = "diagnostics"))]
            probe_command(&app);
        }
    });
}

pub fn open_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    use tauri::{
        menu::{Menu, MenuItem},
        tray::TrayIconBuilder,
    };
    let settings = MenuItem::with_id(app, "settings", "打开浮页设置", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "紧急隐藏", true, None::<&str>)?;
    let recover = MenuItem::with_id(app, "recover", "找回阅读窗口位置", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出浮页", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings, &recover, &hide, &quit])?;
    let mut tray = TrayIconBuilder::with_id("floatleaf")
        .tooltip("浮页 FloatLeaf")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => open_settings(app),
            "recover" => geometry::recover(app, true),
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
    #[cfg(any(debug_assertions, feature = "diagnostics"))]
    if let Some(report) = action.strip_prefix("probe-result:") {
        let directory = std::env::var("FLOATLEAF_PROBE_DIR").map_err(|_| "未启用调试探针")?;
        let value: serde_json::Value = serde_json::from_str(report).map_err(|e| e.to_string())?;
        return std::fs::write(
            std::path::Path::new(&directory).join("ui-result.json"),
            value.to_string(),
        )
        .map_err(|e| e.to_string());
    }
    if action == "reset-position" {
        geometry::recover(app, true);
        return Ok(());
    }
    if ![
        "hide", "hold", "toggle", "pinned", "unlock", "lock", "edit", "finish", "next", "previous",
        "auto",
    ]
    .contains(&action.as_str())
    {
        return Err("未知操作".into());
    }
    if action == "edit" && snapshot(app).controller.position_locked {
        return Err("请先解除位置锁定。".into());
    }
    change(app, move |state| match action.as_str() {
        "hide" => {
            state.controller.emergency_hide();
            state.reader.auto_running = false;
        }
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
        "next" | "previous" => {
            if state.controller.visible {
                state.reader.turn(action == "next", true);
                state.controller.revision += 1;
            }
        }
        "auto" if state.controller.visible => {
            state.reader.toggle_auto();
            state.controller.revision += 1;
        }
        _ => {}
    });
    Ok(())
}

#[cfg(any(debug_assertions, feature = "diagnostics"))]
fn write_probe(app: &AppHandle) {
    if let Ok(directory) = std::env::var("FLOATLEAF_PROBE_DIR")
        && let Some(window) = app.get_webview_window("overlay")
    {
        let value = serde_json::json!({ "pid": std::process::id(), "mainNative": app.get_webview_window("main").map(|w| platform::probe(&w)), "state": snapshot(app), "native": platform::probe(&window),
                "monitors": window.available_monitors().ok(), "visible": window.is_visible().ok(), "position": window.outer_position().ok(), "size": window.outer_size().ok() });
        let dir = std::path::Path::new(&directory);
        let _ = std::fs::create_dir_all(dir);
        if std::fs::write(dir.join("status.tmp"), value.to_string()).is_ok() {
            let _ = std::fs::rename(dir.join("status.tmp"), dir.join("status.json"));
        }
    }
}

#[cfg(any(debug_assertions, feature = "diagnostics"))]
fn probe_command(app: &AppHandle) {
    if let Ok(directory) = std::env::var("FLOATLEAF_PROBE_DIR") {
        // Sample on a later event-loop turn so queued native cursor/focus changes
        // have settled. This filesystem probe is absent from release builds.
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || write_probe(&handle));
        let path = std::path::Path::new(&directory).join("command.txt");
        if let Ok(command) = std::fs::read_to_string(&path) {
            let _ = std::fs::remove_file(path);
            if command.trim() == "evaluate" || command.trim() == "evaluate-overlay" {
                let overlay = command.trim() == "evaluate-overlay";
                if let Ok(script) =
                    std::fs::read_to_string(std::path::Path::new(&directory).join(if overlay {
                        "evaluate-overlay.js"
                    } else {
                        "evaluate.js"
                    }))
                    && let Some(window) =
                        app.get_webview_window(if overlay { "overlay" } else { "main" })
                {
                    let _ = window.eval(script);
                }
                return;
            }
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
            if command.trim() == "hide-settings" {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
                return;
            }
            if command.trim() == "move-test" {
                let app = app.clone();
                let handle = app.clone();
                let _ = app.run_on_main_thread(move || {
                    if let Some(window) = handle.get_webview_window("overlay") {
                        let _ = window.set_position(tauri::PhysicalPosition::new(240, 220));
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
