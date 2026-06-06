use crate::engine::builder;
use crate::model::Canvas;
use crate::templates;
use crate::commands::canvas::ProjectState;
use crate::commands::export::RenderCacheState;
use crate::commands::history::HistoryState;
use tauri::State;

#[tauri::command]
pub fn list_builtin_templates() -> Result<Vec<serde_json::Value>, String> {
    let list = templates::built_in_templates();
    let mut result = Vec::with_capacity(list.len());
    for (meta, project) in &list {
        let preview_svg = builder::build(project).map_err(|e| e.to_string())?;
        result.push(serde_json::json!({
            "meta": meta,
            "preview_svg": preview_svg,
        }));
    }
    Ok(result)
}

#[tauri::command]
pub fn apply_builtin_template(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    history_state: State<'_, HistoryState>,
    index: usize,
) -> Result<Canvas, String> {
    let list = templates::built_in_templates();
    let (_, project) = list
        .into_iter()
        .nth(index)
        .ok_or_else(|| format!("Template index {} out of range (0-{})", index, 7))?;
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let canvas = project.active_canvas().clone();
    *guard = project;
    guard.bump_version();
    cache_state.lock().map_err(|e| e.to_string())?.invalidate_cache();
    history_state.lock().map_err(|e| e.to_string())?.clear();
    Ok(canvas)
}

#[tauri::command]
pub fn save_as_template(
    state: State<'_, ProjectState>,
    name: String,
    description: Option<String>,
) -> Result<(), String> {
    let project = state.lock().map_err(|e| e.to_string())?;
    templates::save_template(&name, &project)?;
    // TODO: Store description in template metadata when format supports it
    let _ = &description;
    Ok(())
}

#[tauri::command]
pub fn list_user_templates_cmd() -> Result<Vec<serde_json::Value>, String> {
    let user_templates = templates::list_user_templates();
    let mut result = Vec::with_capacity(user_templates.len());
    for meta in &user_templates {
        match templates::load_template(&meta.name) {
            Ok(project) => {
                let preview_svg = builder::build(&project).map_err(|e| e.to_string())?;
                result.push(serde_json::json!({
                    "meta": meta,
                    "preview_svg": preview_svg,
                }));
            }
            Err(_) => {
                result.push(serde_json::json!({
                    "meta": meta,
                    "preview_svg": "",
                }));
            }
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn apply_user_template(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    history_state: State<'_, HistoryState>,
    name: String,
) -> Result<Canvas, String> {
    let project = templates::load_template(&name)?;
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let canvas = project.active_canvas().clone();
    *guard = project;
    guard.bump_version();
    cache_state.lock().map_err(|e| e.to_string())?.invalidate_cache();
    history_state.lock().map_err(|e| e.to_string())?.clear();
    Ok(canvas)
}

#[tauri::command]
pub fn delete_user_template(name: String) -> Result<(), String> {
    templates::delete_template(&name)
}
