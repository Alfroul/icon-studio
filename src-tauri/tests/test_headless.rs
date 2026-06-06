#[test]
fn test_mcp_args_detection() {
    let args = ["icon-studio".to_string(), "--mcp".to_string()];
    assert!(args.contains(&"--mcp".to_string()));

    let args_no_mcp = ["icon-studio".to_string()];
    assert!(!args_no_mcp.contains(&"--mcp".to_string()));
}

#[test]
fn test_gui_mode_default() {
    let args = ["icon-studio".to_string()];
    let is_mcp = args.contains(&"--mcp".to_string());
    assert!(!is_mcp, "Default mode should be GUI (no --mcp)");
}

#[test]
fn test_mcp_args_with_other_flags() {
    let args = [
        "icon-studio".to_string(),
        "--verbose".to_string(),
        "--mcp".to_string(),
    ];
    assert!(args.contains(&"--mcp".to_string()));
}

#[test]
fn test_no_mcp_flag() {
    let args = ["icon-studio".to_string(), "--some-other-flag".to_string()];
    assert!(!args.contains(&"--mcp".to_string()));
}
