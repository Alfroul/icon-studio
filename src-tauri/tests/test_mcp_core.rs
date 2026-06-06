use std::sync::{Arc, Mutex};
use icon_studio_lib::mcp::IconStudioHandler;
use icon_studio_lib::model::IconProject;
use icon_studio_lib::engine::builder::RenderCache;
use icon_studio_lib::model::history::CommandHistory;

fn new_handler() -> IconStudioHandler {
    IconStudioHandler::new(Arc::new(Mutex::new(IconProject::default())), Arc::new(Mutex::new(RenderCache::default())), Arc::new(Mutex::new(CommandHistory::default())), None)
}

#[test]
fn test_core_router_has_13_tools() {
    let handler = new_handler();
    let core_tools = handler.core_router.list_all();
    assert_eq!(
        core_tools.len(),
        13,
        "Core router should have 13 tools (12 core + load_extended_tools), got {}",
        core_tools.len()
    );
}

#[test]
fn test_full_router_has_26_tools() {
    let handler = new_handler();
    let full_tools = handler.full_router.list_all();
    assert_eq!(
        full_tools.len(),
        43,
        "Full router should have 43 tools (13 core + 30 extended), got {}",
        full_tools.len()
    );
}

#[test]
fn test_core_tool_names() {
    let handler = new_handler();
    let core_names: Vec<String> = handler
        .core_router
        .list_all()
        .iter()
        .map(|t| t.name.to_string())
        .collect();

    let expected_core = [
        "icon_new",
        "icon_status",
        "icon_preview",
        "add_shape",
        "add_text",
        "add_icon",
        "set_props",
        "remove_element",
        "set_layout",
        "set_gradient",
        "export_svg",
        "export_png",
        "load_extended_tools",
    ];

    for name in &expected_core {
        assert!(
            core_names.iter().any(|n| n == *name),
            "Core router missing tool: {}",
            name
        );
    }
}

#[test]
fn test_handler_starts_in_core_mode() {
    let handler = new_handler();
    assert!(
        !handler.is_full_mode(),
        "Handler should start in core mode (full_mode = false)"
    );
}

#[test]
fn test_handler_project_accessible() {
    let handler = new_handler();
    let project = handler.project.lock().unwrap();
    assert_eq!(project.canvas.width, 512);
    assert_eq!(project.elements.len(), 0);
}
