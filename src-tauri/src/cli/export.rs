use anyhow::{Context, Result};
use clap::Parser;
use std::path::Path;

use crate::engine::builder;
use crate::engine::exporter;
use crate::engine::utils::validate_file_path;
use crate::model::IconProject;

#[derive(Parser)]
pub struct ExportArgs {
    /// Path to .iconproject.json file
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

pub fn run(args: &ExportArgs) -> Result<()> {
    let input_path = Path::new(&args.input);
    let output_dir = validate_file_path(&args.output)
        .map_err(|e| anyhow::anyhow!(e))?;
    let output_dir = output_dir.as_path();

    let project = load_project(input_path)?;
    let svg = builder::build(&project).context("Failed to build SVG")?;

    std::fs::create_dir_all(output_dir).context("Failed to create output directory")?;

    let stem = file_stem(input_path);

    let sizes = parse_sizes(&args.sizes).map_err(|e| anyhow::anyhow!(e))?;

    match args.format.as_str() {
        "svg" => export_svg(&svg, output_dir, &stem)?,
        "png" => export_pngs(&svg, output_dir, &stem, &sizes)?,
        "ico" => export_ico(&svg, output_dir, &stem)?,
        "webp" => export_webp(&svg, output_dir, &stem)?,
        "all" => {
            export_svg(&svg, output_dir, &stem)?;
            export_pngs(&svg, output_dir, &stem, &sizes)?;
            export_ico(&svg, output_dir, &stem)?;
            export_webp(&svg, output_dir, &stem)?;
        }
        other => anyhow::bail!("Unknown format '{}'. Use: svg, png, ico, webp, all", other),
    }

    Ok(())
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
    // Handle .iconproject.json → strip both suffixes
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
            Ok(_) => return Err(format!("尺寸 {} 无效，允许范围: 1-8192", trimmed)),
            Err(_) => return Err(format!("无法解析尺寸: {}", trimmed)),
        }
    }
    if result.is_empty() {
        return Err("No valid sizes provided".to_string());
    }
    Ok(result)
}

fn export_svg(svg: &str, output_dir: &Path, stem: &str) -> Result<()> {
    let path = output_dir.join(format!("{}.svg", stem));
    std::fs::write(&path, svg)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    println!("  {}", path.display());
    Ok(())
}

fn export_pngs(svg: &str, output_dir: &Path, stem: &str, sizes: &[u32]) -> Result<()> {
    for &size in sizes {
        let path = output_dir.join(format!("{}_{}.png", stem, size));
        let png_bytes = exporter::render_to_png(svg, size)
            .with_context(|| format!("Failed to render PNG at {}px", size))?;
        std::fs::write(&path, &png_bytes)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        println!("  {}", path.display());
    }
    Ok(())
}

fn export_ico(svg: &str, output_dir: &Path, stem: &str) -> Result<()> {
    let path = output_dir.join(format!("{}.ico", stem));
    let ico_sizes: Vec<u32> = vec![16, 32, 48, 64, 128, 256];
    exporter::export_ico(svg, &ico_sizes, &path)
        .with_context(|| "Failed to export ICO")?;
    println!("  {}", path.display());
    Ok(())
}

fn export_webp(svg: &str, output_dir: &Path, stem: &str) -> Result<()> {
    let path = output_dir.join(format!("{}.webp", stem));
    let webp_bytes = exporter::render_to_webp(svg, 512)
        .with_context(|| "Failed to render WebP")?;
    std::fs::write(&path, &webp_bytes)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    println!("  {}", path.display());
    Ok(())
}
