mod common;

use icon_studio_lib::model::history::{
    AddElementCommand, CommandHistory, RemoveElementCommand, SetPropsCommand, ReorderCommand,
};
use icon_studio_lib::model::{Element, IconProject};
use icon_studio_lib::model::shapes::ShapeType;

fn make_shape(id: &str, fill: &str) -> Element {
    let mut s = common::make_shape(id, ShapeType::Circle);
    s.fill = fill.to_string();
    common::shape_el(s)
}

fn make_text(id: &str, content: &str) -> Element {
    common::text_el(common::make_text(id, content))
}

#[test]
fn undo_add_shape_restores_element_count() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::default();

    assert_eq!(project.elements.len(), 0);

    let cmd = AddElementCommand::new(make_shape("shape-1", "#ff0000"));
    history.push_and_execute(Box::new(cmd), &mut project).unwrap();
    assert_eq!(project.elements.len(), 1);

    history.undo(&mut project).unwrap();
    assert_eq!(project.elements.len(), 0);
}

#[test]
fn redo_restores_element_count() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::default();

    let cmd = AddElementCommand::new(make_shape("shape-1", "#00ff00"));
    history.push_and_execute(Box::new(cmd), &mut project).unwrap();
    assert_eq!(project.elements.len(), 1);

    history.undo(&mut project).unwrap();
    assert_eq!(project.elements.len(), 0);

    history.redo(&mut project).unwrap();
    assert_eq!(project.elements.len(), 1);
}

#[test]
fn three_operations_undo_three_returns_to_initial() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::default();

    for i in 1..=3 {
        let cmd = AddElementCommand::new(make_shape(&format!("shape-{}", i), "#000"));
        history.push_and_execute(Box::new(cmd), &mut project).unwrap();
    }
    assert_eq!(project.elements.len(), 3);

    for _ in 0..3 {
        history.undo(&mut project).unwrap();
    }
    assert_eq!(project.elements.len(), 0);
    assert!(!history.can_undo());
}

#[test]
fn new_operation_clears_redo_stack() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::default();

    let cmd1 = AddElementCommand::new(make_shape("shape-1", "#000"));
    history.push_and_execute(Box::new(cmd1), &mut project).unwrap();

    history.undo(&mut project).unwrap();
    assert!(history.can_redo());

    let cmd2 = AddElementCommand::new(make_shape("shape-2", "#111"));
    history.push_and_execute(Box::new(cmd2), &mut project).unwrap();
    assert!(!history.can_redo());
}

#[test]
fn history_max_size_drops_oldest() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::new(3);

    for i in 1..=5 {
        let cmd = AddElementCommand::new(make_shape(&format!("s-{}", i), "#000"));
        history.push_and_execute(Box::new(cmd), &mut project).unwrap();
    }
    assert_eq!(project.elements.len(), 5);

    assert!(history.can_undo());
    for _ in 0..3 {
        history.undo(&mut project).unwrap();
    }
    assert!(!history.can_undo());
}

#[test]
fn remove_element_undo_reinserts() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::default();

    let el = make_shape("shape-1", "#abc");
    project.elements.push(el.clone());

    let cmd = RemoveElementCommand::new(el, 0);
    history.push_and_execute(Box::new(cmd), &mut project).unwrap();
    assert_eq!(project.elements.len(), 0);

    history.undo(&mut project).unwrap();
    assert_eq!(project.elements.len(), 1);
}

#[test]
fn set_props_undo_restores_old_values() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::default();

    project.elements.push(make_shape("shape-1", "#ff0000"));

    let old = serde_json::to_value(&project.elements[0]).unwrap();
    let mut new = old.clone();
    new["fill"] = serde_json::Value::String("#00ff00".to_string());

    let cmd = SetPropsCommand::new("shape-1".to_string(), old, new);
    history.push_and_execute(Box::new(cmd), &mut project).unwrap();

    match &project.elements[0] {
        Element::Shape(s) => assert_eq!(s.fill, "#00ff00"),
        _ => panic!("expected shape"),
    }

    history.undo(&mut project).unwrap();
    match &project.elements[0] {
        Element::Shape(s) => assert_eq!(s.fill, "#ff0000"),
        _ => panic!("expected shape"),
    }
}

#[test]
fn reorder_undo_restores_position() {
    let mut project = IconProject::default();
    project.elements.push(make_shape("a", "#aaa"));
    project.elements.push(make_text("b", "hello"));

    let mut history = CommandHistory::default();
    let cmd = ReorderCommand::new("a".to_string(), 0, 1);
    history.push_and_execute(Box::new(cmd), &mut project).unwrap();

    match &project.elements[0] {
        Element::Text(t) => assert_eq!(t.common.id, "b"),
        _ => panic!("expected text at index 0 after reorder"),
    }

    history.undo(&mut project).unwrap();
    match &project.elements[0] {
        Element::Shape(s) => assert_eq!(s.common.id, "a"),
        _ => panic!("expected shape at index 0 after undo"),
    }
}

