use icon_studio_lib::templates;
use icon_studio_lib::model::IconProject;
use tempfile::TempDir;

#[test]
fn built_in_templates_returns_eight() {
    let list = templates::built_in_templates();
    assert_eq!(list.len(), 8);
}

#[test]
fn each_builtin_template_is_serializable() {
    let list = templates::built_in_templates();
    for (meta, project) in &list {
        let json = serde_json::to_string_pretty(project).unwrap_or_else(|e| panic!("serialize {}: {}", meta.name, e));
        let back: IconProject = serde_json::from_str(&json).unwrap_or_else(|e| panic!("deserialize {}: {}", meta.name, e));
        assert_eq!(back.elements.len(), project.elements.len(), "{}", meta.name);
    }
}

#[test]
fn apply_template_replaces_project_state() {
    let list = templates::built_in_templates();
    let (_, template_project) = &list[0];
    assert!(!template_project.elements.is_empty());
}

#[test]
fn save_and_load_user_template_roundtrip() {
    let dir = TempDir::new().unwrap();
    let appdata = dir.path().join("appdata");
    std::fs::create_dir_all(&appdata).unwrap();

    let orig_appdata = std::env::var("APPDATA").ok();
    let orig_home = std::env::var("HOME").ok();
    std::env::set_var("APPDATA", &appdata);
    std::env::set_var("HOME", &appdata);

    let original = IconProject::default();
    templates::save_template("test-roundtrip", &original).unwrap();

    let tmpl_path = appdata.join("iconstudio").join("templates").join("test-roundtrip.json");
    assert!(tmpl_path.exists(), "template file should exist at {:?}", tmpl_path);

    let content = std::fs::read_to_string(&tmpl_path).unwrap();
    let loaded: IconProject = serde_json::from_str(&content).unwrap();
    assert_eq!(loaded.canvas.width, original.canvas.width);
    assert_eq!(loaded.elements.len(), original.elements.len());

    if let Some(v) = orig_appdata { std::env::set_var("APPDATA", v); } else { std::env::remove_var("APPDATA"); }
    if let Some(v) = orig_home { std::env::set_var("HOME", v); } else { std::env::remove_var("HOME"); }
}

#[test]
fn list_user_templates_empty_when_no_dir() {
    let dir = TempDir::new().unwrap();
    let appdata = dir.path().join("nonexistent");
    std::fs::create_dir_all(&appdata).unwrap();
    let orig_appdata = std::env::var("APPDATA").ok();
    let orig_home = std::env::var("HOME").ok();
    std::env::set_var("APPDATA", &appdata);
    std::env::set_var("HOME", &appdata);
    let list = templates::list_user_templates();
    assert!(list.is_empty());
    if let Some(v) = orig_appdata { std::env::set_var("APPDATA", v); } else { std::env::remove_var("APPDATA"); }
    if let Some(v) = orig_home { std::env::set_var("HOME", v); } else { std::env::remove_var("HOME"); }
}

#[test]
fn delete_template_removes_file() {
    let dir = TempDir::new().unwrap();
    let templates_dir = dir.path().join("iconstudio").join("templates");
    std::fs::create_dir_all(&templates_dir).unwrap();

    let file_path = templates_dir.join("to-delete.json");
    let project = IconProject::default();
    let json = serde_json::to_string_pretty(&project).unwrap();
    std::fs::write(&file_path, &json).unwrap();
    assert!(file_path.exists());

    std::fs::remove_file(&file_path).unwrap();
    assert!(!file_path.exists());
}
