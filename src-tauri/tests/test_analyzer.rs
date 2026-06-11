mod common;

use icon_studio_lib::engine::analyzer::{self, ElementFilter};
use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::*;

fn make_shape(id: &str, shape_type: ShapeType, fill: &str, stroke_width: f64, opacity: f64, width: f64) -> Element {
    common::shape_el(common::make_shape_detailed(id, shape_type, fill, stroke_width, opacity, width))
}

fn make_text(id: &str, content: &str, fill: &str, font_size: f64, opacity: f64, width: f64) -> Element {
    common::text_el(common::make_text_detailed(id, content, fill, font_size, opacity, width))
}

fn empty_project() -> IconProject {
    common::make_default_project()
}

// ---------------------------------------------------------------------------
// analyze_colors tests
// ---------------------------------------------------------------------------

#[test]
fn test_analyze_colors_multi_color() {
    let mut project = empty_project();
    project.elements.push(make_shape("s1", ShapeType::Circle, "#FF5733", 0.0, 1.0, 100.0));
    project.elements.push(make_shape("s2", ShapeType::Rect, "#FF5733", 0.0, 1.0, 80.0));
    project.elements.push(make_shape("s3", ShapeType::Circle, "#3366FF", 0.0, 1.0, 60.0));
    project.elements.push(make_text("t1", "Hi", "#FFFFFF", 24.0, 1.0, 50.0));

    let result = analyzer::analyze_colors(&project);

    assert!(result.primary.is_some());
    let primary = result.primary.unwrap();
    assert_eq!(primary.hex, "#FF5733");
    assert_eq!(primary.usage_count, 2);

    assert!(result.all_colors.len() >= 3);
}

#[test]
fn test_analyze_colors_empty_project() {
    let project = empty_project();
    let result = analyzer::analyze_colors(&project);

    assert!(result.primary.is_none());
    assert!(result.all_colors.is_empty());
    assert!(result.secondary.is_empty());
    assert!(result.accent.is_empty());
}

#[test]
fn test_analyze_colors_case_insensitive() {
    let mut project = empty_project();
    project.elements.push(make_shape("s1", ShapeType::Circle, "#ff5733", 0.0, 1.0, 100.0));
    project.elements.push(make_shape("s2", ShapeType::Rect, "#FF5733", 0.0, 1.0, 80.0));

    let result = analyzer::analyze_colors(&project);

    assert_eq!(result.all_colors.len(), 1);
    assert_eq!(result.all_colors[0].usage_count, 2);
}

// ---------------------------------------------------------------------------
// check_consistency tests
// ---------------------------------------------------------------------------

#[test]
fn test_check_consistency_inconsistent_stroke() {
    let mut project = empty_project();
    project.elements.push(make_shape("s1", ShapeType::Circle, "#FF5733", 2.0, 1.0, 100.0));
    project.elements.push(make_shape("s2", ShapeType::Rect, "#3366FF", 2.0, 1.0, 80.0));
    project.elements.push(make_shape("s3", ShapeType::Circle, "#00FF00", 5.0, 1.0, 60.0));

    let report = analyzer::check_consistency(&project);

    assert!(!report.stroke_width_consistent);
    assert!(report.issues.iter().any(|i| i.property == "stroke_width"));
}

#[test]
fn test_check_consistency_inconsistent_font_size() {
    let mut project = empty_project();
    project.elements.push(make_text("t1", "Hi", "#000", 24.0, 1.0, 100.0));
    project.elements.push(make_text("t2", "Lo", "#000", 24.0, 1.0, 80.0));
    project.elements.push(make_text("t3", "Bye", "#000", 48.0, 1.0, 60.0));

    let report = analyzer::check_consistency(&project);

    assert!(!report.font_size_consistent);
    assert!(report.issues.iter().any(|i| i.property == "font_size"));
}

