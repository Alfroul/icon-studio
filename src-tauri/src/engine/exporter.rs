use crate::engine::renderer;
use crate::error::AppError;
use image::ImageFormat;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Phase 3 types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FaviconPackageResult {
    pub paths: Vec<PathBuf>,
    pub html_snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllPlatformsResult {
    pub ios_paths: Vec<String>,
    pub android_paths: Vec<String>,
    pub pwa_paths: Vec<String>,
    pub favicon_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteSheetIcon {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteSheetResult {
    pub image_path: PathBuf,
    pub icons: Vec<SpriteSheetIcon>,
    pub total_width: u32,
    pub total_height: u32,
}

/// Convenience wrapper: render SVG to PNG bytes at given size.
pub fn render_to_png(svg_str: &str, size: u32) -> Result<Vec<u8>, AppError> {
    renderer::render(svg_str, size)
}

/// Convenience wrapper: render a pre-parsed tree to PNG bytes at given size.
/// Use this in batch exports to avoid re-parsing SVG per size.
pub fn render_to_png_from_tree(tree: &usvg::Tree, size: u32) -> Result<Vec<u8>, AppError> {
    renderer::render_from_tree(tree, size)
}

/// Render SVG to WebP bytes at given size.
pub fn render_to_webp(svg_str: &str, size: u32) -> Result<Vec<u8>, AppError> {
    let png_bytes = render_to_png(svg_str, size)?;
    let img = image::load_from_memory(&png_bytes)
        .map_err(|e| AppError::ExportError(format!("Failed to decode PNG for WebP conversion: {}", e)))?;
    let mut webp_buf = Cursor::new(Vec::new());
    img.write_to(&mut webp_buf, ImageFormat::WebP)
        .map_err(|e| AppError::ExportError(format!("Failed to encode WebP: {}", e)))?;
    Ok(webp_buf.into_inner())
}

pub fn export_webp(
    svg_str: &str,
    size: u32,
    output_path: &Path,
) -> Result<PathBuf, AppError> {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let webp_bytes = render_to_webp(svg_str, size)?;
    std::fs::write(output_path, webp_bytes)?;
    Ok(output_path.to_path_buf())
}

/// Export an ICO file containing the icon at multiple sizes.
/// Defaults to [16, 32, 48, 256] when sizes is empty.
pub fn export_ico(
    svg_str: &str,
    sizes: &[u32],
    output_path: &Path,
) -> Result<PathBuf, AppError> {
    let sizes = if sizes.is_empty() {
        &([16u32, 32, 48, 256] as [u32; 4])
    } else {
        sizes
    };

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let tree = renderer::parse_svg(svg_str)?;

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    for &size in sizes {
        let png_bytes = render_to_png_from_tree(&tree, size)?;
        let image = ico::IconImage::read_png(Cursor::new(&png_bytes))
            .map_err(|e| AppError::ExportError(format!("Failed to read PNG for size {}: {}", size, e)))?;
        let entry = ico::IconDirEntry::encode(&image)
            .map_err(|e| AppError::ExportError(format!("Failed to encode ICO entry for size {}: {}", size, e)))?;
        icon_dir.add_entry(entry);
    }

    let file = std::fs::File::create(output_path)?;
    icon_dir.write(file)
        .map_err(|e| AppError::ExportError(format!("Failed to write ICO file: {}", e)))?;

    Ok(output_path.to_path_buf())
}

/// Export Android mipmap icon set.
/// Sizes: mdpi=48, hdpi=72, xhdpi=96, xxhdpi=144, xxxhdpi=192
pub fn export_android_icons(
    svg_str: &str,
    output_dir: &Path,
) -> Result<Vec<PathBuf>, AppError> {
    let densities: &[(&str, u32)] = &[
        ("mipmap-mdpi", 48),
        ("mipmap-hdpi", 72),
        ("mipmap-xhdpi", 96),
        ("mipmap-xxhdpi", 144),
        ("mipmap-xxxhdpi", 192),
    ];

    let tree = renderer::parse_svg(svg_str)?;

    let paths: Vec<PathBuf> = densities
        .par_iter()
        .map(|&(folder, size)| {
            let dir = output_dir.join(folder);
            std::fs::create_dir_all(&dir)?;
            let png_bytes = render_to_png_from_tree(&tree, size)?;
            let file_path = dir.join("ic_launcher.png");
            std::fs::write(&file_path, &png_bytes)?;
            Ok(file_path)
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    Ok(paths)
}

struct IosIconEntry {
    size: &'static str,
    scale: &'static str,
    pixel: u32,
}

/// Apple iOS AppIcon spec (universal idiom).
const IOS_ICON_ENTRIES: &[IosIconEntry] = &[
    IosIconEntry { size: "20x20",     scale: "2x", pixel: 40   },
    IosIconEntry { size: "20x20",     scale: "3x", pixel: 60   },
    IosIconEntry { size: "29x29",     scale: "1x", pixel: 29   },
    IosIconEntry { size: "29x29",     scale: "2x", pixel: 58   },
    IosIconEntry { size: "29x29",     scale: "3x", pixel: 87   },
    IosIconEntry { size: "40x40",     scale: "1x", pixel: 40   },
    IosIconEntry { size: "40x40",     scale: "2x", pixel: 80   },
    IosIconEntry { size: "40x40",     scale: "3x", pixel: 120  },
    IosIconEntry { size: "60x60",     scale: "2x", pixel: 120  },
    IosIconEntry { size: "60x60",     scale: "3x", pixel: 180  },
    IosIconEntry { size: "76x76",     scale: "1x", pixel: 76   },
    IosIconEntry { size: "76x76",     scale: "2x", pixel: 152  },
    IosIconEntry { size: "83.5x83.5", scale: "2x", pixel: 167  },
    IosIconEntry { size: "1024x1024", scale: "1x", pixel: 1024 },
];

/// Export iOS AppIcon.appiconset with Contents.json manifest.
pub fn export_ios_icons(
    svg_str: &str,
    output_dir: &Path,
) -> Result<Vec<PathBuf>, AppError> {
    let appiconset_dir = output_dir.join("AppIcon.appiconset");
    std::fs::create_dir_all(&appiconset_dir)?;

    let mut unique_pixels: Vec<u32> = IOS_ICON_ENTRIES
        .iter()
        .map(|e| e.pixel)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    unique_pixels.sort();

    let tree = renderer::parse_svg(svg_str)?;

    let rendered: std::collections::HashMap<u32, Vec<u8>> = unique_pixels
        .par_iter()
        .map(|&pixel| {
            let png_bytes = render_to_png_from_tree(&tree, pixel)?;
            Ok((pixel, png_bytes))
        })
        .collect::<Result<std::collections::HashMap<_, _>, AppError>>()?;

    for (&pixel, png_bytes) in &rendered {
        let filename = format!("Icon-{}.png", pixel);
        let file_path = appiconset_dir.join(&filename);
        std::fs::write(&file_path, png_bytes)?;
    }

    let mut paths: Vec<PathBuf> = Vec::new();
    let mut images_json = Vec::with_capacity(IOS_ICON_ENTRIES.len());

    for entry in IOS_ICON_ENTRIES {
        let filename = format!("Icon-{}.png", entry.pixel);
        images_json.push(serde_json::json!({
            "size": entry.size,
            "idiom": "universal",
            "filename": filename,
            "scale": entry.scale
        }));
        paths.push(appiconset_dir.join(&filename));
    }

    let contents = serde_json::json!({
        "images": images_json,
        "info": {
            "version": 1,
            "author": "IconStudio"
        }
    });
    let contents_str = serde_json::to_string_pretty(&contents)
        .map_err(|e| AppError::ExportError(format!("Failed to serialize Contents.json: {}", e)))?;

    std::fs::write(appiconset_dir.join("Contents.json"), contents_str)?;

    Ok(paths)
}

// ---------------------------------------------------------------------------
// Favicon export package
// ---------------------------------------------------------------------------

/// Export a complete favicon package for web applications.
///
/// Generates:
/// - favicon.ico (16, 32, 48, 256)
/// - favicon-16x16.png
/// - favicon-32x32.png
/// - favicon.svg
/// - apple-touch-icon.png (180x180)
/// - android-chrome-192x192.png
/// - android-chrome-512x512.png
/// - mstile-70x70.png
/// - mstile-150x150.png
/// - mstile-310x310.png
/// - mstile-310x150.png
/// - site.webmanifest
/// - browserconfig.xml
/// - favicon-snippet.html
pub fn export_favicon_package(
    svg_str: &str,
    output_dir: &Path,
    app_name: &str,
    theme_color: Option<&str>,
    background_color: Option<&str>,
) -> Result<FaviconPackageResult, AppError> {
    std::fs::create_dir_all(output_dir)?;

    let tree = renderer::parse_svg(svg_str)?;

    let mut paths = Vec::new();

    // favicon.ico (16, 32, 48, 256) - uses pre-parsed tree internally
    let ico_path = output_dir.join("favicon.ico");
    export_ico_from_tree(&tree, &[16, 32, 48, 256], &ico_path)?;
    paths.push(ico_path);

    // Specific PNG sizes (including mstile)
    let png_sizes: &[(&str, u32)] = &[
        ("favicon-16x16.png", 16),
        ("favicon-32x32.png", 32),
        ("apple-touch-icon.png", 180),
        ("android-chrome-192x192.png", 192),
        ("android-chrome-512x512.png", 512),
        ("mstile-70x70.png", 70),
        ("mstile-150x150.png", 150),
        ("mstile-310x310.png", 310),
        ("mstile-310x150.png", 310),
    ];

    // Render unique sizes in parallel using pre-parsed tree
    let mut unique_sizes: Vec<u32> = png_sizes
        .iter()
        .map(|&(_, s)| s)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    unique_sizes.sort();

    let rendered: HashMap<u32, Vec<u8>> = unique_sizes
        .par_iter()
        .map(|&size| {
            let png_bytes = render_to_png_from_tree(&tree, size)?;
            Ok((size, png_bytes))
        })
        .collect::<Result<HashMap<_, _>, AppError>>()?;

    for &(filename, size) in png_sizes {
        let file_path = output_dir.join(filename);
        if let Some(png_bytes) = rendered.get(&size) {
            std::fs::write(&file_path, png_bytes)?;
        }
        paths.push(file_path);
    }

    // favicon.svg
    let svg_path = output_dir.join("favicon.svg");
    std::fs::write(&svg_path, svg_str)?;
    paths.push(svg_path);

    // site.webmanifest
    let theme = theme_color.unwrap_or("#FFFFFF");
    let bg = background_color.unwrap_or("#FFFFFF");
    let manifest = serde_json::json!({
        "name": app_name,
        "short_name": app_name,
        "icons": [
            {
                "src": "android-chrome-192x192.png",
                "sizes": "192x192",
                "type": "image/png"
            },
            {
                "src": "android-chrome-512x512.png",
                "sizes": "512x512",
                "type": "image/png"
            }
        ],
        "theme_color": theme,
        "background_color": bg,
        "display": "standalone"
    }).to_string();
    let manifest_path = output_dir.join("site.webmanifest");
    std::fs::write(&manifest_path, &manifest)?;
    paths.push(manifest_path);

    // browserconfig.xml
    let browserconfig = generate_browserconfig_xml(theme);
    let browserconfig_path = output_dir.join("browserconfig.xml");
    std::fs::write(&browserconfig_path, &browserconfig)?;
    paths.push(browserconfig_path);

    // favicon-snippet.html
    let html_snippet = generate_favicon_html_snippet(app_name);
    let snippet_path = output_dir.join("favicon-snippet.html");
    std::fs::write(&snippet_path, &html_snippet)?;
    paths.push(snippet_path);

    Ok(FaviconPackageResult {
        paths,
        html_snippet,
    })
}

/// Internal: export ICO from a pre-parsed tree, avoiding redundant SVG parsing.
fn export_ico_from_tree(
    tree: &usvg::Tree,
    sizes: &[u32],
    output_path: &Path,
) -> Result<PathBuf, AppError> {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    for &size in sizes {
        let png_bytes = render_to_png_from_tree(tree, size)?;
        let image = ico::IconImage::read_png(Cursor::new(&png_bytes))
            .map_err(|e| AppError::ExportError(format!("Failed to read PNG for size {}: {}", size, e)))?;
        let entry = ico::IconDirEntry::encode(&image)
            .map_err(|e| AppError::ExportError(format!("Failed to encode ICO entry for size {}: {}", size, e)))?;
        icon_dir.add_entry(entry);
    }

    let file = std::fs::File::create(output_path)?;
    icon_dir.write(file)
        .map_err(|e| AppError::ExportError(format!("Failed to write ICO file: {}", e)))?;

    Ok(output_path.to_path_buf())
}

// ---------------------------------------------------------------------------
// Phase 3 export functions (skeletons)
// ---------------------------------------------------------------------------

pub fn export_pwa_icons(
    svg: &str,
    output_dir: &Path,
    app_name: &str,
    theme_color: &str,
    bg_color: &str,
) -> Result<Vec<PathBuf>, AppError> {
    std::fs::create_dir_all(output_dir)?;

    let tree = renderer::parse_svg(svg)?;
    let mut paths = Vec::new();

    // Render 192x192 and 512x512 PNGs
    for &size in &[192u32, 512u32] {
        let png_bytes = render_to_png_from_tree(&tree, size)?;
        let filename = format!("pwa-{}x{}.png", size, size);
        let file_path = output_dir.join(&filename);
        std::fs::write(&file_path, &png_bytes)?;
        paths.push(file_path);
    }

    // Generate manifest.json
    let manifest = serde_json::json!({
        "name": app_name,
        "short_name": app_name,
        "icons": [
            {
                "src": "pwa-192x192.png",
                "sizes": "192x192",
                "type": "image/png"
            },
            {
                "src": "pwa-512x512.png",
                "sizes": "512x512",
                "type": "image/png"
            }
        ],
        "theme_color": theme_color,
        "background_color": bg_color,
        "display": "standalone",
        "start_url": "/"
    });
    let manifest_str = serde_json::to_string_pretty(&manifest)
        .map_err(|e| AppError::ExportError(format!("Failed to serialize manifest.json: {}", e)))?;
    let manifest_path = output_dir.join("manifest.json");
    std::fs::write(&manifest_path, &manifest_str)?;
    paths.push(manifest_path);

    Ok(paths)
}

pub fn export_all_platforms(
    svg: &str,
    output_dir: &Path,
    app_name: &str,
    theme_color: &str,
    bg_color: &str,
) -> Result<AllPlatformsResult, AppError> {
    let ios_dir = output_dir.join("ios");
    let android_dir = output_dir.join("android");
    let pwa_dir = output_dir.join("pwa");
    let favicon_dir = output_dir.join("favicon");

    let ios_paths = export_ios_icons(svg, &ios_dir)?;
    let android_paths = export_android_icons(svg, &android_dir)?;
    let pwa_paths = export_pwa_icons(svg, &pwa_dir, app_name, theme_color, bg_color)?;
    let favicon_result = export_favicon_package(svg, &favicon_dir, app_name, Some(theme_color), Some(bg_color))?;

    Ok(AllPlatformsResult {
        ios_paths: ios_paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect(),
        android_paths: android_paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect(),
        pwa_paths: pwa_paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect(),
        favicon_paths: favicon_result.paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect(),
    })
}

pub fn export_sprite_sheet(
    svgs: &[(&str, &str)],
    columns: u32,
    icon_size: u32,
    padding: u32,
    output_path: &Path,
) -> Result<SpriteSheetResult, AppError> {
    if svgs.is_empty() {
        return Err(AppError::ExportError("SVG list is empty".into()));
    }
    if icon_size == 0 {
        return Err(AppError::ExportError("icon_size must be > 0".into()));
    }

    let cols = if columns == 0 { svgs.len() as u32 } else { columns };
    let rows = (svgs.len() as u32).div_ceil(cols);

    let total_width = cols * icon_size + (cols.saturating_sub(1)) * padding;
    let total_height = rows * icon_size + (rows.saturating_sub(1)) * padding;

    let mut canvas = image::RgbaImage::new(total_width, total_height);

    let mut icons = Vec::with_capacity(svgs.len());
    for (i, (name, svg_str)) in svgs.iter().enumerate() {
        let col = i as u32 % cols;
        let row = i as u32 / cols;
        let x = col * (icon_size + padding);
        let y = row * (icon_size + padding);

        let png_bytes = render_to_png(svg_str, icon_size)?;
        let icon_img = image::load_from_memory(&png_bytes)
            .map_err(|e| AppError::ExportError(format!("Failed to decode PNG for '{}': {}", name, e)))?
            .to_rgba8();

        image::imageops::overlay(&mut canvas, &icon_img, x as i64, y as i64);

        icons.push(SpriteSheetIcon {
            name: name.to_string(),
            x,
            y,
            width: icon_size,
            height: icon_size,
        });
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    canvas.save(output_path)
        .map_err(|e| AppError::ExportError(format!("Failed to write sprite sheet: {}", e)))?;

    // Write icons.json alongside the image
    let json_path = output_path.with_extension("icons.json");
    let json_str = serde_json::to_string_pretty(&icons)
        .map_err(|e| AppError::ExportError(format!("Failed to serialize icons.json: {}", e)))?;
    std::fs::write(&json_path, json_str)?;

    Ok(SpriteSheetResult {
        image_path: output_path.to_path_buf(),
        icons,
        total_width,
        total_height,
    })
}

pub fn render_to_png_snapped(
    svg: &str,
    size: u32,
    snap: bool,
    grid_size: f64,
) -> Result<Vec<u8>, AppError> {
    let processed = if snap {
        snap_to_pixel_grid(svg, grid_size)?
    } else {
        svg.to_string()
    };
    render_to_png(&processed, size)
}

pub fn snap_to_pixel_grid(svg: &str, grid_size: f64) -> Result<String, AppError> {
    let grid = if grid_size <= 0.0 { 0.5 } else { grid_size };

    // Attributes where floating-point values should be snapped
    let snap_attrs = [
        "x", "y", "width", "height", "cx", "cy", "r", "rx", "ry",
    ];

    // Step 1: Snap numeric values inside known SVG attributes
    let attr_pattern = regex::Regex::new(
        r#"(x|y|width|height|cx|cy|r|rx|ry|transform)\s*=\s*"([^"]*)""#,
    ).map_err(|e| AppError::ExportError(format!("Regex error: {}", e)))?;

    let result = attr_pattern.replace_all(svg, |caps: &regex::Captures| {
        let attr = &caps[1];
        let value = &caps[2];

        if attr == "transform" {
            // Round all numbers in transform value
            let num_re = regex::Regex::new(r"-?\d+\.?\d*").unwrap();
            let new_val = num_re.replace_all(value, |num_caps: &regex::Captures| {
                snap_number(&num_caps[0], grid)
            });
            format!(r#"{}="{}""#, attr, new_val)
        } else if snap_attrs.contains(&attr) {
            let snapped = snap_number(value, grid);
            format!(r#"{}="{}""#, attr, snapped)
        } else {
            caps[0].to_string()
        }
    });

    // Step 2: Snap numbers inside path d attributes
    let d_pattern = regex::Regex::new(
        r#"\bd\s*=\s*"([^"]*)""#,
    ).map_err(|e| AppError::ExportError(format!("Regex error: {}", e)))?;

    let result = d_pattern.replace_all(&result, |caps: &regex::Captures| {
        let d_val = &caps[1];
        let num_re = regex::Regex::new(r"-?\d+\.?\d*").unwrap();
        let new_d = num_re.replace_all(d_val, |num_caps: &regex::Captures| {
            snap_number(&num_caps[0], grid)
        });
        format!(r#"d="{}""#, new_d)
    });

    Ok(result.into_owned())
}

fn snap_number(s: &str, grid: f64) -> String {
    let val: f64 = match s.parse() {
        Ok(v) => v,
        Err(_) => return s.to_string(),
    };
    let snapped = (val / grid).round() * grid;
    // Format without unnecessary trailing zeros
    if snapped == snapped.floor() {
        format!("{}", snapped as i64)
    } else {
        format!("{}", snapped)
    }
}

pub fn generate_favicon_html_snippet(app_name: &str) -> String {
    format!(
        r##"<link rel="icon" type="image/svg+xml" href="favicon.svg">
<link rel="icon" type="image/png" sizes="32x32" href="favicon-32x32.png">
<link rel="icon" type="image/x-icon" href="favicon.ico">
<link rel="apple-touch-icon" sizes="180x180" href="apple-touch-icon.png">
<link rel="manifest" href="site.webmanifest">
<meta name="application-name" content="{}">
<meta name="msapplication-TileColor" content="#FFFFFF">"##,
        app_name
    )
}

pub fn get_favicon_html_snippet(app_name: &str) -> String {
    generate_favicon_html_snippet(app_name)
}

pub fn generate_browserconfig_xml(theme_color: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<browserconfig>
  <msapplication>
    <tile>
      <square70x70logo src="mstile-70x70.png"/>
      <square150x150logo src="mstile-150x150.png"/>
      <square310x310logo src="mstile-310x310.png"/>
      <wide310x150logo src="mstile-310x150.png"/>
      <TileColor>{}</TileColor>
    </tile>
  </msapplication>
</browserconfig>"#,
        theme_color
    )
}
