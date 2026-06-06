use icon_studio_lib::engine::{builder, exporter, generator};
use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::*;
use std::fs;
use tempfile::TempDir;

fn test_svg() -> &'static str {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><circle cx="50" cy="50" r="40" fill="red"/></svg>"#
}

#[test]
fn test_export_favicon_package() {
    let dir = TempDir::new().unwrap();
    let paths = exporter::export_favicon_package(
        test_svg(),
        dir.path(),
        "TestApp",
        Some("#FFFFFF"),
        Some("#000000"),
    )
    .unwrap();

    assert!(dir.path().join("favicon.ico").exists());
    assert!(dir.path().join("favicon-16x16.png").exists());
    assert!(dir.path().join("favicon-32x32.png").exists());
    assert!(dir.path().join("favicon.svg").exists());
    assert!(dir.path().join("apple-touch-icon.png").exists());
    assert!(dir.path().join("android-chrome-192x192.png").exists());
    assert!(dir.path().join("android-chrome-512x512.png").exists());
    assert!(dir.path().join("site.webmanifest").exists());

    let manifest = fs::read_to_string(dir.path().join("site.webmanifest")).unwrap();
    assert!(manifest.contains("TestApp"));
    assert!(manifest.contains("#FFFFFF"));
    assert!(manifest.contains("android-chrome-192x192.png"));

    assert_eq!(paths.len(), 8);
}

#[test]
fn test_generate_random_minimal() {
    let config = generator::GeneratorConfig {
        style: Some("minimal".to_string()),
        base_color: Some("#3498DB".to_string()),
        text: Some("A".to_string()),
        icon_name: None,
        size: 512,
    };
    let project = generator::generate_random(&config);

    assert_eq!(project.canvas.width, 512);
    assert_eq!(project.canvas.height, 512);
    assert!(!project.elements.is_empty());

    let svg = builder::build(&project).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_generate_random_lettermark() {
    let config = generator::GeneratorConfig {
        style: Some("lettermark".to_string()),
        base_color: Some("#E74C3C".to_string()),
        text: Some("Z".to_string()),
        icon_name: None,
        size: 256,
    };
    let project = generator::generate_random(&config);

    assert_eq!(project.canvas.width, 256);
    assert!(project.elements.len() >= 2);

    let has_text = project.elements.iter().any(|e| matches!(e, Element::Text(t) if t.content == "Z"));
    assert!(has_text, "lettermark should contain the specified letter");
}

#[test]
fn test_generate_random_badge() {
    let config = generator::GeneratorConfig {
        style: Some("badge".to_string()),
        base_color: None,
        text: Some("OK".to_string()),
        icon_name: None,
        size: 512,
    };
    let project = generator::generate_random(&config);

    assert_eq!(project.canvas.background, "transparent");

    let has_shield = project.elements.iter().any(|e| {
        matches!(e, Element::Shape(s) if matches!(s.shape_type, ShapeType::Shield))
    });
    assert!(has_shield, "badge style should include a shield shape");
}

#[test]
fn test_generate_random_geometric() {
    let config = generator::GeneratorConfig {
        style: Some("geometric".to_string()),
        base_color: Some("#2ECC71".to_string()),
        text: None,
        icon_name: None,
        size: 512,
    };
    let project = generator::generate_random(&config);

    assert!(project.elements.len() >= 2, "geometric should have multiple shapes");

    let svg = builder::build(&project).unwrap();
    let png = icon_studio_lib::engine::renderer::render(&svg, 64).unwrap();
    assert!(!png.is_empty());
}

#[test]
fn test_generate_random_default() {
    let config = generator::GeneratorConfig::default();
    let project = generator::generate_random(&config);

    assert_eq!(project.canvas.width, 512);
    assert!(!project.elements.is_empty());
}

#[test]
fn test_generate_and_export_favicon() {
    let config = generator::GeneratorConfig {
        style: Some("lettermark".to_string()),
        base_color: Some("#9B59B6".to_string()),
        text: Some("W".to_string()),
        icon_name: None,
        size: 512,
    };
    let project = generator::generate_random(&config);
    let svg = builder::build(&project).unwrap();

    let dir = TempDir::new().unwrap();
    let paths = exporter::export_favicon_package(
        &svg,
        dir.path(),
        "MyWebsite",
        Some("#9B59B6"),
        Some("#FFFFFF"),
    )
    .unwrap();

    assert!(dir.path().join("favicon.ico").exists());
    assert!(dir.path().join("site.webmanifest").exists());

    let ico_size = fs::metadata(dir.path().join("favicon.ico")).unwrap().len();
    assert!(ico_size > 0);

    assert_eq!(paths.len(), 8);
}