#[test]
fn test_check_consistency_all_consistent() {
    let mut project = empty_project();
    project.elements.push(make_shape("s1", ShapeType::Circle, "#FF5733", 2.0, 1.0, 100.0));
    project.elements.push(make_shape("s2", ShapeType::Rect, "#3366FF", 2.0, 1.0, 80.0));
    project.elements.push(make_text("t1", "Hi", "#000", 24.0, 1.0, 50.0));

    let report = analyzer::check_consistency(&project);

    assert!(report.stroke_width_consistent);
    assert!(report.font_size_consistent);
    assert!(report.opacity_consistent);
    assert!(report.border_radius_consistent);
    assert!(report.fill_style_consistent);
    assert!(report.proportions_consistent);
    assert!(report.stroke_weight_consistent);
}

#[test]
fn test_check_consistency_empty_project() {
    let project = empty_project();
    let report = analyzer::check_consistency(&project);

    assert!(report.border_radius_consistent);
    assert!(report.stroke_width_consistent);
    assert!(report.font_size_consistent);
    assert!(report.opacity_consistent);
    assert!(report.issues.is_empty());
}

// ---------------------------------------------------------------------------
// detect_fill_style tests
// ---------------------------------------------------------------------------

#[test]
fn test_detect_fill_style_outline() {
    let mut s = common::make_shape("s1", ShapeType::Circle);
    s.fill = "none".to_string();
    s.stroke = Some("#000000".to_string());
    s.stroke_width = 2.0;
    let style = analyzer::detect_fill_style(&Element::Shape(s));
    assert!(matches!(style, analyzer::FillStyle::Outline));
}

#[test]
fn test_detect_fill_style_filled() {
    let mut s = common::make_shape("s1", ShapeType::Circle);
    s.fill = "#FF0000".to_string();
    s.stroke = None;
    s.stroke_width = 0.0;
    let style = analyzer::detect_fill_style(&Element::Shape(s));
    assert!(matches!(style, analyzer::FillStyle::Filled));
}

#[test]
fn test_detect_fill_style_duotone() {
    let mut s = common::make_shape("s1", ShapeType::Circle);
    s.fill = "#FF0000".to_string();
    s.stroke = Some("#000000".to_string());
    s.stroke_width = 2.0;
    let style = analyzer::detect_fill_style(&Element::Shape(s));
    assert!(matches!(style, analyzer::FillStyle::Duotone));
}

#[test]
fn test_detect_fill_style_none() {
    let mut s = common::make_shape("s1", ShapeType::Circle);
    s.fill = "none".to_string();
    s.stroke = None;
    s.stroke_width = 0.0;
    let style = analyzer::detect_fill_style(&Element::Shape(s));
    assert!(matches!(style, analyzer::FillStyle::None));
}

#[test]
fn test_detect_fill_style_group_is_none() {
    let g = GroupElement {
        common: CommonProps::new("g1".to_string(), 0.0, 0.0, 100.0, 100.0),
        children: vec![],
        expanded: false,
    };
    let style = analyzer::detect_fill_style(&Element::Group(g));
    assert!(matches!(style, analyzer::FillStyle::None));
}

// ---------------------------------------------------------------------------
// check_fill_style_consistency tests
// ---------------------------------------------------------------------------

#[test]
fn test_fill_style_consistency_mixed() {
    let mut project = empty_project();
    // Two filled shapes + one outline shape
    let mut s1 = common::make_shape("s1", ShapeType::Circle);
    s1.fill = "#FF0000".to_string();
    s1.stroke = None;
    s1.stroke_width = 0.0;

    let mut s2 = common::make_shape("s2", ShapeType::Circle);
    s2.fill = "#00FF00".to_string();
    s2.stroke = None;
    s2.stroke_width = 0.0;

    let mut s3 = common::make_shape("s3", ShapeType::Circle);
    s3.fill = "none".to_string();
    s3.stroke = Some("#000000".to_string());
    s3.stroke_width = 2.0;

    project.elements.push(Element::Shape(s1));
    project.elements.push(Element::Shape(s2));
    project.elements.push(Element::Shape(s3));

    let (consistent, issues) = analyzer::check_fill_style_consistency(&project);
    assert!(!consistent);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].property, "fill_style");
}

