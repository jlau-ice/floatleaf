use crate::controller::DisplayMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ReadingSettings {
    pub font_family: String,
    pub font_size: f64,
    pub color: String,
    pub text_opacity: f64,
    pub background_opacity: f64,
    pub line_height: f64,
    pub letter_spacing: f64,
    pub width: f64,
    pub lines: u8,
    pub interval_seconds: u32,
    pub mode: DisplayMode,
    pub position_locked: bool,
    pub capture_protection: bool,
    pub shortcuts: crate::shortcuts::Bindings,
}
impl Default for ReadingSettings {
    fn default() -> Self {
        Self {
            font_family: "PingFang SC, Microsoft YaHei, sans-serif".into(),
            font_size: 18.0,
            color: "#ffffff".into(),
            text_opacity: 0.7,
            background_opacity: 0.0,
            line_height: 1.5,
            letter_spacing: 0.0,
            width: 600.0,
            lines: 2,
            interval_seconds: 8,
            mode: DisplayMode::Hold,
            position_locked: true,
            capture_protection: false,
            shortcuts: Default::default(),
        }
    }
}
impl ReadingSettings {
    pub fn validate(&self) -> Result<(), String> {
        for (label, value, min, max) in [
            ("字号", self.font_size, 12.0, 48.0),
            ("文字不透明度", self.text_opacity, 0.1, 1.0),
            ("背景不透明度", self.background_opacity, 0.0, 0.8),
            ("行高", self.line_height, 1.0, 2.2),
            ("字间距", self.letter_spacing, -1.0, 8.0),
            ("宽度", self.width, 200.0, 1600.0),
        ] {
            if !(min..=max).contains(&value) {
                return Err(format!("{label}超出允许范围 {min}～{max}"));
            }
        }
        if !(1..=12).contains(&self.lines) || !(2..=60).contains(&self.interval_seconds) {
            return Err("行数或自动翻页间隔超出范围".into());
        }
        if self.font_family.is_empty()
            || self.font_family.len() > 200
            || self.font_family.chars().any(char::is_control)
        {
            return Err("字体名称无效".into());
        }
        if self.color.len() != 7
            || !self.color.starts_with('#')
            || !self.color[1..].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("文字颜色必须是 #RRGGBB".into());
        }
        self.shortcuts.parse()?;
        Ok(())
    }
    pub fn same_layout(&self, other: &Self) -> bool {
        self.font_family == other.font_family
            && self.font_size == other.font_size
            && self.line_height == other.line_height
            && self.letter_spacing == other.letter_spacing
            && self.width == other.width
            && self.lines == other.lines
    }
    pub fn height(&self) -> f64 {
        (self.font_size * self.line_height * f64::from(self.lines)).ceil() + 48.0
    }
}
