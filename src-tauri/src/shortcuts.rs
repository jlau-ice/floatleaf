use crate::{logging, platform, runtime};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{
    GlobalShortcutExt, Modifiers, Shortcut, ShortcutEvent, ShortcutState,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Bindings {
    pub display: String,
    pub emergency: String,
    pub previous: String,
    pub next: String,
    pub auto: String,
}
impl Default for Bindings {
    fn default() -> Self {
        Self {
            display: "Control+Alt+Space".into(),
            emergency: "Control+Alt+Escape".into(),
            previous: "Control+Alt+ArrowLeft".into(),
            next: "Control+Alt+ArrowRight".into(),
            auto: "Control+Alt+KeyP".into(),
        }
    }
}
impl Bindings {
    pub fn values(&self) -> [&str; 5] {
        [
            &self.display,
            &self.emergency,
            &self.previous,
            &self.next,
            &self.auto,
        ]
    }
    pub fn parse(&self) -> Result<[Shortcut; 5], String> {
        let mut parsed = Vec::new();
        for key in self.values() {
            let shortcut: Shortcut = key.parse().map_err(|e| format!("快捷键格式错误：{e}"))?;
            if !platform::supported(&shortcut)
                || !shortcut
                    .mods
                    .intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER)
            {
                return Err(
                    "快捷键须包含 Ctrl、Alt 或 Super，并使用字母、空格、方向键、Esc 或 F1～F12"
                        .into(),
                );
            }
            if parsed.contains(&shortcut) {
                return Err("各项快捷键不能重复".into());
            }
            parsed.push(shortcut);
        }
        Ok(parsed.try_into().unwrap())
    }
}

fn register(app: &AppHandle, key: Shortcut) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    platform::check_hotkey(app, key)?;
    app.global_shortcut()
        .register(key)
        .map_err(|e| e.to_string())
}

// Register additions first. A conflict leaves every old shortcut registered.
pub fn add(app: &AppHandle, keys: &[Shortcut; 5]) -> Result<Vec<Shortcut>, String> {
    let mut added = Vec::new();
    for key in keys {
        if app.global_shortcut().is_registered(*key) {
            continue;
        }
        if let Err(error) = register(app, *key) {
            logging::record(
                app,
                "shortcut-register-error",
                serde_json::json!({"key": key.to_string(), "error": error}),
            );
            let _ = app.global_shortcut().unregister_multiple(added.clone());
            return Err(format!(
                "快捷键 {key} 注册失败：{error}。原快捷键保留，请换一组组合键。"
            ));
        }
        logging::record(
            app,
            "shortcut-registered",
            serde_json::json!({"key": key.to_string(), "keyId": key.id()}),
        );
        added.push(*key);
    }
    Ok(added)
}

fn apply_shortcut(
    state: &mut runtime::Runtime,
    index: usize,
    event: ShortcutState,
    (all, main, any): (bool, bool, bool),
) {
    if index == 0 {
        state.controller.key_sample(all, main, any);
        if event == ShortcutState::Pressed {
            state.controller.press(all);
        }
    } else if event == ShortcutState::Released && !main {
        state.action_held[index - 1] = false;
    } else if event == ShortcutState::Pressed
        && (index == 1 || (all && !state.action_held[index - 1]))
    {
        state.action_held[index - 1] = true;
        match index {
            // A registered emergency event can always hide, even when OS input
            // permissions prevent sampling or a previous release was missed.
            1 => {
                state.controller.emergency_hide();
                state.reader.auto_running = false;
            }
            2 | 3 if state.controller.visible => {
                state.reader.turn(index == 3, true);
                state.controller.revision += 1;
            }
            4 => {
                state.reader.toggle_auto();
                state.controller.revision += 1;
            }
            _ => {}
        }
    }
}

pub fn handle(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    let key = *shortcut;
    let handle = app.clone();
    runtime::change(app, move |state| {
        let Some(index) = state.bindings.iter().position(|s| *s == key) else {
            return;
        };
        let (all, main, any) = platform::chord(&key);
        let before = state.controller.input_flags();
        apply_shortcut(state, index, event.state, (all, main, any));
        let action = ["display", "emergency", "previous", "next", "auto"][index];
        let mut details = serde_json::json!({
            "key": key.to_string(), "keyId": key.id(),
            "action": action,
            "pressed": event.state == ShortcutState::Pressed,
            "chordDown": all, "mainDown": main, "anyDown": any,
            "before": before, "after": state.controller.input_flags(),
        });
        #[cfg(target_os = "macos")]
        {
            details["input"] = platform::input_status();
        }
        #[cfg(windows)]
        if let Some(window) = handle.get_webview_window("overlay") {
            let context = crate::windows_diagnostics::window_context(&window);
            if event.state == ShortcutState::Pressed {
                if main {
                    state.shortcut_blocked_by = None;
                } else if let Some(executable) =
                    crate::windows_diagnostics::higher_integrity_foreground(&context)
                {
                    state.shortcut_blocked_by = Some(executable);
                }
            }
            details["windows"] = context;
        }
        logging::record(&handle, "shortcut", details);
    });
}

