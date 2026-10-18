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
            let _ = app.global_shortcut().unregister_multiple(added.clone());
            return Err(format!(
                "快捷键 {key} 注册失败：{error}。原快捷键保留，请换一组组合键。"
            ));
        }
        added.push(*key);
    }
    Ok(added)
}

pub fn handle(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    let key = *shortcut;
    let handle = app.clone();
    runtime::change(app, move |state| {
        let Some(index) = state.bindings.iter().position(|s| *s == key) else {
            return;
        };
        let (all, main, any) = platform::chord(&key);
        let details = serde_json::json!({
            "key": key.to_string(),
            "pressed": event.state == ShortcutState::Pressed,
            "chordDown": all, "mainDown": main, "anyDown": any,
        });
        #[cfg(target_os = "macos")]
        let details = {
            let mut details = details;
            details["input"] = platform::input_status();
            details
        };
        logging::record(&handle, "shortcut", details);
        if index == 0 {
            state.controller.key_sample(all, main, any);
            if event.state == ShortcutState::Pressed {
                state.controller.press(all);
            }
        } else if event.state == ShortcutState::Released && !main {
            state.action_held[index - 1] = false;
        } else if event.state == ShortcutState::Pressed && all && !state.action_held[index - 1] {
            state.action_held[index - 1] = true;
            match index {
                1 => state.controller.emergency_hide(),
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
                serde_json::json!({ "key": key.to_string() }),
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
