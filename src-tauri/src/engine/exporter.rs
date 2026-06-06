use crate::engine::renderer;
use crate::error::AppError;
use image::ImageFormat;
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};

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
/// - site.webmanifest
pub fn export_favicon_package(
    svg_str: &str,
    output_dir: &Path,
    app_name: &str,
    theme_color: Option<&str>,
    background_color: Option<&str>,
) -> Result<Vec<PathBuf>, AppError> {
    std::fs::create_dir_all(output_dir)?;

    let tree = renderer::parse_svg(svg_str)?;

    let mut paths = Vec::new();

    // favicon.ico (16, 32, 48, 256) - uses pre-parsed tree internally
    let ico_path = output_dir.join("favicon.ico");
    export_ico_from_tree(&tree, &[16, 32, 48, 256], &ico_path)?;
    paths.push(ico_path);

    // Specific PNG sizes
    let png_sizes: &[(&str, u32)] = &[
        ("favicon-16x16.png", 16),
        ("favicon-32x32.png", 32),
        ("apple-touch-icon.png", 180),
        ("android-chrome-192x192.png", 192),
        ("android-chrome-512x512.png", 512),
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
    std::fs::write(&manifest_path, manifest)?;
    paths.push(manifest_path);

    Ok(paths)
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
