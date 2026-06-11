use crate::model::{PresetShape, Shadow, ThemePreset};

/// Returns all 20 built-in theme presets.
pub fn builtin_presets() -> Vec<ThemePreset> {
    vec![
        ThemePreset {
            id: "ios".into(),
            name: "iOS (Apple)".into(),
            corner_radius: 22.37,
            padding_ratio: 0.10,
            background: Some("#FFFFFF".into()),
            shadow: Some(Shadow {
                color: "#0000001A".into(),
                blur: 12.0,
                offset_x: 0.0,
                offset_y: 4.0,
                inset: false,
            }),
            shape: PresetShape::Squircle,
            preview_svg: Some(preview_squircle("#FFFFFF", 22.37)),
        },
        ThemePreset {
            id: "android".into(),
            name: "Android".into(),
            corner_radius: 0.0,
            padding_ratio: 0.10,
            background: Some("#4285F4".into()),
            shadow: None,
            shape: PresetShape::Circle,
            preview_svg: Some(preview_circle("#4285F4")),
        },
        ThemePreset {
            id: "macos".into(),
            name: "macOS".into(),
            corner_radius: 22.37,
            padding_ratio: 0.10,
            background: Some("#F0F0F0".into()),
            shadow: Some(Shadow {
                color: "#00000026".into(),
                blur: 16.0,
                offset_x: 0.0,
                offset_y: 6.0,
                inset: false,
            }),
            shape: PresetShape::Squircle,
            preview_svg: Some(preview_squircle("#F0F0F0", 22.37)),
        },
        ThemePreset {
            id: "windows11".into(),
            name: "Windows 11".into(),
            corner_radius: 8.0,
            padding_ratio: 0.08,
            background: Some("#0078D4".into()),
            shadow: None,
            shape: PresetShape::RoundedRect,
            preview_svg: Some(preview_rounded_rect("#0078D4", 8.0)),
        },
        ThemePreset {
            id: "material".into(),
            name: "Material".into(),
            corner_radius: 0.0,
            padding_ratio: 0.10,
            background: Some("#E8DEF8".into()),
            shadow: Some(Shadow {
                color: "#0000001F".into(),
                blur: 6.0,
                offset_x: 0.0,
                offset_y: 2.0,
                inset: false,
            }),
            shape: PresetShape::Circle,
            preview_svg: Some(preview_circle("#E8DEF8")),
        },
        ThemePreset {
            id: "flat".into(),
            name: "Flat".into(),
            corner_radius: 0.0,
            padding_ratio: 0.08,
            background: Some("#FFFFFF".into()),
            shadow: None,
            shape: PresetShape::Square,
            preview_svg: Some(preview_square("#FFFFFF")),
        },
        ThemePreset {
            id: "glassmorphism".into(),
            name: "Glassmorphism".into(),
            corner_radius: 20.0,
            padding_ratio: 0.10,
            background: Some("#FFFFFF80".into()),
            shadow: Some(Shadow {
                color: "#00000015".into(),
                blur: 20.0,
                offset_x: 0.0,
                offset_y: 8.0,
                inset: false,
            }),
            shape: PresetShape::Squircle,
            preview_svg: Some(preview_squircle("#CCE5FF", 20.0)),
        },
        ThemePreset {
            id: "neon".into(),
            name: "Neon".into(),
            corner_radius: 12.0,
            padding_ratio: 0.10,
            background: Some("#1A1A2E".into()),
            shadow: Some(Shadow {
                color: "#00FF8866".into(),
                blur: 16.0,
                offset_x: 0.0,
                offset_y: 0.0,
                inset: false,
            }),
            shape: PresetShape::RoundedRect,
            preview_svg: Some(preview_neon()),
        },
        ThemePreset {
            id: "pixel-art".into(),
            name: "Pixel Art".into(),
            corner_radius: 0.0,
            padding_ratio: 0.06,
            background: None,
            shadow: None,
            shape: PresetShape::Square,
            preview_svg: Some(preview_square("#F5F5DC")),
        },
        ThemePreset {
            id: "3d-clay".into(),
            name: "3D Clay".into(),
            corner_radius: 25.0,
            padding_ratio: 0.12,
            background: Some("#FF6B6B".into()),
            shadow: Some(Shadow {
                color: "#00000033".into(),
                blur: 20.0,
                offset_x: 0.0,
                offset_y: 8.0,
                inset: false,
            }),
            shape: PresetShape::Squircle,
            preview_svg: Some(preview_clay()),
        },
        ThemePreset {
            id: "minimal".into(),
            name: "Minimal".into(),
            corner_radius: 4.0,
            padding_ratio: 0.06,
            background: Some("#FAFAFA".into()),
            shadow: None,
            shape: PresetShape::RoundedRect,
            preview_svg: Some(preview_rounded_rect("#FAFAFA", 4.0)),
        },
        ThemePreset {
            id: "duotone".into(),
            name: "Duotone".into(),
            corner_radius: 22.37,
            padding_ratio: 0.10,
            background: Some("#667EEA".into()),
            shadow: None,
            shape: PresetShape::Squircle,
            preview_svg: Some(preview_squircle("#667EEA", 22.37)),
        },
        ThemePreset {
            id: "gradient".into(),
            name: "Gradient".into(),
            corner_radius: 12.0,
            padding_ratio: 0.10,
            background: Some("#F093FB".into()),
            shadow: None,
            shape: PresetShape::RoundedRect,
            preview_svg: Some(preview_gradient()),
        },
        ThemePreset {
            id: "outline".into(),
            name: "Outline".into(),
            corner_radius: 0.0,
            padding_ratio: 0.08,
            background: None,
            shadow: None,
            shape: PresetShape::Square,
            preview_svg: Some(preview_outline()),
        },
        ThemePreset {
            id: "round".into(),
            name: "Round".into(),
            corner_radius: 0.0,
            padding_ratio: 0.10,
            background: Some("#10B981".into()),
            shadow: None,
            shape: PresetShape::Circle,
            preview_svg: Some(preview_circle("#10B981")),
        },
        ThemePreset {
            id: "hexagon".into(),
            name: "Hexagon".into(),
            corner_radius: 0.0,
            padding_ratio: 0.10,
            background: Some("#8B5CF6".into()),
            shadow: None,
            shape: PresetShape::Hexagon,
            preview_svg: Some(preview_hexagon("#8B5CF6")),
        },
        ThemePreset {
            id: "shield".into(),
            name: "Shield".into(),
            corner_radius: 0.0,
            padding_ratio: 0.10,
            background: Some("#EF4444".into()),
            shadow: None,
            shape: PresetShape::Square,
            preview_svg: Some(preview_shield("#EF4444")),
        },
        ThemePreset {
            id: "magazine".into(),
            name: "Magazine".into(),
            corner_radius: 16.0,
            padding_ratio: 0.10,
            background: Some("#1F2937".into()),
            shadow: Some(Shadow {
                color: "#00000040".into(),
                blur: 24.0,
                offset_x: 0.0,
                offset_y: 10.0,
                inset: false,
            }),
            shape: PresetShape::RoundedRect,
            preview_svg: Some(preview_rounded_rect("#1F2937", 16.0)),
        },
        ThemePreset {
            id: "retro".into(),
            name: "Retro".into(),
            corner_radius: 8.0,
            padding_ratio: 0.08,
            background: Some("#FBBF24".into()),
            shadow: None,
            shape: PresetShape::RoundedRect,
            preview_svg: Some(preview_rounded_rect("#FBBF24", 8.0)),
        },
        ThemePreset {
            id: "custom".into(),
            name: "Custom".into(),
            corner_radius: 0.0,
            padding_ratio: 0.10,
            background: None,
            shadow: None,
            shape: PresetShape::Square,
            preview_svg: Some(preview_custom()),
        },
    ]
}

