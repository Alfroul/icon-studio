mod common;

use icon_studio_lib::engine::iconset;
use icon_studio_lib::model::*;
use icon_studio_lib::model::shapes::ShapeType;
use std::path::Path;

fn make_test_project(shape: ShapeType, fill: &str, stroke_width: f64) -> IconProject {
    let mut p = IconProject::default();
    p.elements.push(Element::Shape(ShapeElement {
        common: CommonProps::new("shape-1".into(), 10.0, 10.0, 100.0, 100.0),
        shape_type: shape,
        fill: fill.to_string(),
        stroke: if stroke_width > 0.0 {
            Some("#000000".to_string())
        } else {
            None
        },
        stroke_width,
        border_radius: 0.0,
        stroke_dasharray: None,
        gradient: None,
    }));
    p
}

fn cleanup_set(set_id: &str) {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let dir = home.join(".iconstudio").join("sets").join(set_id);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_create_set_creates_directory_and_index() {
    let set = iconset::create_set("Test Set", "A test").unwrap();
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let dir = home.join(".iconstudio").join("sets").join(&set.id);
    assert!(dir.exists());
    let index = dir.join("icon-set.json");
    assert!(index.exists());

    let loaded = iconset::load_set(&set.id).unwrap();
    assert_eq!(loaded.name, "Test Set");
    assert!(loaded.entries.is_empty());

    cleanup_set(&set.id);
}

#[test]
fn test_add_entry_updates_index_and_generates_thumbnail() {
    let mut set = iconset::create_set("Add Test", "").unwrap();
    let project = make_test_project(ShapeType::Circle, "#FF0000", 2.0);

    let entry = iconset::add_entry(
        &mut set,
        &project,
        "home",
        vec!["nav".to_string()],
    )
    .unwrap();
    assert_eq!(entry.name, "home");
    assert_eq!(entry.tags, vec!["nav"]);
    assert!(!entry.thumbnail.is_empty());
    assert!(entry.thumbnail.starts_with("data:image/png;base64,"));

    let loaded = iconset::load_set(&set.id).unwrap();
    assert_eq!(loaded.entries.len(), 1);
    assert_eq!(loaded.entries[0].name, "home");

    cleanup_set(&set.id);
}

#[test]
fn test_remove_entry_deletes_files() {
    let mut set = iconset::create_set("Remove Test", "").unwrap();
    let project = make_test_project(ShapeType::Circle, "#FF0000", 0.0);
    let entry = iconset::add_entry(&mut set, &project, "x", vec![]).unwrap();

    let project_path = entry.project_path.clone();
    assert!(Path::new(&project_path).exists());

    iconset::remove_entry(&set.id, &entry.id).unwrap();

    assert!(!Path::new(&project_path).exists());
    let loaded = iconset::load_set(&set.id).unwrap();
    assert!(loaded.entries.is_empty());

    cleanup_set(&set.id);
}

#[test]
fn test_export_set_empty_returns_zero() {
    let set = iconset::create_set("Empty Export", "").unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let result = iconset::export_set(
        &set.id,
        "png",
        &[16, 32],
        tmp.path().to_str().unwrap(),
    )
    .unwrap();
    assert!(result.is_empty());

    cleanup_set(&set.id);
}

#[test]
fn test_export_set_produces_files() {
    let mut set = iconset::create_set("Export Test", "").unwrap();
    let p1 = make_test_project(ShapeType::Circle, "#FF0000", 0.0);
    let p2 = make_test_project(ShapeType::Rect, "#00FF00", 0.0);
    iconset::add_entry(&mut set, &p1, "circle-icon", vec![]).unwrap();
    iconset::add_entry(&mut set, &p2, "rect-icon", vec![]).unwrap();

    let tmp = tempfile::tempdir().unwrap();
    let result = iconset::export_set(
        &set.id,
        "png",
        &[16, 32],
        tmp.path().to_str().unwrap(),
    )
    .unwrap();
    // 2 icons x 2 sizes = 4 files
    assert_eq!(result.len(), 4);

    cleanup_set(&set.id);
}

#[test]
fn test_check_consistency_detects_stroke_width_issue() {
    let mut set = iconset::create_set("Consistency Test", "").unwrap();

    let p1 = make_test_project(ShapeType::Circle, "#FF0000", 2.0);
    iconset::add_entry(&mut set, &p1, "icon-1", vec![]).unwrap();

    let p2 = make_test_project(ShapeType::Rect, "#00FF00", 2.0);
    iconset::add_entry(&mut set, &p2, "icon-2", vec![]).unwrap();

    // Outlier: stroke_width=3 is 50% deviation from mode 2
    let p3 = make_test_project(ShapeType::Star, "#0000FF", 3.0);
    iconset::add_entry(&mut set, &p3, "icon-3", vec![]).unwrap();

    let report = iconset::check_consistency(&set.id).unwrap();
    assert!(!report.consistent);
    let stroke_issues: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.property == "stroke_width")
        .collect();
    assert_eq!(
        stroke_issues.len(),
        1,
        "Expected 1 stroke_width issue for the 3px outlier"
    );
    assert_eq!(stroke_issues[0].expected, "2.00");
    assert_eq!(stroke_issues[0].actual, "3.00");

    cleanup_set(&set.id);
}

#[test]
fn test_search_entries_by_name_and_tag() {
    let mut set = iconset::create_set("Search Test", "").unwrap();
    let project = make_test_project(ShapeType::Circle, "#FF0000", 0.0);

    iconset::add_entry(&mut set, &project, "home", vec!["navigation".to_string()]).unwrap();
    iconset::add_entry(&mut set, &project, "settings", vec!["system".to_string()]).unwrap();
    iconset::add_entry(&mut set, &project, "back", vec!["navigation".to_string()]).unwrap();

    // Search by name
    let results = iconset::search_entries("home", None, Some(&set.id)).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "home");

    // Search by tag
    let results = iconset::search_entries(
        "",
        Some(&vec!["navigation".to_string()]),
        Some(&set.id),
    )
    .unwrap();
    assert_eq!(results.len(), 2);

    // Search by partial name
    let results = iconset::search_entries("set", None, Some(&set.id)).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "settings");

    cleanup_set(&set.id);
}

#[test]
fn test_list_sets_returns_created_sets() {
    let s1 = iconset::create_set("Set A", "").unwrap();
    let s2 = iconset::create_set("Set B", "").unwrap();

    let sets = iconset::list_sets().unwrap();
    let ids: Vec<&str> = sets.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&s1.id.as_str()));
    assert!(ids.contains(&s2.id.as_str()));

    cleanup_set(&s1.id);
    cleanup_set(&s2.id);
}

#[test]
fn test_tag_entry_updates_tags() {
    let mut set = iconset::create_set("Tag Test", "").unwrap();
    let project = make_test_project(ShapeType::Circle, "#FF0000", 0.0);
    let entry = iconset::add_entry(&mut set, &project, "home", vec![]).unwrap();

    iconset::tag_entry(
        &set.id,
        &entry.id,
        vec!["nav".to_string(), "primary".to_string()],
    )
    .unwrap();

    let loaded = iconset::load_set(&set.id).unwrap();
    assert_eq!(loaded.entries[0].tags, vec!["nav", "primary"]);

    cleanup_set(&set.id);
}
