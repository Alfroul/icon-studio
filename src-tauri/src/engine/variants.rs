use crate::model::{Element, Gradient, IconProject, ThemeRule};

// ---- Color utilities ----

pub fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
    (r, g, b)
}

pub fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < 1e-10 {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = match max {
        x if (x - r).abs() < 1e-10 => (g - b) / d + if g < b { 6.0 } else { 0.0 },
        x if (x - g).abs() < 1e-10 => (b - r) / d + 2.0,
        _ => (r - g) / d + 4.0,
    };
    (h / 6.0, s, l)
}

pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s.abs() < 1e-10 {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;

    let hue_to_rgb = |t: f64| -> f64 {
        let t = if t < 0.0 { t + 1.0 } else if t > 1.0 { t - 1.0 } else { t };
        if t < 1.0 / 6.0 { p + (q - p) * 6.0 * t }
        else if t < 0.5 { q }
        else if t < 2.0 / 3.0 { p + (q - p) * (2.0 / 3.0 - t) * 6.0 }
        else { p }
    };

    let r = (hue_to_rgb(h + 1.0 / 3.0) * 255.0).round() as u8;
    let g = (hue_to_rgb(h) * 255.0).round() as u8;
    let b = (hue_to_rgb(h - 1.0 / 3.0) * 255.0).round() as u8;
    (r, g, b)
}

pub fn invert_color(hex: &str) -> String {
    let (r, g, b) = hex_to_rgb(hex);
    let (h, s, l) = rgb_to_hsl(r, g, b);
    let (r2, g2, b2) = hsl_to_rgb(h, s, 1.0 - l);
    rgb_to_hex(r2, g2, b2)
}

fn grayscale_color(hex: &str) -> String {
    let (r, g, b) = hex_to_rgb(hex);
    let gray = (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64).round() as u8;
    rgb_to_hex(gray, gray, gray)
}

fn desaturate_color(hex: &str, amount: f64) -> String {
    let (r, g, b) = hex_to_rgb(hex);
    let (h, s, l) = rgb_to_hsl(r, g, b);
    let (r2, g2, b2) = hsl_to_rgb(h, s * (1.0 - amount), l);
    rgb_to_hex(r2, g2, b2)
}

fn colors_match(a: &str, b: &str) -> bool {
    let (r1, g1, b1) = hex_to_rgb(a);
    let (r2, g2, b2) = hex_to_rgb(b);
    let diff = (r1 as i32 - r2 as i32).unsigned_abs()
        + (g1 as i32 - g2 as i32).unsigned_abs()
        + (b1 as i32 - b2 as i32).unsigned_abs();
    diff < 30
}

// ---- Element color extraction / replacement ----

pub fn extract_element_colors(elem: &Element) -> Vec<String> {
    let mut colors = Vec::new();
    match elem {
        Element::Shape(e) => {
            colors.push(e.fill.clone());
            if let Some(ref s) = e.stroke { colors.push(s.clone()); }
            if let Some(ref g) = e.gradient {
                colors.extend(g.colors.iter().cloned());
            }
        }
        Element::Text(e) => {
            colors.push(e.fill.clone());
            if let Some(ref s) = e.stroke { colors.push(s.clone()); }
            if let Some(ref g) = e.gradient {
                colors.extend(g.colors.iter().cloned());
            }
        }
        Element::Icon(e) => {
            colors.push(e.fill.clone());
            if let Some(ref s) = e.stroke { colors.push(s.clone()); }
            if let Some(ref g) = e.gradient {
                colors.extend(g.colors.iter().cloned());
            }
        }
        Element::Path(e) => {
            colors.push(e.fill.clone());
            colors.push(e.stroke.clone());
        }
        Element::Group(g) => {
            for child in &g.children {
                colors.extend(extract_element_colors(child));
            }
        }
        Element::Image(_) | Element::Symbol(_) => {}
    }
    // Also extract shadow colors
    for shadow in &elem.common().shadows {
        if let Some(hex) = shadow.color.get(..7) {
            colors.push(hex.to_string());
        }
    }
    colors
}

fn transform_gradient_colors(gradient: &mut Gradient, f: &dyn Fn(&str) -> String) {
    for c in &mut gradient.colors {
        *c = f(c);
    }
}

fn replace_gradient_colors(gradient: &mut Gradient, from: &str, to: &str) {
    for c in &mut gradient.colors {
        if colors_match(c, from) {
            *c = to.to_string();
        }
    }
}

