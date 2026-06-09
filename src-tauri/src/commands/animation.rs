use crate::engine::exporter;
use crate::error::AppError;
use crate::model::helpers::*;
use crate::model::history::SnapshotCommand;
use crate::model::{Animation, AnimationType};
use std::path::Path;
use tauri::State;

use super::canvas::ProjectState;
use super::export::RenderCacheState;
use super::history::HistoryState;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetAnimationRequest {
    pub element_id: String,
    pub animation_type: String,
    pub duration: f64,
    pub delay: f64,
    pub repeat: bool,
    pub easing: String,
    pub params: Option<serde_json::Value>,
}

#[tauri::command]
pub fn set_animation(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    request: SetAnimationRequest,
) -> Result<(), String> {
    let anim_type = match request.animation_type.as_str() {
        "rotate" => AnimationType::Rotate,
        "scale" => AnimationType::Scale,
        "fade" => AnimationType::Fade,
        "translate" => AnimationType::Translate,
        "path" => AnimationType::Path,
        other => return Err(AppError::ValidationError(format!("Invalid animation type: {other}")).into()),
    };

    let animation = Animation {
        animation_type: anim_type,
        duration: request.duration,
        delay: request.delay,
        repeat: request.repeat,
        easing: request.easing,
        params: request.params.unwrap_or(serde_json::Value::Null),
    };

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;
    let elem = find_element_deep_mut(project.active_elements_mut(), &request.element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element not found: {}", request.element_id)))?;
    elem.common_mut().animation = Some(animation);
    project.bump_version();
    let after = project.active_elements().to_vec();
    {
        let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
    }
    Ok(())
}

#[tauri::command]
pub fn clear_animation(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;
    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element not found: {element_id}")))?;
    elem.common_mut().animation = None;
    project.bump_version();
    let after = project.active_elements().to_vec();
    {
        let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
    }
    Ok(())
}

#[tauri::command]
pub fn export_webp(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    size: u32,
    path: String,
) -> Result<String, String> {
    let svg_str = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    };

    let result = exporter::export_webp(&svg_str, size, Path::new(&path))?;
    Ok(result.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn export_lottie(
    state: State<'_, ProjectState>,
    output_path: String,
    fps: Option<f64>,
) -> Result<String, String> {
    let json = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::engine::lottie::export_lottie(&project, fps)?
    };
    if let Some(parent) = Path::new(&output_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }
    std::fs::write(&output_path, &json)
        .map_err(|e| format!("Failed to write Lottie JSON: {}", e))?;
    Ok(output_path)
}

#[tauri::command]
pub fn export_animated_gif(
    state: State<'_, ProjectState>,
    output_path: String,
    fps: u32,
    width: u32,
    height: u32,
) -> Result<String, String> {
    let result = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::engine::lottie::export_gif(&project, fps, width, height, Path::new(&output_path))?
    };
    Ok(result.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn preview_animation_frame(
    state: State<'_, ProjectState>,
    time_ms: f64,
) -> Result<String, String> {
    let png_bytes = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::engine::lottie::preview_frame(&project, time_ms)?
    };
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes);
    Ok(format!("data:image/png;base64,{}", b64))
}
