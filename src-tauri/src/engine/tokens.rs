use crate::model::{BrandColorRole, Element, IconElement, IconProject, PathElement, Shadow, ShapeElement, TextElement};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TokenFormat {
    CssVariables,
    JsonDtcg,
    ScssVariables,
    TailwindConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignTokens {
    pub colors: HashMap<String, String>,
    pub radii: HashMap<String, f64>,
    pub shadows: Vec<String>,
    pub stroke_widths: Vec<f64>,
    #[serde(default)]
    pub icon_size: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenExportResult {
    pub content: String,
    pub format: TokenFormat,
    pub filename: String,
}

const RADIUS_NAMES: &[&str] = &["sm", "md", "lg", "xl", "2xl"];
const SHADOW_NAMES: &[&str] = &["sm", "md", "lg", "xl"];
const STROKE_NAMES: &[&str] = &["thin", "regular", "bold", "heavy"];

fn format_shadow(shadow: &Shadow) -> String {
    if shadow.inset {
        format!(
            "inset {}px {}px {}px {}",
            shadow.offset_x, shadow.offset_y, shadow.blur, shadow.color
        )
    } else {
        format!(
            "{}px {}px {}px {}",
            shadow.offset_x, shadow.offset_y, shadow.blur, shadow.color
        )
    }
}

fn is_valid_color(color: &str) -> bool {
    let c = color.trim().to_lowercase();
    !c.is_empty() && c != "none" && c != "transparent"
}

fn collect_color(color: &str, set: &mut BTreeSet<String>) {
    if is_valid_color(color) {
        set.insert(color.trim().to_string());
    }
}

fn push_unique_f64(vec: &mut Vec<f64>, value: f64) {
    if !vec.iter().any(|v| (v - value).abs() < f64::EPSILON) {
        vec.push(value);
    }
}

fn collect_from_elements(
    elements: &[Element],
    element_colors: &mut BTreeSet<String>,
    radii_list: &mut Vec<f64>,
    shadows: &mut Vec<String>,
    stroke_list: &mut Vec<f64>,
) {
    for elem in elements {
        match elem {
            Element::Shape(s) => {
                collect_shape(s, element_colors, radii_list, stroke_list);
            }
            Element::Text(t) => {
                collect_text(t, element_colors, stroke_list);
            }
            Element::Icon(i) => {
                collect_icon(i, element_colors, stroke_list);
            }
            Element::Path(p) => {
                collect_path(p, element_colors, stroke_list);
            }
            Element::Group(g) => {
                collect_from_elements(
                    &g.children,
                    element_colors,
                    radii_list,
                    shadows,
                    stroke_list,
                );
            }
            _ => {}
        }

        for shadow in &elem.common().shadows {
            let s = format_shadow(shadow);
            if !shadows.contains(&s) {
                shadows.push(s);
            }
        }
    }
}

fn collect_shape(
    s: &ShapeElement,
    colors: &mut BTreeSet<String>,
    radii: &mut Vec<f64>,
    strokes: &mut Vec<f64>,
) {
    collect_color(&s.fill, colors);
    if let Some(ref stroke) = s.stroke {
        collect_color(stroke, colors);
    }
    if s.border_radius > 0.0 {
        push_unique_f64(radii, s.border_radius);
    }
    if s.stroke_width > 0.0 {
        push_unique_f64(strokes, s.stroke_width);
    }
    if let Some(ref g) = s.gradient {
        for c in &g.colors {
            collect_color(c, colors);
        }
    }
}

fn collect_text(
    t: &TextElement,
    colors: &mut BTreeSet<String>,
    strokes: &mut Vec<f64>,
) {
    collect_color(&t.fill, colors);
    if let Some(ref stroke) = t.stroke {
        collect_color(stroke, colors);
    }
    if t.stroke_width > 0.0 {
        push_unique_f64(strokes, t.stroke_width);
    }
    if let Some(ref g) = t.gradient {
        for c in &g.colors {
            collect_color(c, colors);
        }
    }
}

fn collect_icon(
    i: &IconElement,
    colors: &mut BTreeSet<String>,
    strokes: &mut Vec<f64>,
) {
    collect_color(&i.fill, colors);
    if let Some(ref stroke) = i.stroke {
        collect_color(stroke, colors);
    }
    if i.stroke_width > 0.0 {
        push_unique_f64(strokes, i.stroke_width);
    }
    if let Some(ref g) = i.gradient {
        for c in &g.colors {
            collect_color(c, colors);
        }
    }
}

fn collect_path(
    p: &PathElement,
    colors: &mut BTreeSet<String>,
    strokes: &mut Vec<f64>,
) {
    collect_color(&p.fill, colors);
    collect_color(&p.stroke, colors);
    if p.stroke_width > 0.0 {
        push_unique_f64(strokes, p.stroke_width);
    }
}

pub fn extract_tokens(project: &IconProject) -> DesignTokens {
    let mut colors = HashMap::new();
    let mut radii_list = Vec::new();
    let mut shadows = Vec::new();
    let mut stroke_list = Vec::new();
    let mut element_colors = BTreeSet::new();

    // Brand kit colors (first brand kit, semantic names)
    if let Some(brand_kit) = project.brand_kits.first() {
        for (role, color) in &brand_kit.colors {
            let name = match role {
                BrandColorRole::Primary => "primary",
                BrandColorRole::Secondary => "secondary",
                BrandColorRole::Accent => "accent",
                BrandColorRole::Neutral => "neutral",
                BrandColorRole::Surface => "surface",
                BrandColorRole::Error => "error",
            };
            colors.insert(name.to_string(), color.clone());
        }
    }

    // Collect from all active elements
    collect_from_elements(
        project.active_elements(),
        &mut element_colors,
        &mut radii_list,
        &mut shadows,
        &mut stroke_list,
    );

    // Remove brand colors, then map remaining to numbered names
    let brand_colors: std::collections::HashSet<String> =
        colors.values().cloned().collect();
    element_colors.retain(|c| !brand_colors.contains(c));

    for (i, color) in element_colors.iter().enumerate() {
        colors.insert(format!("{}", i + 1), color.clone());
    }

    // Sort radii and map to semantic size names
    radii_list.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut radii = HashMap::new();
    for (i, &r) in radii_list.iter().enumerate() {
        let name = RADIUS_NAMES
            .get(i)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}", i + 1));
        radii.insert(name, r);
    }

    stroke_list.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let canvas = project.active_canvas();
    let icon_size = if canvas.width > 0 && canvas.height > 0 {
        Some((canvas.width, canvas.height))
    } else {
        None
    };

    DesignTokens {
        colors,
        radii,
        shadows,
        stroke_widths: stroke_list,
        icon_size,
    }
}

// ---------------------------------------------------------------------------
// Formatters
// ---------------------------------------------------------------------------

fn format_css(tokens: &DesignTokens) -> String {
    let mut lines = vec![
        "/* IconStudio Design Tokens */".to_string(),
        ":root {".to_string(),
    ];

    if !tokens.colors.is_empty() {
        lines.push("  /* Colors */".to_string());
        let mut sorted: Vec<_> = tokens.colors.iter().collect();
        sorted.sort_by(|a, b| {
            let brand_a = !a.0.chars().all(|c| c.is_ascii_digit());
            let brand_b = !b.0.chars().all(|c| c.is_ascii_digit());
            match (brand_a, brand_b) {
                (true, true) => a.0.cmp(b.0),
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (false, false) => a.0.cmp(b.0),
            }
        });
        for (name, value) in sorted {
            lines.push(format!("  --color-{}: {};", name, value));
        }
    }

    if !tokens.radii.is_empty() {
        lines.push("".to_string());
        lines.push("  /* Border Radius */".to_string());
        let mut sorted: Vec<_> = tokens.radii.iter().collect();
        sorted.sort_by_key(|(k, _)| {
            RADIUS_NAMES
                .iter()
                .position(|&n| n == k.as_str())
                .unwrap_or(999)
        });
        for (name, &value) in sorted {
            lines.push(format!("  --radius-{}: {}px;", name, value));
        }
    }

    if !tokens.shadows.is_empty() {
        lines.push("".to_string());
        lines.push("  /* Shadows */".to_string());
        for (i, shadow) in tokens.shadows.iter().enumerate() {
            let name = SHADOW_NAMES.get(i).unwrap_or(&"xl");
            lines.push(format!("  --shadow-{}: {};", name, shadow));
        }
    }

    if !tokens.stroke_widths.is_empty() {
        lines.push("".to_string());
        lines.push("  /* Stroke Width */".to_string());
        for (i, &width) in tokens.stroke_widths.iter().enumerate() {
            let name = STROKE_NAMES.get(i).unwrap_or(&"heavy");
            lines.push(format!("  --stroke-{}: {}px;", name, width));
        }
    }

    if let Some((w, _h)) = tokens.icon_size {
        lines.push("".to_string());
        lines.push("  /* Icon Size */".to_string());
        lines.push(format!("  --icon-size: {}px;", w));
    }

    lines.push("}".to_string());
    lines.join("\n")
}

fn format_json(tokens: &DesignTokens) -> String {
    let mut root = serde_json::Map::new();

    // Colors
    if !tokens.colors.is_empty() {
        let mut color_obj = serde_json::Map::new();
        let mut sorted: Vec<_> = tokens.colors.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));
        for (name, value) in sorted {
            let mut token = serde_json::Map::new();
            token.insert(
                "$value".to_string(),
                serde_json::Value::String(value.clone()),
            );
            token.insert(
                "$type".to_string(),
                serde_json::Value::String("color".to_string()),
            );
            color_obj.insert(name.clone(), serde_json::Value::Object(token));
        }
        root.insert("color".to_string(), serde_json::Value::Object(color_obj));
    }

    // Dimensions (radii + icon size)
    if !tokens.radii.is_empty() || tokens.icon_size.is_some() {
        let mut dim_obj = serde_json::Map::new();

        if !tokens.radii.is_empty() {
            let mut radius_obj = serde_json::Map::new();
            let mut sorted: Vec<_> = tokens.radii.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(b.0));
            for (name, &value) in sorted {
                let mut token = serde_json::Map::new();
                token.insert(
                    "$value".to_string(),
                    serde_json::Value::String(format!("{}px", value)),
                );
                token.insert(
                    "$type".to_string(),
                    serde_json::Value::String("dimension".to_string()),
                );
                radius_obj.insert(name.clone(), serde_json::Value::Object(token));
            }
            dim_obj.insert(
                "radius".to_string(),
                serde_json::Value::Object(radius_obj),
            );
        }

        if let Some((w, _h)) = tokens.icon_size {
            let mut token = serde_json::Map::new();
            token.insert(
                "$value".to_string(),
                serde_json::Value::String(format!("{}px", w)),
            );
            token.insert(
                "$type".to_string(),
                serde_json::Value::String("dimension".to_string()),
            );
            dim_obj.insert("iconSize".to_string(), serde_json::Value::Object(token));
        }

        root.insert(
            "dimension".to_string(),
            serde_json::Value::Object(dim_obj),
        );
    }

    // Shadows
    if !tokens.shadows.is_empty() {
        let mut shadow_obj = serde_json::Map::new();
        for (i, shadow) in tokens.shadows.iter().enumerate() {
            let name = SHADOW_NAMES.get(i).unwrap_or(&"xl");
            let mut token = serde_json::Map::new();
            token.insert(
                "$value".to_string(),
                serde_json::Value::String(shadow.clone()),
            );
            token.insert(
                "$type".to_string(),
                serde_json::Value::String("shadow".to_string()),
            );
            shadow_obj.insert(name.to_string(), serde_json::Value::Object(token));
        }
        root.insert(
            "shadow".to_string(),
            serde_json::Value::Object(shadow_obj),
        );
    }

    // Stroke widths
    if !tokens.stroke_widths.is_empty() {
        let mut stroke_obj = serde_json::Map::new();
        for (i, &width) in tokens.stroke_widths.iter().enumerate() {
            let name = STROKE_NAMES.get(i).unwrap_or(&"heavy");
            let mut token = serde_json::Map::new();
            token.insert(
                "$value".to_string(),
                serde_json::Value::String(format!("{}px", width)),
            );
            token.insert(
                "$type".to_string(),
                serde_json::Value::String("dimension".to_string()),
            );
            stroke_obj.insert(name.to_string(), serde_json::Value::Object(token));
        }
        root.insert(
            "strokeWidth".to_string(),
            serde_json::Value::Object(stroke_obj),
        );
    }

    serde_json::to_string_pretty(&serde_json::Value::Object(root)).unwrap_or_default()
}