#[test]
fn test_fill_style_consistency_all_same() {
    let mut project = empty_project();
    let mut s1 = common::make_shape("s1", ShapeType::Circle);
    s1.fill = "#FF0000".to_string();
    let mut s2 = common::make_shape("s2", ShapeType::Circle);
    s2.fill = "#00FF00".to_string();

    project.elements.push(Element::Shape(s1));
    project.elements.push(Element::Shape(s2));

    let (consistent, issues) = analyzer::check_fill_style_consistency(&project);
    assert!(consistent);
    assert!(issues.is_empty());
}

// ---------------------------------------------------------------------------
// check_proportions_consistency tests
// ---------------------------------------------------------------------------

#[test]
fn test_proportions_consistency_major_outlier() {
    let mut project = empty_project();
    // Canvas is 512x512 = 262144 area
    // Normal elements: ~100x100 = 10000 area (~3.8% of canvas)
    project.elements.push(make_shape("s1", ShapeType::Circle, "#FF0000", 0.0, 1.0, 100.0));
    project.elements.push(make_shape("s2", ShapeType::Circle, "#00FF00", 0.0, 1.0, 100.0));
    // Outlier: 400x400 = 160000 area (~61% of canvas) — huge deviation
    project.elements.push(make_shape("s3", ShapeType::Circle, "#0000FF", 0.0, 1.0, 400.0));

    let (consistent, issues) = analyzer::check_proportions_consistency(&project);
    assert!(!consistent);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|i| i.element_id == "s3"));
}

#[test]
fn test_proportions_consistency_all_similar() {
    let mut project = empty_project();
    project.elements.push(make_shape("s1", ShapeType::Circle, "#FF0000", 0.0, 1.0, 100.0));
    project.elements.push(make_shape("s2", ShapeType::Circle, "#00FF00", 0.0, 1.0, 105.0));

    let (consistent, issues) = analyzer::check_proportions_consistency(&project);
    assert!(consistent);
    assert!(issues.is_empty());
}

// ---------------------------------------------------------------------------
// compute_visual_center_drift tests
// ---------------------------------------------------------------------------

#[test]
fn test_visual_center_drift_centered() {
    let mut project = empty_project();
    // Canvas 512x512, center at (256, 256). Element at center.
    let mut s = common::make_shape_sized("s1", ShapeType::Circle, 206.0, 206.0, 100.0, 100.0);
    project.elements.push(Element::Shape(s));

    let drift = analyzer::compute_visual_center_drift(&project);
    assert!(drift.is_some());
    // Center of element is at (256, 256) — perfectly centered
    assert!(drift.unwrap() < 0.01);
}

#[test]
fn test_visual_center_drift_off_center() {
    let mut project = empty_project();
    let mut s = common::make_shape_sized("s1", ShapeType::Circle, 0.0, 0.0, 100.0, 100.0);
    project.elements.push(Element::Shape(s));

    let drift = analyzer::compute_visual_center_drift(&project);
    assert!(drift.is_some());
    assert!(drift.unwrap() > 0.1);
}

#[test]
fn test_visual_center_drift_empty() {
    let project = empty_project();
    let drift = analyzer::compute_visual_center_drift(&project);
    assert!(drift.is_none());
}

// ---------------------------------------------------------------------------
// severity classification tests
// ---------------------------------------------------------------------------

#[test]
fn test_severity_info_for_small_deviation() {
    let mut project = empty_project();
    // Stroke width 2.0 vs 2.1 — 5% deviation, should be Info severity
    let mut s1 = common::make_shape("s1", ShapeType::Circle);
    s1.stroke = Some("#000".to_string());
    s1.stroke_width = 2.0;

    let mut s2 = common::make_shape("s2", ShapeType::Circle);
    s2.stroke = Some("#000".to_string());
    s2.stroke_width = 2.0;

    let mut s3 = common::make_shape("s3", ShapeType::Circle);
    s3.stroke = Some("#000".to_string());
    s3.stroke_width = 2.3; // 15% deviation — Warning

    project.elements.push(Element::Shape(s1));
    project.elements.push(Element::Shape(s2));
    project.elements.push(Element::Shape(s3));

    let report = analyzer::check_consistency(&project);
    let stroke_issues: Vec<_> = report.issues.iter().filter(|i| i.property == "stroke_width").collect();
    assert!(!stroke_issues.is_empty());
    // 2.3 vs 2.0 = 15% deviation — should be Warning
    assert!(matches!(stroke_issues[0].severity, analyzer::IssueSeverity::Warning));
}

