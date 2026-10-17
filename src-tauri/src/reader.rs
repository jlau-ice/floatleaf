use crate::storage::utf16_boundary;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Reader {
    pub book_id: Option<String>,
    pub anchor: usize,
    pub total: usize,
    pub end: usize,
    pub previous: Option<usize>,
    pub layout_version: u64,
    pub ready: bool,
    pub auto_running: bool,
    pub remaining_ms: u64,
    pub interval_ms: u64,
    #[serde(skip)]
    pub text: std::sync::Arc<str>,
}
impl Reader {
    pub fn open(&mut self, id: String, text: String, anchor: usize, interval: u32) {
        self.book_id = Some(id);
        self.total = text.encode_utf16().count();
        self.anchor = if utf16_boundary(&text, anchor) {
            anchor
        } else {
            0
        };
        self.text = text.into();
        self.auto_running = false;
        self.interval_ms = u64::from(interval) * 1000;
        self.remaining_ms = self.interval_ms;
        self.relayout();
    }
    pub fn relayout(&mut self) {
        self.layout_version += 1;
        self.ready = false;
        self.end = self.anchor;
        self.previous = None;
    }
    pub fn accept_layout(
        &mut self,
        id: &str,
        anchor: usize,
        version: u64,
        end: usize,
        previous: Option<usize>,
    ) -> Result<bool, String> {
        if self.book_id.as_deref() != Some(id)
            || self.anchor != anchor
            || self.layout_version != version
        {
            return Ok(false);
        }
        if end > self.total
            || (end <= anchor && anchor != self.total)
            || !utf16_boundary(&self.text, end)
            || previous.is_some_and(|p| p >= anchor || !utf16_boundary(&self.text, p))
        {
            return Err("分页结果包含无效边界".into());
        }
        self.end = end;
        self.previous = previous;
        self.ready = true;
        if end == self.total {
            self.auto_running = false;
        }
        Ok(true)
    }
    pub fn turn(&mut self, forward: bool, visible: bool) -> bool {
        if !visible || !self.ready {
            return false;
        }
        self.remaining_ms = self.interval_ms;
        let target = if forward {
            (self.end < self.total).then_some(self.end)
        } else {
            self.previous
        };
        if let Some(target) = target {
            self.anchor = target;
            self.relayout();
            return true;
        }
        if forward {
            self.auto_running = false;
        }
        false
    }
    pub fn toggle_auto(&mut self) {
        if self.book_id.is_some() && self.end < self.total {
            self.auto_running = !self.auto_running;
        }
    }
    pub fn set_interval(&mut self, seconds: u32) {
        let interval = u64::from(seconds) * 1000;
        if self.interval_ms != interval {
            self.interval_ms = interval;
            self.remaining_ms = interval;
        }
    }
    pub fn tick(&mut self, elapsed_ms: u64, visible: bool, editing: bool) -> bool {
        if !visible || editing || !self.ready || !self.auto_running {
            return false;
        }
        self.remaining_ms = self.remaining_ms.saturating_sub(elapsed_ms);
        if self.remaining_ms == 0 {
            return self.turn(true, visible);
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn reader() -> Reader {
        let mut r = Reader::default();
        r.open("book".into(), "a😀bcdefghij".into(), 0, 2);
        r.accept_layout("book", 0, r.layout_version, 4, None)
            .unwrap();
        r
    }
    #[test]
    fn hidden_pause_resume_manual_reset_and_end_stop() {
        let mut r = reader();
        r.toggle_auto();
        assert!(!r.tick(750, true, false));
        assert_eq!(r.remaining_ms, 1250);
        r.set_interval(2); // Saving appearance or mode must not restart the countdown.
        assert_eq!(r.remaining_ms, 1250);
        r.tick(9000, false, false);
        assert_eq!(r.remaining_ms, 1250);
        r.tick(9000, true, true);
        assert_eq!(r.remaining_ms, 1250);
        assert!(!r.turn(true, false));
        assert_eq!(r.anchor, 0);
        assert!(r.turn(true, true));
        assert_eq!(r.remaining_ms, 2000);
        assert!(!r.tick(9000, true, false)); // Pending layout never advances twice.
        r.accept_layout("book", 4, r.layout_version, 8, Some(0))
            .unwrap();
        assert!(r.tick(2000, true, false));
        assert_eq!(r.anchor, 8);
        r.accept_layout("book", 8, r.layout_version, r.total, Some(4))
            .unwrap();
        assert!(!r.auto_running);
        assert!(!r.turn(true, true));
    }
    #[test]
    fn interval_changes_restart_countdown_but_hidden_controls_never_turn_pages() {
        let mut r = reader();
        r.toggle_auto();
        r.tick(500, true, false);
        r.set_interval(9);
        assert_eq!(r.remaining_ms, 9000);
        assert!(r.auto_running);
        assert!(!r.tick(20_000, false, false));
        assert_eq!((r.anchor, r.remaining_ms), (0, 9000));
        r.toggle_auto();
        r.set_interval(3);
        assert!(!r.auto_running);
        assert!(!r.tick(10_000, true, false));
        assert_eq!((r.anchor, r.remaining_ms), (0, 3000));
        r.toggle_auto();
        assert!(!r.tick(10_000, false, false));
        assert!(r.tick(3000, true, false));
        assert_eq!(r.anchor, 4);
    }
    #[test]
    fn stale_layout_after_book_switch_or_reflow_is_ignored() {
        let mut r = reader();
        let old = r.layout_version;
        r.relayout();
        assert_eq!(r.anchor, 0);
        assert!(!r.accept_layout("book", 0, old, 4, None).unwrap());
        r.open("second".into(), "another text".into(), 3, 8);
        assert!(!r.accept_layout("book", 0, old, 4, None).unwrap());
        assert_eq!(r.anchor, 3);
    }
    #[test]
    fn invalid_surrogate_and_backwards_boundaries_are_rejected() {
        let mut r = reader();
        assert!(
            r.accept_layout("book", 0, r.layout_version, 2, None)
                .is_err()
        );
        assert!(
            r.accept_layout("book", 0, r.layout_version, 4, Some(0))
                .is_err()
        );
        assert!(
            r.accept_layout("book", 0, r.layout_version, r.total + 1, None)
                .is_err()
        );
    }
}
