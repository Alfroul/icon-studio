use icon_studio_lib::engine::exporter;
use std::fs;
use tempfile::TempDir;

fn test_svg() -> &'static str {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><circle cx="50" cy="50" r="40" fill="red"/></svg>"#
}

#[test]
fn test_export_ico() {
    let dir = TempDir::new().unwrap();
    let ico_path = dir.path().join("test.ico");
    let result = exporter::export_ico(test_svg(), &[16, 32, 48], &ico_path).unwrap();
    assert!(ico_path.exists());
    assert!(fs::metadata(&ico_path).unwrap().len() > 0);
    let _ = result;
}

#[test]
fn test_export_ico_default_sizes() {
    let dir = TempDir::new().unwrap();
    let ico_path = dir.path().join("default.ico");
    exporter::export_ico(test_svg(), &[], &ico_path).unwrap();
    assert!(ico_path.exists());
}

#[test]
fn test_export_android_icons() {
    let dir = TempDir::new().unwrap();
    let paths = exporter::export_android_icons(test_svg(), dir.path()).unwrap();
    assert_eq!(paths.len(), 5);
    assert!(dir.path().join("mipmap-mdpi/ic_launcher.png").exists());
    assert!(dir.path().join("mipmap-xxxhdpi/ic_launcher.png").exists());
    let mdpi_size = fs::metadata(dir.path().join("mipmap-mdpi/ic_launcher.png")).unwrap().len();
    let xxxhdpi_size = fs::metadata(dir.path().join("mipmap-xxxhdpi/ic_launcher.png")).unwrap().len();
    assert!(xxxhdpi_size > mdpi_size);
}

#[test]
fn test_export_ios_icons() {
    let dir = TempDir::new().unwrap();
    let paths = exporter::export_ios_icons(test_svg(), dir.path()).unwrap();
    assert_eq!(paths.len(), 14);
    assert!(dir.path().join("AppIcon.appiconset/Contents.json").exists());
    assert!(dir.path().join("AppIcon.appiconset/Icon-40.png").exists());
    assert!(dir.path().join("AppIcon.appiconset/Icon-1024.png").exists());
    let contents = fs::read_to_string(dir.path().join("AppIcon.appiconset/Contents.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
    let images = parsed["images"].as_array().unwrap();
    assert_eq!(images.len(), 14);
    assert_eq!(images[0]["size"].as_str().unwrap(), "20x20");
    assert_eq!(images[0]["scale"].as_str().unwrap(), "2x");
}

#[test]
fn test_render_to_png() {
    let png = exporter::render_to_png(test_svg(), 64).unwrap();
    assert!(!png.is_empty());
    assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
}
