use crate::{commands, reading_settings::ReadingSettings, runtime};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

/// Offset in logical pixels from the monitor work area, independent of its scale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    pub monitor: Option<String>,
    pub x: f64,
    pub y: f64,
}

fn fit(settings: &mut ReadingSettings, placement: &mut Placement, width: f64, height: f64) {
    settings.width = settings.width.min(width).max(200.0);
    let rows = ((height - 48.0) / (settings.font_size * settings.line_height)).floor();
    settings.lines = settings.lines.min(rows.max(1.0) as u8);
    placement.x = placement.x.clamp(0.0, (width - settings.width).max(0.0));
    placement.y = placement
        .y
        .clamp(0.0, (height - settings.height()).max(0.0));
}

pub fn apply(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let before = runtime::snapshot(app);
    let monitors = window.available_monitors().map_err(|e| e.to_string())?;
    let saved = before.placement;
    let monitor = saved
        .as_ref()
        .and_then(|p| monitors.iter().find(|m| m.name().cloned() == p.monitor))
        .cloned()
        .or(window.primary_monitor().map_err(|e| e.to_string())?);
    let Some(monitor) = monitor else {
        return Ok(());
    };
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let width = area.size.width as f64 / scale;
    let height = area.size.height as f64 / scale;
    let mut placement = saved
        .filter(|p| p.monitor == monitor.name().cloned())
        .unwrap_or(Placement {
            monitor: monitor.name().cloned(),
            x: (width - before.settings.width) / 2.0,
            y: height * 0.72,
        });
    let mut settings = before.settings;
    let old_settings = settings.clone();
    fit(&mut settings, &mut placement, width, height);
    // Store intent before native calls, which may dispatch moved/resized events.
    {
        let shared = app.state::<runtime::Shared>();
        let mut state = shared.lock().unwrap();
        state.placement = Some(placement.clone());
        if state.settings != settings {
            state.settings = settings.clone();
            state.reader.relayout();
            state.controller.revision += 1;
        }
    }
    let position = PhysicalPosition::new(
        area.position.x + (placement.x * scale).round() as i32,
        area.position.y + (placement.y * scale).round() as i32,
    );
    let size = PhysicalSize::new(
        (settings.width * scale).round() as u32,
        (settings.height() * scale).round() as u32,
    );
    // AppKit keeps the bottom edge when resizing; restore the intended top-left
    // after changing size so startup and font changes do not drift vertically.
    let resized = window.inner_size().map_err(|e| e.to_string())? != size;
    if resized {
        window.set_size(size).map_err(|e| e.to_string())?;
    }
    if resized || window.outer_position().map_err(|e| e.to_string())? != position {
        window.set_position(position).map_err(|e| e.to_string())?;
    }
    if old_settings != settings {
        commands::with_store(app, |s| s.set_settings(settings))?;
    }
    Ok(())
}

pub fn capture(app: &AppHandle) {
    let result = (|| -> Result<_, String> {
        let window = app.get_webview_window("overlay").ok_or("阅读窗口不存在")?;
        let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())? else {
            return Ok(None);
        };
        let position = window.outer_position().map_err(|e| e.to_string())?;
        let area = monitor.work_area();
        Ok(Some(Placement {
            monitor: monitor.name().cloned(),
            x: (position.x - area.position.x) as f64 / monitor.scale_factor(),
            y: (position.y - area.position.y) as f64 / monitor.scale_factor(),
        }))
    })();
    if let Ok(Some(placement)) = result {
        app.state::<runtime::Shared>().lock().unwrap().placement = Some(placement);
    }
}

pub fn recover(app: &AppHandle, reset: bool) {
    runtime::change(app, move |state| {
        if reset {
            state.placement = None;
        }
        state.reader.relayout();
        state.controller.revision += 1;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disconnect_or_smaller_work_area_keeps_complete_rows_and_visible_bounds() {
        let mut settings = ReadingSettings {
            width: 1500.0,
            font_size: 48.0,
            lines: 12,
            ..Default::default()
        };
        let mut placement = Placement {
            monitor: None,
            x: -300.0,
            y: 1600.0,
        };
        fit(&mut settings, &mut placement, 900.0, 600.0);
        assert_eq!(settings.width, 900.0);
        assert_eq!(settings.lines, 7);
        assert_eq!(placement.x, 0.0);
        assert_eq!(placement.y + settings.height(), 600.0);
        assert_eq!(settings.height(), 552.0);
    }
}