pub fn replace_element_color(elem: &mut Element, from: &str, to: &str) {
    match elem {
        Element::Shape(e) => {
            if colors_match(&e.fill, from) { e.fill = to.to_string(); }
            if let Some(ref mut s) = e.stroke {
                if colors_match(s, from) { *s = to.to_string(); }
            }
            if let Some(ref mut g) = e.gradient {
                replace_gradient_colors(g, from, to);
            }
        }
        Element::Text(e) => {
            if colors_match(&e.fill, from) { e.fill = to.to_string(); }
            if let Some(ref mut s) = e.stroke {
                if colors_match(s, from) { *s = to.to_string(); }
            }
            if let Some(ref mut g) = e.gradient {
                replace_gradient_colors(g, from, to);
            }
        }
        Element::Icon(e) => {
            if colors_match(&e.fill, from) { e.fill = to.to_string(); }
            if let Some(ref mut s) = e.stroke {
                if colors_match(s, from) { *s = to.to_string(); }
            }
            if let Some(ref mut g) = e.gradient {
                replace_gradient_colors(g, from, to);
            }
        }
        Element::Path(e) => {
            if colors_match(&e.fill, from) { e.fill = to.to_string(); }
            if colors_match(&e.stroke, from) { e.stroke = to.to_string(); }
        }
        Element::Group(g) => {
            for child in &mut g.children {
                replace_element_color(child, from, to);
            }
        }
        Element::Image(_) | Element::Symbol(_) => {}
    }
}

fn apply_color_transform(elem: &mut Element, f: &dyn Fn(&str) -> String) {
    match elem {
        Element::Shape(e) => {
            e.fill = f(&e.fill);
            if let Some(ref mut s) = e.stroke { *s = f(s); }
            if let Some(ref mut g) = e.gradient { transform_gradient_colors(g, f); }
        }
        Element::Text(e) => {
            e.fill = f(&e.fill);
            if let Some(ref mut s) = e.stroke { *s = f(s); }
            if let Some(ref mut g) = e.gradient { transform_gradient_colors(g, f); }
        }
        Element::Icon(e) => {
            e.fill = f(&e.fill);
            if let Some(ref mut s) = e.stroke { *s = f(s); }
            if let Some(ref mut g) = e.gradient { transform_gradient_colors(g, f); }
        }
        Element::Path(e) => {
            e.fill = f(&e.fill);
            e.stroke = f(&e.stroke);
        }
        Element::Group(g) => {
            for child in &mut g.children {
                apply_color_transform(child, f);
            }
        }
        Element::Image(_) | Element::Symbol(_) => {}
    }
}

// ---- Rule application ----

fn apply_rule(elem: &mut Element, rule: &ThemeRule) {
    match rule {
        ThemeRule::InvertColors => {
            apply_color_transform(elem, &|c| invert_color(c));
        }
        ThemeRule::ReplaceColor { from, to } => {
            replace_element_color(elem, from, to);
        }
        ThemeRule::AdjustOpacity { factor } => {
            let common = elem.common_mut();
            common.opacity = (common.opacity * factor).clamp(0.0, 1.0);
            if let Element::Group(g) = elem {
                for child in &mut g.children {
                    apply_rule(child, rule);
                }
            }
        }
        ThemeRule::Grayscale => {
            apply_color_transform(elem, &|c| grayscale_color(c));
        }
        ThemeRule::Desaturate { factor } => {
            let amt = *factor;
            apply_color_transform(elem, &move |c| desaturate_color(c, amt));
        }
        ThemeRule::CustomFill { color } => {
            apply_color_transform(elem, &|_| color.clone());
        }
    }
}

pub fn apply_rules(elem: &mut Element, rules: &[ThemeRule]) {
    for rule in rules {
        apply_rule(elem, rule);
    }
}

// ---- Variant generation ----