fn format_scss(tokens: &DesignTokens) -> String {
    let mut lines = vec!["// IconStudio Design Tokens".to_string()];

    let mut sorted: Vec<_> = tokens.colors.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));
    for (name, value) in sorted {
        lines.push(format!("$color-{}: {};", name, value));
    }

    let mut sorted_radii: Vec<_> = tokens.radii.iter().collect();
    sorted_radii.sort_by_key(|(k, _)| {
        RADIUS_NAMES
            .iter()
            .position(|&n| n == k.as_str())
            .unwrap_or(999)
    });
    for (name, &value) in sorted_radii {
        lines.push(format!("$radius-{}: {}px;", name, value));
    }

    for (i, shadow) in tokens.shadows.iter().enumerate() {
        let name = SHADOW_NAMES.get(i).unwrap_or(&"xl");
        lines.push(format!("$shadow-{}: {};", name, shadow));
    }

    for (i, &width) in tokens.stroke_widths.iter().enumerate() {
        let name = STROKE_NAMES.get(i).unwrap_or(&"heavy");
        lines.push(format!("$stroke-{}: {}px;", name, width));
    }

    if let Some((w, _h)) = tokens.icon_size {
        lines.push(format!("$icon-size: {}px;", w));
    }

    lines.join("\n")
}

