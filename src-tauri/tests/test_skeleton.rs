#[test]
fn test_model_serialize() {
    let canvas = icon_studio_lib::model::Canvas::default();
    assert_eq!(canvas.width, 512);
    assert_eq!(canvas.height, 512);

    let project = icon_studio_lib::model::IconProject::default();
    let json = serde_json::to_string(&project).expect("serialize");
    let back: icon_studio_lib::model::IconProject =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.canvas.width, 512);
    assert_eq!(back.elements.len(), 0);
}

#[test]
fn test_builder_returns_svg() {
    let project = icon_studio_lib::model::IconProject::default();
    let svg = icon_studio_lib::engine::builder::build(&project).expect("build");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.contains("#FFFFFF"));
}

#[test]
fn test_shape_enum() {
    use icon_studio_lib::model::shapes::ShapeType;

    let shapes = vec![
        ShapeType::Circle,
        ShapeType::Rect,
        ShapeType::RoundedRect,
        ShapeType::Hexagon,
        ShapeType::Star,
        ShapeType::Shield,
        ShapeType::Diamond,
    ];

    for shape in &shapes {
        let json = serde_json::to_string(shape).expect("serialize shape");
        let back: ShapeType = serde_json::from_str(&json).expect("deserialize shape");
        assert_eq!(format!("{:?}", shape), format!("{:?}", back));
    }
}