pub fn setup(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    logging::record(app, "input-status", platform::input_status());
    let keys = app.state::<runtime::Shared>().lock().unwrap().bindings;
    // Keep independently available actions usable even if one key is occupied.
    let mut errors = Vec::new();
    for key in keys {
        if let Err(e) = register(app, key) {
            logging::record(
                app,
                "shortcut-register-error",
                serde_json::json!({ "key": key.to_string(), "error": e }),
            );
            errors.push(format!("{key}: {e}"));
        } else {
            logging::record(
                app,
                "shortcut-registered",
                serde_json::json!({ "key": key.to_string(), "keyId": key.id() }),
            );
        }
    }
    if !errors.is_empty() {
        app.state::<runtime::Shared>().lock().unwrap().error = Some(format!(
            "快捷键注册失败：{}。请在设置中修改，托盘仍可使用。",
            errors.join("；")
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::DisplayMode;

    #[test]
    fn emergency_event_hides_without_key_sampling_in_every_mode() {
        for mode in [DisplayMode::Hold, DisplayMode::Toggle, DisplayMode::Pinned] {
            let mut state = runtime::Runtime::default();
            state.controller.set_mode(mode);
            apply_shortcut(
                &mut state,
                0,
                ShortcutState::Released,
                (false, false, false),
            );
            apply_shortcut(&mut state, 0, ShortcutState::Pressed, (true, true, true));
            assert!(state.controller.visible);
            state.reader.auto_running = true;

            apply_shortcut(&mut state, 1, ShortcutState::Pressed, (false, false, false));
            assert!(!state.controller.visible, "{mode:?} must hide on the event");
            assert!(state.controller.emergency_latched);
            assert!(!state.reader.auto_running);

            apply_shortcut(&mut state, 0, ShortcutState::Pressed, (true, true, true));
            assert!(
                !state.controller.visible,
                "old display chord must stay blocked"
            );
            apply_shortcut(
                &mut state,
                0,
                ShortcutState::Released,
                (false, false, false),
            );
            apply_shortcut(&mut state, 0, ShortcutState::Pressed, (true, true, true));
            assert!(state.controller.visible, "fresh display chord must recover");
            apply_shortcut(&mut state, 1, ShortcutState::Pressed, (false, false, false));
            assert!(
                !state.controller.visible,
                "a missed emergency release cannot block hiding"
            );
        }
    }

    #[test]
    fn unavailable_sampling_does_not_display_or_change_reading() {
        let mut state = runtime::Runtime::default();
        state.reader.open("test".into(), "0123456789".into(), 0, 8);
        let version = state.reader.layout_version;
        state
            .reader
            .accept_layout("test", 0, version, 3, None)
            .unwrap();
        apply_shortcut(&mut state, 0, ShortcutState::Pressed, (false, false, false));
        assert!(!state.controller.visible);

        state.controller.set_mode(DisplayMode::Pinned);
        apply_shortcut(
            &mut state,
            1,
            ShortcutState::Released,
            (false, false, false),
        );
        assert!(
            state.controller.visible,
            "release alone must not emergency-hide"
        );
        apply_shortcut(&mut state, 3, ShortcutState::Pressed, (false, false, false));
        apply_shortcut(&mut state, 4, ShortcutState::Pressed, (false, false, false));
        assert_eq!(state.reader.anchor, 0);
        assert!(!state.reader.auto_running);
    }

    #[test]
    fn shortcut_validation_rejects_alias_duplicates_and_typing_keys() {
        assert!(Bindings::default().parse().is_ok());
        let mut keys = Bindings {
            display: "KeyA".into(),
            ..Default::default()
        };
        assert!(keys.parse().is_err());
        keys.display = "Ctrl+Alt+Escape".into();
        assert!(keys.parse().is_err());
        keys.display = "Ctrl+Shift+KeyR".into();
        assert!(keys.parse().is_ok());
    }
}
