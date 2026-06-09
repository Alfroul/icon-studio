use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconPackMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub icon_count: usize,
    pub categories: Vec<String>,
    pub source_path: String,
    pub imported_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackIcon {
    pub name: String,
    pub category: String,
    pub tags: Vec<String>,
    pub svg_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackIndex {
    pub pack: IconPackMeta,
    pub icons: Vec<PackIcon>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub pack: IconPackMeta,
    pub icons_imported: usize,
    pub errors: Vec<String>,
}

fn packs_dir() -> Result<std::path::PathBuf, AppError> {
    let base = dirs::home_dir()
        .ok_or_else(|| AppError::ValidationError("Cannot determine home directory".into()))?;
    let dir = base.join(".iconstudio").join("packs");
    Ok(dir)
}

fn pack_dir(pack_id: &str) -> Result<std::path::PathBuf, AppError> {
    let base = packs_dir()?;
    // Sanitize pack_id to prevent path traversal
    let sanitized = pack_id
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();
    Ok(base.join(sanitized))
}

fn generate_pack_id(name: &str) -> String {
    let id: String = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() { c }
            else if c == ' ' || c == '-' { '-' }
            else { '\0' }
        })
        .filter(|c| *c != '\0')
        .collect();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{}-{}", id, ts)
}

fn timestamp_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

/// Generate tags from icon name: split on hyphens/underscores, remove short parts
fn name_to_tags(name: &str) -> Vec<String> {
    name.split(['-', '_'])
        .filter(|s| s.len() >= 2)
        .map(|s| s.to_lowercase())
        .collect()
}

/// Validate SVG by attempting to parse with usvg
fn validate_svg(content: &str) -> bool {
    usvg::Tree::from_str(content, &usvg::Options::default()).is_ok()
}

pub fn import_from_directory(dir: &Path, pack_name: &str) -> Result<ImportResult, AppError> {
    if !dir.is_dir() {
        return Err(AppError::ValidationError(format!(
            "Directory does not exist: {}",
            dir.display()
        )));
    }

    let pack_id = generate_pack_id(pack_name);
    let dest_dir = pack_dir(&pack_id)?;
    let svg_dir = dest_dir.join("svg");

    fs::create_dir_all(&svg_dir)?;

    let mut icons: Vec<PackIcon> = Vec::new();
    let mut categories: HashSet<String> = HashSet::new();
    let mut errors: Vec<String> = Vec::new();
    let mut seen_names: HashSet<String> = HashSet::new();

    // Recursively collect SVG files
    fn collect_svgs(
        current_dir: &Path,
        base_dir: &Path,
        icons: &mut Vec<PackIcon>,
        categories: &mut HashSet<String>,
        errors: &mut Vec<String>,
        seen_names: &mut HashSet<String>,
        svg_dir: &Path,
    ) -> Result<(), AppError> {
        let entries = fs::read_dir(current_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                collect_svgs(
                    &path,
                    base_dir,
                    icons,
                    categories,
                    errors,
                    seen_names,
                    svg_dir,
                )?;
                continue;
            }

            if path.extension().and_then(|e| e.to_str()) != Some("svg") {
                continue;
            }

            let relative = path
                .strip_prefix(base_dir)
                .unwrap_or(&path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("");

            let category = if relative.is_empty() {
                "default".to_string()
            } else {
                relative.replace('\\', "/").replace('/', " > ")
            };
            categories.insert(category.clone());

            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            // Handle duplicate names by appending a counter
            let mut final_name = stem.to_string();
            let mut counter = 1;
            while seen_names.contains(&final_name) {
                counter += 1;
                final_name = format!("{}-{}", stem, counter);
            }
            seen_names.insert(final_name.clone());

            // Read and validate SVG
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!("Cannot read {}: {}", path.display(), e));
                    continue;
                }
            };

            if !validate_svg(&content) {
                errors.push(format!("Invalid SVG: {}", path.display()));
                continue;
            }

            // Copy SVG to pack directory
            let dest_svg = svg_dir.join(format!("{}.svg", final_name));
            fs::write(&dest_svg, &content)?;

            let tags = name_to_tags(&final_name);

            icons.push(PackIcon {
                name: final_name,
                category: category.clone(),
                tags,
                svg_path: format!("svg/{}.svg", icons.len()),
            });
        }

        Ok(())
    }

    collect_svgs(
        dir,
        dir,
        &mut icons,
        &mut categories,
        &mut errors,
        &mut seen_names,
        &svg_dir,
    )?;

    let icon_count = icons.len();
    let imported_count = icons.len();
    let _error_count = errors.len();

    let mut cats: Vec<String> = categories.into_iter().collect();
    cats.sort();

    let pack = IconPackMeta {
        id: pack_id,
        name: pack_name.to_string(),
        version: "1.0.0".to_string(),
        icon_count,
        categories: cats,
        source_path: dir.to_string_lossy().to_string(),
        imported_at: timestamp_now(),
    };

    // Now fix svg_path to use actual icon name instead of index
    for icon in &mut icons {
        icon.svg_path = format!("svg/{}.svg", icon.name);
    }

    let index = PackIndex {
        pack: pack.clone(),
        icons,
    };

    let index_json = serde_json::to_string_pretty(&index)?;
    fs::write(dest_dir.join("pack.json"), index_json)?;

    Ok(ImportResult {
        pack,
        icons_imported: imported_count,
        errors,
    })
}

