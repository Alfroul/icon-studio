use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use icon_studio_lib::engine::builder;
use icon_studio_lib::engine::exporter;
use icon_studio_lib::engine::analyzer;
use icon_studio_lib::model::IconProject;

fn create_test_project(dir: &std::path::Path, name: &str) -> PathBuf {
    let project = serde_json::json!({
        "schema_version": "1.0",
        "canvas": {
            "width": 64,
            "height": 64,
            "background": "#3B82F6",
            "corner_radius": 0
        },
        "elements": [
            {
                "type": "shape",
                "id": "shape-1",
                "shape_type": "circle",
                "x": 12.0,
                "y": 12.0,
                "width": 40.0,
                "height": 40.0,
                "fill": "#FFFFFF",
                "stroke": null,
                "stroke_width": 0.0,
                "opacity": 1.0,
                "rotation": 0.0
            }
        ],
        "exports": {
            "formats": ["svg", "png"],
            "sizes": [16, 32, 64]
        },
        "templates": {}
    });
    let path = dir.join(format!("{}.iconproject.json", name));
    fs::write(&path, serde_json::to_string_pretty(&project).unwrap()).unwrap();
    path
}

fn load_project(path: &std::path::Path) -> IconProject {
    let data = fs::read_to_string(path).unwrap();
    let mut project: IconProject = serde_json::from_str(&data).unwrap();
    project.recalc_next_element_id();
    project
}

fn file_stem(path: &std::path::Path) -> String {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("icon");
    name.strip_suffix(".iconproject.json")
        .or_else(|| name.strip_suffix(".json"))
        .unwrap_or(name)
        .to_string()
}

#[test]
fn test_export_svg() {
    let tmp = TempDir::new().unwrap();
    let input = create_test_project(tmp.path(), "test-icon");
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let project = load_project(&input);
    let svg = builder::build(&project).unwrap();

    let svg_path = output_dir.join(format!("{}.svg", file_stem(&input)));
    fs::write(&svg_path, &svg).unwrap();

    assert!(svg_path.exists());
    let content = fs::read_to_string(&svg_path).unwrap();
    assert!(content.contains("<svg"));
}

#[test]
fn test_export_png() {
    let tmp = TempDir::new().unwrap();
    let input = create_test_project(tmp.path(), "test-icon");
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let project = load_project(&input);
    let svg = builder::build(&project).unwrap();

    for &size in &[16u32, 32] {
        let path = output_dir.join(format!("{}_{}.png", file_stem(&input), size));
        let png_bytes = exporter::render_to_png(&svg, size).unwrap();
        fs::write(&path, &png_bytes).unwrap();
        assert!(path.exists());
    }
}

#[test]
fn test_export_ico() {
    let tmp = TempDir::new().unwrap();
    let input = create_test_project(tmp.path(), "test-icon");
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let project = load_project(&input);
    let svg = builder::build(&project).unwrap();

    let ico_path = output_dir.join(format!("{}.ico", file_stem(&input)));
    exporter::export_ico(&svg, &[16, 32, 48], &ico_path).unwrap();
    assert!(ico_path.exists());
}

#[test]
fn test_analyze_single_file() {
    let tmp = TempDir::new().unwrap();
    let input = create_test_project(tmp.path(), "app");

    let project = load_project(&input);
    let colors = analyzer::analyze_colors(&project);
    let consistency = analyzer::check_consistency(&project);

    assert!(colors.primary.is_some(), "Should have a primary color");
    assert!(consistency.opacity_consistent, "Opacity should be consistent");
}

#[test]
fn test_analyze_json_output() {
    let tmp = TempDir::new().unwrap();
    let input = create_test_project(tmp.path(), "app");

    let project = load_project(&input);
    let result = analyzer::AnalysisResult {
        colors: analyzer::analyze_colors(&project),
        consistency: analyzer::check_consistency(&project),
    };

    let json = serde_json::to_string_pretty(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["colors"].is_object());
    assert!(parsed["consistency"].is_object());
}

#[test]
fn test_batch_export() {
    let tmp = TempDir::new().unwrap();
    let input_a = create_test_project(tmp.path(), "icon-a");
    let input_b = create_test_project(tmp.path(), "icon-b");
    let output_dir = tmp.path().join("batch-output");
    fs::create_dir_all(&output_dir).unwrap();

    for input in &[input_a, input_b] {
        let project = load_project(input);
        let svg = builder::build(&project).unwrap();
        let stem = file_stem(input);
        let svg_path = output_dir.join(format!("{}.svg", stem));
        fs::write(&svg_path, &svg).unwrap();
    }

    assert!(output_dir.join("icon-a.svg").exists());
    assert!(output_dir.join("icon-b.svg").exists());
}
