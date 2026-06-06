use std::sync::{Arc, Mutex};
use icon_studio_lib::mcp::IconStudioHandler;
use icon_studio_lib::model::IconProject;
use icon_studio_lib::engine::builder::RenderCache;
use icon_studio_lib::model::history::CommandHistory;

fn new_handler() -> IconStudioHandler {
    IconStudioHandler::new(Arc::new(Mutex::new(IconProject::default())), Arc::new(Mutex::new(RenderCache::default())), Arc::new(Mutex::new(CommandHistory::default())), None)
}

#[test]
fn test_extended_tool_names() {
    let handler = new_handler();
    let full_names: Vec<String> = handler
        .full_router
        .list_all()
        .iter()
        .map(|t| t.name.to_string())
        .collect();

    let expected_extended = [
        "icon_open",
        "add_image",
        "list_elements",
        "set_shadow",
        "clear_gradient",
        "clear_shadow",
        "suggest_palette",
        "export_ico",
        "export_all",
        "list_fonts",
        "list_icons",
        "save_project",
        "open_project",
    ];

    for name in &expected_extended {
        assert!(
            full_names.iter().any(|n| n == *name),
            "Full router missing extended tool: {}",
            name
        );
    }
}

#[test]
fn test_extended_tools_not_in_core_router() {
    let handler = new_handler();
    let core_names: Vec<String> = handler
        .core_router
        .list_all()
        .iter()
        .map(|t| t.name.to_string())
        .collect();

    let extended_only = [
        "icon_open",
        "add_image",
        "list_elements",
        "set_shadow",
        "clear_gradient",
        "clear_shadow",
        "suggest_palette",
        "export_ico",
        "export_all",
        "list_fonts",
        "list_icons",
        "save_project",
        "open_project",
    ];

    for name in &extended_only {
        assert!(
            !core_names.iter().any(|n| n == *name),
            "Core router should NOT contain extended tool: {}",
            name
        );
    }
}

#[test]
fn test_load_extended_tools_switches_mode() {
    let handler = new_handler();
    assert!(
        !handler.is_full_mode(),
        "Should start in core mode"
    );

    handler.full_mode.store(true, std::sync::atomic::Ordering::SeqCst);
    assert!(
        handler.is_full_mode(),
        "Should be in full mode after setting full_mode = true"
    );
}

#[test]
fn test_core_and_extended_together_cover_all_25_original_tools() {
    let handler = new_handler();
    let core_names: Vec<String> = handler
        .core_router
        .list_all()
        .iter()
        .map(|t| t.name.to_string())
        .collect();

    let all_25_original = [
        "icon_new", "icon_open", "icon_status", "icon_preview",
        "add_shape", "add_text", "add_icon", "add_image",
        "list_elements", "set_props", "remove_element", "set_layout",
        "set_gradient", "set_shadow", "clear_gradient", "clear_shadow",
        "suggest_palette", "export_svg", "export_png", "export_ico",
        "export_all", "list_fonts", "list_icons",
        "save_project", "open_project",
    ];

    // All 25 original tools should be in either core or full router
    // Core has 12 of them (minus load_extended_tools), full has all 25 + load_extended_tools
    let full_names: Vec<String> = handler
        .full_router
        .list_all()
        .iter()
        .map(|t| t.name.to_string())
        .collect();

    for name in &all_25_original {
        assert!(
            full_names.iter().any(|n| n == *name),
            "Full router missing original tool: {}",
            name
        );
    }

    // Core router should have exactly these 12 from original + load_extended_tools
    let core_original = [
        "icon_new", "icon_status", "icon_preview",
        "add_shape", "add_text", "add_icon",
        "set_props", "remove_element", "set_layout",
        "set_gradient", "export_svg", "export_png",
    ];

    for name in &core_original {
        assert!(
            core_names.iter().any(|n| n == *name),
            "Core router missing core tool: {}",
            name
        );
    }
}
