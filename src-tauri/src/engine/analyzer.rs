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
    pub issues: Vec<ConsistencyIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyIssue {
    pub property: String,
    pub expected: String,
    pub actual: String,
    pub element_id: String,
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

    ConsistencyReport {
        border_radius_consistent,
        stroke_width_consistent,
        font_size_consistent,
        opacity_consistent,
        issues,
    }
}

/// Check consistency of float values: mode is the "expected" value,
/// deviations > 10% are flagged as issues.
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
        // If mode is 0, use absolute tolerance for comparison
        for &(val, ref id) in values {
            if (val - mode_val).abs() > 0.01 {
                issues.push(ConsistencyIssue {
                    property: property.to_string(),
                    expected: format!("{:.2}", mode_val),
                    actual: format!("{:.2}", val),
                    element_id: id.clone(),
                });
            }
        }
    } else {
        for &(val, ref id) in values {
            let deviation = ((val - mode_val) / mode_val).abs();
            if deviation > 0.10 {
                issues.push(ConsistencyIssue {
                    property: property.to_string(),
                    expected: format!("{:.2}", mode_val),
                    actual: format!("{:.2}", val),
                    element_id: id.clone(),
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
