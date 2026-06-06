use crate::engine::adaptive;
use crate::error::AppError;
use tauri::State;

use super::canvas::ProjectState;

#[tauri::command]
pub fn preview_adaptive_icon(
    state: State<'_, ProjectState>,
    shape: String,
    size: u32,
) -> Result<String, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let adaptive_shape = parse_shape(&shape)?;
    adaptive::render_adaptive_base64(&project, adaptive_shape, size)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_adaptive_safe_zone(
    state: State<'_, ProjectState>,
    margin: f64,
) -> Result<String, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let result = adaptive::check_safe_zone(&project, margin);
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_adaptive_foreground(
    state: State<'_, ProjectState>,
    ids: Vec<String>,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let config = project.adaptive.get_or_insert_with(|| crate::model::AdaptiveConfig {
        foreground_ids: Vec::new(),
        background_ids: Vec::new(),
    });
    config.foreground_ids = ids;
    project.bump_version();
    Ok(())
}

#[tauri::command]
pub fn set_adaptive_background(
    state: State<'_, ProjectState>,
    ids: Vec<String>,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let config = project.adaptive.get_or_insert_with(|| crate::model::AdaptiveConfig {
        foreground_ids: Vec::new(),
        background_ids: Vec::new(),
    });
    config.background_ids = ids;
    project.bump_version();
    Ok(())
}

#[tauri::command]
pub fn export_adaptive_android(
    state: State<'_, ProjectState>,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let result = adaptive::export_adaptive_layers(&project, &output_dir)
        .map_err(|e| e.to_string())?;
    Ok(result.files)
}

fn parse_shape(s: &str) -> Result<crate::model::AdaptiveShape, String> {
    match s {
        "circle" => Ok(crate::model::AdaptiveShape::Circle),
        "squircle" => Ok(crate::model::AdaptiveShape::Squircle),
        "rounded-rect" => Ok(crate::model::AdaptiveShape::RoundedRect),
        "pill" => Ok(crate::model::AdaptiveShape::Pill),
        "square" => Ok(crate::model::AdaptiveShape::Square),
        _ => Err(format!(
            "Invalid shape '{}'. Must be: circle, squircle, rounded-rect, pill, square",
            s
        )),
    }
}