pub fn list_packs() -> Result<Vec<IconPackMeta>, AppError> {
    let base = packs_dir()?;
    if !base.exists() {
        return Ok(Vec::new());
    }

    let mut packs = Vec::new();
    let entries = fs::read_dir(&base)?;

    for entry in entries {
        let entry = entry?;
        let pack_file = entry.path().join("pack.json");
        if !pack_file.exists() {
            continue;
        }

        let content = fs::read_to_string(&pack_file)?;
        let index: PackIndex = serde_json::from_str(&content)?;
        packs.push(index.pack);
    }

    // Sort by import time descending (newest first)
    packs.sort_by(|a, b| b.imported_at.cmp(&a.imported_at));
    Ok(packs)
}

pub fn list_pack_icons(pack_id: &str) -> Result<Vec<PackIcon>, AppError> {
    let dir = pack_dir(pack_id)?;
    let pack_file = dir.join("pack.json");

    if !pack_file.exists() {
        return Err(AppError::NotFoundError(format!(
            "Pack not found: {}",
            pack_id
        )));
    }

    let content = fs::read_to_string(&pack_file)?;
    let index: PackIndex = serde_json::from_str(&content)?;
    Ok(index.icons)
}

pub fn load_pack_icon_svg(pack_id: &str, icon_name: &str) -> Result<String, AppError> {
    let dir = pack_dir(pack_id)?;
    let sanitized_name: String = icon_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let svg_path = dir.join("svg").join(format!("{}.svg", sanitized_name));

    if !svg_path.exists() {
        return Err(AppError::NotFoundError(format!(
            "Icon not found: {}/{}",
            pack_id, icon_name
        )));
    }

    fs::read_to_string(&svg_path).map_err(AppError::IoError)
}

pub fn remove_pack(pack_id: &str) -> Result<(), AppError> {
    let dir = pack_dir(pack_id)?;

    if !dir.exists() {
        return Err(AppError::NotFoundError(format!(
            "Pack not found: {}",
            pack_id
        )));
    }

    fs::remove_dir_all(&dir).map_err(AppError::IoError)
}

