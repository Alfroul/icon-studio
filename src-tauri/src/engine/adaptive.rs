use crate::engine::builder;
use crate::engine::renderer;
use crate::error::AppError;
use crate::model::{AdaptiveConfig, AdaptiveShape, IconProject};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeZoneViolation {
    pub element_id: String,
    pub overshoot_px: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeZoneResult {
    pub safe: bool,
    pub violations: Vec<SafeZoneViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveExportResult {
    pub files: Vec<String>,
}

// ---------------------------------------------------------------------------
// Core rendering
// ---------------------------------------------------------------------------

/// Build an SVG string for the project with an adaptive clipPath applied.
fn build_adaptive_svg(project: &IconProject, shape: &AdaptiveShape) -> Result<String, AppError> {
    let base_svg = builder::build(project)?;
    let canvas = project.active_canvas();
    let w = canvas.width as f64;
    let h = canvas.height as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;

    let clip_content = match shape {
        AdaptiveShape::Circle => {
            let r = cx.min(cy);
            format!(r#"<circle cx="{cx:.2}" cy="{cy:.2}" r="{r:.2}"/>"#, cx = cx, cy = cy, r = r)
        }
        AdaptiveShape::Squircle => {
            // Superellipse: (x/a)^n + (y/b)^n = 1, n≈5
            let a = cx;
            let b = cy;
            let n: f64 = 5.0;
            let points = generate_superellipse_points(cx, cy, a, b, n, 120);
            let d = points_to_svg_path(&points);
            format!(r#"<path d="{d}"/>"#, d = d)
        }
        AdaptiveShape::RoundedRect => {
            let rx = w * 0.2;
            let ry = h * 0.2;
            format!(
                r#"<rect x="0" y="0" width="{w:.2}" height="{h:.2}" rx="{rx:.2}" ry="{ry:.2}"/>"#,
                w = w, h = h, rx = rx, ry = ry,
            )
        }
        AdaptiveShape::Pill => {
            let rx = w / 2.0;
            format!(
                r#"<rect x="0" y="0" width="{w:.2}" height="{h:.2}" rx="{rx:.2}" ry="{rx:.2}"/>"#,
                w = w, h = h, rx = rx,
            )
        }
        AdaptiveShape::Square => {
            // No clipping
            return Ok(base_svg);
        }
    };

    // Insert clipPath into the SVG
    let clip_id = "adaptive-clip";
    let clip_def = format!(
        r#"<defs><clipPath id="{clip_id}">{content}</clipPath></defs>"#,
        clip_id = clip_id, content = clip_content,
    );

    // Wrap content in clipped group
    let svg = if let Some(pos) = base_svg.find('>') {
        let (open_tag, rest) = base_svg.split_at(pos + 1);
        // Remove closing </svg>
        let inner = rest.strip_suffix("</svg>").unwrap_or(rest);
        format!(
            r#"{open_tag}{clip_def}<g clip-path="url(#{clip_id})">{inner}</g></svg>"#,
            open_tag = open_tag, clip_def = clip_def, clip_id = clip_id, inner = inner,
        )
    } else {
        base_svg
    };

    Ok(svg)
}

/// Render the project as an adaptive icon in the given shape and size.
/// Returns PNG bytes.
pub fn render_adaptive(
    project: &IconProject,
    shape: AdaptiveShape,
    size: u32,
) -> Result<Vec<u8>, AppError> {
    let svg = build_adaptive_svg(project, &shape)?;
    renderer::render(&svg, size)
}

/// Render adaptive preview as base64 PNG string.
pub fn render_adaptive_base64(
    project: &IconProject,
    shape: AdaptiveShape,
    size: u32,
) -> Result<String, AppError> {
    let png = render_adaptive(project, shape, size)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&png))
}

// ---------------------------------------------------------------------------
// Safe zone check
// ---------------------------------------------------------------------------

/// Check which elements fall outside the safe zone.
/// `margin_percent` is the margin from the edge (e.g., 34 means 66% safe zone center).
pub fn check_safe_zone(project: &IconProject, margin_percent: f64) -> SafeZoneResult {
    let canvas = project.active_canvas();
    let w = canvas.width as f64;
    let h = canvas.height as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let safe_radius = (w.min(h) / 2.0) * (1.0 - margin_percent / 100.0);

    let elements = project.active_elements();
    let mut violations = Vec::new();

    for elem in elements {
        if !elem.common().visible {
            continue;
        }
        let c = elem.common();
        // Check all four corners of the bounding box
        let corners = [
            (c.x, c.y),
            (c.x + c.width, c.y),
            (c.x, c.y + c.height),
            (c.x + c.width, c.y + c.height),
        ];

        let max_overshoot = corners
            .iter()
            .map(|(px, py)| {
                let dist = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
                (dist - safe_radius).max(0.0)
            })
            .fold(0.0_f64, f64::max);

        if max_overshoot > 0.0 {
            violations.push(SafeZoneViolation {
                element_id: elem.id().to_string(),
                overshoot_px: max_overshoot,
            });
        }
    }

    let safe = violations.is_empty();
    SafeZoneResult { safe, violations }
}

// ---------------------------------------------------------------------------
// Android Adaptive Icon export
// ---------------------------------------------------------------------------

/// Build an SVG containing only the specified layer elements.
fn build_layer_svg(
    project: &IconProject,
    element_ids: &[String],
    _size: u32,
) -> Result<String, AppError> {
    let canvas = project.active_canvas();

    let id_set: std::collections::HashSet<&str> =
        element_ids.iter().map(|s| s.as_str()).collect();

    // Build a temporary project with only the filtered elements
    let mut layer_project = IconProject {
        canvas: canvas.clone(),
        symbols: project.symbols.clone(),
        ..Default::default()
    };

    for elem in project.active_elements() {
        if !elem.common().visible {
            continue;
        }
        if id_set.contains(elem.id()) {
            layer_project.elements.push(elem.clone());
        }
    }

    builder::build(&layer_project)
}

/// Export Android Adaptive Icon package with foreground + background layers.
pub fn export_adaptive_layers(
    project: &IconProject,
    output_dir: &str,
) -> Result<AdaptiveExportResult, AppError> {
    use crate::engine::utils::validate_file_path;
    validate_file_path(output_dir)?;

    let config = project.adaptive.clone().unwrap_or(AdaptiveConfig {
        foreground_ids: Vec::new(),
        background_ids: Vec::new(),
    });

    let all_elements = project.active_elements();
    let all_ids: Vec<String> = all_elements.iter().map(|e| e.id().to_string()).collect();

    // If no foreground_ids set, treat all elements as foreground
    let fg_ids = if config.foreground_ids.is_empty() {
        all_ids.clone()
    } else {
        config.foreground_ids.clone()
    };

    // Background ids: if not set, use empty (transparent background)
    let bg_ids = config.background_ids.clone();

    let dir = Path::new(output_dir);
    let mut files = Vec::new();

    // Android mipmap density sizes (based on 108dp base)
    let densities = [
        ("mipmap-mdpi", 108),
        ("mipmap-hdpi", 162),
        ("mipmap-xhdpi", 216),
        ("mipmap-xxhdpi", 324),
        ("mipmap-xxxhdpi", 432),
    ];

    // Render and save foreground layers
    for (density_dir, size) in &densities {
        let fg_svg = build_layer_svg(project, &fg_ids, *size)?;
        let fg_png = renderer::render(&fg_svg, *size)?;

        let mipmap_dir = dir.join(density_dir);
        std::fs::create_dir_all(&mipmap_dir)?;

        let fg_path = mipmap_dir.join("ic_launcher_foreground.png");
        std::fs::write(&fg_path, &fg_png)?;
        files.push(fg_path.to_string_lossy().into_owned());

        // Render background (solid color or elements)
        if !bg_ids.is_empty() {
            let bg_svg = build_layer_svg(project, &bg_ids, *size)?;
            let bg_png = renderer::render(&bg_svg, *size)?;
            let bg_path = mipmap_dir.join("ic_launcher_background.png");
            std::fs::write(&bg_path, &bg_png)?;
            files.push(bg_path.to_string_lossy().into_owned());
        } else {
            // Write a white background as placeholder
            let bg_svg = format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\"><rect width=\"{}\" height=\"{}\" fill=\"#FFFFFF\"/></svg>",
                size, size, size, size
            );
            let bg_png = renderer::render(&bg_svg, *size)?;
            let bg_path = mipmap_dir.join("ic_launcher_background.png");
            std::fs::write(&bg_path, &bg_png)?;
            files.push(bg_path.to_string_lossy().into_owned());
        }
    }

    // Generate ic_launcher.xml
    let xml_content = r#"<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@mipmap/ic_launcher_background"/>
    <foreground android:drawable="@mipmap/ic_launcher_foreground"/>
</adaptive-icon>"#;

    let mipmap_anydpi = dir.join("mipmap-anydpi-v26");
    std::fs::create_dir_all(&mipmap_anydpi)?;
    let xml_path = mipmap_anydpi.join("ic_launcher.xml");
    std::fs::write(&xml_path, xml_content)?;
    files.push(xml_path.to_string_lossy().into_owned());

    // Generate ic_launcher_round.xml (same content)
    let round_xml_path = mipmap_anydpi.join("ic_launcher_round.xml");
    std::fs::write(&round_xml_path, xml_content)?;
    files.push(round_xml_path.to_string_lossy().into_owned());

    Ok(AdaptiveExportResult { files })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Generate points on a superellipse curve.
fn generate_superellipse_points(
    cx: f64,
    cy: f64,
    a: f64,
    b: f64,
    n: f64,
    num_points: usize,
) -> Vec<(f64, f64)> {
    let mut points = Vec::with_capacity(num_points);
    let exp = 2.0 / n;
    for i in 0..num_points {
        let t = 2.0 * std::f64::consts::PI * i as f64 / num_points as f64;
        let cos_t = t.cos();
        let sin_t = t.sin();
        let x = cx + a * cos_t.signum() * cos_t.abs().powf(exp);
        let y = cy + b * sin_t.signum() * sin_t.abs().powf(exp);
        points.push((x, y));
    }
    points
}

/// Convert a list of points to an SVG path d attribute (closed polygon).
fn points_to_svg_path(points: &[(f64, f64)]) -> String {
    if points.is_empty() {
        return String::new();
    }
    let mut d = format!("M{:.2},{:.2}", points[0].0, points[0].1);
    for &(x, y) in &points[1..] {
        d.push_str(&format!(" L{:.2},{:.2}", x, y));
    }
    d.push('Z');
    d
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CommonProps, Element, ShapeElement, shapes::ShapeType};

    fn make_test_project() -> IconProject {
        let mut p = IconProject::default();
        p.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-1".to_string(), 50.0, 50.0, 100.0, 100.0),
            shape_type: ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        }));
        p.bump_version();
        p
    }

    #[test]
    fn test_render_adaptive_circle_produces_png() {
        let project = make_test_project();
        let png = render_adaptive(&project, AdaptiveShape::Circle, 128);
        assert!(png.is_ok());
        let bytes = png.unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_render_adaptive_squircle_produces_png() {
        let project = make_test_project();
        let png = render_adaptive(&project, AdaptiveShape::Squircle, 128);
        assert!(png.is_ok());
        let bytes = png.unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_render_adaptive_rounded_rect_produces_png() {
        let project = make_test_project();
        let png = render_adaptive(&project, AdaptiveShape::RoundedRect, 128);
        assert!(png.is_ok());
    }

    #[test]
    fn test_render_adaptive_pill_produces_png() {
        let project = make_test_project();
        let png = render_adaptive(&project, AdaptiveShape::Pill, 128);
        assert!(png.is_ok());
    }

    #[test]
    fn test_render_adaptive_square_no_clip() {
        let project = make_test_project();
        let png = render_adaptive(&project, AdaptiveShape::Square, 128);
        assert!(png.is_ok());
    }

    #[test]
    fn test_render_adaptive_base64() {
        let project = make_test_project();
        let b64 = render_adaptive_base64(&project, AdaptiveShape::Circle, 64);
        assert!(b64.is_ok());
        let s = b64.unwrap();
        assert!(base64::engine::general_purpose::STANDARD.decode(&s).is_ok());
    }

    #[test]
    fn test_build_adaptive_svg_contains_clippath() {
        let project = make_test_project();
        let svg = build_adaptive_svg(&project, &AdaptiveShape::Circle).unwrap();
        assert!(svg.contains(r#"id="adaptive-clip""#));
        assert!(svg.contains("clip-path"));
    }

    #[test]
    fn test_build_adaptive_svg_squircle_path_syntax() {
        let project = make_test_project();
        let svg = build_adaptive_svg(&project, &AdaptiveShape::Squircle).unwrap();
        // Should contain a <path d="M...Z"/> inside the clipPath
        assert!(svg.contains(r#"id="adaptive-clip""#));
        let clip_start = svg.find("<clipPath").unwrap();
        let clip_end = svg.find("</clipPath>").unwrap();
        let clip_content = &svg[clip_start..clip_end];
        assert!(clip_content.contains("<path d=\"M"));
        assert!(clip_content.ends_with("Z\"/>") || clip_content.contains("Z\"/>"));
    }

    #[test]
    fn test_build_adaptive_svg_square_no_clip() {
        let project = make_test_project();
        let svg = build_adaptive_svg(&project, &AdaptiveShape::Square).unwrap();
        assert!(!svg.contains("adaptive-clip"));
    }

    #[test]
    fn test_safe_zone_element_inside() {
        // Element fully inside the 66% safe zone center circle
        let mut project = IconProject::default();
        // On 512×512 canvas, safe zone radius = 256 * 0.66 = 168.96
        // Place element at center, fully within
        project.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-1".to_string(), 200.0, 200.0, 100.0, 100.0),
            shape_type: ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        }));

        let result = check_safe_zone(&project, 34.0);
        assert!(result.safe, "Element should be inside safe zone");
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_safe_zone_element_outside() {
        // Element at edge, outside safe zone
        let mut project = IconProject::default();
        // On 512×512 canvas, place element near corner
        project.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-1".to_string(), 400.0, 400.0, 100.0, 100.0),
            shape_type: ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        }));

        let result = check_safe_zone(&project, 34.0);
        assert!(!result.safe, "Element should be outside safe zone");
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].element_id, "shape-1");
        assert!(result.violations[0].overshoot_px > 0.0);
    }

    #[test]
    fn test_safe_zone_margin_34_is_66_percent() {
        // margin=34 → safe radius = 512/2 * (1 - 34/100) = 256 * 0.66 = 168.96
        let project = IconProject::default();
        let canvas = project.active_canvas();
        let safe_radius = (canvas.width as f64 / 2.0) * (1.0 - 34.0 / 100.0);
        // 512 / 2 * 0.66 = 168.96
        assert!((safe_radius - 168.96).abs() < 0.01);
    }

    #[test]
    fn test_safe_zone_ignores_invisible_elements() {
        let mut project = IconProject::default();
        let mut elem = ShapeElement {
            common: CommonProps::new("shape-1".to_string(), 450.0, 450.0, 100.0, 100.0),
            shape_type: ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        };
        elem.common.visible = false;
        project.elements.push(Element::Shape(elem));

        let result = check_safe_zone(&project, 34.0);
        assert!(result.safe, "Invisible element should be ignored");
    }

    #[test]
    fn test_export_adaptive_layers_no_config_treats_all_as_foreground() {
        let project = make_test_project();
        // No adaptive config set — all elements should be treated as foreground
        assert!(project.adaptive.is_none());

        let dir = std::env::temp_dir().join("iconstudio_test_adaptive_no_config");
        let _ = std::fs::remove_dir_all(&dir);
        let result = export_adaptive_layers(&project, dir.to_str().unwrap());
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.files.is_empty());
        // Should have foreground PNGs for each density
        assert!(output.files.iter().any(|f| f.contains("ic_launcher_foreground.png")));
        // Should have background PNGs
        assert!(output.files.iter().any(|f| f.contains("ic_launcher_background.png")));
        // Should have XML files
        assert!(output.files.iter().any(|f| f.contains("ic_launcher.xml")));
        assert!(output.files.iter().any(|f| f.contains("ic_launcher_round.xml")));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_export_adaptive_layers_with_config() {
        let mut project = make_test_project();
        project.adaptive = Some(AdaptiveConfig {
            foreground_ids: vec!["shape-1".to_string()],
            background_ids: vec![],
        });

        let dir = std::env::temp_dir().join("iconstudio_test_adaptive_with_config");
        let _ = std::fs::remove_dir_all(&dir);
        let result = export_adaptive_layers(&project, dir.to_str().unwrap());
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.files.iter().any(|f| f.contains("ic_launcher_foreground.png")));
        assert!(output.files.iter().any(|f| f.contains("ic_launcher_background.png")));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_superellipse_points() {
        let points = generate_superellipse_points(256.0, 256.0, 256.0, 256.0, 5.0, 4);
        assert_eq!(points.len(), 4);
        // All points should be within canvas bounds
        for &(x, y) in &points {
            assert!(x >= 0.0 && x <= 512.0);
            assert!(y >= 0.0 && y <= 512.0);
        }
    }

    #[test]
    fn test_points_to_svg_path() {
        let points = vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0)];
        let d = points_to_svg_path(&points);
        assert!(d.starts_with("M0.00,0.00"));
        assert!(d.contains("L100.00,0.00"));
        assert!(d.ends_with('Z'));
    }
}
