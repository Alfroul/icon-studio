use crate::engine::builder::RenderCache;
use crate::engine::codegen::{self, CodeExportOptions};
use crate::engine::exporter::{self, AllPlatformsResult, SpriteSheetResult};
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
    pixel_snap: Option<bool>,
) -> Result<Vec<String>, String> {
    let svg_str = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    };

    let svg_str = if pixel_snap.unwrap_or(false) {
        exporter::snap_to_pixel_grid(&svg_str, 0.5).map_err(|e| e.to_string())?
    } else {
        svg_str
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
    pixel_snap: Option<bool>,
) -> Result<Vec<String>, String> {
    let formats = formats.unwrap_or_else(|| vec!["svg".into(), "png".into(), "ico".into()]);
    let png_sizes = png_sizes.unwrap_or_else(|| vec![16, 32, 64, 128, 256, 512]);

    let svg_str = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    };

    let svg_str = if pixel_snap.unwrap_or(false) {
        exporter::snap_to_pixel_grid(&svg_str, 0.5).map_err(|e| e.to_string())?
    } else {
        svg_str
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

#[tauri::command]
pub fn export_overlay_svg(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    include_overlays: Option<bool>,
) -> Result<String, String> {
    let include = include_overlays.unwrap_or(true);
    let svg_str = if include {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    } else {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut stripped = project.clone();
        for elem in stripped.active_elements_mut() {
            strip_overlays(elem);
        }
        crate::engine::builder::build(&stripped)?
    };
    Ok(svg_str)
}

fn strip_overlays(elem: &mut crate::model::Element) {
    elem.common_mut().overlay = None;
    if let crate::model::Element::Group(g) = elem {
        for child in &mut g.children {
            strip_overlays(child);
        }
    }
}

#[tauri::command]
pub fn quick_export(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    profile: String,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let svg_str = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    };

    crate::engine::utils::validate_file_path(&output_dir)?;

    match profile.as_str() {
        "ios" => {
            let paths = crate::engine::exporter::export_ios_icons(&svg_str, Path::new(&output_dir))?;
            Ok(paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect())
        }
        "android" => {
            let paths = crate::engine::exporter::export_android_icons(&svg_str, Path::new(&output_dir))?;
            Ok(paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect())
        }
        "all" => {
            crate::services::export::export_all_formats(
                &svg_str,
                &output_dir,
                &["svg".into(), "png".into(), "ico".into()],
                &[16, 32, 64, 128, 256, 512],
            ).map_err(|e| e.to_string())
        }
        "svg" => {
            let path = format!("{}/icon.svg", output_dir.trim_end_matches('/'));
            crate::services::export::write_svg_to_file(&svg_str, &path)?;
            Ok(vec![path])
        }
        "png" => {
            crate::services::export::write_pngs_to_dir(
                &svg_str,
                &[16, 32, 64, 128, 256, 512],
                &output_dir,
            ).map_err(|e| e.to_string())
        }
        "font" => {
            // Font export requires glyphs data - return a hint to use full export
            Err("Use export_icon_font command with glyph data".into())
        }
        other => Err(format!("Unknown export profile: {}. Use ios, android, all, svg, png, or font", other)),
    }
}

#[tauri::command]
pub fn get_favicon_html_snippet(app_name: String) -> Result<String, String> {
    Ok(exporter::get_favicon_html_snippet(&app_name))
}

#[tauri::command]
pub fn export_pwa_icons(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    output_dir: String,
    app_name: String,
    theme_color: Option<String>,
    bg_color: Option<String>,
) -> Result<Vec<String>, String> {
    let svg_str = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    };

    let theme = theme_color.as_deref().unwrap_or("#FFFFFF");
    let bg = bg_color.as_deref().unwrap_or("#FFFFFF");
    let paths = exporter::export_pwa_icons(&svg_str, Path::new(&output_dir), &app_name, theme, bg)
        .map_err(|e| e.to_string())?;

    Ok(paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect())
}

#[tauri::command]
pub fn export_all_platforms(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    output_dir: String,
    app_name: String,
    theme_color: Option<String>,
    bg_color: Option<String>,
) -> Result<AllPlatformsResult, String> {
    let svg_str = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    };

    let theme = theme_color.as_deref().unwrap_or("#FFFFFF");
    let bg = bg_color.as_deref().unwrap_or("#FFFFFF");
    exporter::export_all_platforms(&svg_str, Path::new(&output_dir), &app_name, theme, bg)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_sprite_sheet(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    output_path: String,
    columns: u32,
    icon_size: u32,
    padding: Option<u32>,
) -> Result<SpriteSheetResult, String> {
    let svg_str = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::services::export::build_svg(&project, &mut cache)?
    };

    let padding = padding.unwrap_or(0);
    let svgs: &[(&str, &str)] = &[("icon", &svg_str)];
    exporter::export_sprite_sheet(svgs, columns, icon_size, padding, Path::new(&output_path))
        .map_err(|e| e.to_string())
}
