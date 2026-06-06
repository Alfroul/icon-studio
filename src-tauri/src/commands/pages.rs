use crate::error::AppError;
use crate::model::Page;

use super::canvas::ProjectState;
use super::export::RenderCacheState;

use tauri::{AppHandle, Emitter, State};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PageInfo {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub element_count: usize,
    pub active: bool,
}

#[tauri::command]
pub fn list_pages(state: State<'_, ProjectState>) -> Result<Vec<PageInfo>, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    if project.pages.is_empty() {
        return Ok(vec![PageInfo {
            id: "default".to_string(),
            name: "Default".to_string(),
            width: project.canvas.width,
            height: project.canvas.height,
            element_count: project.elements.len(),
            active: true,
        }]);
    }

    let active_idx = project.active_page_index_clamped();
    let result: Vec<PageInfo> = project.pages.iter().enumerate().map(|(i, p)| PageInfo {
        id: p.id.clone(),
        name: p.name.clone(),
        width: p.canvas.width,
        height: p.canvas.height,
        element_count: p.elements.len(),
        active: i == active_idx,
    }).collect();

    Ok(result)
}

#[tauri::command]
pub fn add_page(
    app_handle: AppHandle,
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    name: String,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<PageInfo, String> {
    let w = width.unwrap_or(512);
    let h = height.unwrap_or(512);
    if w == 0 || h == 0 || w > 8192 || h > 8192 {
        return Err(AppError::ValidationError("Invalid canvas dimensions".into()).into());
    }

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    // Migrate to pages if needed
    if project.pages.is_empty() {
        let first_page = Page::from_project(&project);
        project.pages.push(first_page);
    }

    let new_page = Page::new(&name, w, h);
    let info = PageInfo {
        id: new_page.id.clone(),
        name: new_page.name.clone(),
        width: new_page.canvas.width,
        height: new_page.canvas.height,
        element_count: 0,
        active: true,
    };

    project.pages.push(new_page);
    project.active_page_index = project.pages.len() - 1;
    project.bump_version();
    drop(project);

    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();
    drop(cache);

    let _ = app_handle.emit("project-changed", ());
    Ok(info)
}

#[tauri::command]
pub fn switch_page(
    app_handle: AppHandle,
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    page_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    if project.pages.is_empty() {
        return Err(AppError::ValidationError("Project has no pages".into()).into());
    }

    let idx = project.pages.iter().position(|p| p.id == page_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Page '{}' not found", page_id)))?;

    project.active_page_index = idx;
    project.bump_version();
    drop(project);

    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();
    drop(cache);

    let _ = app_handle.emit("project-changed", ());
    Ok(())
}

#[tauri::command]
pub fn delete_page(
    app_handle: AppHandle,
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    page_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    if project.pages.is_empty() {
        return Err(AppError::ValidationError("Project has no pages to delete".into()).into());
    }

    if project.pages.len() <= 1 {
        return Err(AppError::ValidationError("Cannot delete the last page".into()).into());
    }

    let idx = project.pages.iter().position(|p| p.id == page_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Page '{}' not found", page_id)))?;

    project.pages.remove(idx);

    if project.active_page_index >= project.pages.len() {
        project.active_page_index = project.pages.len() - 1;
    } else if idx < project.active_page_index {
        project.active_page_index -= 1;
    } else if idx == project.active_page_index {
        project.active_page_index = project.active_page_index.min(project.pages.len() - 1);
    }

    project.bump_version();
    drop(project);

    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();
    drop(cache);

    let _ = app_handle.emit("project-changed", ());
    Ok(())
}

#[tauri::command]
pub fn duplicate_page(
    app_handle: AppHandle,
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    page_id: String,
    name: String,
) -> Result<PageInfo, String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    let idx = project.pages.iter().position(|p| p.id == page_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Page '{}' not found", page_id)))?;

    let mut clone = project.pages[idx].clone();
    clone.id = format!("page-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis());
    clone.name = name.clone();

    let info = PageInfo {
        id: clone.id.clone(),
        name: clone.name.clone(),
        width: clone.canvas.width,
        height: clone.canvas.height,
        element_count: clone.elements.len(),
        active: false,
    };

    project.pages.push(clone);
    project.bump_version();
    drop(project);

    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();
    drop(cache);

    let _ = app_handle.emit("project-changed", ());
    Ok(info)
}

#[tauri::command]
pub fn rename_page(
    app_handle: AppHandle,
    state: State<'_, ProjectState>,
    page_id: String,
    name: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    let page = project.pages.iter_mut().find(|p| p.id == page_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Page '{}' not found", page_id)))?;

    page.name = name;
    project.bump_version();
    drop(project);

    let _ = app_handle.emit("project-changed", ());
    Ok(())
}

#[tauri::command]
pub fn get_active_page(state: State<'_, ProjectState>) -> Result<usize, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    Ok(project.active_page_index_clamped())
}
