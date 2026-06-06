use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported SVG filter types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum FilterType {
    Noise,
    Blur,
    Pixelate,
    Emboss,
    Posterize,
    Turbulence,
}

/// SVG filter applied to an element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SvgFilter {
    pub filter_type: FilterType,
    #[serde(default)]
    pub params: HashMap<String, f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_filter_serde_roundtrip() {
        let mut params = HashMap::new();
        params.insert("baseFrequency".to_string(), 0.05);
        params.insert("numOctaves".to_string(), 3.0);

        let filter = SvgFilter {
            filter_type: FilterType::Turbulence,
            params,
        };

        let json = serde_json::to_string(&filter).unwrap();
        let parsed: SvgFilter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, filter);
    }

    #[test]
    fn test_filter_type_kebab_case() {
        let ft = FilterType::Noise;
        let json = serde_json::to_string(&ft).unwrap();
        assert_eq!(json, "\"noise\"");

        let ft = FilterType::Pixelate;
        let json = serde_json::to_string(&ft).unwrap();
        assert_eq!(json, "\"pixelate\"");
    }

    #[test]
    fn test_svg_filter_default_params() {
        // Old JSON without params should default to empty HashMap
        let json = r#"{"filter_type": "blur"}"#;
        let parsed: SvgFilter = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.filter_type, FilterType::Blur);
        assert!(parsed.params.is_empty());
    }

    #[test]
    fn test_svg_filter_backward_compat() {
        // Ensure new fields default correctly when missing from old JSON
        let json = r#"{"filter_type": "noise"}"#;
        let parsed: SvgFilter = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.filter_type, FilterType::Noise);
        assert!(parsed.params.is_empty());
    }
}