pub fn generate_variant(project: &IconProject, rules: &[ThemeRule]) -> IconProject {
    let mut derived = project.clone();

    // Apply rules to all elements on all pages
    if !derived.pages.is_empty() {
        for page in &mut derived.pages {
            for elem in &mut page.elements {
                apply_rules(elem, rules);
            }
        }
    }
    // Also apply to legacy elements
    for elem in &mut derived.elements {
        apply_rules(elem, rules);
    }

    // Also transform canvas background
    let bg = &derived.canvas.background;
    if bg.starts_with('#') && bg.len() >= 7 {
        let transformed_bg = rules.iter().fold(bg.clone(), |acc, rule| match rule {
            ThemeRule::InvertColors => invert_color(&acc),
            ThemeRule::Grayscale => grayscale_color(&acc),
            ThemeRule::Desaturate { factor } => desaturate_color(&acc, *factor),
            ThemeRule::CustomFill { color } => color.clone(),
            _ => acc,
        });
        derived.canvas.background = transformed_bg;
    }
    if let Some(ref mut grad) = derived.canvas.background_gradient {
        for rule in rules {
            match rule {
                ThemeRule::InvertColors => transform_gradient_colors(grad, &|c| invert_color(c)),
                ThemeRule::Grayscale => transform_gradient_colors(grad, &|c| grayscale_color(c)),
                ThemeRule::Desaturate { factor } => {
                    let amt = *factor;
                    transform_gradient_colors(grad, &move |c| desaturate_color(c, amt));
                }
                ThemeRule::CustomFill { color } => transform_gradient_colors(grad, &|_| color.clone()),
                _ => {}
            }
        }
    }

    derived.version = project.version;
    derived
}

// ---- Preset rule sets ----

pub fn dark_mode_rules() -> Vec<ThemeRule> {
    vec![ThemeRule::InvertColors]
}

pub fn hover_rules() -> Vec<ThemeRule> {
    // Slightly darker + opacity boost
    vec![
        ThemeRule::Desaturate { factor: -0.1 },
        ThemeRule::AdjustOpacity { factor: 1.1 },
    ]
}

pub fn active_rules() -> Vec<ThemeRule> {
    vec![
        ThemeRule::Desaturate { factor: -0.2 },
        ThemeRule::AdjustOpacity { factor: 0.8 },
    ]
}

