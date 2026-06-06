use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::{CommonProps, Element, IconProject, ShapeElement};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_save_and_load_project() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.iconproject.json");

    let mut original = IconProject::default();
    original.canvas.width = 256;
    original.canvas.height = 256;
    original.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id: "shape-1".to_string(),
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 100.0,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
        shape_type: ShapeType::Circle,
        fill: "#FF0000".to_string(),
        stroke: None,
        stroke_width: 0.0,
        border_radius: 0.0,
        stroke_dasharray: None,
        gradient: None,
    }));

    let json = serde_json::to_string_pretty(&original).unwrap();
    fs::write(&path, &json).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    let loaded: IconProject = serde_json::from_str(&content).unwrap();

    assert_eq!(loaded.canvas.width, 256);
    assert_eq!(loaded.canvas.height, 256);
    assert_eq!(loaded.elements.len(), 1);
    match &loaded.elements[0] {
        Element::Shape(s) => {
            assert!(matches!(s.shape_type, ShapeType::Circle));
            assert_eq!(s.fill, "#FF0000");
        }
        _ => panic!("Expected Shape element"),
    }
}

#[test]
fn test_template_save_and_load() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("my-template.iconstudio-template.json");

    let project = IconProject::default();
    let json = serde_json::to_string_pretty(&project).unwrap();
    fs::write(&path, &json).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    let loaded: IconProject = serde_json::from_str(&content).unwrap();
    assert_eq!(loaded.canvas.width, 512);
}
