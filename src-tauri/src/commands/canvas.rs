use crate::error::AppError;
use crate::model::{Canvas, ExportConfig, IconProject};
use crate::services::websocket;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};

use super::history::HistoryState;
use super::export::RenderCacheState;

pub type ProjectState = Arc<Mutex<IconProject>>;

#[tauri::command]
pub fn new_canvas(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
) -> Result<Canvas, String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    project.canvas = Canvas::default();
    project.elements.clear();
    project.pages.clear();
    project.active_page_index = 0;
    project.next_element_id = 1;
    project.exports = ExportConfig::default();
    project.bump_version();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.clear();
    Ok(project.canvas.clone())
}

#[tauri::command]
pub fn get_ws_info() -> Result<serde_json::Value, String> {
    let port: u16 = std::env::var("ICONSTUDIO_WS_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(9250);
    let token = websocket::get_auth_token().to_string();
    Ok(serde_json::json!({ "port": port, "token": token }))
}

#[tauri::command]
pub fn get_status(state: State<'_, ProjectState>) -> Result<String, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    Ok(format!(
        "Canvas: {}x{}, Elements: {}",
        project.active_canvas().width,
        project.active_canvas().height,
        project.active_elements().len()
    ))
}

#[tauri::command]
pub fn get_canvas(state: State<'_, ProjectState>) -> Result<Canvas, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    Ok(project.active_canvas().clone())
}

#[tauri::command]
pub fn save_project(state: State<'_, ProjectState>, path: Option<String>) -> Result<String, String> {
    let (json, file_path) = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let json = serde_json::to_string_pretty(&*project).map_err(AppError::from)?;

        let file_path = match path {
            Some(p) => {
                crate::engine::utils::validate_file_path(&p)?;
                if p.ends_with(".iconproject.json") {
                    p
                } else if p.ends_with(".iconproject") {
                    format!("{}.json", p)
                } else if p.ends_with(".json") {
                    format!("{}.iconproject.json", p.trim_end_matches(".json"))
                } else {
                    format!("{}.iconproject.json", p)
                }
            }
            None => "project.iconproject.json".to_string(),
        };

        (json, file_path)
    };

    if let Some(parent) = Path::new(&file_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(AppError::from)?;
        }
    }

    fs::write(&file_path, json).map_err(AppError::from)?;
    Ok(file_path)
}

#[tauri::command]
pub fn open_project(
    app_handle: tauri::AppHandle,
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    cache_state: State<'_, RenderCacheState>,
    path: String,
) -> Result<IconProject, String> {
    let loaded = crate::services::project::load_project_auto(&path)?;
    let result = loaded.clone();
    let mut guard = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    *guard = loaded;
    guard.recalc_next_element_id();
    guard.bump_version();
    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.clear();
    drop(guard);
    let _ = app_handle.emit("project-changed", ());
    Ok(result)
}

#[tauri::command]
pub fn export_template(state: State<'_, ProjectState>, name: String) -> Result<String, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    crate::templates::save_template(&name, &project)?;

    let file_path = format!("{}.json", name);
    Ok(file_path)
}

#[tauri::command]
pub fn import_svg_file(
    app_handle: tauri::AppHandle,
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    cache_state: State<'_, RenderCacheState>,
    path: String,
) -> Result<IconProject, String> {
    let imported = crate::services::project::load_svg_project(&path)?;
    let result = imported.clone();
    let mut guard = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    *guard = imported;
    guard.recalc_next_element_id();
    guard.bump_version();
    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.clear();
    drop(guard);
    let _ = app_handle.emit("project-changed", ());
    Ok(result)
}
