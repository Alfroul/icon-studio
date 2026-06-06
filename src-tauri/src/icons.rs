//! Lucide icons data module.
//!
//! Provides embedded SVG data for ~1700 Lucide icons with lookup and search.
//! Data is loaded from `icons_data.json` at compile time via `include_str!`
//! and parsed once lazily on first access.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Metadata for a Lucide icon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconInfo {
    pub name: String,
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
struct IconData {
    tags: Vec<String>,
    svg: String,
}

#[derive(Deserialize)]
struct IconMap(std::collections::HashMap<String, IconData>);

struct IconStore {
    map: std::collections::HashMap<String, IconData>,
    sorted_names: Vec<String>,
}

static STORE: OnceLock<IconStore> = OnceLock::new();

fn store() -> &'static IconStore {
    STORE.get_or_init(|| {
        let raw = include_str!("icons_data.json");
        let map: IconMap =
            serde_json::from_str(raw).expect("icons_data.json should be valid JSON");
        let mut sorted_names: Vec<String> = map.0.keys().cloned().collect();
        sorted_names.sort();
        IconStore {
            map: map.0,
            sorted_names,
        }
    })
}

/// Look up an icon's SVG content by name (exact match).
///
/// Returns the inner SVG elements string (e.g., <path d="..."/><circle .../>)
/// designed for a 24x24 viewBox with stroke-based rendering.
pub fn get_icon_path(name: &str) -> Option<&'static str> {
    store().map.get(name).map(|d| d.svg.as_str())
}

/// Search icons by keyword (case-insensitive).
///
/// Matches against icon name and associated tags.
pub fn search_icons(keyword: &str) -> Vec<IconInfo> {
    let kw = keyword.to_lowercase();
    store()
        .sorted_names
        .iter()
        .filter(|name| {
            let name_lower = name.to_lowercase();
            if name_lower.contains(&kw) {
                return true;
            }
            let data = store().map.get(*name).expect("icon name came from sorted_names which are map keys");
            data.tags.iter().any(|t| t.to_lowercase().contains(&kw))
        })
        .map(|name| {
            let data = store().map.get(name).expect("icon name came from sorted_names which are map keys");
            IconInfo {
                name: name.clone(),
                tags: data.tags.clone(),
            }
        })
        .collect()
}

/// Return all available icons with their metadata.
pub fn list_all_icons() -> Vec<IconInfo> {
    store()
        .sorted_names
        .iter()
        .map(|name| {
            let data = store().map.get(name).expect("icon name came from sorted_names which are map keys");
            IconInfo {
                name: name.clone(),
                tags: data.tags.clone(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_icon_path_existing() {
        assert!(get_icon_path("heart").is_some());
        assert!(get_icon_path("heart").unwrap().contains("path"));
    }

    #[test]
    fn test_get_icon_path_missing() {
        assert!(get_icon_path("nonexistent-icon-xyz").is_none());
    }

    #[test]
    fn test_search_icons_by_name() {
        let results = search_icons("arrow");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.name == "arrow-right"));
    }

    #[test]
    fn test_search_icons_case_insensitive() {
        let results = search_icons("HEART");
        assert!(results.iter().any(|r| r.name == "heart"));
    }

    #[test]
    fn test_search_icons_by_tag() {
        let results = search_icons("cart");
        assert!(results.iter().any(|r| r.name == "shopping-cart"));
    }

    #[test]
    fn test_list_all_icons_count() {
        let all = list_all_icons();
        assert!(all.len() >= 1000, "should have 1000+ icons, got {}", all.len());
        assert!(all.iter().any(|r| r.name == "heart"));
        assert!(all.iter().any(|r| r.name == "star"));
    }
}