#[test]
fn batch_undo_reverts_all_operations_at_once() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::default();

    history.begin_batch("add three shapes").unwrap();
    for i in 1..=3 {
        let cmd = AddElementCommand::new(make_shape(&format!("shape-{}", i), "#000"));
        history.push_and_execute(Box::new(cmd), &mut project).unwrap();
    }
    history.commit_batch().unwrap();

    assert_eq!(project.elements.len(), 3);
    assert!(history.can_undo());

    history.undo(&mut project).unwrap();
    assert_eq!(project.elements.len(), 0, "Single undo should revert all 3 batched operations");
    assert!(!history.can_undo());
}

#[test]
fn batch_redo_reapplies_all_operations() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::default();

    history.begin_batch("add three shapes").unwrap();
    for i in 1..=3 {
        let cmd = AddElementCommand::new(make_shape(&format!("shape-{}", i), "#000"));
        history.push_and_execute(Box::new(cmd), &mut project).unwrap();
    }
    history.commit_batch().unwrap();

    history.undo(&mut project).unwrap();
    assert_eq!(project.elements.len(), 0);

    history.redo(&mut project).unwrap();
    assert_eq!(project.elements.len(), 3, "Single redo should reapply all 3 batched operations");
}

#[test]
fn run_batch_convenience_wrapper() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::default();

    history.run_batch("add shapes", |h| {
        for i in 1..=3 {
            let cmd = AddElementCommand::new(make_shape(&format!("s-{}", i), "#000"));
            h.push_and_execute(Box::new(cmd), &mut project).unwrap();
        }
    }).unwrap();

    assert_eq!(project.elements.len(), 3);
    history.undo(&mut project).unwrap();
    assert_eq!(project.elements.len(), 0, "run_batch undo should revert all operations");
}

#[test]
fn nested_batch_panics() {
    let mut h = CommandHistory::default();
    h.begin_batch("outer").unwrap();
    let result = h.begin_batch("inner");
    assert!(result.is_err(), "Nested batch should return Err");
}

#[test]
fn batch_with_no_commands_is_noop() {
    let mut history = CommandHistory::default();

    history.begin_batch("empty batch").unwrap();
    history.commit_batch().unwrap();

    assert!(!history.can_undo(), "Empty batch should not push to undo stack");
}

#[test]
fn version_bumps_on_push_and_execute() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::default();
    let initial_version = project.version;

    let cmd = AddElementCommand::new(make_shape("shape-1", "#000"));
    history.push_and_execute(Box::new(cmd), &mut project).unwrap();

    assert_eq!(project.version, initial_version + 1, "push_and_execute should bump version");
}

#[test]
fn version_bumps_on_undo_and_redo() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::default();

    let cmd = AddElementCommand::new(make_shape("shape-1", "#000"));
    history.push_and_execute(Box::new(cmd), &mut project).unwrap();
    let version_after_execute = project.version;

    history.undo(&mut project).unwrap();
    assert_eq!(project.version, version_after_execute + 1, "undo should bump version");

    history.redo(&mut project).unwrap();
    assert_eq!(project.version, version_after_execute + 2, "redo should bump version");
}

#[test]
fn batch_version_bumps_per_operation() {
    let mut project = IconProject::default();
    let mut history = CommandHistory::default();
    let initial_version = project.version;

    history.begin_batch("add shapes").unwrap();
    for i in 1..=3 {
        let cmd = AddElementCommand::new(make_shape(&format!("s-{}", i), "#000"));
        history.push_and_execute(Box::new(cmd), &mut project).unwrap();
    }
    history.commit_batch().unwrap();

    assert_eq!(project.version, initial_version + 3, "Each push_and_execute in batch should bump version");
}

#[test]
fn default_max_size_is_200() {
    let mut history = CommandHistory::default();
    let mut project = IconProject::default();

    for i in 0..250 {
        let cmd = AddElementCommand::new(make_shape(&format!("s-{}", i), "#000"));
        history.push_and_execute(Box::new(cmd), &mut project).unwrap();
    }

    assert_eq!(project.elements.len(), 250);
    let mut undo_count = 0;
    while history.undo(&mut project).unwrap() {
        undo_count += 1;
    }
    assert_eq!(undo_count, 200, "Should be able to undo exactly 200 operations (default max_size)");
}

#[test]
fn serde_backward_compatibility_no_version_field() {
    let json = r##"{
        "schema_version": "1.0",
        "canvas": { "width": 512, "height": 512, "background": "#FFFFFF", "corner_radius": 0 },
        "elements": [],
        "exports": { "formats": ["svg"], "sizes": [512] },
        "templates": {}
    }"##;
    let project: IconProject = serde_json::from_str(json).expect("should deserialize without version field");
    assert_eq!(project.version, 0, "Missing version field should default to 0");
}