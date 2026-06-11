use crate::engine::theme_presets;
use crate::error::AppError;
use crate::model::{Element, ShapeElement, CommonProps, shapes::ShapeType, ThemePreset};
use crate::model::history::{AddElementCommand, CanvasCommand};
use tauri::State;

use super::canvas::ProjectState;
use super::history::HistoryState;

#[tauri::command]
pub fn list_theme_presets() -> Result<Vec<ThemePreset>, String> {
    Ok(theme_presets::builtin_presets())
}

#[tauri::command]
pub fn apply_theme_preset(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    preset_id: String,
) -> Result<(), String> {
    let preset = theme_presets::builtin_presets()
        .into_iter()
        .find(|p| p.id == preset_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Preset '{}' not found", preset_id)))?;

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let old_canvas = project.active_canvas().clone();

    // Apply canvas corner radius from preset
    let cr_percent = (preset.corner_radius / 100.0 * 50.0).round() as u32;
    project.active_canvas_mut().corner_radius = cr_percent.min(50);

    // Apply background color
    if let Some(ref bg) = preset.background {
        project.active_canvas_mut().background = bg.clone();
    }

    project.bump_version();
    let new_canvas = project.active_canvas().clone();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(CanvasCommand::new(old_canvas, new_canvas)));

    // Add a background shape element if preset has a background
    if preset.background.is_some() {
        let canvas_w = project.active_canvas().width as f64;
        let canvas_h = project.active_canvas().height as f64;
        let bg_color = preset.background.clone().unwrap_or_default();

        let (shape_type, border_radius) = match preset.shape {
            crate::model::PresetShape::Circle => (ShapeType::Circle, 0.0),
            crate::model::PresetShape::Square => (ShapeType::Rect, 0.0),
            crate::model::PresetShape::Hexagon => (ShapeType::Hexagon, 0.0),
            crate::model::PresetShape::Shield => (ShapeType::Shield, 0.0),
            _ => (ShapeType::Rect, preset.corner_radius / 100.0 * canvas_w),
        };

        let id = project.alloc_element_id("shape");
        let bg_element = ShapeElement {
            common: CommonProps {
                id,
                x: 0.0,
                y: 0.0,
                width: canvas_w,
                height: canvas_h,
                opacity: 1.0,
                rotation: 0.0,
                shadows: preset.shadow.map(|s| vec![s]).unwrap_or_default(),
                animation: None,
                blend_mode: None,
                clip_element_id: None,
                mask_element_id: None,
                locked: true,
                visible: true,
                svg_filter: None,
                overlay: None,
            },
            shape_type,
            fill: bg_color,
            stroke: None,
            stroke_width: 0.0,
            border_radius,
            stroke_dasharray: None,
            gradient: None,
        };

        // Insert background at position 0
        let cmd = AddElementCommand::new(Element::Shape(bg_element));
        // We need to insert at position 0 - use push_and_execute then reorder
        let mut history2 = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        history2.push_and_execute(Box::new(cmd), &mut project)?;

        // Move to front
        let elements = project.active_elements_mut();
        let last_idx = elements.len() - 1;
        if last_idx > 0 {
            let bg_elem = elements.remove(last_idx);
            elements.insert(0, bg_elem);
        }
        project.bump_version();
    }

    Ok(())
}

#[tauri::command]
pub fn save_custom_theme_preset(
    _state: State<'_, ProjectState>,
    name: String,
    shape: String,
    corner_radius: f64,
    background: Option<String>,
    padding_ratio: f64,
) -> Result<ThemePreset, String> {
    if name.trim().is_empty() {
        return Err(AppError::ValidationError("Name must not be empty".into()).into());
    }

    let shape_type = match shape.as_str() {
        "squircle" => crate::model::PresetShape::Squircle,
        "circle" => crate::model::PresetShape::Circle,
        "roundedRect" => crate::model::PresetShape::RoundedRect,
        "square" => crate::model::PresetShape::Square,
        "hexagon" => crate::model::PresetShape::Hexagon,
        "shield" => crate::model::PresetShape::Shield,
        _ => return Err(AppError::ValidationError(format!("Unknown shape: {}", shape)).into()),
    };

    let preset = ThemePreset {
        id: format!("custom-{:x}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()),
        name,
        corner_radius,
        padding_ratio,
        background,
        shadow: None,
        shape: shape_type,
        preview_svg: None,
    };

    // In a full implementation, this would persist to disk
    // For now, we just return the created preset
    Ok(preset)
}
