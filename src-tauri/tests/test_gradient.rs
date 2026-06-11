use icon_studio_lib::engine::builder;
use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::{CommonProps, Element, Gradient, GradientKind, IconProject, ShapeElement};

/// Helper: create a project with one shape that has a gradient.
fn project_with_gradient(kind: GradientKind, colors: Vec<String>, angle: f64, stops: Vec<f64>) -> IconProject {
    let mut project = IconProject::default();
    project.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id: "shape-1".into(),
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 200.0,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        overlay: None,
        },
        shape_type: ShapeType::Circle,
        fill: "#FF0000".into(),
        stroke: None,
        stroke_width: 0.0,
        border_radius: 0.0,
        stroke_dasharray: None,
        gradient: Some(Gradient {
            gradient_type: kind,
            colors,
            angle,
            stops,
        }),
    }));
    project
}

/// Helper: create a project with N shapes, each with a linear gradient.
fn project_with_n_gradients(n: usize) -> IconProject {
    let mut project = IconProject::default();
    for i in 0..n {
        project.elements.push(Element::Shape(ShapeElement {
            common: CommonProps {
                id: format!("shape-{}", i + 1),
                x: 50.0 * i as f64,
                y: 50.0,
                width: 100.0,
                height: 100.0,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
                blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        overlay: None,
        },
            shape_type: ShapeType::Rect,
            fill: "#000000".into(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: Some(Gradient {
                gradient_type: GradientKind::Linear,
                colors: vec!["#FF0000".into(), "#0000FF".into()],
                angle: 90.0,
                stops: vec![],
            }),
        }));
    }
    project
}

#[test]
fn linear_gradient_generates_correct_svg() {
    let project = project_with_gradient(
        GradientKind::Linear,
        vec!["#FF0000".into(), "#0000FF".into()],
        90.0,
        vec![],
    );
    let svg = builder::build(&project).unwrap();

    assert!(svg.contains("<linearGradient"), "SVG should contain <linearGradient>");
    assert!(svg.contains("gradient-shape-1"), "SVG should contain gradient-shape-1 id");
    // Should have at least 2 stops for 2 colors
    assert!(
        svg.matches("<stop").count() >= 2,
        "SVG should contain at least 2 <stop> elements"
    );
    assert!(
        svg.contains(r#"fill="url(#gradient-shape-1)""#),
        "Element fill should reference url(#gradient-shape-1)"
    );
}

#[test]
fn radial_gradient_generates_correct_svg() {
    let project = project_with_gradient(
        GradientKind::Radial,
        vec!["#00FF00".into(), "#000000".into()],
        0.0,
        vec![],
    );
    let svg = builder::build(&project).unwrap();

    assert!(svg.contains("<radialGradient"), "SVG should contain <radialGradient>");
    assert!(svg.contains("gradient-shape-1"), "SVG should contain gradient-shape-1 id");
    assert!(
        svg.matches("<stop").count() >= 2,
        "SVG should contain at least 2 <stop> elements"
    );
}

#[test]
fn element_fill_references_correct_gradient_id() {
    let project = project_with_gradient(
        GradientKind::Linear,
        vec!["#AABBCC".into(), "#DDEEFF".into()],
        45.0,
        vec![],
    );
    let svg = builder::build(&project).unwrap();

    // The element should reference gradient-shape-1
    assert!(
        svg.contains(r#"fill="url(#gradient-shape-1)""#),
        "Element should have fill='url(#gradient-shape-1)'"
    );
}

#[test]
fn multiple_elements_with_gradients_get_unique_ids() {
    let project = project_with_n_gradients(2);
    let svg = builder::build(&project).unwrap();

    assert!(svg.contains("gradient-shape-1"), "SVG should contain gradient-shape-1");
    assert!(svg.contains("gradient-shape-2"), "SVG should contain gradient-shape-2");
    assert!(
        svg.contains(r#"fill="url(#gradient-shape-1)""#),
        "First element should reference gradient-shape-1"
    );
    assert!(
        svg.contains(r#"fill="url(#gradient-shape-2)""#),
        "Second element should reference gradient-shape-2"
    );
}

#[test]
fn gradient_with_custom_stop_positions() {
    let project = project_with_gradient(
        GradientKind::Linear,
        vec!["#FF0000".into(), "#00FF00".into(), "#0000FF".into()],
        180.0,
        vec![0.0, 0.5, 1.0],
    );
    let svg = builder::build(&project).unwrap();

    // Check stop offsets: 0% → 0.0000%, 50% → 50.0000%, 100% → 100.0000%
    assert!(
        svg.contains(r#"offset="0.0000%""#),
        "First stop should have offset 0%"
    );
    assert!(
        svg.contains(r#"offset="50.0000%""#),
        "Second stop should have offset 50%"
    );
    assert!(
        svg.contains(r#"offset="100.0000%""#),
        "Third stop should have offset 100%"
    );
    // Verify colors
    assert!(svg.contains("stop-color=\"#FF0000\""), "First stop color");
    assert!(svg.contains("stop-color=\"#00FF00\""), "Second stop color");
    assert!(svg.contains("stop-color=\"#0000FF\""), "Third stop color");
}