fn format_tailwind(tokens: &DesignTokens) -> String {
    let mut sections = Vec::new();

    // Colors
    if !tokens.colors.is_empty() {
        let mut pairs: Vec<_> = tokens.colors.iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(b.0));
        let entries: Vec<String> = pairs
            .iter()
            .map(|(k, v)| format!("        {}: '{}',", k, v))
            .collect();
        sections.push(format!(
            "      colors: {{\n{}\n      }},",
            entries.join("\n")
        ));
    }

    // Border radius
    if !tokens.radii.is_empty() {
        let mut pairs: Vec<_> = tokens.radii.iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(b.0));
        let entries: Vec<String> = pairs
            .iter()
            .map(|(k, v)| format!("        {}: '{}px',", k, v))
            .collect();
        sections.push(format!(
            "      borderRadius: {{\n{}\n      }},",
            entries.join("\n")
        ));
    }

    // Shadows
    if !tokens.shadows.is_empty() {
        let entries: Vec<String> = tokens
            .shadows
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let name = SHADOW_NAMES.get(i).unwrap_or(&"xl");
                format!("        {}: '{}',", name, s)
            })
            .collect();
        sections.push(format!(
            "      boxShadow: {{\n{}\n      }},",
            entries.join("\n")
        ));
    }

    // Stroke widths
    if !tokens.stroke_widths.is_empty() {
        let entries: Vec<String> = tokens
            .stroke_widths
            .iter()
            .enumerate()
            .map(|(i, &w)| {
                let name = STROKE_NAMES.get(i).unwrap_or(&"heavy");
                format!("        {}: '{}px',", name, w)
            })
            .collect();
        sections.push(format!(
            "      strokeWidth: {{\n{}\n      }},",
            entries.join("\n")
        ));
    }

    if sections.is_empty() {
        return "module.exports = {\n  theme: {\n    extend: {},\n  },\n};\n".to_string();
    }

    format!(
        "module.exports = {{\n  theme: {{\n    extend: {{\n{}\n    }},\n  }},\n}};\n",
        sections.join("\n")
    )
}

