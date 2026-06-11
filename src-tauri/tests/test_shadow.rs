use icon_studio_lib::engine::builder;
use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::{CommonProps, Element, IconProject, Shadow, ShapeElement};

fn project_with_shadow(shadow: Shadow) -> IconProject {
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
            shadows: vec![shadow],
            animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        overlay: None,
        },
        shape_type: ShapeType::Rect,
        fill: "#336699".into(),
        stroke: None,
        stroke_width: 0.0,
        border_radius: 0.0,
        stroke_dasharray: None,
        gradient: None,
    }));
    project
}

#[test]
fn shadow_generates_correct_filter() {
    let project = project_with_shadow(Shadow::default());
    let svg = builder::build(&project).unwrap();

    assert!(
        svg.contains(r#"<filter id="shadow-shape-1-0">"#),
        "SVG should contain filter with id shadow-shape-1-0"
    );
    assert!(svg.contains("feDropShadow"), "SVG should contain feDropShadow element");
}

#[test]
fn element_references_correct_filter() {
    let project = project_with_shadow(Shadow::default());
    let svg = builder::build(&project).unwrap();

    assert!(
        svg.contains(r#"filter="url(#shadow-shape-1-0)""#),
        "Element should reference filter url(#shadow-shape-1-0)"
    );
}

#[test]
fn shadow_with_custom_params() {
    let project = project_with_shadow(Shadow {
        color: "#FF000080".into(),
        blur: 12.0,
        offset_x: 3.0,
        offset_y: 5.0,
        inset: false,
    });
    let svg = builder::build(&project).unwrap();

    assert!(
        svg.contains(r#"dx="3.00""#),
        "Shadow dx should be 3.00"
    );
    assert!(
        svg.contains(r#"dy="5.00""#),
        "Shadow dy should be 5.00"
    );
    assert!(
        svg.contains(r#"stdDeviation="12.00""#),
        "Shadow stdDeviation should be 12.00"
    );
    assert!(
        svg.contains("flood-color=\"#FF0000\""),
        "Shadow flood-color should be #FF0000 (alpha stripped)"
    );
    assert!(
        svg.contains("flood-opacity"),
        "Shadow should have flood-opacity when alpha is present"
    );
}

#[test]
fn multiple_elements_with_shadows_get_unique_filter_ids() {
    let mut project = IconProject::default();
    for i in 0..2 {
        project.elements.push(Element::Shape(ShapeElement {
            common: CommonProps {
                id: format!("shape-{}", i + 1),
                x: 50.0 * i as f64,
                y: 50.0,
                width: 100.0,
                height: 100.0,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![Shadow::default()],
                animation: None,
                blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        overlay: None,
        },
            shape_type: ShapeType::Circle,
            fill: "#000000".into(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        }));
    }
    let svg = builder::build(&project).unwrap();

    assert!(svg.contains(r#"<filter id="shadow-shape-1-0">"#), "Should have shadow-shape-1-0");
    assert!(svg.contains(r#"<filter id="shadow-shape-2-0">"#), "Should have shadow-shape-2-0");
    assert!(svg.contains(r#"url(#shadow-shape-1-0)"#), "First element references shadow-shape-1-0");
    assert!(svg.contains(r#"url(#shadow-shape-2-0)"#), "Second element references shadow-shape-2-0");
}
