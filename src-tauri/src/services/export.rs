//! Shared export helpers — used by both Tauri commands and MCP tools.

use crate::engine::builder;
use crate::engine::exporter;
use crate::engine::utils::validate_file_path;
use crate::error::AppError;
use crate::model::IconProject;
use std::path::Path;

pub fn build_svg(
    project: &IconProject,
    cache: &mut crate::engine::builder::RenderCache,
) -> Result<String, AppError> {
    cache.build(project)
}

pub fn build_svg_uncached(project: &IconProject) -> Result<String, AppError> {
    builder::build(project)
}

pub fn write_svg_to_file(svg_str: &str, path: &str) -> Result<String, AppError> {
    validate_file_path(path)?;
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(p, svg_str)?;
    Ok(path.to_string())
}

pub fn write_pngs_to_dir(
    svg_str: &str,
    sizes: &[u32],
    output_dir: &str,
) -> Result<Vec<String>, AppError> {
    validate_file_path(output_dir)?;
    let dir = Path::new(output_dir);
    std::fs::create_dir_all(dir)?;

    let mut paths = Vec::with_capacity(sizes.len());
    for size in sizes {
        let png_bytes = exporter::render_to_png(svg_str, *size)?;
        let file_name = format!("icon-{}.png", size);
        let file_path = dir.join(&file_name);
        std::fs::write(&file_path, &png_bytes)?;
        paths.push(file_path.to_string_lossy().into_owned());
    }
    Ok(paths)
}

pub fn write_ico_to_file(
    svg_str: &str,
    sizes: &[u32],
    path: &str,
) -> Result<String, AppError> {
    validate_file_path(path)?;
    let result = exporter::export_ico(svg_str, sizes, Path::new(path))?;
    Ok(result.to_string_lossy().into_owned())
}

pub fn export_all_formats(
    svg_str: &str,
    output_dir: &str,
    formats: &[String],
    png_sizes: &[u32],
) -> Result<Vec<String>, AppError> {
    validate_file_path(output_dir)?;
    let dir = Path::new(output_dir);
    std::fs::create_dir_all(dir)?;

    let mut all_paths = Vec::new();

    if formats.iter().any(|f| f == "svg") {
        let svg_path = dir.join("icon.svg");
        std::fs::write(&svg_path, svg_str)?;
        all_paths.push(svg_path.to_string_lossy().into_owned());
    }

    if formats.iter().any(|f| f == "png") {
        for &size in png_sizes {
            let png_bytes = exporter::render_to_png(svg_str, size)?;
            let file_path = dir.join(format!("icon-{}.png", size));
            std::fs::write(&file_path, &png_bytes)?;
            all_paths.push(file_path.to_string_lossy().into_owned());
        }
    }

    if formats.iter().any(|f| f == "ico") {
        let ico_path = dir.join("icon.ico");
        let result = exporter::export_ico(svg_str, &[], &ico_path)?;
        all_paths.push(result.to_string_lossy().into_owned());
    }

    if formats.iter().any(|f| f == "webp") {
        for &size in png_sizes {
            let webp_path = dir.join(format!("icon-{}.webp", size));
            let result = exporter::export_webp(svg_str, size, &webp_path)?;
            all_paths.push(result.to_string_lossy().into_owned());
        }
    }

    Ok(all_paths)
}