#[test]
fn test_severity_error_for_large_deviation() {
    let mut project = empty_project();
    let mut s1 = common::make_shape("s1", ShapeType::Circle);
    s1.stroke = Some("#000".to_string());
    s1.stroke_width = 2.0;

    let mut s2 = common::make_shape("s2", ShapeType::Circle);
    s2.stroke = Some("#000".to_string());
    s2.stroke_width = 2.0;

    let mut s3 = common::make_shape("s3", ShapeType::Circle);
    s3.stroke = Some("#000".to_string());
    s3.stroke_width = 5.0; // 150% deviation — Error

    project.elements.push(Element::Shape(s1));
    project.elements.push(Element::Shape(s2));
    project.elements.push(Element::Shape(s3));

    let report = analyzer::check_consistency(&project);
    let stroke_issues: Vec<_> = report.issues.iter().filter(|i| i.property == "stroke_width").collect();
    assert!(!stroke_issues.is_empty());
    assert!(matches!(stroke_issues[0].severity, analyzer::IssueSeverity::Error));
}

// ---------------------------------------------------------------------------
// fix_consistency_issues tests
// ---------------------------------------------------------------------------

#[test]
fn test_fix_stroke_width_to_mode() {
    let mut project = empty_project();
    let mut s1 = common::make_shape("s1", ShapeType::Circle);
    s1.stroke = Some("#000".to_string());
    s1.stroke_width = 2.0;

    let mut s2 = common::make_shape("s2", ShapeType::Circle);
    s2.stroke = Some("#000".to_string());
    s2.stroke_width = 2.0;

    let mut s3 = common::make_shape("s3", ShapeType::Circle);
    s3.stroke = Some("#000".to_string());
    s3.stroke_width = 5.0;

    project.elements.push(Element::Shape(s1));
    project.elements.push(Element::Shape(s2));
    project.elements.push(Element::Shape(s3));

    let fixed = analyzer::fix_consistency_issues(&project, &["s3".to_string()]).unwrap();

    // After fix, s3's stroke_width should be 2.0 (the mode)
    if let Element::Shape(s) = &fixed.elements[2] {
        assert_eq!(s.stroke_width, 2.0);
    } else {
        panic!("Expected shape");
    }

    // s1 and s2 should be unchanged
    if let Element::Shape(s) = &fixed.elements[0] {
        assert_eq!(s.stroke_width, 2.0);
    }
    if let Element::Shape(s) = &fixed.elements[1] {
        assert_eq!(s.stroke_width, 2.0);
    }
}

#[test]
fn test_fix_font_size_to_mode() {
    let mut project = empty_project();
    project.elements.push(make_text("t1", "Hi", "#000", 24.0, 1.0, 100.0));
    project.elements.push(make_text("t2", "Lo", "#000", 24.0, 1.0, 80.0));
    project.elements.push(make_text("t3", "Bye", "#000", 48.0, 1.0, 60.0));

    let fixed = analyzer::fix_consistency_issues(&project, &["t3".to_string()]).unwrap();

    if let Element::Text(t) = &fixed.elements[2] {
        assert_eq!(t.font_size, 24.0);
    } else {
        panic!("Expected text");
    }
}

#[test]
fn test_fix_nothing_when_no_issues() {
    let mut project = empty_project();
    project.elements.push(make_shape("s1", ShapeType::Circle, "#FF5733", 2.0, 1.0, 100.0));
    project.elements.push(make_shape("s2", ShapeType::Rect, "#3366FF", 2.0, 1.0, 80.0));

    let fixed = analyzer::fix_consistency_issues(&project, &["s1".to_string()]).unwrap();

    if let Element::Shape(s) = &fixed.elements[0] {
        assert_eq!(s.stroke_width, 2.0);
    }
}

