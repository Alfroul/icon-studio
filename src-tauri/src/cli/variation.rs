use anyhow::{Context, Result};
use clap::Parser;
use rayon::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::engine::builder;
use crate::engine::exporter;
use crate::engine::utils::validate_file_path;
use crate::engine::variation::{self, VariationConfig};
use crate::model::IconProject;

#[derive(Parser)]
pub struct VariationArgs {
    /// Path to .iconproject.json file
    #[arg(long)]
    pub input: String,

    /// Path to variation config JSON file
    #[arg(long)]
    pub config: String,

    /// Output directory (overrides config's output_dir)
    #[arg(long)]
    pub output: Option<String>,

    /// Export format: png or svg (default: png)
    #[arg(long, default_value = "png")]
    pub format: String,

    /// Comma-separated export sizes (default: 16,32,64,128,256,512)
    #[arg(long, default_value = "16,32,64,128,256,512")]
    pub sizes: String,
}

pub fn run(args: &VariationArgs) -> Result<()> {
    let input_path = Path::new(&args.input);
    let config_path = Path::new(&args.config);

    let project = load_project(input_path)?;
    let config = load_config(config_path)?;

    let output_dir = match &args.output {
        Some(dir) => dir.clone(),
        None => config.output_dir.clone(),
    };
    let output_path = validate_file_path(&output_dir)
        .map_err(|e| anyhow::anyhow!(e))?;
    std::fs::create_dir_all(&output_path).context("Failed to create output directory")?;
    let output_path = output_path.as_path();

    let sizes = parse_sizes(&args.sizes).map_err(|e| anyhow::anyhow!(e))?;
    let format = args.format.as_str();
    let naming = config.naming.clone();
    let stem = file_stem(input_path);

    let variations = variation::generate_variations(&project, &config);
    let total = variations.len();

    println!("Generating {} variations...", total);

    let counter = AtomicUsize::new(0);
    let errors: Mutex<Vec<String>> = Mutex::new(Vec::new());

    variations.par_iter().for_each(|(var_name, var_project)| {
        let idx = counter.fetch_add(1, Ordering::SeqCst) + 1;
        let out_name = naming
            .replace("{name}", &stem)
            .replace("{variation}", var_name);

        if let Err(e) = export_variation(var_project, output_path, &out_name, format, &sizes) {
            eprintln!("[{}/{}] ✗ {}: {}", idx, total, var_name, e);
            errors
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(format!("{}: {}", var_name, e));
        } else {
            println!("[{}/{}] ✓ {}", idx, total, var_name);
        }
    });

    let error_list = errors.lock().unwrap_or_else(|e| e.into_inner());
    let err_count = error_list.len();
    if err_count > 0 {
        eprintln!("\n{} of {} variations failed:", err_count, total);
        for err in error_list.iter() {
            eprintln!("  - {}", err);
        }
        anyhow::bail!(
            "{} of {} variations failed",
            err_count,
            total
        );
    } else {
        println!("\nAll {} variations exported successfully.", total);
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

fn load_config(path: &Path) -> Result<VariationConfig> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config {}", path.display()))?;
    serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse config {}", path.display()))
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
            Ok(_) => return Err(format!("尺寸 {} 无效，允许范围: 1-8192", trimmed)),
            Err(_) => return Err(format!("无法解析尺寸: {}", trimmed)),
        }
    }
    if result.is_empty() {
        return Err("No valid sizes provided".to_string());
    }
    Ok(result)
}

fn export_variation(
    project: &IconProject,
    output_dir: &Path,
    name: &str,
    format: &str,
    sizes: &[u32],
) -> Result<()> {
    let svg = builder::build(project).context("Failed to build SVG")?;

    match format {
        "svg" => {
            let path = output_dir.join(format!("{}.svg", name));
            std::fs::write(&path, &svg)?;
        }
        "png" => {
            for &size in sizes {
                let path = output_dir.join(format!("{}_{}.png", name, size));
                let png_bytes = exporter::render_to_png(&svg, size)?;
                std::fs::write(&path, &png_bytes)?;
            }
        }
        other => anyhow::bail!("Unknown format '{}'. Use: svg, png", other),
    }

    Ok(())
}
