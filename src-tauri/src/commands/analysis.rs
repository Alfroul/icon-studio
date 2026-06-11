use tauri::State;

use super::canvas::ProjectState;
use crate::engine::analyzer::{self, ElementFilter};

#[tauri::command]
pub fn analyze_colors(state: State<'_, ProjectState>) -> Result<analyzer::ColorAnalysis, String> {
    let project = state.lock().map_err(|e| e.to_string())?;
    Ok(analyzer::analyze_colors(&project))
}

#[tauri::command]
pub fn check_consistency(
    state: State<'_, ProjectState>,
) -> Result<analyzer::ConsistencyReport, String> {
    let project = state.lock().map_err(|e| e.to_string())?;
    Ok(analyzer::check_consistency(&project))
}

#[tauri::command]
pub fn find_elements(
    state: State<'_, ProjectState>,
    element_type: Option<String>,
    fill: Option<String>,
    min_width: Option<f64>,
    max_width: Option<f64>,
) -> Result<analyzer::FindResult, String> {
    let project = state.lock().map_err(|e| e.to_string())?;
    let filter = ElementFilter {
        element_type,
        fill,
        min_width,
        max_width,
    };
    Ok(analyzer::find_elements(&project, &filter))
}

#[tauri::command]
pub fn fix_consistency(
    state: State<'_, ProjectState>,
    element_ids: Vec<String>,
) -> Result<String, String> {
    let project = state.lock().map_err(|e| e.to_string())?;
    let fixed = analyzer::fix_consistency_issues(&project, &element_ids)?;
    drop(project);

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    *guard = fixed;

    Ok("ok".to_string())
}
