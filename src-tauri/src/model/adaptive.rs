use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Target clipping shape for adaptive icons.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AdaptiveShape {
    Circle,
    Squircle,
    RoundedRect,
    Pill,
    Square,
}

/// Configuration for adaptive icon foreground/background layering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct AdaptiveConfig {
    pub foreground_ids: Vec<String>,
    pub background_ids: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_config_serde_roundtrip() {
        let config = AdaptiveConfig {
            foreground_ids: vec!["icon-1".to_string()],
            background_ids: vec!["shape-1".to_string()],
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AdaptiveConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, config);
    }

    #[test]
    fn test_adaptive_shape_kebab_case() {
        let json = serde_json::to_string(&AdaptiveShape::RoundedRect).unwrap();
        assert_eq!(json, "\"rounded-rect\"");
    }
}
