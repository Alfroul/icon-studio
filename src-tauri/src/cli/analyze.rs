use anyhow::{Context, Result};
use clap::Parser;
use std::path::Path;

use crate::engine::analyzer::{self, AnalysisResult};
use crate::model::IconProject;

#[derive(Parser)]
pub struct AnalyzeArgs {
    /// Path to .iconproject.json file or directory
    #[arg(long)]
    pub input: String,

    /// Output format: text or json
    #[arg(long, default_value = "text")]
    pub format: String,
}

pub fn run(args: &AnalyzeArgs) -> Result<()> {
    let input_path = Path::new(&args.input);

    if input_path.is_dir() {
        let files = collect_project_files(input_path);
        if files.is_empty() {
            anyhow::bail!("No .iconproject.json files found in {}", input_path.display());
        }
        for file in &files {
            analyze_file(file, &args.format)?;
        }
    } else {
        analyze_file(input_path, &args.format)?;
    }

    Ok(())
}

fn collect_project_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    collect_project_files_recursive(dir, &mut files);
    files.sort();
    files
}

fn collect_project_files_recursive(dir: &Path, result: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_project_files_recursive(&path, result);
            } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".iconproject.json") {
                        result.push(path);
                    }
                }
            }
        }
    }
}

fn analyze_file(path: &Path, format: &str) -> Result<()> {
    let project = load_project(path)?;
    let result = AnalysisResult {
        colors: analyzer::analyze_colors(&project),
        consistency: analyzer::check_consistency(&project),
    };

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&result)
                .context("Failed to serialize analysis result")?;
            println!("{}", json);
        }
        _ => print_text_report(path, &result),
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

fn print_text_report(path: &Path, result: &AnalysisResult) {
    let filename = path.file_name().unwrap_or_default().to_string_lossy();
    println!("📊 {}", filename);

    if let Some(ref primary) = result.colors.primary {
        println!("  主色: {} (使用 {} 次)", primary.hex, primary.usage_count);
    }
    if !result.colors.secondary.is_empty() {
        let secs: Vec<&str> = result.colors.secondary.iter().map(|c| c.hex.as_str()).collect();
        println!("  辅色: {}", secs.join(", "));
    }

    let report = &result.consistency;
    if report.stroke_width_consistent {
        println!("  一致性: ✅ 描边粗细一致");
    } else {
        let issues: Vec<String> = report
            .issues
            .iter()
            .filter(|i| i.property == "stroke_width")
            .map(|i| format!("{}px vs {}px, 元素 {}", i.expected, i.actual, i.element_id))
            .collect();
        println!("  一致性: ⚠️ 描边粗细不一致 ({})", issues.join("; "));
    }

    if report.font_size_consistent {
        println!("         ✅ 字号一致");
    } else {
        let issues: Vec<String> = report
            .issues
            .iter()
            .filter(|i| i.property == "font_size")
            .map(|i| format!("{}px vs {}px, 元素 {}", i.expected, i.actual, i.element_id))
            .collect();
        println!("         ⚠️ 字号不一致 ({})", issues.join("; "));
    }

    if report.opacity_consistent {
        println!("         ✅ 透明度一致");
    } else {
        let issues: Vec<String> = report
            .issues
            .iter()
            .filter(|i| i.property == "opacity")
            .map(|i| format!("{} vs {}, 元素 {}", i.expected, i.actual, i.element_id))
            .collect();
        println!("         ⚠️ 透明度不一致 ({})", issues.join("; "));
    }

    println!();
}
