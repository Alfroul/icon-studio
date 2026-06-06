//! Pre-made image library module.
//!
//! Provides curated SVG assets organized by category (filled icons, shapes, brand logos).
//! Data loaded from `library_data.json` at compile time via `include_str!`.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// A library asset with full metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryAsset {
    pub name: String,
    pub tags: Vec<String>,
    pub svg: String,
    pub category: String,
    pub label: String,
}

#[derive(Deserialize)]
struct AssetData {
    tags: Vec<String>,
    svg: String,
    category: String,
    label: String,
}

struct LibraryStore {
    assets: std::collections::HashMap<String, AssetData>,
    sorted_names: Vec<String>,
    categories: Vec<String>,
}

static STORE: OnceLock<LibraryStore> = OnceLock::new();

fn store() -> &'static LibraryStore {
    STORE.get_or_init(|| {
        let raw = include_str!("library_data.json");
        let map: std::collections::HashMap<String, AssetData> =
            serde_json::from_str(raw).expect("library_data.json should be valid JSON");
        let mut sorted_names: Vec<String> = map.keys().cloned().collect();
        sorted_names.sort();

        let mut categories: Vec<String> = map.values().map(|d| d.category.clone()).collect();
        categories.sort();
        categories.dedup();

        LibraryStore {
            assets: map,
            sorted_names,
            categories,
        }
    })
}

/// Get an asset's SVG content by name (exact match).
pub fn get_asset_svg(name: &str) -> Option<&'static str> {
    store().assets.get(name).map(|d| d.svg.as_str())
}

/// Get full asset info by name.
pub fn get_asset(name: &str) -> Option<LibraryAsset> {
    store().assets.get(name).map(|d| LibraryAsset {
        name: name.to_string(),
        tags: d.tags.clone(),
        svg: d.svg.clone(),
        category: d.category.clone(),
        label: d.label.clone(),
    })
}

/// List all available categories.
pub fn list_categories() -> Vec<String> {
    store().categories.clone()
}

/// List assets, optionally filtered by category and/or keyword.
pub fn list_assets(category: Option<&str>, keyword: Option<&str>) -> Vec<LibraryAsset> {
    let kw = keyword.map(|k| k.to_lowercase());
    store()
        .sorted_names
        .iter()
        .filter(|name| {
            let data = match store().assets.get(*name) {
                Some(d) => d,
                None => return false,
            };

            if let Some(cat) = category {
                if data.category != cat {
                    return false;
                }
            }

            if let Some(ref k) = kw {
                let name_match = name.to_lowercase().contains(k);
                let tag_match = data.tags.iter().any(|t| t.to_lowercase().contains(k));
                let label_match = data.label.to_lowercase().contains(k);
                return name_match || tag_match || label_match;
            }

            true
        })
        .map(|name| {
            let data = match store().assets.get(name) {
                Some(d) => d,
                None => return LibraryAsset { name: name.clone(), tags: vec![], label: name.clone(), category: "unknown".into(), svg: String::new() },
            };
            LibraryAsset {
                name: name.clone(),
                tags: data.tags.clone(),
                svg: data.svg.clone(),
                category: data.category.clone(),
                label: data.label.clone(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_asset_svg_existing() {
        assert!(get_asset_svg("filled-home").is_some());
        assert!(get_asset_svg("filled-home").unwrap().contains("svg"));
    }

    #[test]
    fn test_get_asset_svg_missing() {
        assert!(get_asset_svg("nonexistent-asset-xyz").is_none());
    }

    #[test]
    fn test_get_asset() {
        let asset = get_asset("filled-heart").unwrap();
        assert_eq!(asset.name, "filled-heart");
        assert_eq!(asset.category, "filled-icons");
        assert_eq!(asset.label, "Heart");
        assert!(asset.tags.contains(&"heart".to_string()));
    }

    #[test]
    fn test_list_categories() {
        let cats = list_categories();
        assert!(cats.contains(&"filled-icons".to_string()));
        assert!(cats.contains(&"shapes".to_string()));
        assert!(cats.contains(&"brand-logos".to_string()));
    }

    #[test]
    fn test_list_assets_all() {
        let all = list_assets(None, None);
        assert!(all.len() >= 50, "should have 50+ assets, got {}", all.len());
    }

    #[test]
    fn test_list_assets_by_category() {
        let shapes = list_assets(Some("shapes"), None);
        assert!(!shapes.is_empty());
        assert!(shapes.iter().all(|a| a.category == "shapes"));
    }

    #[test]
    fn test_list_assets_by_keyword() {
        let results = list_assets(None, Some("heart"));
        assert!(results.len() >= 2, "should find filled-heart and deco-heart-shape");
    }

    #[test]
    fn test_list_assets_keyword_case_insensitive() {
        let results = list_assets(None, Some("GITHUB"));
        assert!(results.iter().any(|a| a.name == "brand-github"));
    }
}
