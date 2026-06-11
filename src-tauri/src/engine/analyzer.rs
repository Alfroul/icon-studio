use crate::colors;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::model::{Element, IconProject};

/// Recursively visit all elements, including children inside groups.
fn visit_all_elements<F>(elements: &[Element], visitor: &mut F)
where
    F: FnMut(&Element),
{
    for elem in elements {
        visitor(elem);
        if let Element::Group(g) = elem {
            visit_all_elements(&g.children, visitor);
        }
    }
}

/// Recursively collect elements matching a predicate, including children inside groups.
fn collect_all_elements<'a, F>(elements: &'a [Element], predicate: &F) -> Vec<&'a Element>
where
    F: Fn(&Element) -> bool,
{
    let mut result = Vec::new();
    for elem in elements {
        if predicate(elem) {
            result.push(elem);
        }
        if let Element::Group(g) = elem {
            result.extend(collect_all_elements(&g.children, predicate));
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub colors: ColorAnalysis,
    pub consistency: ConsistencyReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorAnalysis {
    pub all_colors: Vec<ColorInfo>,
    pub primary: Option<ColorInfo>,
    pub secondary: Vec<ColorInfo>,
    pub accent: Vec<ColorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorInfo {
    pub hex: String,
    pub usage_count: usize,
    pub element_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyReport {
    pub border_radius_consistent: bool,
    pub stroke_width_consistent: bool,
    pub font_size_consistent: bool,
    pub opacity_consistent: bool,
    #[serde(default)]
    pub stroke_weight_consistent: bool,
    #[serde(default)]
    pub fill_style_consistent: bool,
    #[serde(default)]
    pub proportions_consistent: bool,
    #[serde(default)]
    pub visual_center_drift: Option<f64>,
    pub issues: Vec<ConsistencyIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyIssue {
    pub property: String,
    pub expected: String,
    pub actual: String,
    pub element_id: String,
    #[serde(default)]
    pub severity: IssueSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum IssueSeverity {
    #[default]
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FillStyle {
    Outline,
    Filled,
    Duotone,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementFilter {
    pub element_type: Option<String>,
    pub fill: Option<String>,
    pub min_width: Option<f64>,
    pub max_width: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindResult {
    pub matching_ids: Vec<String>,
    pub count: usize,
}

// ---------------------------------------------------------------------------
// Color analysis
// ---------------------------------------------------------------------------

/// Collect fill and stroke colors from all elements.
fn collect_colors(project: &IconProject) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    visit_all_elements(project.active_elements(), &mut |elem| {
        let id = get_element_id(elem);
        let fill = get_element_fill(elem);
        if !fill.is_empty() && !fill.starts_with("url(") {
            let norm = colors::normalize_hex(fill);
            if norm.starts_with('#') {
                map.entry(norm).or_default().push(id.to_string());
            }
        }
        if let Some(stroke) = get_element_stroke(elem) {
            if !stroke.is_empty() && !stroke.starts_with("url(") {
                let norm = colors::normalize_hex(stroke);
                if norm.starts_with('#') {
                    map.entry(norm).or_default().push(id.to_string());
                }
            }
        }
    });

    map
}

/// Analyze colors in the project. Returns ColorAnalysis with primary,
/// secondary, and accent colors identified by usage frequency.
pub fn analyze_colors(project: &IconProject) -> ColorAnalysis {
    let color_map = collect_colors(project);

    if color_map.is_empty() {
        return ColorAnalysis {
            all_colors: Vec::new(),
            primary: None,
            secondary: Vec::new(),
            accent: Vec::new(),
        };
    }

    // Build ColorInfo list sorted by usage_count descending
    let mut all_colors: Vec<ColorInfo> = color_map
        .into_iter()
        .map(|(hex, ids)| {
            let count = ids.len();
            ColorInfo {
                hex,
                usage_count: count,
                element_ids: ids,
            }
        })
        .collect();
    all_colors.sort_by_key(|b| std::cmp::Reverse(b.usage_count));

    let primary = all_colors.first().cloned();
    let secondary: Vec<ColorInfo> = if all_colors.len() > 2 {
        all_colors[1..all_colors.len() / 2 + 1].to_vec()
    } else if all_colors.len() > 1 {
        all_colors[1..2].to_vec()
    } else {
        Vec::new()
    };

    // Accent: colors with lowest usage count but high contrast relative to primary
    let accent = compute_accent_colors(&all_colors);

    ColorAnalysis {
        all_colors,
        primary,
        secondary,
        accent,
    }
}

/// Accent colors: those with the fewest uses that contrast well with primary.
/// Uses a simple heuristic — pick the bottom third by usage, limited to 3,
/// then filter by luminance contrast against the primary color.
fn compute_accent_colors(all_colors: &[ColorInfo]) -> Vec<ColorInfo> {
    if all_colors.len() <= 2 {
        return Vec::new();
    }

    // Take the last third of colors (lowest usage), max 3
    let accent_count = (all_colors.len() / 3).clamp(1, 3);
    let start = all_colors.len().saturating_sub(accent_count);
    let mut candidates = all_colors[start..].to_vec();

    // Filter by luminance contrast with primary (first color = highest usage)
    if let Some(primary) = all_colors.first() {
        let pr = colors::ParsedColor::from_hex(&primary.hex);
        candidates.retain(|c| {
            let cr = colors::ParsedColor::from_hex(&c.hex);
            match (pr, cr) {
                (Some(p), Some(q)) => luminance_diff_u8(&p, &q) > 0.3,
                _ => false,
            }
        });
    }

    candidates
}

fn luminance_diff_u8(c1: &colors::ParsedColor, c2: &colors::ParsedColor) -> f64 {
    let l1 = 0.299 * c1.r as f64 + 0.587 * c1.g as f64 + 0.114 * c1.b as f64;
    let l2 = 0.299 * c2.r as f64 + 0.587 * c2.g as f64 + 0.114 * c2.b as f64;
    (l1 - l2).abs() / 255.0
}

// ---------------------------------------------------------------------------
// Consistency check
// ---------------------------------------------------------------------------

/// Find the mode (most common value) in a list of f64 values.
/// Returns (mode_value, count_of_mode).
fn find_mode(values: &[f64]) -> Option<(f64, usize)> {
    if values.is_empty() {
        return None;
    }

    let mut freq: HashMap<u64, usize> = HashMap::new(); // use ordered_f64 bits as key
    let mut first_val: HashMap<u64, f64> = HashMap::new();

    for &v in values {
        // Normalize -0.0 to 0.0 to avoid treating them as different values
        let normalized = if v == 0.0 { 0.0 } else { v };
        let key = normalized.to_bits();
        *freq.entry(key).or_insert(0) += 1;
        first_val.entry(key).or_insert(v);
    }

    freq.into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(key, count)| (first_val[&key], count))
}

pub fn check_consistency(project: &IconProject) -> ConsistencyReport {
    let mut issues: Vec<ConsistencyIssue> = Vec::new();

    // --- Border radius (for rounded-rect shapes) ---
    let mut border_radii: Vec<(f64, String)> = Vec::new();
    visit_all_elements(project.active_elements(), &mut |elem| {
        if let Element::Shape(s) = elem {
            if matches!(s.shape_type, crate::model::shapes::ShapeType::Rect | crate::model::shapes::ShapeType::RoundedRect)
                && s.border_radius.abs() > f64::EPSILON
            {
                border_radii.push((s.border_radius, s.common.id.clone()));
            }
        }
    });
    let border_radius_consistent = check_float_consistency(
        &border_radii,
        "border_radius",
        &mut issues,
    );

    // --- Stroke width ---
    let mut stroke_widths: Vec<(f64, String)> = Vec::new();
    visit_all_elements(project.active_elements(), &mut |elem| {
        let (sw, id) = match elem {
            Element::Shape(s) => (s.stroke_width, s.common.id.clone()),
            Element::Text(t) => (t.stroke_width, t.common.id.clone()),
            Element::Icon(i) => (i.stroke_width, i.common.id.clone()),
            Element::Image(_) => return,
            Element::Path(p) => (p.stroke_width, p.common.id.clone()),
            Element::Group(_) => return,
            Element::Symbol(_) => return,
        };
        if sw > 0.0 {
            stroke_widths.push((sw, id));
        }
    });

    let stroke_width_consistent = check_float_consistency(
        &stroke_widths,
        "stroke_width",
        &mut issues,
    );

    // --- Font size ---
    let mut font_sizes: Vec<(f64, String)> = Vec::new();
    visit_all_elements(project.active_elements(), &mut |elem| {
        if let Element::Text(t) = elem {
            font_sizes.push((t.font_size, t.common.id.clone()));
        }
    });

    let font_size_consistent = check_float_consistency(
        &font_sizes,
        "font_size",
        &mut issues,
    );

    // --- Opacity ---
    let mut opacities: Vec<(f64, String)> = Vec::new();
    visit_all_elements(project.active_elements(), &mut |elem| {
        let c = elem.common();
        if c.opacity < 1.0 {
            opacities.push((c.opacity, c.id.clone()));
        }
    });

    let opacity_consistent = if opacities.len() < 2 {
        true
    } else {
        check_float_consistency(
            &opacities,
            "opacity",
            &mut issues,
        )
    };

    // --- Stroke weight (same as stroke_width but reported separately) ---
    let stroke_weight_consistent = stroke_width_consistent;

    // --- Fill style ---
    let (fill_style_consistent, fill_issues) = check_fill_style_consistency(project);
    issues.extend(fill_issues);

    // --- Proportions ---
    let (proportions_consistent, prop_issues) = check_proportions_consistency(project);
    issues.extend(prop_issues);

    // --- Visual center drift ---
    let visual_center_drift = compute_visual_center_drift(project);

    ConsistencyReport {
        border_radius_consistent,
        stroke_width_consistent,
        font_size_consistent,
        opacity_consistent,
        stroke_weight_consistent,
        fill_style_consistent,
        proportions_consistent,
        visual_center_drift,
        issues,
    }
}

/// Check consistency of float values: mode is the "expected" value,
/// deviations > 10% are flagged as issues with severity based on deviation.
fn check_float_consistency(
    values: &[(f64, String)],
    property: &str,
    issues: &mut Vec<ConsistencyIssue>,
) -> bool {
    if values.len() < 2 {
        return true;
    }

    let raw: Vec<f64> = values.iter().map(|(v, _)| *v).collect();
    let (mode_val, _mode_count) = match find_mode(&raw) {
        Some(m) => m,
        None => return true,
    };

    if mode_val == 0.0 {
        for &(val, ref id) in values {
            if (val - mode_val).abs() > 0.01 {
                issues.push(ConsistencyIssue {
                    property: property.to_string(),
                    expected: format!("{:.2}", mode_val),
                    actual: format!("{:.2}", val),
                    element_id: id.clone(),
                    severity: IssueSeverity::Warning,
                });
            }
        }
    } else {
        for &(val, ref id) in values {
            let deviation_pct = ((val - mode_val) / mode_val).abs() * 100.0;
            if deviation_pct > 10.0 {
                issues.push(ConsistencyIssue {
                    property: property.to_string(),
                    expected: format!("{:.2}", mode_val),
                    actual: format!("{:.2}", val),
                    element_id: id.clone(),
                    severity: classify_severity(deviation_pct),
                });
            }
        }
    }

    issues.iter().all(|i| i.property != property)
}

// ---------------------------------------------------------------------------
// Element query
// ---------------------------------------------------------------------------

pub fn find_elements(project: &IconProject, filter: &ElementFilter) -> FindResult {
    let matching: Vec<String> = collect_all_elements(project.active_elements(), &|elem| {
        // Type filter
        if let Some(ref ty) = filter.element_type {
            let actual_type = match elem {
                Element::Shape(_) => "shape",
                Element::Text(_) => "text",
                Element::Icon(_) => "icon",
                Element::Image(_) => "image",
                Element::Path(_) => "path",
                Element::Group(_) => "group",
                Element::Symbol(_) => "symbol",
            };
            if actual_type != ty.as_str() {
                return false;
            }
        }

        // Fill filter (exact match, case-insensitive)
        if let Some(ref fill) = filter.fill {
            let actual_fill = get_element_fill(elem);
            if colors::normalize_hex(actual_fill) != colors::normalize_hex(fill) {
                return false;
            }
        }

        // Width range filter
        let (w, _) = get_element_dimensions(elem);
        if let Some(min_w) = filter.min_width {
            if w < min_w {
                return false;
            }
        }
        if let Some(max_w) = filter.max_width {
            if w > max_w {
                return false;
            }
        }

        true
    })
    .iter()
    .map(|elem| get_element_id(elem).to_string())
    .collect();

    let count = matching.len();
    FindResult {
        matching_ids: matching,
        count,
    }
}

// ---------------------------------------------------------------------------
// Helpers — element property accessors
// ---------------------------------------------------------------------------

fn get_element_id(elem: &Element) -> &str {
    elem.id()
}

fn get_element_fill(elem: &Element) -> &str {
    match elem {
        Element::Shape(s) => &s.fill,
        Element::Text(t) => &t.fill,
        Element::Icon(i) => &i.fill,
        Element::Image(_) => "",
        Element::Path(p) => &p.fill,
        Element::Group(_) => "",
        Element::Symbol(_) => "",
    }
}

fn get_element_stroke(elem: &Element) -> Option<&str> {
    match elem {
        Element::Shape(s) => s.stroke.as_deref(),
        Element::Text(t) => t.stroke.as_deref(),
        Element::Icon(i) => i.stroke.as_deref(),
        Element::Image(_) => None,
        Element::Path(p) => Some(&p.stroke),
        Element::Group(_) => None,
        Element::Symbol(_) => None,
    }
}

fn get_element_dimensions(elem: &Element) -> (f64, f64) {
    let c = elem.common();
    (c.width, c.height)
}

fn get_element_stroke_width(elem: &Element) -> f64 {
    match elem {
        Element::Shape(s) => s.stroke_width,
        Element::Text(t) => t.stroke_width,
        Element::Icon(i) => i.stroke_width,
        Element::Path(p) => p.stroke_width,
        _ => 0.0,
    }
}

// ---------------------------------------------------------------------------
// Fill style detection
// ---------------------------------------------------------------------------

/// Detect the fill style of a single element.
pub fn detect_fill_style(elem: &Element) -> FillStyle {
    match elem {
        Element::Group(_) | Element::Symbol(_) | Element::Image(_) => return FillStyle::None,
        _ => {}
    }

    let fill = get_element_fill(elem);
    let has_fill = !fill.is_empty()
        && fill != "none"
        && fill != "transparent";
    let stroke = get_element_stroke(elem);
    let has_stroke = stroke.is_some()
        && !stroke.unwrap().is_empty()
        && stroke.unwrap() != "none"
        && get_element_stroke_width(elem) > 0.0;

    match (has_fill, has_stroke) {
        (true, true) => FillStyle::Duotone,
        (true, false) => FillStyle::Filled,
        (false, true) => FillStyle::Outline,
        (false, false) => FillStyle::None,
    }
}

/// Classify severity based on deviation percentage.
fn classify_severity(deviation_pct: f64) -> IssueSeverity {
    if deviation_pct > 30.0 {
        IssueSeverity::Error
    } else if deviation_pct > 15.0 {
        IssueSeverity::Warning
    } else {
        IssueSeverity::Info
    }
}

/// Check fill style consistency across all elements.
pub fn check_fill_style_consistency(project: &IconProject) -> (bool, Vec<ConsistencyIssue>) {
    let mut styles: Vec<(FillStyle, String)> = Vec::new();
    visit_all_elements(project.active_elements(), &mut |elem| {
        match elem {
            Element::Group(_) | Element::Symbol(_) | Element::Image(_) => return,
            _ => {}
        }
        let style = detect_fill_style(elem);
        styles.push((style, elem.id().to_string()));
    });

    if styles.len() < 2 {
        return (true, Vec::new());
    }

    // Find mode fill style
    let mut freq: HashMap<String, usize> = HashMap::new();
    for (style, _) in &styles {
        let key = format!("{:?}", style);
        *freq.entry(key).or_insert(0) += 1;
    }
    let mode_key = freq.iter().max_by_key(|(_, c)| **c).map(|(k, _)| k.clone());
    let mode_key = match mode_key {
        Some(k) => k,
        None => return (true, Vec::new()),
    };

    let mut issues: Vec<ConsistencyIssue> = Vec::new();
    let mut consistent = true;
    for (style, id) in &styles {
        let key = format!("{:?}", style);
        if key != mode_key {
            consistent = false;
            issues.push(ConsistencyIssue {
                property: "fill_style".to_string(),
                expected: mode_key.clone(),
                actual: key,
                element_id: id.clone(),
                severity: IssueSeverity::Warning,
            });
        }
    }
    (consistent, issues)
}

/// Check proportions consistency (element area / canvas area).
pub fn check_proportions_consistency(project: &IconProject) -> (bool, Vec<ConsistencyIssue>) {
    let canvas_w = project.active_canvas().width as f64;
    let canvas_h = project.active_canvas().height as f64;
    let canvas_area = canvas_w * canvas_h;
    if canvas_area <= 0.0 {
        return (true, Vec::new());
    }

    let mut proportions: Vec<(f64, String)> = Vec::new();
    visit_all_elements(project.active_elements(), &mut |elem| {
        let (w, h) = get_element_dimensions(elem);
        if w > 0.0 && h > 0.0 {
            let area = w * h;
            let ratio = area / canvas_area;
            proportions.push((ratio, elem.id().to_string()));
        }
    });

    if proportions.len() < 2 {
        return (true, Vec::new());
    }

    let raw: Vec<f64> = proportions.iter().map(|(v, _)| *v).collect();
    let (mode_val, _) = match find_mode(&raw) {
        Some(m) => m,
        None => return (true, Vec::new()),
    };

    if mode_val <= 0.0 {
        return (true, Vec::new());
    }

    let mut issues: Vec<ConsistencyIssue> = Vec::new();
    let mut consistent = true;
    for &(val, ref id) in &proportions {
        let deviation = ((val - mode_val) / mode_val).abs() * 100.0;
        if deviation > 20.0 {
            consistent = false;
            issues.push(ConsistencyIssue {
                property: "proportions".to_string(),
                expected: format!("{:.1}%", mode_val * 100.0),
                actual: format!("{:.1}%", val * 100.0),
                element_id: id.clone(),
                severity: classify_severity(deviation),
            });
        }
    }
    (consistent, issues)
}

/// Compute visual center drift: max distance of any element's weighted center
/// from the canvas center, normalized to [0, 1].
pub fn compute_visual_center_drift(project: &IconProject) -> Option<f64> {
    let canvas_w = project.active_canvas().width as f64;
    let canvas_h = project.active_canvas().height as f64;
    let cx = canvas_w / 2.0;
    let cy = canvas_h / 2.0;
    let diag = (canvas_w * canvas_w + canvas_h * canvas_h).sqrt();

    let mut centers: Vec<(f64, f64, f64)> = Vec::new(); // (center_x, center_y, area)
    visit_all_elements(project.active_elements(), &mut |elem| {
        let c = elem.common();
        let center_x = c.x + c.width / 2.0;
        let center_y = c.y + c.height / 2.0;
        let area = c.width * c.height;
        if area > 0.0 {
            centers.push((center_x, center_y, area));
        }
    });

    if centers.is_empty() {
        return None;
    }

    let total_area: f64 = centers.iter().map(|(_, _, a)| *a).sum();
    if total_area <= 0.0 {
        return None;
    }

    let mut max_drift = 0.0f64;
    for (x, y, _area) in &centers {
        let dx = x - cx;
        let dy = y - cy;
        let dist = (dx * dx + dy * dy).sqrt();
        let normalized = dist / (diag / 2.0);
        if normalized > max_drift {
            max_drift = normalized;
        }
    }

    Some(max_drift)
}

// ---------------------------------------------------------------------------
// Auto-fix
// ---------------------------------------------------------------------------

/// Recursively find and mutate an element by ID.
fn find_element_mut<'a>(elements: &'a mut [Element], id: &str) -> Option<&'a mut Element> {
    for elem in elements.iter_mut() {
        if elem.id() == id {
            return Some(elem);
        }
        if let Element::Group(g) = elem {
            if let Some(found) = find_element_mut(&mut g.children, id) {
                return Some(found);
            }
        }
    }
    None
}

/// Fix consistency issues for specified element IDs by setting deviated
/// properties to their mode values. Returns a modified deep copy of the project.
pub fn fix_consistency_issues(
    project: &IconProject,
    element_ids: &[String],
) -> Result<IconProject, String> {
    let mut fixed = project.clone();

    // Compute mode values from the original project
    let mut border_radii: Vec<f64> = Vec::new();
    let mut stroke_widths: Vec<f64> = Vec::new();
    let mut font_sizes: Vec<f64> = Vec::new();
    let mut opacities: Vec<f64> = Vec::new();

    visit_all_elements(project.active_elements(), &mut |elem| {
        if let Element::Shape(s) = elem {
            if matches!(s.shape_type, crate::model::shapes::ShapeType::Rect | crate::model::shapes::ShapeType::RoundedRect)
                && s.border_radius.abs() > f64::EPSILON
            {
                border_radii.push(s.border_radius);
            }
            if s.stroke_width > 0.0 {
                stroke_widths.push(s.stroke_width);
            }
        }
        if let Element::Text(t) = elem {
            font_sizes.push(t.font_size);
            if t.stroke_width > 0.0 {
                stroke_widths.push(t.stroke_width);
            }
        }
        if let Element::Icon(i) = elem {
            if i.stroke_width > 0.0 {
                stroke_widths.push(i.stroke_width);
            }
        }
        if let Element::Path(p) = elem {
            if p.stroke_width > 0.0 {
                stroke_widths.push(p.stroke_width);
            }
        }
        let c = elem.common();
        if c.opacity < 1.0 {
            opacities.push(c.opacity);
        }
    });

    let mode_border_radius = find_mode(&border_radii).map(|(v, _)| v);
    let mode_stroke_width = find_mode(&stroke_widths).map(|(v, _)| v);
    let mode_font_size = find_mode(&font_sizes).map(|(v, _)| v);
    let mode_opacity = find_mode(&opacities).map(|(v, _)| v);

    // Get the issues for reference
    let report = check_consistency(project);
    let issue_map: HashMap<String, Vec<&ConsistencyIssue>> = {
        let mut map: HashMap<String, Vec<&ConsistencyIssue>> = HashMap::new();
        for issue in &report.issues {
            map.entry(issue.element_id.clone()).or_default().push(issue);
        }
        map
    };

    let elements = fixed.active_elements_mut();
    for id in element_ids {
        let elem = match find_element_mut(elements, id) {
            Some(e) => e,
            None => continue,
        };

        let issues = match issue_map.get(id) {
            Some(v) => v,
            None => continue,
        };

        for issue in issues {
            match issue.property.as_str() {
                "border_radius" => {
                    if let Some(mode) = mode_border_radius {
                        if let Element::Shape(s) = elem {
                            s.border_radius = mode;
                        }
                    }
                }
                "stroke_width" => {
                    if let Some(mode) = mode_stroke_width {
                        match elem {
                            Element::Shape(s) => s.stroke_width = mode,
                            Element::Text(t) => t.stroke_width = mode,
                            Element::Icon(i) => i.stroke_width = mode,
                            Element::Path(p) => p.stroke_width = mode,
                            _ => {}
                        }
                    }
                }
                "font_size" => {
                    if let Some(mode) = mode_font_size {
                        if let Element::Text(t) = elem {
                            t.font_size = mode;
                        }
                    }
                }
                "opacity" => {
                    if let Some(mode) = mode_opacity {
                        elem.common_mut().opacity = mode;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(fixed)
}
