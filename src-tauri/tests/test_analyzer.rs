mod common;

use icon_studio_lib::engine::analyzer::{self, ElementFilter};
use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::Element;

fn make_shape(id: &str, shape_type: ShapeType, fill: &str, stroke_width: f64, opacity: f64, width: f64) -> Element {
    common::shape_el(common::make_shape_detailed(id, shape_type, fill, stroke_width, opacity, width))
}

fn make_text(id: &str, content: &str, fill: &str, font_size: f64, opacity: f64, width: f64) -> Element {
    common::text_el(common::make_text_detailed(id, content, fill, font_size, opacity, width))
}

fn empty_project() -> icon_studio_lib::model::IconProject {
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

    // All colors should include all distinct colors
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
    project.elements.push(make_shape("s3", ShapeType::Circle, "#00FF00", 5.0, 1.0, 60.0)); // inconsistent stroke

    let report = analyzer::check_consistency(&project);

    assert!(!report.stroke_width_consistent);
    assert!(report.issues.iter().any(|i| i.property == "stroke_width"));
}

#[test]
fn test_check_consistency_inconsistent_font_size() {
    let mut project = empty_project();
    project.elements.push(make_text("t1", "Hi", "#000", 24.0, 1.0, 100.0));
    project.elements.push(make_text("t2", "Lo", "#000", 24.0, 1.0, 80.0));
    project.elements.push(make_text("t3", "Bye", "#000", 48.0, 1.0, 60.0)); // inconsistent

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
    assert!(report.issues.is_empty());
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
    project.elements.push(make_shape("s3", ShapeType::Circle, "#ff5733", 0.0, 1.0, 60.0)); // same as s1, diff case

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