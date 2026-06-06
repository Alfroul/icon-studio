use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semantic color roles in a brand kit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum BrandColorRole {
    Primary,
    Secondary,
    Accent,
    Neutral,
    Surface,
    Error,
}

/// A complete brand kit with semantic color definitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct BrandKit {
    pub id: String,
    pub name: String,
    pub colors: HashMap<BrandColorRole, String>,
    #[serde(default)]
    pub variants: HashMap<String, BrandVariant>,
}

/// A brand variant (e.g. dark, light, high-contrast).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct BrandVariant {
    pub variant_type: String,
    pub colors: HashMap<BrandColorRole, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brand_kit_serde_roundtrip() {
        let mut colors = HashMap::new();
        colors.insert(BrandColorRole::Primary, "#FF6B35".to_string());
        colors.insert(BrandColorRole::Secondary, "#004E89".to_string());

        let kit = BrandKit {
            id: "brand-1".to_string(),
            name: "Test Brand".to_string(),
            colors,
            variants: HashMap::new(),
        };
        let json = serde_json::to_string(&kit).unwrap();
        let parsed: BrandKit = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, kit);
    }

    #[test]
    fn test_brand_color_role_kebab_case() {
        let json = serde_json::to_string(&BrandColorRole::Primary).unwrap();
        assert_eq!(json, "\"primary\"");
    }
}
