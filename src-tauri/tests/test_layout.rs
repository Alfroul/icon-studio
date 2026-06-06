mod common;

use icon_studio_lib::commands::layout::{get_element_bounds, set_element_position};
use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::Element;

fn make_shape(id: &str, x: f64, y: f64, w: f64, h: f64) -> Element {
    common::shape_el(common::make_shape_sized(id, ShapeType::Rect, x, y, w, h))
}

fn canvas_size() -> (f64, f64) {
    (512.0, 512.0)
}

#[test]
fn center_layout_all_elements_centered() {
    let mut elements = vec![
        make_shape("s1", 10.0, 20.0, 100.0, 80.0),
        make_shape("s2", 300.0, 400.0, 60.0, 60.0),
    ];
    let (cw, ch) = canvas_size();
    let padding = 0.0;

    for elem in &mut elements {
        let (_, _, ew, eh) = get_element_bounds(elem);
        let nx = padding + (cw - padding * 2.0 - ew) / 2.0;
        let ny = padding + (ch - padding * 2.0 - eh) / 2.0;
        set_element_position(elem, nx, ny);
    }

    let (x1, y1, w1, h1) = get_element_bounds(&elements[0]);
    assert!(
        (x1 - (cw - w1) / 2.0).abs() < f64::EPSILON,
        "Element 1 x should be centered"
    );
    assert!(
        (y1 - (ch - h1) / 2.0).abs() < f64::EPSILON,
        "Element 1 y should be centered"
    );

    let (x2, y2, w2, h2) = get_element_bounds(&elements[1]);
    assert!(
        (x2 - (cw - w2) / 2.0).abs() < f64::EPSILON,
        "Element 2 x should be centered"
    );
    assert!(
        (y2 - (ch - h2) / 2.0).abs() < f64::EPSILON,
        "Element 2 y should be centered"
    );
}

#[test]
fn horizontal_layout_elements_do_not_overlap() {
    let mut elements = vec![
        make_shape("s1", 0.0, 0.0, 100.0, 100.0),
        make_shape("s2", 0.0, 0.0, 100.0, 100.0),
    ];
    let (cw, ch) = canvas_size();
    let gap = 20.0;
    let padding = 0.0;

    let total_width: f64 = elements.iter().map(|e| {
        let (_, _, w, _) = get_element_bounds(e);
        w
    }).sum();
    let total_gaps = (elements.len() - 1) as f64 * gap;
    let start_x = padding + (cw - padding * 2.0 - total_width - total_gaps) / 2.0;

    let mut current_x = start_x;
    for elem in &mut elements {
        let (_, _, ew, eh) = get_element_bounds(elem);
        let ny = padding + (ch - padding * 2.0 - eh) / 2.0;
        set_element_position(elem, current_x, ny);
        current_x += ew + gap;
    }

    let (x1, _, w1, _) = get_element_bounds(&elements[0]);
    let (x2, _, _, _) = get_element_bounds(&elements[1]);
    assert!(
        x1 + w1 <= x2 + f64::EPSILON,
        "Element 1 right edge ({}) should not overlap element 2 left edge ({})",
        x1 + w1,
        x2
    );
    let actual_gap = x2 - (x1 + w1);
    assert!(
        (actual_gap - gap).abs() < f64::EPSILON,
        "Gap between elements should be {}, got {}",
        gap,
        actual_gap
    );
}

#[test]
fn vertical_layout_elements_do_not_overlap() {
    let mut elements = vec![
        make_shape("s1", 0.0, 0.0, 100.0, 80.0),
        make_shape("s2", 0.0, 0.0, 120.0, 60.0),
    ];
    let (cw, ch) = canvas_size();
    let gap = 30.0;
    let padding = 0.0;

    let total_height: f64 = elements.iter().map(|e| {
        let (_, _, _, h) = get_element_bounds(e);
        h
    }).sum();
    let total_gaps = (elements.len() - 1) as f64 * gap;
    let start_y = padding + (ch - padding * 2.0 - total_height - total_gaps) / 2.0;

    let mut current_y = start_y;
    for elem in &mut elements {
        let (_, _, ew, eh) = get_element_bounds(elem);
        let nx = padding + (cw - padding * 2.0 - ew) / 2.0;
        set_element_position(elem, nx, current_y);
        current_y += eh + gap;
    }

    let (_, y1, _, h1) = get_element_bounds(&elements[0]);
    let (_, y2, _, _) = get_element_bounds(&elements[1]);
    assert!(
        y1 + h1 <= y2 + f64::EPSILON,
        "Element 1 bottom ({}) should not overlap element 2 top ({})",
        y1 + h1,
        y2
    );
    let actual_gap = y2 - (y1 + h1);
    assert!(
        (actual_gap - gap).abs() < f64::EPSILON,
        "Gap between elements should be {}, got {}",
        gap,
        actual_gap
    );
}

#[test]
fn stack_layout_all_at_same_center_position() {
    let mut elements = vec![
        make_shape("s1", 10.0, 20.0, 200.0, 150.0),
        make_shape("s2", 300.0, 100.0, 100.0, 100.0),
        make_shape("s3", 50.0, 50.0, 80.0, 300.0),
    ];
    let (cw, ch) = canvas_size();

    for elem in &mut elements {
        let (_, _, ew, eh) = get_element_bounds(elem);
        let nx = (cw - ew) / 2.0;
        let ny = (ch - eh) / 2.0;
        set_element_position(elem, nx, ny);
    }

    let (x1, y1, w1, h1) = get_element_bounds(&elements[0]);
    let (x2, y2, w2, h2) = get_element_bounds(&elements[1]);
    let (x3, y3, w3, h3) = get_element_bounds(&elements[2]);

    let cx1 = x1 + w1 / 2.0;
    let cy1 = y1 + h1 / 2.0;
    let cx2 = x2 + w2 / 2.0;
    let cy2 = y2 + h2 / 2.0;
    let cx3 = x3 + w3 / 2.0;
    let cy3 = y3 + h3 / 2.0;

    assert!(
        (cx1 - cw / 2.0).abs() < f64::EPSILON && (cy1 - ch / 2.0).abs() < f64::EPSILON,
        "Element 1 center should be at canvas center"
    );
    assert!(
        (cx2 - cw / 2.0).abs() < f64::EPSILON && (cy2 - ch / 2.0).abs() < f64::EPSILON,
        "Element 2 center should be at canvas center"
    );
    assert!(
        (cx3 - cw / 2.0).abs() < f64::EPSILON && (cy3 - ch / 2.0).abs() < f64::EPSILON,
        "Element 3 center should be at canvas center"
    );
}