fn preview_squircle(bg: &str, cr: f64) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><rect width="48" height="48" rx="{rx}" ry="{rx}" fill="{bg}"/><rect x="14" y="14" width="20" height="20" rx="3" fill="rgba(0,0,0,0.15)"/></svg>"#,
        rx = cr / 100.0 * 48.0,
        bg = bg
    )
}

fn preview_circle(bg: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><circle cx="24" cy="24" r="24" fill="{bg}"/><rect x="14" y="14" width="20" height="20" rx="3" fill="rgba(0,0,0,0.15)"/></svg>"#,
        bg = bg
    )
}

fn preview_rounded_rect(bg: &str, cr: f64) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><rect width="48" height="48" rx="{rx}" fill="{bg}"/><rect x="14" y="14" width="20" height="20" rx="3" fill="rgba(0,0,0,0.15)"/></svg>"#,
        rx = cr,
        bg = bg
    )
}

fn preview_square(bg: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><rect width="48" height="48" fill="{bg}"/><rect x="14" y="14" width="20" height="20" rx="2" fill="rgba(0,0,0,0.15)"/></svg>"#,
        bg = bg
    )
}

fn preview_neon() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><rect width="48" height="48" rx="12" fill="#1A1A2E"/><rect x="14" y="14" width="20" height="20" rx="3" fill="none" stroke="#00FF88" stroke-width="1.5"/></svg>"##.into()
}

fn preview_clay() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><rect width="48" height="48" rx="12" fill="#FF6B6B"/><ellipse cx="24" cy="42" rx="14" ry="3" fill="rgba(0,0,0,0.15)"/><rect x="16" y="14" width="16" height="16" rx="4" fill="rgba(255,255,255,0.3)"/></svg>"##.into()
}

fn preview_gradient() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0%" stop-color="#F093FB"/><stop offset="100%" stop-color="#F5576C"/></linearGradient></defs><rect width="48" height="48" rx="12" fill="url(#g)"/><rect x="14" y="14" width="20" height="20" rx="3" fill="rgba(255,255,255,0.3)"/></svg>"##.into()
}

fn preview_outline() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><rect x="1" y="1" width="46" height="46" rx="2" fill="none" stroke="#333" stroke-width="2"/><rect x="14" y="14" width="20" height="20" rx="2" fill="none" stroke="#333" stroke-width="1.5"/></svg>"##.into()
}