pub fn format_tokens(tokens: &DesignTokens, format: TokenFormat) -> TokenExportResult {
    let (content, filename) = match format {
        TokenFormat::CssVariables => (format_css(tokens), "tokens.css".to_string()),
        TokenFormat::JsonDtcg => (format_json(tokens), "tokens.json".to_string()),
        TokenFormat::ScssVariables => (format_scss(tokens), "tokens.scss".to_string()),
        TokenFormat::TailwindConfig => (format_tailwind(tokens), "tailwind.config.js".to_string()),
    };
    TokenExportResult {
        content,
        format,
        filename,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{shapes, BrandKit, CommonProps};

    fn make_shape(
        id: &str,
        fill: &str,
        stroke: Option<&str>,
        stroke_width: f64,
        border_radius: f64,
    ) -> Element {
        Element::Shape(ShapeElement {
            common: CommonProps::new(id.to_string(), 0.0, 0.0, 100.0, 100.0),
            shape_type: shapes::ShapeType::Rect,
            fill: fill.to_string(),
            stroke: stroke.map(|s| s.to_string()),
            stroke_width,
            border_radius,
            stroke_dasharray: None,
            gradient: None,
        })
    }

    fn make_text(id: &str, fill: &str, stroke: Option<&str>, stroke_width: f64) -> Element {
        Element::Text(TextElement {
            common: CommonProps::new(id.to_string(), 0.0, 0.0, 100.0, 20.0),
            content: "Hello".to_string(),
            fill: fill.to_string(),
            font_family: "Arial".to_string(),
            font_size: 16.0,
            font_weight: "400".to_string(),
            letter_spacing: 0.0,
            stroke: stroke.map(|s| s.to_string()),
            stroke_width,
            gradient: None,
        })
    }

    fn make_shadow_element(id: &str, color: &str, blur: f64, ox: f64, oy: f64) -> Element {
        let mut e = make_shape(id, "#000000", None, 0.0, 0.0);
        e.common_mut().shadows.push(Shadow {
            color: color.to_string(),
            blur,
            offset_x: ox,
            offset_y: oy,
            inset: false,
        });
        e
    }

    #[test]
    fn test_extract_colors_from_project() {
        let mut project = IconProject::default();
        project.elements.push(make_shape("s1", "#FF0000", None, 0.0, 0.0));
        project.elements.push(make_shape("s2", "#00FF00", Some("#0000FF"), 0.0, 0.0));
        project
            .elements
            .push(make_shape("s3", "#FF0000", None, 0.0, 0.0)); // duplicate
        project
            .elements
            .push(make_text("t1", "#FF0000", None, 0.0)); // duplicate color

        let tokens = extract_tokens(&project);

        // 3 unique colors: #FF0000, #00FF00, #0000FF
        assert_eq!(tokens.colors.len(), 3);
        assert!(tokens.colors.values().any(|v| v == "#FF0000"));
        assert!(tokens.colors.values().any(|v| v == "#00FF00"));
        assert!(tokens.colors.values().any(|v| v == "#0000FF"));
    }

    #[test]
    fn test_extract_radii() {
        let mut project = IconProject::default();
        project
            .elements
            .push(make_shape("s1", "#000", None, 0.0, 4.0));
        project
            .elements
            .push(make_shape("s2", "#000", None, 0.0, 8.0));
        project
            .elements
            .push(make_shape("s3", "#000", None, 0.0, 4.0)); // duplicate
        project
            .elements
            .push(make_shape("s4", "#000", None, 0.0, 16.0));

        let tokens = extract_tokens(&project);

        // 3 unique radii: 4, 8, 16 → sm, md, lg
        assert_eq!(tokens.radii.len(), 3);
        assert_eq!(tokens.radii.get("sm"), Some(&4.0));
        assert_eq!(tokens.radii.get("md"), Some(&8.0));
        assert_eq!(tokens.radii.get("lg"), Some(&16.0));
    }

    #[test]
    fn test_extract_shadows() {
        let mut project = IconProject::default();
        project
            .elements
            .push(make_shadow_element("s1", "#00000040", 8.0, 0.0, 4.0));
        project
            .elements
            .push(make_shadow_element("s2", "#00000080", 16.0, 2.0, 8.0));

        let tokens = extract_tokens(&project);

        assert_eq!(tokens.shadows.len(), 2);
        assert!(tokens.shadows[0].contains("0px"));
        assert!(tokens.shadows[0].contains("4px"));
        assert!(tokens.shadows[0].contains("8px"));
        assert!(tokens.shadows[0].contains("#00000040"));
    }

    #[test]
    fn test_format_css() {
        let tokens = DesignTokens {
            colors: {
                let mut m = HashMap::new();
                m.insert("primary".to_string(), "#FBBF24".to_string());
                m.insert("1".to_string(), "#FF0000".to_string());
                m
            },
            radii: {
                let mut m = HashMap::new();
                m.insert("sm".to_string(), 4.0);
                m
            },
            shadows: vec!["0px 4px 8px #00000040".to_string()],
            stroke_widths: vec![1.5],
            icon_size: Some((512, 512)),
        };

        let css = format_css(&tokens);

        assert!(css.starts_with("/* IconStudio Design Tokens */"));
        assert!(css.contains(":root {"));
        assert!(css.contains("}"));
        assert!(css.contains("--color-primary: #FBBF24;"));
        assert!(css.contains("--color-1: #FF0000;"));
        assert!(css.contains("--radius-sm: 4px;"));
        assert!(css.contains("--shadow-sm:"));
        assert!(css.contains("--stroke-thin: 1.5px;"));
        assert!(css.contains("--icon-size: 512px;"));
    }

    #[test]
    fn test_format_json_dtcg() {
        let tokens = DesignTokens {
            colors: {
                let mut m = HashMap::new();
                m.insert("primary".to_string(), "#FBBF24".to_string());
                m
            },
            radii: {
                let mut m = HashMap::new();
                m.insert("sm".to_string(), 4.0);
                m
            },
            shadows: vec![],
            stroke_widths: vec![],
            icon_size: None,
        };

        let json = format_json(&tokens);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Check color token
        let primary = &parsed["color"]["primary"];
        assert_eq!(primary["$value"], "#FBBF24");
        assert_eq!(primary["$type"], "color");

        // Check dimension token
        let radius_sm = &parsed["dimension"]["radius"]["sm"];
        assert_eq!(radius_sm["$value"], "4px");
        assert_eq!(radius_sm["$type"], "dimension");
    }

    #[test]
    fn test_format_scss() {
        let tokens = DesignTokens {
            colors: {
                let mut m = HashMap::new();
                m.insert("primary".to_string(), "#FBBF24".to_string());
                m.insert("1".to_string(), "#FF0000".to_string());
                m
            },
            radii: {
                let mut m = HashMap::new();
                m.insert("sm".to_string(), 4.0);
                m
            },
            shadows: vec!["0px 4px 8px #00000040".to_string()],
            stroke_widths: vec![1.5],
            icon_size: Some((512, 512)),
        };

        let scss = format_scss(&tokens);

        assert!(scss.starts_with("// IconStudio Design Tokens"));
        assert!(scss.contains("$color-1: #FF0000;"));
        assert!(scss.contains("$color-primary: #FBBF24;"));
        assert!(scss.contains("$radius-sm: 4px;"));
        assert!(scss.contains("$shadow-sm:"));
        assert!(scss.contains("$stroke-thin: 1.5px;"));
        assert!(scss.contains("$icon-size: 512px;"));
    }

    #[test]
    fn test_format_tailwind() {
        let tokens = DesignTokens {
            colors: {
                let mut m = HashMap::new();
                m.insert("primary".to_string(), "#FBBF24".to_string());
                m
            },
            radii: {
                let mut m = HashMap::new();
                m.insert("sm".to_string(), 4.0);
                m
            },
            shadows: vec![],
            stroke_widths: vec![],
            icon_size: None,
        };

        let tw = format_tailwind(&tokens);

        assert!(tw.contains("module.exports"));
        assert!(tw.contains("colors:"));
        assert!(tw.contains("primary: '#FBBF24'"));
        assert!(tw.contains("borderRadius:"));
        assert!(tw.contains("sm: '4px'"));
    }

    #[test]
    fn test_empty_project_tokens() {
        let project = IconProject::default();
        let tokens = extract_tokens(&project);

        assert!(tokens.colors.is_empty());
        assert!(tokens.radii.is_empty());
        assert!(tokens.shadows.is_empty());
        assert!(tokens.stroke_widths.is_empty());

        // Verify all formatters produce valid output on empty tokens
        let css = format_css(&tokens);
        assert!(css.contains(":root"));
        assert!(css.contains("}"));

        let json = format_json(&tokens);
        assert_eq!(json, "{}");

        let scss = format_scss(&tokens);
        assert!(scss.starts_with("//"));

        let tw = format_tailwind(&tokens);
        assert!(tw.contains("module.exports"));
    }

    #[test]
    fn test_tokens_from_brand_kit() {
        let mut project = IconProject::default();
        let mut brand_colors = HashMap::new();
        brand_colors.insert(BrandColorRole::Primary, "#FBBF24".to_string());
        brand_colors.insert(BrandColorRole::Secondary, "#1E293B".to_string());
        brand_colors.insert(BrandColorRole::Error, "#EF4444".to_string());

        project.brand_kits.push(BrandKit {
            id: "brand-1".to_string(),
            name: "Test Brand".to_string(),
            colors: brand_colors,
            variants: HashMap::new(),
        });

        // Add elements with some colors overlapping brand kit
        project
            .elements
            .push(make_shape("s1", "#FBBF24", None, 0.0, 0.0)); // same as primary
        project
            .elements
            .push(make_shape("s2", "#FF0000", None, 0.0, 0.0)); // unique

        let tokens = extract_tokens(&project);

        // Brand colors should have semantic names
        assert_eq!(tokens.colors.get("primary"), Some(&"#FBBF24".to_string()));
        assert_eq!(
            tokens.colors.get("secondary"),
            Some(&"#1E293B".to_string())
        );
        assert_eq!(tokens.colors.get("error"), Some(&"#EF4444".to_string()));

        // #FBBF24 should NOT appear as numbered color (already in brand kit)
        // #FF0000 should appear as numbered color "1"
        assert_eq!(tokens.colors.get("1"), Some(&"#FF0000".to_string()));

        // Total: 3 brand + 1 element = 4
        assert_eq!(tokens.colors.len(), 4);
    }
}
