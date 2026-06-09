use crate::model::{BrandColorRole, BrandKit, BrandVariant, Element, IconProject};
use palette::{FromColor, Hsl, Srgb};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn generate_brand_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("brand-{}", ts)
}

fn parse_hex_to_srgb(hex: &str) -> Option<Srgb> {
    let c = crate::colors::ParsedColor::from_hex(hex)?;
    Some(Srgb::new(
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
    ))
}

fn srgb_to_hex(color: Srgb) -> String {
    let r = (color.red.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (color.green.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (color.blue.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

fn normalize_hex(hex: &str) -> String {
    match parse_hex_to_srgb(hex) {
        Some(rgb) => srgb_to_hex(rgb),
        None => hex.to_string(),
    }
}

/// Euclidean distance in HSL space (hue normalized to [0,1]).
fn hsl_distance(a: Hsl, b: Hsl) -> f32 {
    let ah = a.hue.into_degrees();
    let bh = b.hue.into_degrees();
    let dh = ((ah - bh).abs()).min(360.0 - (ah - bh).abs()) / 180.0;
    let ds = a.saturation - b.saturation;
    let dl = a.lightness - b.lightness;
    (dh * dh + ds * ds + dl * dl).sqrt()
}

/// Find the closest brand color to a given hex color.
fn closest_brand_color(
    hex: &str,
    brand_colors: &HashMap<BrandColorRole, String>,
) -> Option<String> {
    let target = parse_hex_to_srgb(hex)?;
    let target_hsl: Hsl = Hsl::from_color(target);

    let mut best_dist = f32::MAX;
    let mut best_color: Option<String> = None;

    for brand_hex in brand_colors.values() {
        if let Some(brand_rgb) = parse_hex_to_srgb(brand_hex) {
            let brand_hsl: Hsl = Hsl::from_color(brand_rgb);
            let dist = hsl_distance(target_hsl, brand_hsl);
            if dist < best_dist {
                best_dist = dist;
                best_color = Some(brand_hex.clone());
            }
        }
    }

    best_color
}

fn linearize(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance(rgb: Srgb) -> f64 {
    let r = linearize(rgb.red as f64);
    let g = linearize(rgb.green as f64);
    let b = linearize(rgb.blue as f64);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast_ratio(a: Srgb, b: Srgb) -> f64 {
    let l1 = relative_luminance(a);
    let l2 = relative_luminance(b);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

fn adjust_lightness(hsl: Hsl, delta: f32) -> Hsl {
    Hsl::new(
        hsl.hue,
        hsl.saturation,
        (hsl.lightness + delta).clamp(0.0, 1.0),
    )
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create a brand kit with auto-derived colors.
pub fn create_brand_kit(
    name: &str,
    primary: &str,
    secondary: Option<&str>,
    accent: Option<&str>,
    neutral: Option<&str>,
) -> Result<BrandKit, String> {
    let primary_rgb = parse_hex_to_srgb(primary)
        .ok_or_else(|| format!("Invalid primary color: {}", primary))?;
    let primary_hsl: Hsl = Hsl::from_color(primary_rgb);
    let primary_hex = normalize_hex(primary);

    let mut colors = HashMap::new();
    colors.insert(BrandColorRole::Primary, primary_hex);

    // Secondary: complementary if not provided
    let secondary_hex = match secondary {
        Some(s) => {
            parse_hex_to_srgb(s).ok_or_else(|| format!("Invalid secondary color: {}", s))?;
            normalize_hex(s)
        }
        None => {
            let hue = (primary_hsl.hue.into_degrees() + 180.0) % 360.0;
            let comp = Srgb::from_color(Hsl::new(hue, primary_hsl.saturation, primary_hsl.lightness));
            srgb_to_hex(comp)
        }
    };
    colors.insert(BrandColorRole::Secondary, secondary_hex);

    // Accent: triadic if not provided
    let accent_hex = match accent {
        Some(a) => {
            parse_hex_to_srgb(a).ok_or_else(|| format!("Invalid accent color: {}", a))?;
            normalize_hex(a)
        }
        None => {
            let hue = (primary_hsl.hue.into_degrees() + 120.0) % 360.0;
            let triad = Srgb::from_color(Hsl::new(
                hue,
                primary_hsl.saturation.min(0.8),
                (primary_hsl.lightness + 0.1).min(0.7),
            ));
            srgb_to_hex(triad)
        }
    };
    colors.insert(BrandColorRole::Accent, accent_hex);

    // Neutral: desaturated version of primary
    let neutral_hex = match neutral {
        Some(n) => {
            parse_hex_to_srgb(n).ok_or_else(|| format!("Invalid neutral color: {}", n))?;
            normalize_hex(n)
        }
        None => {
            let n = Srgb::from_color(Hsl::new(
                primary_hsl.hue.into_degrees(),
                primary_hsl.saturation * 0.1,
                0.5,
            ));
            srgb_to_hex(n)
        }
    };
    colors.insert(BrandColorRole::Neutral, neutral_hex);

    // Surface: light tinted
    let surface = Srgb::from_color(Hsl::new(
        primary_hsl.hue.into_degrees(),
        primary_hsl.saturation * 0.05,
        0.97,
    ));
    colors.insert(BrandColorRole::Surface, srgb_to_hex(surface));

    // Error: red
    colors.insert(BrandColorRole::Error, "#DC2626".to_string());

    Ok(BrandKit {
        id: generate_brand_id(),
        name: name.to_string(),
        colors,
        variants: HashMap::new(),
    })
}

/// Apply brand colors to all elements in the project.
pub fn apply_brand(
    project: &mut IconProject,
    kit: &BrandKit,
    _mode: &str,
) -> Result<(), String> {
    let elements = project.active_elements_mut();
    for elem in elements.iter_mut() {
        apply_brand_to_element(elem, &kit.colors);
    }

    // Also replace canvas background if not transparent
    let canvas = project.active_canvas_mut();
    if canvas.background != "transparent" {
        if let Some(replacement) = closest_brand_color(&canvas.background, &kit.colors) {
            canvas.background = replacement;
        }
    }

    Ok(())
}

fn apply_brand_to_element(elem: &mut Element, brand_colors: &HashMap<BrandColorRole, String>) {
    match elem {
        Element::Shape(e) => {
            if let Some(c) = closest_brand_color(&e.fill, brand_colors) {
                e.fill = c;
            }
            if let Some(ref stroke) = e.stroke {
                if let Some(c) = closest_brand_color(stroke, brand_colors) {
                    e.stroke = Some(c);
                }
            }
        }
        Element::Text(e) => {
            if let Some(c) = closest_brand_color(&e.fill, brand_colors) {
                e.fill = c;
            }
            if let Some(ref stroke) = e.stroke {
                if let Some(c) = closest_brand_color(stroke, brand_colors) {
                    e.stroke = Some(c);
                }
            }
        }
        Element::Icon(e) => {
            if let Some(c) = closest_brand_color(&e.fill, brand_colors) {
                e.fill = c;
            }
            if let Some(ref stroke) = e.stroke {
                if let Some(c) = closest_brand_color(stroke, brand_colors) {
                    e.stroke = Some(c);
                }
            }
        }
        Element::Path(e) => {
            if let Some(c) = closest_brand_color(&e.fill, brand_colors) {
                e.fill = c;
            }
            if let Some(c) = closest_brand_color(&e.stroke, brand_colors) {
                e.stroke = c;
            }
        }
        Element::Group(g) => {
            for child in g.children.iter_mut() {
                apply_brand_to_element(child, brand_colors);
            }
        }
        Element::Image(_) | Element::Symbol(_) => {}
    }
}

/// Generate a brand variant (dark/light/high-contrast).
pub fn generate_variant(kit: &BrandKit, variant_type: &str) -> Result<BrandKit, String> {
    let primary_rgb = kit
        .colors
        .get(&BrandColorRole::Primary)
        .and_then(|h| parse_hex_to_srgb(h))
        .ok_or("Brand kit missing valid primary color")?;
    let primary_hsl: Hsl = Hsl::from_color(primary_rgb);

    let variant_colors = match variant_type {
        "dark" => generate_dark_variant(&primary_hsl, &kit.colors),
        "light" => generate_light_variant(&primary_hsl, &kit.colors),
        "high-contrast" => generate_high_contrast_variant(&primary_hsl),
        _ => {
            return Err(format!(
                "Unknown variant type: {}. Use dark, light, or high-contrast.",
                variant_type
            ))
        }
    }?;

    let variant = BrandVariant {
        variant_type: variant_type.to_string(),
        colors: variant_colors.clone(),
    };

    let mut new_kit = BrandKit {
        id: generate_brand_id(),
        name: format!("{} ({})", kit.name, variant_type),
        colors: variant_colors,
        variants: HashMap::new(),
    };

    // Store reference to original variant
    new_kit
        .variants
        .insert(variant_type.to_string(), variant);

    Ok(new_kit)
}

fn generate_dark_variant(
    primary_hsl: &Hsl,
    original: &HashMap<BrandColorRole, String>,
) -> Result<HashMap<BrandColorRole, String>, String> {
    let mut colors = HashMap::new();

    let primary = Srgb::from_color(adjust_lightness(*primary_hsl, 0.15));
    colors.insert(BrandColorRole::Primary, srgb_to_hex(primary));

    let sec_hex = original
        .get(&BrandColorRole::Secondary)
        .map(|s| s.as_str())
        .unwrap_or("#FFFFFF");
    let sec_rgb = parse_hex_to_srgb(sec_hex).unwrap_or(Srgb::new(0.5, 0.5, 0.5));
    let sec_hsl = Hsl::from_color(sec_rgb);
    let secondary = Srgb::from_color(adjust_lightness(sec_hsl, 0.15));
    colors.insert(BrandColorRole::Secondary, srgb_to_hex(secondary));

    let acc_hex = original
        .get(&BrandColorRole::Accent)
        .map(|s| s.as_str())
        .unwrap_or("#FFFFFF");
    let acc_rgb = parse_hex_to_srgb(acc_hex).unwrap_or(Srgb::new(0.5, 0.5, 0.5));
    let acc_hsl = Hsl::from_color(acc_rgb);
    let accent = Srgb::from_color(Hsl::new(
        acc_hsl.hue,
        acc_hsl.saturation.max(0.7),
        (acc_hsl.lightness + 0.1).min(0.75),
    ));
    colors.insert(BrandColorRole::Accent, srgb_to_hex(accent));

    let neutral = Srgb::from_color(Hsl::new(primary_hsl.hue.into_degrees(), 0.05, 0.15));
    colors.insert(BrandColorRole::Neutral, srgb_to_hex(neutral));

    let surface = Srgb::from_color(Hsl::new(primary_hsl.hue.into_degrees(), 0.08, 0.08));
    colors.insert(BrandColorRole::Surface, srgb_to_hex(surface));

    colors.insert(BrandColorRole::Error, "#EF4444".to_string());

    Ok(colors)
}

fn generate_light_variant(
    primary_hsl: &Hsl,
    original: &HashMap<BrandColorRole, String>,
) -> Result<HashMap<BrandColorRole, String>, String> {
    let mut colors = HashMap::new();

    let primary_hex = original
        .get(&BrandColorRole::Primary)
        .cloned()
        .unwrap_or_else(|| "#000000".to_string());
    colors.insert(BrandColorRole::Primary, primary_hex);

    let sec_hex = original
        .get(&BrandColorRole::Secondary)
        .cloned()
        .unwrap_or_else(|| "#666666".to_string());
    colors.insert(BrandColorRole::Secondary, sec_hex);

    let acc_hex = original
        .get(&BrandColorRole::Accent)
        .cloned()
        .unwrap_or_else(|| "#888888".to_string());
    colors.insert(BrandColorRole::Accent, acc_hex);

    let neutral = Srgb::from_color(Hsl::new(primary_hsl.hue.into_degrees(), 0.03, 0.92));
    colors.insert(BrandColorRole::Neutral, srgb_to_hex(neutral));

    let surface = Srgb::from_color(Hsl::new(primary_hsl.hue.into_degrees(), 0.02, 0.98));
    colors.insert(BrandColorRole::Surface, srgb_to_hex(surface));

    colors.insert(BrandColorRole::Error, "#DC2626".to_string());

    Ok(colors)
}

fn generate_high_contrast_variant(
    primary_hsl: &Hsl,
) -> Result<HashMap<BrandColorRole, String>, String> {
    let mut colors = HashMap::new();

    let is_light = primary_hsl.lightness > 0.5;
    let (fg, bg) = if is_light {
        ("#000000".to_string(), "#FFFFFF".to_string())
    } else {
        ("#FFFFFF".to_string(), "#000000".to_string())
    };

    // Verify WCAG AAA (7:1) — hardcoded black/white always passes
    let fg_rgb = parse_hex_to_srgb(&fg).unwrap_or(Srgb::new(0.0, 0.0, 0.0));
    let bg_rgb = parse_hex_to_srgb(&bg).unwrap_or(Srgb::new(1.0, 1.0, 1.0));
    let ratio = contrast_ratio(fg_rgb, bg_rgb);
    if ratio < 7.0 {
        return Err(format!("High-contrast variant failed WCAG AAA ({:.1}:1)", ratio));
    }

    colors.insert(BrandColorRole::Primary, fg.clone());
    colors.insert(BrandColorRole::Secondary, fg.clone());
    colors.insert(BrandColorRole::Accent, fg);
    colors.insert(BrandColorRole::Neutral, bg.clone());
    colors.insert(BrandColorRole::Surface, bg);
    colors.insert(
        BrandColorRole::Error,
        if is_light {
            "#CC0000".to_string()
        } else {
            "#FF4444".to_string()
        },
    );

    Ok(colors)
}

/// Export brand guide as Markdown.
pub fn export_brand_guide(kit: &BrandKit) -> String {
    let mut md = String::new();

    md.push_str(&format!("# {} — Brand Guide\n\n", kit.name));

    md.push_str("## Color Palette\n\n");
    md.push_str("| Role | Hex | Preview |\n");
    md.push_str("|------|-----|--------|\n");

    for role in &[
        BrandColorRole::Primary,
        BrandColorRole::Secondary,
        BrandColorRole::Accent,
        BrandColorRole::Neutral,
        BrandColorRole::Surface,
        BrandColorRole::Error,
    ] {
        if let Some(hex) = kit.colors.get(role) {
            let role_name = serde_json::to_string(role)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            md.push_str(&format!("| {} | `{}` | {} |\n", role_name, hex, hex));
        }
    }

    md.push_str("\n## Usage Guidelines\n\n");
    md.push_str("- **Primary**: Main brand color for headers, buttons, key UI elements\n");
    md.push_str("- **Secondary**: Supporting color for secondary elements and accents\n");
    md.push_str("- **Accent**: Highlight color for CTAs and attention-drawing elements\n");
    md.push_str("- **Neutral**: Text and borders for body text and dividers\n");
    md.push_str("- **Surface**: Background for cards, panels, and page backgrounds\n");
    md.push_str("- **Error**: Validation messages and destructive actions\n");

    if !kit.variants.is_empty() {
        md.push_str("\n## Variants\n\n");
        for (name, variant) in &kit.variants {
            md.push_str(&format!("### {}\n\n", name));
            md.push_str("| Role | Hex |\n");
            md.push_str("|------|-----|\n");
            for role in &[
                BrandColorRole::Primary,
                BrandColorRole::Secondary,
                BrandColorRole::Accent,
                BrandColorRole::Neutral,
                BrandColorRole::Surface,
                BrandColorRole::Error,
            ] {
                if let Some(hex) = variant.colors.get(role) {
                    let role_name = serde_json::to_string(role)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_string();
                    md.push_str(&format!("| {} | `{}` |\n", role_name, hex));
                }
            }
            md.push('\n');
        }
    }

    md.push_str("## CSS Variables\n\n");
    md.push_str("```css\n");
    md.push_str(":root {\n");
    for role in &[
        BrandColorRole::Primary,
        BrandColorRole::Secondary,
        BrandColorRole::Accent,
        BrandColorRole::Neutral,
        BrandColorRole::Surface,
        BrandColorRole::Error,
    ] {
        if let Some(hex) = kit.colors.get(role) {
            let role_name = serde_json::to_string(role)
                .unwrap_or_default()
                .trim_matches('"')
                .replace('-', "_");
            md.push_str(&format!("  --color-{}: {};\n", role_name, hex));
        }
    }
    md.push_str("}\n");
    md.push_str("```\n");

    md
}

/// Keyword-to-color mapping for brand suggestions.
const BRAND_SUGGESTIONS: &[(&str, &str, &str)] = &[
    ("tech", "科技", "#0EA5E9"),
    ("nature", "自然", "#22C55E"),
    ("eco", "环保", "#16A34A"),
    ("young", "年轻", "#FF6B35"),
    ("energy", "活力", "#F59E0B"),
    ("luxury", "高端", "#D4A574"),
    ("minimal", "极简", "#6B7280"),
    ("creative", "创意", "#A855F7"),
    ("health", "健康", "#10B981"),
    ("finance", "金融", "#1D4ED8"),
    ("education", "教育", "#7C3AED"),
    ("food", "美食", "#EA580C"),
    ("travel", "旅行", "#0891B2"),
    ("social", "社交", "#EC4899"),
    ("security", "安全", "#059669"),
    ("music", "音乐", "#7C3AED"),
    ("sport", "运动", "#EF4444"),
    ("fashion", "时尚", "#BE185D"),
    ("kids", "儿童", "#F472B6"),
    ("corporate", "企业", "#1E40AF"),
];

/// Suggest a brand kit from a text description.
pub fn suggest_brand_from_description(description: &str) -> Result<BrandKit, String> {
    let desc_lower = description.to_lowercase();

    let mut best_color: Option<&str> = None;
    for &(eng, cn, color) in BRAND_SUGGESTIONS {
        if desc_lower.contains(eng) || desc_lower.contains(cn) {
            best_color = Some(color);
            break;
        }
    }

    let primary = best_color.unwrap_or("#4488FF");

    create_brand_kit(
        &format!("Brand ({})", description),
        primary,
        None,
        None,
        None,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{shapes::ShapeType, CommonProps, ShapeElement};

    #[test]
    fn test_create_brand_kit_primary_only() {
        let kit = create_brand_kit("Test", "#FF6B35", None, None, None).unwrap();
        assert_eq!(kit.name, "Test");
        assert_eq!(kit.colors[&BrandColorRole::Primary], "#FF6B35");
        assert_eq!(kit.colors.len(), 6);
    }

    #[test]
    fn test_create_brand_kit_auto_secondary_complementary() {
        let kit = create_brand_kit("Test", "#FF6B35", None, None, None).unwrap();
        let primary = parse_hex_to_srgb(&kit.colors[&BrandColorRole::Primary]).unwrap();
        let secondary = parse_hex_to_srgb(&kit.colors[&BrandColorRole::Secondary]).unwrap();
        let ph: Hsl = Hsl::from_color(primary);
        let sh: Hsl = Hsl::from_color(secondary);
        let hue_diff = (ph.hue.into_degrees() - sh.hue.into_degrees()).abs();
        assert!(
            (hue_diff - 180.0).abs() < 5.0,
            "Complementary should be ~180° apart, got {}",
            hue_diff
        );
    }

    #[test]
    fn test_apply_brand_closest() {
        let kit =
            create_brand_kit("Test", "#FF6B35", Some("#2DD4BF"), None, None).unwrap();
        let mut project = IconProject::default();

        project.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-1".into(), 0.0, 0.0, 100.0, 100.0),
            shape_type: ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        }));
        project.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-2".into(), 0.0, 0.0, 100.0, 100.0),
            shape_type: ShapeType::Circle,
            fill: "#00FF00".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        }));
        project.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-3".into(), 0.0, 0.0, 100.0, 100.0),
            shape_type: ShapeType::Circle,
            fill: "#0000FF".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        }));

        apply_brand(&mut project, &kit, "closest").unwrap();

        let elements = project.active_elements();
        for elem in elements {
            if let Element::Shape(s) = elem {
                assert!(
                    kit.colors.values().any(|c| c == &s.fill),
                    "Fill {} should be a brand color",
                    s.fill
                );
            }
        }
    }

    #[test]
    fn test_generate_variant_dark() {
        let kit = create_brand_kit("Test", "#FF6B35", None, None, None).unwrap();
        let dark = generate_variant(&kit, "dark").unwrap();

        let orig_hsl: Hsl = Hsl::from_color(
            parse_hex_to_srgb(&kit.colors[&BrandColorRole::Primary]).unwrap(),
        );
        let dark_hsl: Hsl = Hsl::from_color(
            parse_hex_to_srgb(&dark.colors[&BrandColorRole::Primary]).unwrap(),
        );
        assert!(
            dark_hsl.lightness > orig_hsl.lightness,
            "Dark variant primary should be brighter"
        );

        let surface_hsl: Hsl = Hsl::from_color(
            parse_hex_to_srgb(&dark.colors[&BrandColorRole::Surface]).unwrap(),
        );
        assert!(
            surface_hsl.lightness < 0.2,
            "Dark variant surface should be very dark"
        );
    }

    #[test]
    fn test_generate_variant_high_contrast() {
        let kit = create_brand_kit("Test", "#FF6B35", None, None, None).unwrap();
        let hc = generate_variant(&kit, "high-contrast").unwrap();

        let fg_rgb = parse_hex_to_srgb(&hc.colors[&BrandColorRole::Primary]).unwrap();
        let bg_rgb = parse_hex_to_srgb(&hc.colors[&BrandColorRole::Surface]).unwrap();
        let ratio = contrast_ratio(fg_rgb, bg_rgb);
        assert!(ratio >= 7.0, "Should meet WCAG AAA, got {:.1}", ratio);
    }

    #[test]
    fn test_export_brand_guide() {
        let kit = create_brand_kit("MyBrand", "#FF6B35", None, None, None).unwrap();
        let guide = export_brand_guide(&kit);
        assert!(guide.contains("# MyBrand — Brand Guide"));
        assert!(guide.contains("primary"));
        assert!(guide.contains("#FF6B35"));
        assert!(guide.contains("CSS Variables"));
    }

    #[test]
    fn test_suggest_brand_from_description() {
        let kit = suggest_brand_from_description("科技").unwrap();
        let rgb = parse_hex_to_srgb(&kit.colors[&BrandColorRole::Primary]).unwrap();
        let hsl: Hsl = Hsl::from_color(rgb);
        let hue = hsl.hue.into_degrees();
        assert!(
            hue > 180.0 && hue < 260.0,
            "Tech should suggest blue, got hue {}",
            hue
        );
    }
}