// ---------------------------------------------------------------------------
// check_consistency extended tests (new fields)
// ---------------------------------------------------------------------------

#[test]
fn test_check_consistency_includes_new_fields() {
    let project = empty_project();
    let report = analyzer::check_consistency(&project);

    assert!(report.stroke_weight_consistent);
    assert!(report.fill_style_consistent);
    assert!(report.proportions_consistent);
    assert!(report.visual_center_drift.is_none());
}

#[test]
fn test_consistency_report_has_severity() {
    let mut project = empty_project();
    project.elements.push(make_shape("s1", ShapeType::Circle, "#FF5733", 2.0, 1.0, 100.0));
    project.elements.push(make_shape("s2", ShapeType::Circle, "#FF5733", 2.0, 1.0, 100.0));
    project.elements.push(make_shape("s3", ShapeType::Circle, "#FF5733", 5.0, 1.0, 100.0));

    let report = analyzer::check_consistency(&project);
    let stroke_issue = report.issues.iter().find(|i| i.property == "stroke_width");
    assert!(stroke_issue.is_some());
    let issue = stroke_issue.unwrap();
    // 5.0 vs 2.0 = 150% deviation → Error
    assert!(matches!(issue.severity, analyzer::IssueSeverity::Error));
}

// ---------------------------------------------------------------------------
// find_elements tests
// ---------------------------------------------------------------------------

#[test]
fn test_find_elements_by_type() {
    let mut project = empty_project();
    project.elements.push(make_shape("s1", ShapeType::Circle, "#FF5733", 0.0, 1.0, 100.0));
    project.elements.push(make_shape("s2", ShapeType::Rect, "#3366FF", 0.0, 1.0, 80.0));
    project.elements.push(make_text("t1", "Hi", "#000", 24.0, 1.0, 50.0));

    let filter = ElementFilter {
        element_type: Some("shape".to_string()),
        fill: None,
        min_width: None,
        max_width: None,
    };
    let result = analyzer::find_elements(&project, &filter);

    assert_eq!(result.count, 2);
    assert!(result.matching_ids.contains(&"s1".to_string()));
    assert!(result.matching_ids.contains(&"s2".to_string()));
}

#[test]
fn test_find_elements_by_fill() {
    let mut project = empty_project();
    project.elements.push(make_shape("s1", ShapeType::Circle, "#FF5733", 0.0, 1.0, 100.0));
    project.elements.push(make_shape("s2", ShapeType::Rect, "#3366FF", 0.0, 1.0, 80.0));
    project.elements.push(make_shape("s3", ShapeType::Circle, "#ff5733", 0.0, 1.0, 60.0));

    let filter = ElementFilter {
        element_type: None,
        fill: Some("#ff5733".to_string()),
        min_width: None,
        max_width: None,
    };
    let result = analyzer::find_elements(&project, &filter);

    assert_eq!(result.count, 2);
    assert!(result.matching_ids.contains(&"s1".to_string()));
    assert!(result.matching_ids.contains(&"s3".to_string()));
}

#[test]
fn test_find_elements_by_width_range() {
    let mut project = empty_project();
    project.elements.push(make_shape("s1", ShapeType::Circle, "#FF5733", 0.0, 1.0, 100.0));
    project.elements.push(make_shape("s2", ShapeType::Rect, "#3366FF", 0.0, 1.0, 50.0));
    project.elements.push(make_shape("s3", ShapeType::Circle, "#00FF00", 0.0, 1.0, 20.0));

    let filter = ElementFilter {
        element_type: None,
        fill: None,
        min_width: Some(40.0),
        max_width: Some(80.0),
    };
    let result = analyzer::find_elements(&project, &filter);

    assert_eq!(result.count, 1);
    assert!(result.matching_ids.contains(&"s2".to_string()));
}

#[test]
fn test_find_elements_empty_project() {
    let project = empty_project();
    let filter = ElementFilter {
        element_type: Some("shape".to_string()),
        fill: None,
        min_width: None,
        max_width: None,
    };
    let result = analyzer::find_elements(&project, &filter);
    assert_eq!(result.count, 0);
}