use icon_studio_lib::model::shapes::{shape_to_svg, ShapeType};

fn default_shape_args() -> (f64, f64, f64, f64) {
    (10.0, 20.0, 100.0, 80.0)
}

#[test]
fn test_circle_has_circle_tag() {
    let (x, y, w, h) = default_shape_args();
    let svg = shape_to_svg(&ShapeType::Circle, x, y, w, h, "#000", None, 0.0, 1.0, 0.0, 0.0, None);
    assert!(svg.contains("<circle"), "Expected <circle> tag, got: {}", svg);
}

#[test]
fn test_rect_has_rect_tag() {
    let (x, y, w, h) = default_shape_args();
    let svg = shape_to_svg(&ShapeType::Rect, x, y, w, h, "#000", None, 0.0, 1.0, 0.0, 0.0, None);
    assert!(svg.contains("<rect"), "Expected <rect> tag, got: {}", svg);
}

#[test]
fn test_rounded_rect_has_rx() {
    let (x, y, w, h) = default_shape_args();
    let svg = shape_to_svg(&ShapeType::RoundedRect, x, y, w, h, "#000", None, 0.0, 1.0, 0.0, 0.0, None);
    assert!(
        svg.contains("rx=") && svg.contains("ry="),
        "Expected rx= and ry= attributes, got: {}",
        svg
    );
}

#[test]
fn test_hexagon_has_six_points() {
    let (x, y, w, h) = default_shape_args();
    let svg = shape_to_svg(&ShapeType::Hexagon, x, y, w, h, "#000", None, 0.0, 1.0, 0.0, 0.0, None);
    assert!(svg.contains("<polygon"), "Expected <polygon> tag, got: {}", svg);

    let points = extract_points(&svg);
    let comma_count = points.chars().filter(|c| *c == ',').count();
    assert_eq!(comma_count, 6, "Expected 6 coordinate pairs (6 commas), got {} commas in points='{}'", comma_count, points);
}

#[test]
fn test_star_has_ten_points() {
    let (x, y, w, h) = default_shape_args();
    let svg = shape_to_svg(&ShapeType::Star, x, y, w, h, "#000", None, 0.0, 1.0, 0.0, 0.0, None);
    assert!(svg.contains("<polygon"), "Expected <polygon> tag, got: {}", svg);

    let points = extract_points(&svg);
    let comma_count = points.chars().filter(|c| *c == ',').count();
    assert_eq!(comma_count, 10, "Expected 10 coordinate pairs (10 commas), got {} commas in points='{}'", comma_count, points);
}

#[test]
fn test_shield_has_path_tag() {
    let (x, y, w, h) = default_shape_args();
    let svg = shape_to_svg(&ShapeType::Shield, x, y, w, h, "#000", None, 0.0, 1.0, 0.0, 0.0, None);
    assert!(svg.contains("<path"), "Expected <path> tag, got: {}", svg);
}

#[test]
fn test_diamond_has_four_points() {
    let (x, y, w, h) = default_shape_args();
    let svg = shape_to_svg(&ShapeType::Diamond, x, y, w, h, "#000", None, 0.0, 1.0, 0.0, 0.0, None);
    assert!(svg.contains("<polygon"), "Expected <polygon> tag, got: {}", svg);

    let points = extract_points(&svg);
    let comma_count = points.chars().filter(|c| *c == ',').count();
    assert_eq!(comma_count, 4, "Expected 4 coordinate pairs (4 commas), got {} commas in points='{}'", comma_count, points);
}

#[test]
fn test_fill_attribute() {
    let (x, y, w, h) = default_shape_args();
    let svg = shape_to_svg(&ShapeType::Rect, x, y, w, h, "#FF0000", None, 0.0, 1.0, 0.0, 0.0, None);
    assert!(
        svg.contains("fill=\"#FF0000\""),
        "Expected fill=\"#FF0000\", got: {}",
        svg
    );
}

#[test]
fn test_stroke_attribute() {
    let (x, y, w, h) = default_shape_args();
    let svg = shape_to_svg(&ShapeType::Rect, x, y, w, h, "#000", Some("#333"), 2.0, 1.0, 0.0, 0.0, None);
    assert!(
        svg.contains("stroke=\"#333\""),
        "Expected stroke=\"#333\", got: {}",
        svg
    );
    assert!(
        svg.contains("stroke-width=\"2.00\""),
        "Expected stroke-width=\"2.00\", got: {}",
        svg
    );
}

#[test]
fn test_rotation_transform() {
    let (x, y, w, h) = default_shape_args();
    let svg = shape_to_svg(&ShapeType::Rect, x, y, w, h, "#000", None, 0.0, 1.0, 45.0, 0.0, None);
    assert!(
        svg.contains("rotate(45"),
        "Expected rotate(45...) in transform, got: {}",
        svg
    );
}

/// Helper: extract the `points="..."` attribute value from a polygon SVG string.
fn extract_points(svg: &str) -> String {
    let start = svg.find("points=\"").expect("no points attribute found") + "points=\"".len();
    let end = svg[start..].find("\"").expect("unclosed points attribute") + start;
    svg[start..end].to_string()
}
