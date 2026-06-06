use icon_studio_lib::engine::{analyzer, builder, renderer};
use icon_studio_lib::model::history::{AddElementCommand, CommandHistory};
use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::*;

#[test]
fn test_create_shape_gradient_export_png() {
    let mut project = IconProject::default();
    project.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id: "shape-1".into(),
            x: 156.0,
            y: 156.0,
            width: 200.0,
            height: 200.0,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
        shape_type: ShapeType::Circle,
        fill: "#FF5733".into(),
        stroke: None,
        stroke_width: 0.0,
        border_radius: 0.0,
        stroke_dasharray: None,
        gradient: Some(Gradient {
            gradient_type: GradientKind::Linear,
            colors: vec!["#FF5733".into(), "#33C1FF".into()],
            angle: 45.0,
            stops: vec![],
        }),
    }));

    let svg = builder::build(&project).expect("build");
    assert!(svg.contains("<linearGradient"), "SVG should contain gradient def");
    assert!(svg.contains("url(#gradient-shape-1)"), "Element should reference gradient URL");

    let png = renderer::render(&svg, 512).expect("render");
    assert!(!png.is_empty());
    assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);

    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("gradient_icon.png");
    std::fs::write(&out_path, &png).unwrap();
    assert!(out_path.exists());
    assert!(std::fs::metadata(&out_path).unwrap().len() > 0);
}

#[test]
fn test_create_text_modify_font_export_svg() {
    let mut project = IconProject::default();
    project.elements.push(Element::Text(TextElement {
        common: CommonProps {
            id: "text-1".into(),
            x: 100.0,
            y: 200.0,
            width: 300.0,
            height: 60.0,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
        content: "Hello".into(),
        fill: "#000000".into(),
        font_family: "Microsoft YaHei".into(),
        font_size: 24.0,
        font_weight: "normal".into(),
        letter_spacing: 0.0,
        stroke: None,
        stroke_width: 0.0,
        gradient: None,
    }));

    let svg_v1 = builder::build(&project).expect("build");
    assert!(svg_v1.contains("font-size=\"24.00\""));

    project.elements[0] = {
        let mut props = serde_json::to_value(&project.elements[0]).unwrap();
        if let Some(obj) = props.as_object_mut() {
            obj.insert("font_size".into(), serde_json::Value::Number(serde_json::Number::from_f64(48.0).unwrap()));
        }
        serde_json::from_value(props).unwrap()
    };

    let svg_v2 = builder::build(&project).expect("build");
    assert!(svg_v2.contains("font-size=\"48.00\""), "font-size should be updated to 48");

    let dir = tempfile::tempdir().unwrap();
    let svg_path = dir.path().join("text_icon.svg");
    std::fs::write(&svg_path, &svg_v2).unwrap();
    let content = std::fs::read_to_string(&svg_path).unwrap();
    assert!(content.contains("<svg"));
    assert!(content.contains("Hello"));
}

#[test]
fn test_undo_redo_full_cycle() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::new(50);

    let elem1 = Element::Shape(ShapeElement {
        common: CommonProps {
            id: "shape-1".into(),
            x: 0.0, y: 0.0, width: 100.0, height: 100.0,
            opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
        shape_type: ShapeType::Circle,
        fill: "#FF0000".into(), stroke: None, stroke_width: 0.0,
        border_radius: 0.0, stroke_dasharray: None,
        gradient: None,
    });
    let elem2 = Element::Shape(ShapeElement {
        common: CommonProps {
            id: "shape-2".into(),
            x: 100.0, y: 100.0, width: 80.0, height: 80.0,
            opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
        shape_type: ShapeType::Rect,
        fill: "#00FF00".into(), stroke: None, stroke_width: 0.0,
        border_radius: 0.0, stroke_dasharray: None,
        gradient: None,
    });

    history.push_and_execute(Box::new(AddElementCommand::new(elem1)), &mut project).unwrap();
    history.push_and_execute(Box::new(AddElementCommand::new(elem2)), &mut project).unwrap();
    assert_eq!(project.elements.len(), 2);

    history.undo(&mut project).unwrap();
    history.undo(&mut project).unwrap();
    assert_eq!(project.elements.len(), 0, "All elements should be undone");

    history.redo(&mut project).unwrap();
    history.redo(&mut project).unwrap();
    assert_eq!(project.elements.len(), 2, "All elements should be back after redo");

    if let Element::Shape(ref s) = project.elements[0] {
        assert!(matches!(s.shape_type, ShapeType::Circle));
    }
    if let Element::Shape(ref s) = project.elements[1] {
        assert!(matches!(s.shape_type, ShapeType::Rect));
    }
}

#[test]
fn test_batch_undo_scenario() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::new(50);

    history.begin_batch("add 3 elements").unwrap();

    for i in 0..3 {
        let id = project.alloc_element_id("shape");
        let elem = Element::Shape(ShapeElement {
            common: CommonProps {
                id,
                x: i as f64 * 100.0, y: 0.0, width: 80.0, height: 80.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
                blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            shape_type: ShapeType::Circle,
            fill: "#FF0000".into(), stroke: None, stroke_width: 0.0,
            border_radius: 0.0, stroke_dasharray: None,
            gradient: None,
        });
        history.push_and_execute(Box::new(AddElementCommand::new(elem)), &mut project).unwrap();
    }

    history.commit_batch().unwrap();
    assert_eq!(project.elements.len(), 3, "Should have 3 elements after batch");

    history.undo(&mut project).unwrap();
    assert_eq!(project.elements.len(), 0, "Single undo should remove all 3 batched elements");

    history.redo(&mut project).unwrap();
    assert_eq!(project.elements.len(), 3, "Redo should restore all 3 elements");
}

#[test]
fn test_analyze_colors_scenario() {
    let mut project = IconProject::default();
    project.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id: "shape-1".into(),
            x: 0.0, y: 0.0, width: 200.0, height: 200.0,
            opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
        shape_type: ShapeType::Circle,
        fill: "#FF0000".into(), stroke: None, stroke_width: 0.0,
        border_radius: 0.0, stroke_dasharray: None,
        gradient: None,
    }));
    project.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id: "shape-2".into(),
            x: 200.0, y: 200.0, width: 100.0, height: 100.0,
            opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
        shape_type: ShapeType::Rect,
        fill: "#00FF00".into(), stroke: None, stroke_width: 0.0,
        border_radius: 0.0, stroke_dasharray: None,
        gradient: None,
    }));
    project.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id: "shape-3".into(),
            x: 100.0, y: 100.0, width: 100.0, height: 100.0,
            opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
        shape_type: ShapeType::Star,
        fill: "#FF0000".into(), stroke: None, stroke_width: 0.0,
        border_radius: 0.0, stroke_dasharray: None,
        gradient: None,
    }));

    let analysis = analyzer::analyze_colors(&project);
    assert!(!analysis.all_colors.is_empty(), "Should find colors");
    assert!(
        analysis.primary.is_some(),
        "Should identify a primary color"
    );
    let primary = analysis.primary.as_ref().unwrap();
    assert!(
        primary.usage_count >= 2,
        "Red (#FF0000) used twice should be primary, got count {}",
        primary.usage_count
    );
}
