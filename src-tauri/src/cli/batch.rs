use anyhow::{Context, Result};
use clap::Parser;
use std::path::Path;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::engine::builder;
use crate::engine::exporter;
use crate::engine::utils::validate_file_path;
use crate::model::IconProject;

#[derive(Parser)]
pub struct BatchArgs {
    /// Directory containing .iconproject.json files
    #[arg(long)]
    pub input: String,

    /// Export format: svg, png, ico, webp, all
    #[arg(long, value_name = "FORMAT")]
    pub format: String,

    /// Comma-separated sizes for PNG export (default: 16,32,64,128,256,512)
    #[arg(long, default_value = "16,32,64,128,256,512")]
    pub sizes: String,

    /// Output directory (default: current directory)
    #[arg(long, default_value = ".")]
    pub output: String,
}

pub fn run(args: &BatchArgs) -> Result<()> {
    let input_dir = Path::new(&args.input);
    if !input_dir.is_dir() {
        anyhow::bail!("{} is not a directory", input_dir.display());
    }

    let files = collect_project_files(input_dir);
    if files.is_empty() {
        anyhow::bail!("No .iconproject.json files found in {}", input_dir.display());
    }

    let output_dir = validate_file_path(&args.output)
        .map_err(|e| anyhow::anyhow!(e))?;
    std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;
    let output_dir = output_dir.as_path();

    let total = files.len();
    let format = args.format.as_str();
    let sizes = parse_sizes(&args.sizes).map_err(|e| anyhow::anyhow!(e))?;
    let counter = AtomicUsize::new(0);
    let errors: Mutex<Vec<String>> = Mutex::new(Vec::new());

    files.par_iter().for_each(|file| {
        let idx = counter.fetch_add(1, Ordering::SeqCst) + 1;
        let stem = file_stem(file);
        println!("[{}/{}] exporting {}...", idx, total, stem);

        if let Err(e) = export_single(file, output_dir, &stem, format, &sizes) {
            eprintln!("[{}/{}] ✗ {}: {}", idx, total, stem, e);
            errors.lock().unwrap_or_else(|e| e.into_inner()).push(format!("{}: {}", stem, e));
        }
    });

    let error_list = errors.lock().unwrap_or_else(|e| e.into_inner());
    let err_count = error_list.len();
    if err_count > 0 {
        eprintln!("\n{} of {} files failed:", err_count, total);
        for err in error_list.iter() {
            eprintln!("  - {}", err);
        }
        anyhow::bail!("{} of {} files failed during batch export", err_count, total);
    } else {
        println!("\nAll {} files exported successfully.", total);
    }

    Ok(())
}

fn collect_project_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    collect_recursive(dir, &mut files);
    files.sort();
    files
}

fn collect_recursive(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_recursive(&path, files);
            } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".iconproject.json") {
                        files.push(path);
                    }
                }
            }
        }
    }
}

fn load_project(path: &Path) -> Result<IconProject> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let mut project: IconProject = serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    project.recalc_next_element_id();
    Ok(project)
}

fn file_stem(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("icon");
    let stem = name
        .strip_suffix(".iconproject.json")
        .or_else(|| name.strip_suffix(".json"))
        .unwrap_or(name);
    stem.to_string()
}

fn parse_sizes(sizes: &str) -> Result<Vec<u32>, String> {
    let mut result = Vec::new();
    for part in sizes.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        match trimmed.parse::<u32>() {
            Ok(v) if v > 0 && v <= 8192 => result.push(v),
            Ok(_) => return Err(format!("尺寸 {} 无效，有效范围 1-8192", trimmed)),
            Err(_) => return Err(format!("无法解析尺寸: {}", trimmed)),
        }
    }
    if result.is_empty() {
        return Err("No valid sizes provided".to_string());
    }
    Ok(result)
}

fn export_single(
    file: &Path,
    output_dir: &Path,
    stem: &str,
    format: &str,
    sizes: &[u32],
) -> Result<()> {
    let project = load_project(file)?;
    let svg = builder::build(&project).context("Failed to build SVG")?;

    match format {
        "svg" => {
            let path = output_dir.join(format!("{}.svg", stem));
            std::fs::write(&path, &svg)?;
        }
        "png" => {
            for &size in sizes {
                let path = output_dir.join(format!("{}_{}.png", stem, size));
                let png_bytes = exporter::render_to_png(&svg, size)?;
                std::fs::write(&path, &png_bytes)?;
            }
        }
        "ico" => {
            let path = output_dir.join(format!("{}.ico", stem));
            let ico_sizes: Vec<u32> = vec![16, 32, 48, 64, 128, 256];
            exporter::export_ico(&svg, &ico_sizes, &path)?;
        }
        "webp" => {
            let path = output_dir.join(format!("{}.webp", stem));
            let webp_bytes = exporter::render_to_webp(&svg, 512)?;
            std::fs::write(&path, &webp_bytes)?;
        }
        "all" => {
            let svg_path = output_dir.join(format!("{}.svg", stem));
            std::fs::write(&svg_path, &svg)?;

            for &size in sizes {
                let png_path = output_dir.join(format!("{}_{}.png", stem, size));
                let png_bytes = exporter::render_to_png(&svg, size)?;
                std::fs::write(&png_path, &png_bytes)?;
            }

            let ico_path = output_dir.join(format!("{}.ico", stem));
            let ico_sizes: Vec<u32> = vec![16, 32, 48, 64, 128, 256];
            exporter::export_ico(&svg, &ico_sizes, &ico_path)?;

            let webp_path = output_dir.join(format!("{}.webp", stem));
            let webp_bytes = exporter::render_to_webp(&svg, 512)?;
            std::fs::write(&webp_path, &webp_bytes)?;
        }
        other => anyhow::bail!("Unknown format '{}'. Use: svg, png, ico, webp, all", other),
    }

    Ok(())
}
