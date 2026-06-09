use crate::engine::pack_importer;
use crate::engine::utils::validate_file_path;
use crate::error::AppError;
use crate::model::history::AddElementCommand;
use std::path::Path;
use tauri::{Emitter, State};

use super::canvas::ProjectState;
use super::export::RenderCacheState;
use super::history::HistoryState;

#[tauri::command]
pub fn import_icon_pack(
    dir: String,
    pack_name: String,
) -> Result<pack_importer::ImportResult, String> {
    validate_file_path(&dir)?;
    pack_importer::import_from_directory(Path::new(&dir), &pack_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_icon_packs() -> Result<Vec<pack_importer::IconPackMeta>, String> {
    pack_importer::list_packs().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_pack_icons(
    pack_id: String,
) -> Result<Vec<pack_importer::PackIcon>, String> {
    pack_importer::list_pack_icons(&pack_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_pack_icons(
    pack_id: String,
    query: String,
) -> Result<Vec<pack_importer::PackIcon>, String> {
    pack_importer::search_pack_icons(&pack_id, &query).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_pack_icon(
    pack_id: String,
    icon_name: String,
) -> Result<String, String> {
    pack_importer::load_pack_icon_svg(&pack_id, &icon_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_pack_icon_to_canvas(
    app_handle: tauri::AppHandle,
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    cache_state: State<'_, RenderCacheState>,
    pack_id: String,
    icon_name: String,
) -> Result<Vec<String>, String> {
    let svg = pack_importer::load_pack_icon_svg(&pack_id, &icon_name)
        .map_err(|e| e.to_string())?;

    let mut elements = crate::engine::importer::import_svg_as_elements(&svg)
        .map_err(|e| format!("SVG parse error: {}", e))?;

    if elements.is_empty() {
        return Ok(Vec::new());
    }

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let mut ids = Vec::new();

    for elem in &mut elements {
        let prefix = match elem {
            crate::model::Element::Shape(_) => "shape",
            crate::model::Element::Text(_) => "text",
            crate::model::Element::Icon(_) => "icon",
            crate::model::Element::Image(_) => "image",
            crate::model::Element::Path(_) => "path",
            crate::model::Element::Group(_) => "group",
            crate::model::Element::Symbol(_) => "symbol",
        };
        let new_id = project.alloc_element_id(prefix);
        elem.common_mut().id = new_id.clone();
        ids.push(new_id);
    }

    {
        let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        history.begin_batch("add_pack_icon").map_err(|e| e.to_string())?;
        for e in elements {
            history.push_and_execute(
                Box::new(AddElementCommand::new(e)),
                &mut project,
            ).map_err(|e| e.to_string())?;
        }
        history.commit_batch().map_err(|e| e.to_string())?;
    }

    project.bump_version();
    drop(project);

    {
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        cache.invalidate_cache();
    }

    let _ = app_handle.emit("project-changed", ());
    Ok(ids)
}

#[tauri::command]
pub fn remove_icon_pack(pack_id: String) -> Result<(), String> {
    pack_importer::remove_pack(&pack_id).map_err(|e| e.to_string())
}
