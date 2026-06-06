use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Supported style preset types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum StyleType {
    Soft3d,
    Neumorphism,
    Glassmorphism,
    Flat,
}

/// Parameters controlling style preset appearance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct StyleParams {
    #[serde(default = "default_depth")]
    pub depth: f64,
    #[serde(default = "default_light_angle")]
    pub light_angle: f64,
    #[serde(default = "default_highlight")]
    pub highlight: f64,
    #[serde(default = "default_shadow_softness")]
    pub shadow_softness: f64,
}

fn default_depth() -> f64 {
    5.0
}
fn default_light_angle() -> f64 {
    135.0
}
fn default_highlight() -> f64 {
    0.3
}
fn default_shadow_softness() -> f64 {
    8.0
}

impl Default for StyleParams {
    fn default() -> Self {
        Self {
            depth: default_depth(),
            light_angle: default_light_angle(),
            highlight: default_highlight(),
            shadow_softness: default_shadow_softness(),
        }
    }
}

/// A style preset that combines multiple visual effects into one macro operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct StylePreset {
    pub style_type: StyleType,
    #[serde(default)]
    pub params: StyleParams,
}

/// A user-saved custom style preset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct CustomStylePreset {
    pub name: String,
    pub style_type: StyleType,
    #[serde(default)]
    pub params: StyleParams,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_preset_serde_roundtrip() {
        let preset = StylePreset {
            style_type: StyleType::Soft3d,
            params: StyleParams::default(),
        };
        let json = serde_json::to_string(&preset).unwrap();
        let parsed: StylePreset = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, preset);
    }

    #[test]
    fn test_style_type_kebab_case() {
        let json = serde_json::to_string(&StyleType::Soft3d).unwrap();
        assert_eq!(json, "\"soft-3d\"");
        let json = serde_json::to_string(&StyleType::Glassmorphism).unwrap();
        assert_eq!(json, "\"glassmorphism\"");
    }
}
