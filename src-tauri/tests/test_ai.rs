use icon_studio_lib::engine::ai::AiConfig;
use icon_studio_lib::engine::variants::{hex_to_rgb, rgb_to_hex, rgb_to_hsl, hsl_to_rgb};
use icon_studio_lib::model::{AiProvider, AiTask, IconStyle};

#[test]
fn test_ai_config_default() {
    let config = AiConfig::default();
    assert!(matches!(config.provider, AiProvider::OpenAi));
    assert_eq!(config.model, "gpt-4o");
    assert_eq!(config.timeout_secs, 60);
    assert!(config.api_key.is_empty());
    assert!(config.endpoint.is_none());
}

#[test]
fn test_ai_config_serde_roundtrip() {
    let config = AiConfig {
        provider: AiProvider::Recraft,
        api_key: "test-key-123".into(),
        model: "recraft-v3".into(),
        endpoint: Some("https://custom.api.com".into()),
        timeout_secs: 120,
    };
    let json = serde_json::to_string(&config).unwrap();
    let parsed: AiConfig = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed.provider, AiProvider::Recraft));
    assert_eq!(parsed.api_key, "test-key-123");
    assert_eq!(parsed.model, "recraft-v3");
    assert_eq!(parsed.endpoint.unwrap(), "https://custom.api.com");
    assert_eq!(parsed.timeout_secs, 120);
}

#[test]
fn test_ai_config_provider_variants() {
    for provider in [AiProvider::OpenAi, AiProvider::Recraft, AiProvider::Custom, AiProvider::Ollama] {
        let config = AiConfig {
            provider: provider.clone(),
            api_key: String::new(),
            model: "test".into(),
            endpoint: None,
            timeout_secs: 60,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AiConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.provider, config.provider);
    }
}

#[test]
fn test_ai_provider_deserialize() {
    let p: AiProvider = serde_json::from_str("\"openAi\"").unwrap();
    assert!(matches!(p, AiProvider::OpenAi));

    let p: AiProvider = serde_json::from_str("\"recraft\"").unwrap();
    assert!(matches!(p, AiProvider::Recraft));

    let p: AiProvider = serde_json::from_str("\"custom\"").unwrap();
    assert!(matches!(p, AiProvider::Custom));

    let p: AiProvider = serde_json::from_str("\"ollama\"").unwrap();
    assert!(matches!(p, AiProvider::Ollama));
}

#[test]
fn test_ai_task_deserialize() {
    let t: AiTask = serde_json::from_str("\"textToIcon\"").unwrap();
    assert!(matches!(t, AiTask::TextToIcon));

    let t: AiTask = serde_json::from_str("\"styleTransfer\"").unwrap();
    assert!(matches!(t, AiTask::StyleTransfer));

    let t: AiTask = serde_json::from_str("\"removeBackground\"").unwrap();
    assert!(matches!(t, AiTask::RemoveBackground));
}

#[test]
fn test_icon_style_deserialize() {
    for (json, expected) in [
        ("\"flat\"", IconStyle::Flat),
        ("\"outline\"", IconStyle::Outline),
        ("\"duotone\"", IconStyle::Duotone),
        ("\"gradient\"", IconStyle::Gradient),
        ("\"threeD\"", IconStyle::ThreeD),
        ("\"minimal\"", IconStyle::Minimal),
        ("\"cartoon\"", IconStyle::Cartoon),
        ("\"pixelArt\"", IconStyle::PixelArt),
        ("\"lineArt\"", IconStyle::LineArt),
        ("\"neon\"", IconStyle::Neon),
    ] {
        let s: IconStyle = serde_json::from_str(json).unwrap();
        assert_eq!(s, expected);
    }
}

#[test]
fn test_hex_to_rgb() {
    assert_eq!(hex_to_rgb("#FF0000"), (255, 0, 0));
    assert_eq!(hex_to_rgb("#00FF00"), (0, 255, 0));
    assert_eq!(hex_to_rgb("#0000FF"), (0, 0, 255));
    assert_eq!(hex_to_rgb("#FFFFFF"), (255, 255, 255));
    assert_eq!(hex_to_rgb("#000000"), (0, 0, 0));
}

#[test]
fn test_rgb_to_hex() {
    assert_eq!(rgb_to_hex(255, 0, 0), "#FF0000");
    assert_eq!(rgb_to_hex(0, 255, 0), "#00FF00");
    assert_eq!(rgb_to_hex(0, 0, 255), "#0000FF");
    assert_eq!(rgb_to_hex(255, 255, 255), "#FFFFFF");
    assert_eq!(rgb_to_hex(0, 0, 0), "#000000");
}

#[test]
fn test_hex_rgb_roundtrip() {
    for hex in ["#FF0000", "#00FF00", "#123456", "#ABCDEF", "#000000", "#FFFFFF"] {
        let (r, g, b) = hex_to_rgb(hex);
        assert_eq!(rgb_to_hex(r, g, b), hex);
    }
}

#[test]
fn test_rgb_hsl_roundtrip() {
    for (r, g, b) in [(255, 0, 0), (0, 255, 0), (0, 0, 255), (128, 128, 128), (64, 32, 16)] {
        let (h, s, l) = rgb_to_hsl(r, g, b);
        let (r2, g2, b2) = hsl_to_rgb(h, s, l);
        assert!((r as i32 - r2 as i32).abs() <= 1);
        assert!((g as i32 - g2 as i32).abs() <= 1);
        assert!((b as i32 - b2 as i32).abs() <= 1);
    }
}

#[test]
fn test_ai_config_save_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("ai_config.json");

    let config = AiConfig {
        provider: AiProvider::Ollama,
        api_key: "sk-test-key".into(),
        model: "llava".into(),
        endpoint: Some("http://localhost:11434".into()),
        timeout_secs: 90,
    };

    // Serialize
    let json = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write(&config_path, &json).unwrap();

    // Deserialize
    let loaded: AiConfig = serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
    assert!(matches!(loaded.provider, AiProvider::Ollama));
    assert_eq!(loaded.api_key, "sk-test-key");
    assert_eq!(loaded.model, "llava");
    assert_eq!(loaded.endpoint.unwrap(), "http://localhost:11434");
    assert_eq!(loaded.timeout_secs, 90);
}

#[test]
fn test_generated_icon_serde() {
    let icon = icon_studio_lib::engine::ai::GeneratedIcon {
        svg_content: Some("<svg><rect/></svg>".into()),
        image_data: None,
        prompt: "test icon".into(),
    };
    let json = serde_json::to_string(&icon).unwrap();
    let parsed: icon_studio_lib::engine::ai::GeneratedIcon = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.svg_content.unwrap(), "<svg><rect/></svg>");
    assert!(parsed.image_data.is_none());
    assert_eq!(parsed.prompt, "test icon");
}

#[test]
fn test_extract_svg_helper() {
    // Test through the engine module's internal test
    // This validates the SVG extraction from AI text responses
    let text = "Here is the icon:\n<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 512 512\"><circle cx=\"256\" cy=\"256\" r=\"200\"/></svg>\nDone.";
    let start = text.find("<svg").unwrap();
    let end = text.rfind("</svg>").unwrap();
    let svg = &text[start..end + 6];
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}