fn preview_hexagon(bg: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><polygon points="24,2 44,14 44,34 24,46 4,34 4,14" fill="{bg}"/><rect x="16" y="16" width="16" height="16" rx="2" fill="rgba(0,0,0,0.15)"/></svg>"#,
        bg = bg
    )
}

fn preview_shield(bg: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><path d="M24 4 L42 12 L42 28 Q42 40 24 46 Q6 40 6 28 L6 12 Z" fill="{bg}"/><rect x="16" y="16" width="16" height="16" rx="2" fill="rgba(0,0,0,0.15)"/></svg>"#,
        bg = bg
    )
}

fn preview_custom() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><rect width="48" height="48" rx="4" fill="none" stroke="#999" stroke-width="2" stroke-dasharray="4 2"/><text x="24" y="28" text-anchor="middle" font-size="10" fill="#999">?</text></svg>"##.into()
}

/// Compute the position and size for centering an element on the canvas
/// with aspect ratio preserved and 10% padding.
/// Returns (x, y, width, height).
pub fn fit_to_canvas(
    canvas_w: f64,
    canvas_h: f64,
    elem_w: f64,
    elem_h: f64,
) -> (f64, f64, f64, f64) {
    let padding = 0.10;
    let avail_w = canvas_w * (1.0 - 2.0 * padding);
    let avail_h = canvas_h * (1.0 - 2.0 * padding);

    let scale_x = avail_w / elem_w.max(1.0);
    let scale_y = avail_h / elem_h.max(1.0);
    let scale = scale_x.min(scale_y).min(1.0);

    let final_w = elem_w * scale;
    let final_h = elem_h * scale;
    let x = (canvas_w - final_w) / 2.0;
    let y = (canvas_h - final_h) / 2.0;

    (x, y, final_w, final_h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_presets_count() {
        let presets = builtin_presets();
        assert_eq!(presets.len(), 20);
    }

    #[test]
    fn test_presets_have_unique_ids() {
        let presets = builtin_presets();
        let mut ids: Vec<&str> = presets.iter().map(|p| p.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 20, "All preset IDs must be unique");
    }

    #[test]
    fn test_presets_non_empty_fields() {
        let presets = builtin_presets();
        for p in &presets {
            assert!(!p.id.is_empty(), "Preset {:?} has empty id", p.name);
            assert!(!p.name.is_empty(), "Preset {:?} has empty name", p.name);
            assert!(
                p.preview_svg.is_some(),
                "Preset {:?} missing preview_svg",
                p.name
            );
            if let Some(ref svg) = p.preview_svg {
                assert!(!svg.is_empty(), "Preset {:?} has empty preview_svg", p.name);
                assert!(
                    svg.contains("<svg"),
                    "Preset {:?} preview_svg not valid SVG",
                    p.name
                );
            }
        }
    }

    #[test]
    fn test_fit_to_canvas_square() {
        let (x, y, w, h) = fit_to_canvas(512.0, 512.0, 256.0, 256.0);
        assert!((x - 128.0).abs() < 0.01);
        assert!((y - 128.0).abs() < 0.01);
        assert!((w - 256.0).abs() < 0.01);
        assert!((h - 256.0).abs() < 0.01);
    }

    #[test]
    fn test_fit_to_canvas_oversize_element() {
        let (x, y, w, h) = fit_to_canvas(512.0, 512.0, 1024.0, 1024.0);
        let expected = 512.0 * 0.8;
        assert!((w - expected).abs() < 0.01);
        assert!((h - expected).abs() < 0.01);
        assert!((x - 512.0 * 0.1).abs() < 0.01);
        assert!((y - 512.0 * 0.1).abs() < 0.01);
    }

    #[test]
    fn test_fit_to_canvas_wide_element() {
        let (x, y, w, h) = fit_to_canvas(512.0, 512.0, 800.0, 400.0);
        let expected_w = 512.0 * 0.8;
        let expected_h = expected_w * 400.0 / 800.0;
        assert!((w - expected_w).abs() < 0.01);
        assert!((h - expected_h).abs() < 0.01);
        assert!((x - (512.0 - expected_w) / 2.0).abs() < 0.01);
        assert!((y - (512.0 - expected_h) / 2.0).abs() < 0.01);
    }

    #[test]
    fn test_fit_to_canvas_preserves_aspect_ratio() {
        let (_x1, _y1, w1, h1) = fit_to_canvas(512.0, 512.0, 200.0, 400.0);
        let ratio_orig = 200.0 / 400.0;
        let ratio_fit = w1 / h1;
        assert!((ratio_orig - ratio_fit).abs() < 0.001);
    }

    #[test]
    fn test_fit_to_canvas_no_upscale() {
        let (_x, _y, w, h) = fit_to_canvas(1024.0, 1024.0, 100.0, 100.0);
        assert!((w - 100.0).abs() < 0.01, "Should not upscale");
        assert!((h - 100.0).abs() < 0.01);
    }
}
