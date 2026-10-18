use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DisplayMode {
    #[default]
    Hold,
    Toggle,
    Pinned,
}

/// All visibility decisions live here. Native calls are applied on the UI thread.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Controller {
    pub mode: DisplayMode,
    pub visible: bool,
    pub emergency_latched: bool,
    pub editing_layout: bool,
    pub position_locked: bool,
    pub revision: u64,
    #[serde(skip)]
    held: bool,
    #[serde(skip)]
    wait_for_all_up: bool,
    #[serde(skip)]
    wait_for_main_up: bool,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            mode: DisplayMode::Hold,
            visible: false,
            emergency_latched: false,
            editing_layout: false,
            position_locked: true,
            revision: 0,
            held: false,
            wait_for_all_up: true,
            wait_for_main_up: false,
        }
    }
}

impl Controller {
    pub fn input_flags(&self) -> serde_json::Value {
        serde_json::json!({
            "visible": self.visible, "mode": self.mode, "held": self.held,
            "waitForAllUp": self.wait_for_all_up, "waitForMainUp": self.wait_for_main_up,
            "emergencyLatched": self.emergency_latched,
        })
    }

    pub fn needs_key_watch(&self) -> bool {
        self.held || self.wait_for_all_up || self.wait_for_main_up
    }

    pub fn key_sample(&mut self, chord_down: bool, main_down: bool, any_down: bool) {
        if !any_down {
            self.wait_for_all_up = false;
        }
        if !main_down {
            self.wait_for_main_up = false;
        }
        if self.held && !chord_down {
            self.release();
            // Re-pressing only a modifier while Space is still down is not a new gesture.
            self.wait_for_main_up = main_down;
        }
    }

    pub fn press(&mut self, physical_chord_down: bool) {
        if !physical_chord_down || self.held || self.wait_for_all_up || self.wait_for_main_up {
            return;
        }
        self.held = true;
        self.emergency_latched = false;
        self.visible = match self.mode {
            DisplayMode::Hold | DisplayMode::Pinned => true,
            DisplayMode::Toggle => !self.visible,
        };
        self.revision += 1;
    }

    pub fn release(&mut self) {
        if !self.held {
            return;
        }
        self.held = false;
        if self.mode == DisplayMode::Hold && !self.editing_layout {
            self.visible = false;
        }
        self.revision += 1;
    }

    pub fn emergency_hide(&mut self) {
        self.visible = false;
        self.editing_layout = false;
        self.emergency_latched = true;
        self.held = false;
        self.wait_for_all_up = true;
        self.revision += 1;
    }

    pub fn set_mode(&mut self, mode: DisplayMode) {
        self.mode = mode;
        self.held = false;
        self.wait_for_all_up = true;
        self.editing_layout = false;
        self.visible = mode == DisplayMode::Pinned && !self.emergency_latched;
        self.revision += 1;
    }

    pub fn set_editing(&mut self, editing: bool) -> Result<(), String> {
        if editing && self.position_locked {
            return Err("请先解除位置锁定。".into());
        }
        self.held = false;
        self.wait_for_all_up = true;
        self.editing_layout = editing;
        if editing {
            self.emergency_latched = false;
        }
        self.visible = editing || (self.mode == DisplayMode::Pinned && !self.emergency_latched);
        self.revision += 1;
        Ok(())
    }

    pub fn set_locked(&mut self, locked: bool) {
        if locked && self.editing_layout {
            let _ = self.set_editing(false);
        }
        self.position_locked = locked;
        self.revision += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ready() -> Controller {
        let mut c = Controller::default();
        c.key_sample(false, false, false);
        c
    }
    #[test]
    fn either_release_order_hides_and_modifier_repress_is_not_a_gesture() {
        for main_down in [false, true] {
            let mut c = ready();
            c.press(true);
            assert!(c.visible);
            c.key_sample(false, main_down, true);
            assert!(!c.visible);
            if main_down {
                c.press(true);
                assert!(!c.visible);
            }
            c.key_sample(false, false, true);
            c.press(true);
            assert!(c.visible);
        }
    }
    #[test]
    fn emergency_blocks_repeat_until_all_members_released() {
        let mut c = ready();
        c.press(true);
        c.emergency_hide();
        c.press(true);
        c.key_sample(false, false, true);
        c.press(true);
        assert!(!c.visible);
        c.key_sample(false, false, false);
        c.press(false); // Late queued event after physical release.
        assert!(!c.visible);
        c.press(true);
        assert!(c.visible);
    }
    #[test]
    fn repeats_do_not_toggle_and_mode_changes_invalidate_old_chord() {
        let mut c = ready();
        c.set_mode(DisplayMode::Toggle);
        c.press(true);
        assert!(!c.visible);
        c.key_sample(false, false, false);
        c.press(true);
        c.press(true);
        assert!(c.visible);
        c.release();
        assert!(c.visible);
        c.press(true);
        assert!(!c.visible);
    }
    #[test]
    fn emergency_overrides_pinned_and_mode_changes() {
        let mut c = ready();
        c.set_mode(DisplayMode::Pinned);
        assert!(c.visible);
        c.emergency_hide();
        c.set_mode(DisplayMode::Pinned);
        assert!(!c.visible);
        c.release();
        assert!(!c.visible);
    }
    #[test]
    fn locking_is_independent_and_ends_editing() {
        let mut c = ready();
        c.set_mode(DisplayMode::Pinned);
        assert!(c.set_editing(true).is_err());
        c.set_locked(false);
        c.set_editing(true).unwrap();
        c.set_locked(true);
        assert!(!c.editing_layout);
        assert!(c.visible);
        assert_eq!(c.mode, DisplayMode::Pinned);
    }
}
