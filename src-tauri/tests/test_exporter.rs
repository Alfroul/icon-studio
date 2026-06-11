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

// ---------------------------------------------------------------------------
// Stage 17: Favicon HTML snippet tests
// ---------------------------------------------------------------------------

#[test]
fn test_generate_favicon_html_snippet() {
    let snippet = exporter::generate_favicon_html_snippet("MyApp");
    assert!(snippet.contains(r#"rel="icon" type="image/svg+xml""#));
    assert!(snippet.contains(r#"rel="icon" type="image/png" sizes="32x32""#));
    assert!(snippet.contains(r#"rel="icon" type="image/x-icon""#));
    assert!(snippet.contains(r#"rel="apple-touch-icon" sizes="180x180""#));
    assert!(snippet.contains(r#"rel="manifest""#));
    assert!(snippet.contains(r#"content="MyApp""#));
}

#[test]
fn test_generate_browserconfig_xml() {
    let xml = exporter::generate_browserconfig_xml("#FF0000");
    assert!(xml.contains("<?xml"));
    assert!(xml.contains("<browserconfig>"));
    assert!(xml.contains("<square70x70logo"));
    assert!(xml.contains("<square150x150logo"));
    assert!(xml.contains("<square310x310logo"));
    assert!(xml.contains("<wide310x150logo"));
    assert!(xml.contains("<TileColor>#FF0000</TileColor>"));
    assert!(xml.contains("</browserconfig>"));
}

#[test]
fn test_snap_to_pixel_grid() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
  <rect x="10.234" y="20.789" width="50.123" height="30.456"/>
  <path d="M10.234 20.789 L60.123 50.456"/>
  <circle cx="25.3" cy="30.7" r="15.2"/>
</svg>"#;
    let result = exporter::snap_to_pixel_grid(svg, 0.5).unwrap();
    // 10.234 with grid 0.5 → round(10.234 / 0.5) * 0.5 = round(20.468) * 0.5 = 20 * 0.5 = 10.0
    assert!(result.contains("x=\"10\""), "x should be snapped to 10, got: {}", result);
    // 20.789 → round(20.789/0.5)*0.5 = round(41.578)*0.5 = 42*0.5 = 21.0
    assert!(result.contains("y=\"21\""), "y should be snapped to 21, got: {}", result);
}

#[test]
fn test_render_to_png_snapped() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect x="10" y="10" width="80" height="80" fill="blue"/></svg>"#;
    let png = exporter::render_to_png_snapped(svg, 16, true, 0.5).unwrap();
    assert!(!png.is_empty());
    assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
}

#[test]
fn test_export_favicon_package_includes_mstile_and_snippet() {
    let dir = TempDir::new().unwrap();
    let result = exporter::export_favicon_package(
        test_svg(),
        dir.path(),
        "TestApp",
        Some("#FFFFFF"),
        Some("#FFFFFF"),
    ).unwrap();

    assert!(!result.paths.is_empty());
    assert!(!result.html_snippet.is_empty());

    // Check mstile files
    assert!(dir.path().join("mstile-70x70.png").exists());
    assert!(dir.path().join("mstile-150x150.png").exists());
    assert!(dir.path().join("mstile-310x310.png").exists());
    assert!(dir.path().join("mstile-310x150.png").exists());

    // Check browserconfig.xml
    let bc = fs::read_to_string(dir.path().join("browserconfig.xml")).unwrap();
    assert!(bc.contains("<browserconfig>"));

    // Check favicon-snippet.html
    let snippet = fs::read_to_string(dir.path().join("favicon-snippet.html")).unwrap();
    assert!(snippet.contains(r#"rel="icon" type="image/x-icon""#));
}

// ---------------------------------------------------------------------------
// Stage 18: PWA + All-platforms export tests
// ---------------------------------------------------------------------------

#[test]
fn test_export_pwa_icons() {
    let dir = TempDir::new().unwrap();
    let paths = exporter::export_pwa_icons(
        test_svg(),
        dir.path(),
        "TestApp",
        "#000000",
        "#FFFFFF",
    ).unwrap();

    assert_eq!(paths.len(), 3);
    assert!(dir.path().join("pwa-192x192.png").exists());
    assert!(dir.path().join("pwa-512x512.png").exists());
    assert!(dir.path().join("manifest.json").exists());

    // Verify manifest.json is valid JSON with correct fields
    let manifest_str = fs::read_to_string(dir.path().join("manifest.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&manifest_str).unwrap();
    assert_eq!(manifest["name"].as_str().unwrap(), "TestApp");
    assert_eq!(manifest["short_name"].as_str().unwrap(), "TestApp");
    assert_eq!(manifest["theme_color"].as_str().unwrap(), "#000000");
    assert_eq!(manifest["background_color"].as_str().unwrap(), "#FFFFFF");
    assert_eq!(manifest["display"].as_str().unwrap(), "standalone");
    assert_eq!(manifest["start_url"].as_str().unwrap(), "/");

    let icons = manifest["icons"].as_array().unwrap();
    assert_eq!(icons.len(), 2);
    assert_eq!(icons[0]["src"].as_str().unwrap(), "pwa-192x192.png");
    assert_eq!(icons[0]["sizes"].as_str().unwrap(), "192x192");
    assert_eq!(icons[0]["type"].as_str().unwrap(), "image/png");
    assert_eq!(icons[1]["src"].as_str().unwrap(), "pwa-512x512.png");
    assert_eq!(icons[1]["sizes"].as_str().unwrap(), "512x512");
    assert_eq!(icons[1]["type"].as_str().unwrap(), "image/png");
}

#[test]
fn test_export_all_platforms() {
    let dir = TempDir::new().unwrap();
    let result = exporter::export_all_platforms(
        test_svg(),
        dir.path(),
        "TestApp",
        "#000000",
        "#FFFFFF",
    ).unwrap();

    // iOS: 14 icon files + Contents.json
    assert!(!result.ios_paths.is_empty());
    assert!(dir.path().join("ios/AppIcon.appiconset/Contents.json").exists());
    assert!(dir.path().join("ios/AppIcon.appiconset/Icon-1024.png").exists());

    // Android: 5 mipmap dirs
    assert!(!result.android_paths.is_empty());
    assert!(dir.path().join("android/mipmap-mdpi/ic_launcher.png").exists());
    assert!(dir.path().join("android/mipmap-xxxhdpi/ic_launcher.png").exists());

    // PWA: 2 PNGs + manifest.json
    assert!(!result.pwa_paths.is_empty());
    assert!(dir.path().join("pwa/pwa-192x192.png").exists());
    assert!(dir.path().join("pwa/pwa-512x512.png").exists());
    assert!(dir.path().join("pwa/manifest.json").exists());

    // Favicon: full package
    assert!(!result.favicon_paths.is_empty());
    assert!(dir.path().join("favicon/favicon.ico").exists());
    assert!(dir.path().join("favicon/favicon.svg").exists());
    assert!(dir.path().join("favicon/site.webmanifest").exists());
    assert!(dir.path().join("favicon/browserconfig.xml").exists());

    // Verify PWA manifest is valid JSON
    let manifest_str = fs::read_to_string(dir.path().join("pwa/manifest.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&manifest_str).unwrap();
    assert_eq!(manifest["name"].as_str().unwrap(), "TestApp");
    assert!(manifest["icons"].as_array().unwrap().len() >= 2);
}

// ---------------------------------------------------------------------------
// Stage 21: Sprite Sheet tests
// ---------------------------------------------------------------------------

fn colored_svg(color: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><circle cx="50" cy="50" r="40" fill="{}"/></svg>"#,
        color
    )
}

#[test]
fn test_sprite_sheet_horizontal_layout() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("sprite.png");
    let svg_red = colored_svg("red");
    let svg_green = colored_svg("green");
    let svg_blue = colored_svg("blue");
    let svgs: Vec<(&str, &str)> = vec![
        ("red", &svg_red),
        ("green", &svg_green),
        ("blue", &svg_blue),
    ];

    let result = exporter::export_sprite_sheet(&svgs, 0, 32, 4, &out).unwrap();

    // 3 icons horizontal: (3*32 + 2*4) x 32
    assert_eq!(result.total_width, 104);
    assert_eq!(result.total_height, 32);
    assert_eq!(result.icons.len(), 3);
    assert!(out.exists());

    // Verify icons.json was written
    let json_path = dir.path().join("sprite.icons.json");
    assert!(json_path.exists());
}

#[test]
fn test_sprite_sheet_grid_layout() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("grid.png");
    let s1 = colored_svg("red");
    let s2 = colored_svg("green");
    let s3 = colored_svg("blue");
    let s4 = colored_svg("yellow");
    let s5 = colored_svg("purple");
    let svgs: Vec<(&str, &str)> = vec![
        ("a", &s1), ("b", &s2), ("c", &s3), ("d", &s4), ("e", &s5),
    ];

    let result = exporter::export_sprite_sheet(&svgs, 2, 24, 2, &out).unwrap();

    // 5 icons, 2 cols → 3 rows; width = 2*24 + 1*2 = 50, height = 3*24 + 2*2 = 76
    assert_eq!(result.total_width, 50);
    assert_eq!(result.total_height, 76);
    assert_eq!(result.icons.len(), 5);
    assert!(out.exists());
}

#[test]
fn test_sprite_sheet_icons_json_coordinates() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("coords.png");
    let s1 = colored_svg("red");
    let s2 = colored_svg("green");
    let s3 = colored_svg("blue");
    let svgs: Vec<(&str, &str)> = vec![
        ("icon-a", &s1),
        ("icon-b", &s2),
        ("icon-c", &s3),
    ];

    let result = exporter::export_sprite_sheet(&svgs, 0, 32, 8, &out).unwrap();

    // Verify coordinate positions
    assert_eq!(result.icons[0].name, "icon-a");
    assert_eq!(result.icons[0].x, 0);
    assert_eq!(result.icons[0].y, 0);
    assert_eq!(result.icons[0].width, 32);
    assert_eq!(result.icons[0].height, 32);

    assert_eq!(result.icons[1].name, "icon-b");
    assert_eq!(result.icons[1].x, 40); // 32 + 8 padding
    assert_eq!(result.icons[1].y, 0);

    assert_eq!(result.icons[2].name, "icon-c");
    assert_eq!(result.icons[2].x, 80); // 2 * (32 + 8)
    assert_eq!(result.icons[2].y, 0);

    // Verify icons.json content
    let json_path = dir.path().join("coords.icons.json");
    let json_str = fs::read_to_string(&json_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0]["name"].as_str().unwrap(), "icon-a");
    assert_eq!(arr[1]["x"].as_u64().unwrap(), 40);
}

#[test]
fn test_sprite_sheet_zero_padding() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("tight.png");
    let s1 = colored_svg("red");
    let s2 = colored_svg("green");
    let svgs: Vec<(&str, &str)> = vec![("a", &s1), ("b", &s2)];

    let result = exporter::export_sprite_sheet(&svgs, 0, 48, 0, &out).unwrap();

    // 2 icons, no padding: 96 x 48
    assert_eq!(result.total_width, 96);
    assert_eq!(result.total_height, 48);
    assert_eq!(result.icons[0].x, 0);
    assert_eq!(result.icons[1].x, 48);
}

#[test]
fn test_sprite_sheet_empty_list_error() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("empty.png");
    let svgs: Vec<(&str, &str)> = vec![];

    let result = exporter::export_sprite_sheet(&svgs, 4, 32, 0, &out);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("empty"), "Expected empty error, got: {}", err);
}
