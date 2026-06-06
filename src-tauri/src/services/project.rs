//! Shared project loading — used by both Tauri commands and MCP tools.

use crate::engine::importer;
use crate::engine::utils::validate_file_path;
use crate::error::AppError;
use crate::model::IconProject;

/// Load an SVG file and convert it to a new IconProject.
pub fn load_svg_project(path: &str) -> Result<IconProject, AppError> {
    validate_file_path(path)?;
    let svg_str = std::fs::read_to_string(path)?;
    let imported = importer::import_svg(&svg_str)?;
    Ok(imported)
}

/// Load a `.iconproject.json` file.
pub fn load_icon_project(path: &str) -> Result<IconProject, AppError> {
    validate_file_path(path)?;
    let content = std::fs::read_to_string(path)?;
    let mut project: IconProject = serde_json::from_str(&content)?;
    project.recalc_next_element_id();
    Ok(project)
}

/// Load either an SVG or an IconProject file, dispatching by extension.
pub fn load_project_auto(path: &str) -> Result<IconProject, AppError> {
    if path.to_lowercase().ends_with(".svg") {
        load_svg_project(path)
    } else {
        load_icon_project(path)
    }
}
