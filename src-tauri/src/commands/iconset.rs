use crate::engine;
use crate::error::AppError;
use crate::model::{IconSet, SetEntry};

use super::canvas::ProjectState;

use tauri::State;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct IconSetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub entry_count: usize,
    pub created_at: String,
}

fn set_to_info(set: &IconSet) -> IconSetInfo {
    IconSetInfo {
        id: set.id.clone(),
        name: set.name.clone(),
        description: set.description.clone(),
        entry_count: set.entries.len(),
        created_at: set.created_at.clone(),
    }
}

#[tauri::command]
pub fn list_icon_sets() -> Result<Vec<IconSetInfo>, String> {
    let sets = engine::iconset::list_sets().map_err(|e| e.to_string())?;
    Ok(sets.iter().map(set_to_info).collect())
}

#[tauri::command]
pub fn create_icon_set(
    name: String,
    description: Option<String>,
) -> Result<IconSetInfo, String> {
    let set = engine::iconset::create_set(&name, description.as_deref().unwrap_or(""))
        .map_err(|e| e.to_string())?;
    Ok(set_to_info(&set))
}

#[tauri::command]
pub fn add_to_icon_set(
    state: State<'_, ProjectState>,
    set_id: String,
    name: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<SetEntry, String> {
    let mut set = engine::iconset::load_set(&set_id)
        .map_err(|e| AppError::NotFoundError(format!("Set '{}' not found: {}", set_id, e)))
        .map_err(|e| e.to_string())?;

    let project = state.lock().map_err(|e| AppError::LockError(e.to_string())).map_err(|e| e.to_string())?;
    let entry = engine::iconset::add_entry(
        &mut set,
        &project,
        name.as_deref().unwrap_or(""),
        tags.unwrap_or_default(),
    )
    .map_err(|e| e.to_string())?;
    drop(project);

    Ok(entry)
}

#[tauri::command]
pub fn remove_from_icon_set(
    set_id: String,
    entry_id: String,
) -> Result<(), String> {
    engine::iconset::remove_entry(&set_id, &entry_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_icon_set(set_id: String) -> Result<IconSet, String> {
    engine::iconset::load_set(&set_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_icon_set(
    set_id: String,
    format: Option<String>,
    sizes: Option<Vec<u32>>,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let fmt = format.unwrap_or_else(|| "png".to_string());
    let sz = sizes.unwrap_or_else(|| vec![16, 32, 64, 128, 256, 512]);

    crate::engine::utils::validate_file_path(&output_dir)
        .map_err(|e| e.to_string())?;

    let files = engine::iconset::export_set(&set_id, &fmt, &sz, &output_dir)
        .map_err(|e| e.to_string())?;
    Ok(files.iter().map(|p| p.to_string_lossy().to_string()).collect())
}

#[derive(serde::Serialize)]
pub struct ConsistencyReportInfo {
    pub consistent: bool,
    pub issues: Vec<crate::model::ConsistencyIssue>,
    pub summary: String,
}

#[tauri::command]
pub fn check_icon_set_consistency(set_id: String) -> Result<ConsistencyReportInfo, String> {
    let report = engine::iconset::check_consistency(&set_id)
        .map_err(|e| e.to_string())?;
    Ok(ConsistencyReportInfo {
        consistent: report.consistent,
        issues: report.issues,
        summary: report.summary,
    })
}

#[tauri::command]
pub fn tag_icon_entry(
    set_id: String,
    entry_id: String,
    tags: Vec<String>,
) -> Result<(), String> {
    engine::iconset::tag_entry(&set_id, &entry_id, tags)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_icons(
    query: String,
    set_id: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<Vec<SetEntry>, String> {
    engine::iconset::search_entries(&query, tags.as_ref(), set_id.as_deref())
        .map_err(|e| e.to_string())
}
