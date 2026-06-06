mod common;

use icon_studio_lib::engine::builder;
use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::*;

#[test]
fn test_empty_project_generates_svg() {
    let project = common::make_default_project();
    let svg = builder::build(&project).expect("build");
    assert!(svg.contains("<svg"), "SVG should contain <svg tag");
    assert!(svg.contains("</svg>"), "SVG should contain closing </svg>");
}

#[test]
fn test_empty_project_has_background() {
    let project = common::make_default_project();
    let svg = builder::build(&project).expect("build");
    assert!(
        svg.contains("fill=\"#FFFFFF\""),
        "Expected background rect with fill=\"#FFFFFF\", got: {}",
        svg
    );
}

#[test]
fn test_single_circle() {
    let mut project = common::make_default_project();
    let mut s = common::make_shape("shape-1", ShapeType::Circle);
    s.common.x = 50.0;
    s.common.y = 50.0;
    project.elements.push(common::shape_el(s));

    let svg = builder::build(&project).expect("build");
    assert!(svg.contains("<circle"), "Expected <circle> tag, got: {}", svg);
}

#[test]
fn test_text_element() {
    let mut project = common::make_default_project();
    let mut t = common::make_text("text-1", "Hello");
    t.common.x = 100.0;
    t.common.y = 200.0;
    t.common.width = 300.0;
    t.common.height = 60.0;
    t.font_family = "Microsoft YaHei".to_string();
    t.font_size = 32.0;
    t.font_weight = "bold".to_string();
    project.elements.push(common::text_el(t));

    let svg = builder::build(&project).expect("build");
    assert!(svg.contains("<text"), "Expected <text> tag, got: {}", svg);
    assert!(svg.contains("Hello"), "Expected text content 'Hello', got: {}", svg);
}

#[test]
fn test_icon_element() {
    let mut project = common::make_default_project();
    let mut i = common::make_icon("icon-1", "heart");
    i.common.x = 100.0;
    i.common.y = 100.0;
    i.common.width = 200.0;
    i.common.height = 200.0;
    i.fill = "#FF0000".to_string();
    project.elements.push(common::icon_el(i));

    let svg = builder::build(&project).expect("build");
    assert!(svg.contains("<path"), "Expected <path> tag from heart icon, got: {}", svg);
}

#[test]
fn test_multiple_elements() {
    let mut project = common::make_default_project();
    let mut s = common::make_shape("shape-1", ShapeType::Rect);
    s.common.x = 0.0;
    s.common.y = 0.0;
    s.common.width = 512.0;
    s.common.height = 512.0;
    s.fill = "#00FF00".to_string();
    project.elements.push(common::shape_el(s));

    let mut t = common::make_text("text-1", "Icon");
    t.common.x = 150.0;
    t.common.y = 200.0;
    t.common.width = 200.0;
    t.common.height = 60.0;
    t.fill = "#000".to_string();
    t.font_family = "Microsoft YaHei".to_string();
    t.font_size = 48.0;
    t.font_weight = "normal".to_string();
    project.elements.push(common::text_el(t));

    let svg = builder::build(&project).expect("build");
    assert!(svg.contains("<rect"), "Expected <rect> tag, got: {}", svg);
    assert!(svg.contains("<text"), "Expected <text> tag, got: {}", svg);
}

#[test]
fn test_image_element() {
    let mut project = common::make_default_project();
    let red_png_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==";
    let img = common::make_image("image-1", &format!("data:image/png;base64,{}", red_png_b64));
    project.elements.push(common::image_el(img));

    let svg = builder::build(&project).expect("build");
    assert!(svg.contains("<image"), "Expected <image> tag, got: {}", svg);
    assert!(
        svg.contains("data:image/png;base64,"),
        "Expected base64 data URI in image tag, got: {}",
        svg
    );
}

#[test]
fn test_corner_radius_clip() {
    let mut project = common::make_default_project();
    project.canvas.corner_radius = 20;

    let svg = builder::build(&project).expect("build");
    assert!(
        svg.contains("<clipPath"),
        "Expected <clipPath> when corner_radius > 0, got: {}",
        svg
    );
    assert!(
        svg.contains("clip-path=\"url(#app-icon-clip)\""),
        "Expected clip-path reference, got: {}",
        svg
    );
}