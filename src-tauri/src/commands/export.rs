use crate::engine::builder::RenderCache;
use crate::engine::codegen::{self, CodeExportOptions};
use crate::engine::tokens::{self, TokenFormat};
use crate::engine::fontgen::{self, FontExportOptions, GlyphEntry};
use crate::error::AppError;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::State;

use super::canvas::ProjectState;

pub type RenderCacheState = Arc<Mutex<RenderCache>>;

#[tauri::command]
pub fn render_preview(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
) -> Result<String, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.build(&project).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_svg(state: State<'_, ProjectState>, cache_state: State<'_, RenderCacheState>, path: Option<String>) -> Result<String, String> {
    let svg_str = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    };

    match path {
        Some(p) => {
            crate::services::export::write_svg_to_file(&svg_str, &p)?;
            Ok(p)
        }
        None => Ok(svg_str),
    }
}

#[tauri::command]
pub fn export_png(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    sizes: Vec<u32>,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let svg_str = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    };

    crate::services::export::write_pngs_to_dir(&svg_str, &sizes, &output_dir)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_ico(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    sizes: Vec<u32>,
    path: String,
) -> Result<String, String> {
    let svg_str = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    };

    crate::services::export::write_ico_to_file(&svg_str, &sizes, &path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_android_icons(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let svg_str = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    };

    crate::engine::utils::validate_file_path(&output_dir)?;
    let paths = crate::engine::exporter::export_android_icons(&svg_str, Path::new(&output_dir))?;

    Ok(paths
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}

#[tauri::command]
pub fn export_ios_icons(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let svg_str = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    };

    crate::engine::utils::validate_file_path(&output_dir)?;
    let paths = crate::engine::exporter::export_ios_icons(&svg_str, Path::new(&output_dir))?;

    Ok(paths
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}

#[tauri::command]
pub fn export_all(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    output_dir: String,
    formats: Option<Vec<String>>,
    png_sizes: Option<Vec<u32>>,
) -> Result<Vec<String>, String> {
    let formats = formats.unwrap_or_else(|| vec!["svg".into(), "png".into(), "ico".into()]);
    let png_sizes = png_sizes.unwrap_or_else(|| vec![16, 32, 64, 128, 256, 512]);

    let svg_str = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    };

    crate::services::export::export_all_formats(&svg_str, &output_dir, &formats, &png_sizes)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_code(
    svg_content: String,
    options: CodeExportOptions,
) -> Result<codegen::CodeExportResult, String> {
    codegen::export_code(&svg_content, &options).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_tokens(
    state: State<'_, super::canvas::ProjectState>,
    format: TokenFormat,
) -> Result<tokens::TokenExportResult, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let tokens = tokens::extract_tokens(&project);
    Ok(tokens::format_tokens(&tokens, format))
}

#[tauri::command]
pub fn export_icon_font(
    glyphs: Vec<GlyphEntry>,
    options: FontExportOptions,
) -> Result<fontgen::FontExportResult, String> {
    fontgen::generate_font(&glyphs, &options).map_err(|e| e.to_string())
}