pub fn disabled_rules() -> Vec<ThemeRule> {
    vec![
        ThemeRule::Grayscale,
        ThemeRule::AdjustOpacity { factor: 0.4 },
    ]
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetRuleSet {
    pub name: String,
    pub rules: Vec<ThemeRule>,
}

pub fn list_preset_rule_sets() -> Vec<PresetRuleSet> {
    vec![
        PresetRuleSet { name: "Dark".into(), rules: dark_mode_rules() },
        PresetRuleSet { name: "Hover".into(), rules: hover_rules() },
        PresetRuleSet { name: "Active".into(), rules: active_rules() },
        PresetRuleSet { name: "Disabled".into(), rules: disabled_rules() },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn make_shape(id: &str, fill: &str) -> Element {
        Element::Shape(ShapeElement {
            common: CommonProps::new(id.to_string(), 0.0, 0.0, 100.0, 100.0),
            shape_type: shapes::ShapeType::Circle,
            fill: fill.to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        })
    }

    #[test]
    fn test_invert_color() {
        assert_eq!(invert_color("#000000"), "#FFFFFF");
        assert_eq!(invert_color("#FFFFFF"), "#000000");
        // Mid-gray inverts to mid-gray
        let inverted = invert_color("#808080");
        let (r, g, b) = hex_to_rgb(&inverted);
        assert!((r as i32 - 0x7F).unsigned_abs() <= 1);
        assert!((g as i32 - 0x7F).unsigned_abs() <= 1);
        assert!((b as i32 - 0x7F).unsigned_abs() <= 1);
    }

    #[test]
    fn test_grayscale() {
        let result = grayscale_color("#FF0000");
        let (r, g, b) = hex_to_rgb(&result);
        assert_eq!(r, g);
        assert_eq!(g, b);

        // Pure white stays white
        assert_eq!(grayscale_color("#FFFFFF"), "#FFFFFF");
        // Pure black stays black
        assert_eq!(grayscale_color("#000000"), "#000000");
    }

    #[test]
    fn test_adjust_opacity() {
        let mut elem = make_shape("s1", "#FF0000");
        elem.common_mut().opacity = 1.0;
        apply_rule(&mut elem, &ThemeRule::AdjustOpacity { factor: 0.5 });
        assert!((elem.common().opacity - 0.5).abs() < 1e-10);

        // Clamps to 1.0
        elem.common_mut().opacity = 0.9;
        apply_rule(&mut elem, &ThemeRule::AdjustOpacity { factor: 1.5 });
        assert!((elem.common().opacity - 1.0).abs() < 1e-10);

        // Clamps to 0.0
        elem.common_mut().opacity = 0.1;
        apply_rule(&mut elem, &ThemeRule::AdjustOpacity { factor: 0.0 });
        assert!((elem.common().opacity - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_generate_variant_dark() {
        let mut project = IconProject::default();
        project.elements.push(make_shape("s1", "#333333"));

        let variant = generate_variant(&project, &dark_mode_rules());

        // Dark gray should invert to light
        if let Element::Shape(s) = &variant.elements[0] {
            let (r, _g, _b) = hex_to_rgb(&s.fill);
            assert!(r > 200, "Expected light color after inversion, got {}", s.fill);
        }
    }

    #[test]
    fn test_generate_variant_disabled() {
        let mut project = IconProject::default();
        project.elements.push(make_shape("s1", "#FF0000"));

        let variant = generate_variant(&project, &disabled_rules());

        if let Element::Shape(s) = &variant.elements[0] {
            let (r, g, b) = hex_to_rgb(&s.fill);
            assert_eq!(r, g, "Should be grayscale");
            assert_eq!(g, b, "Should be grayscale");
            let _ = (r, b); // suppress warnings
        }
        assert!((variant.elements[0].common().opacity - 0.4).abs() < 1e-10);
    }

    #[test]
    fn test_replace_color() {
        let mut elem = make_shape("s1", "#FF0000");
        apply_rule(&mut elem, &ThemeRule::ReplaceColor {
            from: "#FF0000".into(),
            to: "#0000FF".into(),
        });
        if let Element::Shape(s) = &elem {
            assert_eq!(s.fill, "#0000FF");
        }

        // Fuzzy match (close color)
        let mut elem2 = make_shape("s2", "#FE0001");
        apply_rule(&mut elem2, &ThemeRule::ReplaceColor {
            from: "#FF0000".into(),
            to: "#00FF00".into(),
        });
        if let Element::Shape(s) = &elem2 {
            assert_eq!(s.fill, "#00FF00", "Fuzzy match should work");
        }
    }

    #[test]
    fn test_desaturate() {
        let result = desaturate_color("#FF0000", 0.5);
        let (_h, s, _l) = {
            let (r, g, b) = hex_to_rgb(&result);
            rgb_to_hsl(r, g, b)
        };
        let (_, orig_s, _) = rgb_to_hsl(255, 0, 0);
        assert!(s < orig_s, "Saturation should decrease: {} < {}", s, orig_s);
    }

    #[test]
    fn test_hex_to_rgb_roundtrip() {
        assert_eq!(rgb_to_hex(255, 0, 0), "#FF0000");
        assert_eq!(rgb_to_hex(0, 255, 0), "#00FF00");
        assert_eq!(rgb_to_hex(0, 0, 255), "#0000FF");
        assert_eq!(rgb_to_hex(0, 0, 0), "#000000");
        assert_eq!(rgb_to_hex(255, 255, 255), "#FFFFFF");
    }

    #[test]
    fn test_hsl_to_rgb_roundtrip() {
        let test_colors = [("#FF0000", 0), ("#00FF00", 0), ("#0000FF", 0), ("#808080", 0)];
        for (hex, _) in &test_colors {
            let (r, g, b) = hex_to_rgb(hex);
            let (h, s, l) = rgb_to_hsl(r, g, b);
            let (r2, g2, b2) = hsl_to_rgb(h, s, l);
            assert!((r as i32 - r2 as i32).unsigned_abs() <= 1, "R mismatch for {}", hex);
            assert!((g as i32 - g2 as i32).unsigned_abs() <= 1, "G mismatch for {}", hex);
            assert!((b as i32 - b2 as i32).unsigned_abs() <= 1, "B mismatch for {}", hex);
        }
    }

    #[test]
    fn test_extract_element_colors() {
        let elem = make_shape("s1", "#FF0000");
        let colors = extract_element_colors(&elem);
        assert!(colors.contains(&"#FF0000".to_string()));
    }

    #[test]
    fn test_variant_preserves_structure() {
        let mut project = IconProject::default();
        project.elements.push(make_shape("s1", "#FF0000"));
        project.elements.push(make_shape("s2", "#00FF00"));

        let variant = generate_variant(&project, &dark_mode_rules());
        assert_eq!(variant.elements.len(), 2);
        assert_eq!(variant.canvas.width, project.canvas.width);
    }
}