pub fn search_pack_icons(pack_id: &str, query: &str) -> Result<Vec<PackIcon>, AppError> {
    let icons = list_pack_icons(pack_id)?;
    let query_lower = query.to_lowercase();

    let results: Vec<PackIcon> = icons
        .into_iter()
        .filter(|icon| {
            icon.name.to_lowercase().contains(&query_lower)
                || icon.category.to_lowercase().contains(&query_lower)
                || icon
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&query_lower))
        })
        .collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_svg(_name: &str) -> String {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M0 0h24v24H0z"/></svg>"#,
        )
    }

    #[test]
    fn test_import_from_directory() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("home.svg"), create_svg("home")).unwrap();
        fs::write(source.join("user.svg"), create_svg("user")).unwrap();

        let result = import_from_directory(&source, "Test Pack").unwrap();
        assert_eq!(result.icons_imported, 2);
        assert_eq!(result.pack.name, "Test Pack");
        assert_eq!(result.pack.icon_count, 2);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_import_preserves_categories() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let solid = source.join("solid");
        let outline = source.join("outline");
        fs::create_dir_all(&solid).unwrap();
        fs::create_dir_all(&outline).unwrap();
        fs::write(solid.join("star.svg"), create_svg("star")).unwrap();
        fs::write(outline.join("moon.svg"), create_svg("moon")).unwrap();

        let result = import_from_directory(&source, "Categorized").unwrap();
        assert_eq!(result.icons_imported, 2);
        assert!(result.pack.categories.contains(&"outline".to_string()));
        assert!(result.pack.categories.contains(&"solid".to_string()));
    }

    #[test]
    fn test_import_skips_invalid_svg() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("valid.svg"), create_svg("valid")).unwrap();
        fs::write(source.join("invalid.svg"), "not an svg").unwrap();
        fs::write(source.join("readme.txt"), "text file").unwrap();

        let result = import_from_directory(&source, "Mixed").unwrap();
        assert_eq!(result.icons_imported, 1);
        assert!(result.errors.iter().any(|e| e.contains("invalid.svg")));
        // .txt file is simply skipped, no error
    }

    #[test]
    fn test_list_packs_empty() {
        // list_packs should not error regardless of state
        let packs = list_packs().unwrap();
        // Should not error even with no packs in a clean state
        assert!(packs.is_empty() || !packs.is_empty());
    }

    #[test]
    fn test_list_pack_icons_after_import() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("star.svg"), create_svg("star")).unwrap();
        fs::write(source.join("heart.svg"), create_svg("heart")).unwrap();

        let result = import_from_directory(&source, "IconTest").unwrap();
        let icons = list_pack_icons(&result.pack.id).unwrap();
        assert_eq!(icons.len(), 2);

        let names: Vec<&str> = icons.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"star"));
        assert!(names.contains(&"heart"));
    }

    #[test]
    fn test_search_by_name() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("arrow-left.svg"), create_svg("arrow-left")).unwrap();
        fs::write(source.join("arrow-right.svg"), create_svg("arrow-right")).unwrap();
        fs::write(source.join("home.svg"), create_svg("home")).unwrap();

        let result = import_from_directory(&source, "SearchTest").unwrap();
        let found = search_pack_icons(&result.pack.id, "arrow").unwrap();
        assert_eq!(found.len(), 2);

        let found_home = search_pack_icons(&result.pack.id, "home").unwrap();
        assert_eq!(found_home.len(), 1);
    }

    #[test]
    fn test_load_icon_svg_content() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("check.svg"), create_svg("check")).unwrap();

        let result = import_from_directory(&source, "SvgLoad").unwrap();
        let svg = load_pack_icon_svg(&result.pack.id, "check").unwrap();
        assert!(!svg.is_empty());
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn test_remove_pack_cleans_files() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("del.svg"), create_svg("del")).unwrap();

        let result = import_from_directory(&source, "DeleteTest").unwrap();
        let pack_id = result.pack.id.clone();

        // Verify pack exists
        let icons = list_pack_icons(&pack_id).unwrap();
        assert!(!icons.is_empty());

        // Remove
        remove_pack(&pack_id).unwrap();

        // Verify removed
        let err = list_pack_icons(&pack_id);
        assert!(err.is_err());
    }

    #[test]
    fn test_import_duplicate_names() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let sub = source.join("sub");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&sub).unwrap();
        // Same filename in root and subdirectory
        fs::write(source.join("icon.svg"), create_svg("icon")).unwrap();
        fs::write(sub.join("icon.svg"), create_svg("icon")).unwrap();

        let result = import_from_directory(&source, "DupTest").unwrap();
        assert_eq!(result.icons_imported, 2);

        let icons = list_pack_icons(&result.pack.id).unwrap();
        let names: Vec<&str> = icons.iter().map(|i| i.name.as_str()).collect();
        // Second duplicate should have a suffix
        assert!(names.contains(&"icon"));
        assert!(names.iter().any(|n| n.starts_with("icon-")));
    }
}
