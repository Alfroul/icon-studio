mod common;

use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::*;

#[test]
fn test_alloc_element_id() {
    let mut project = IconProject::default();

    let id1 = project.alloc_element_id("shape");
    assert_eq!(id1, "shape-1", "First ID should be shape-1");

    let id2 = project.alloc_element_id("shape");
    assert_eq!(id2, "shape-2", "Second ID should be shape-2");

    let id3 = project.alloc_element_id("text");
    assert_eq!(id3, "text-3", "Third ID with different prefix should be text-3");
}

#[test]
fn test_add_element() {
    let mut project = IconProject::default();
    assert_eq!(project.elements.len(), 0);

    let mut s = common::make_shape("shape-1", ShapeType::Circle);
    s.common.x = 10.0;
    s.common.y = 20.0;
    project.elements.push(common::shape_el(s));

    assert_eq!(project.elements.len(), 1);
}

#[test]
fn test_remove_element() {
    let mut project = IconProject::default();
    let mut s = common::make_shape("shape-1", ShapeType::Circle);
    s.common.x = 10.0;
    s.common.y = 20.0;
    project.elements.push(common::shape_el(s));
    assert_eq!(project.elements.len(), 1);

    project.elements.retain(|e| match e {
        Element::Shape(s) => s.common.id != "shape-1",
        _ => true,
    });
    assert_eq!(project.elements.len(), 0, "Element should be removed");
}

#[test]
fn test_update_element_props() {
    let mut project = IconProject::default();
    let mut s = common::make_shape("shape-1", ShapeType::Circle);
    s.common.x = 10.0;
    s.common.y = 20.0;
    project.elements.push(common::shape_el(s));

    if let Element::Shape(ref mut s) = project.elements[0] {
        s.fill = "#00FF00".to_string();
    }

    if let Element::Shape(ref s) = project.elements[0] {
        assert_eq!(s.fill, "#00FF00", "Fill should be updated to #00FF00");
    } else {
        panic!("Expected Shape element");
    }
}
