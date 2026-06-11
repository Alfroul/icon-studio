use crate::error::AppError;
use crate::model::{AiProvider, AiTask, IconStyle};
use serde::{Deserialize, Serialize};

// ---- Config ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: AiProvider,
    pub api_key: String,
    pub model: String,
    pub endpoint: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    60
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::OpenAi,
            api_key: String::new(),
            model: "gpt-4o".to_string(),
            endpoint: None,
            timeout_secs: default_timeout(),
        }
    }
}

// ---- Result types ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedIcon {
    pub svg_content: Option<String>,
    pub image_data: Option<Vec<u8>>,
    pub prompt: String,
}

// ---- Provider request/response structs ----

#[derive(Serialize)]
struct OpenAiImageRequest {
    model: String,
    prompt: String,
    n: u32,
    size: String,
    response_format: String,
}

#[derive(Deserialize)]
struct OpenAiImageResponse {
    data: Vec<OpenAiImageData>,
}

#[derive(Deserialize)]
struct OpenAiImageData {
    b64_json: Option<String>,
}

#[derive(Serialize)]
struct RecraftVectorRequest {
    prompt: String,
    #[serde(rename = "image_size")]
    image_size: String,
    #[serde(rename = "style")]
    style: Option<String>,
}

#[derive(Deserialize)]
struct RecraftVectorResponse {
    image: Option<String>,
}

#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

// ---- Main entry point ----

pub async fn generate_icon(
    task: AiTask,
    style: IconStyle,
    prompt: &str,
    config: &AiConfig,
) -> Result<Vec<GeneratedIcon>, AppError> {
    let full_prompt = build_prompt(task, style, prompt);

    match config.provider {
        AiProvider::OpenAi => generate_openai(&full_prompt, config).await,
        AiProvider::Recraft => generate_recraft(&full_prompt, style, config).await,
        AiProvider::Custom => generate_custom(&full_prompt, config).await,
        AiProvider::Ollama => generate_ollama(&full_prompt, config).await,
    }
}

pub async fn generate_icon_set(
    prompts: &[String],
    style: IconStyle,
    config: &AiConfig,
) -> Result<Vec<GeneratedIcon>, AppError> {
    let mut all = Vec::new();
    for p in prompts {
        let icons = generate_icon(AiTask::TextToIcon, style, p, config).await?;
        all.extend(icons);
    }
    Ok(all)
}

pub async fn remove_background(
    image_data: &[u8],
    config: &AiConfig,
) -> Result<Vec<u8>, AppError> {
    let client = build_client(config.timeout_secs)?;
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, image_data);

    match config.provider {
        AiProvider::OpenAi => {
            let url = "https://api.openai.com/v1/images/edits";
            let body = serde_json::json!({
                "image": b64,
                "model": &config.model,
            });
            let resp = client
                .post(url)
                .bearer_auth(&config.api_key)
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::BuildError(format!("OpenAI request failed: {}", e)))?;
            let data: OpenAiImageResponse = resp
                .json()
                .await
                .map_err(|e| AppError::BuildError(format!("OpenAI parse failed: {}", e)))?;
            data.data
                .first()
                .and_then(|d| d.b64_json.as_ref())
                .map(|b| {
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b)
                        .unwrap_or_default()
                })
                .ok_or_else(|| AppError::BuildError("No image data in response".into()))
        }
        _ => Err(AppError::ValidationError(
            "Background removal only supported with OpenAI provider".into(),
        )),
    }
}

// ---- Provider implementations ----

async fn generate_openai(
    prompt: &str,
    config: &AiConfig,
) -> Result<Vec<GeneratedIcon>, AppError> {
    let client = build_client(config.timeout_secs)?;

    let model = if config.model.is_empty() {
        "gpt-4o".to_string()
    } else {
        config.model.clone()
    };

    let request = OpenAiImageRequest {
        model,
        prompt: prompt.to_string(),
        n: 1,
        size: "1024x1024".to_string(),
        response_format: "b64_json".to_string(),
    };

    let resp = client
        .post("https://api.openai.com/v1/images/generations")
        .bearer_auth(&config.api_key)
        .json(&request)
        .send()
        .await
        .map_err(|e| AppError::BuildError(format!("OpenAI request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::BuildError(format!(
            "OpenAI API error {}: {}",
            status, body
        )));
    }

    let data: OpenAiImageResponse = resp
        .json()
        .await
        .map_err(|e| AppError::BuildError(format!("OpenAI parse failed: {}", e)))?;

    let icons: Vec<GeneratedIcon> = data
        .data
        .into_iter()
        .map(|item| {
            let image_data = item.b64_json.as_ref().map(|b| {
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b)
                    .unwrap_or_default()
            });
            GeneratedIcon {
                svg_content: None,
                image_data,
                prompt: prompt.to_string(),
            }
        })
        .collect();

    Ok(icons)
}

async fn generate_recraft(
    prompt: &str,
    style: IconStyle,
    config: &AiConfig,
) -> Result<Vec<GeneratedIcon>, AppError> {
    let client = build_client(config.timeout_secs)?;

    let request = RecraftVectorRequest {
        prompt: prompt.to_string(),
        image_size: "1024x1024".to_string(),
        style: Some(style_to_recraft_style(&style)),
    };

    let resp = client
        .post("https://api.recraft.ai/v1/images")
        .bearer_auth(&config.api_key)
        .json(&request)
        .send()
        .await
        .map_err(|e| AppError::BuildError(format!("Recraft request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::BuildError(format!(
            "Recraft API error {}: {}",
            status, body
        )));
    }

    let data: RecraftVectorResponse = resp
        .json()
        .await
        .map_err(|e| AppError::BuildError(format!("Recraft parse failed: {}", e)))?;

    let svg = data.image.unwrap_or_default();
    Ok(vec![GeneratedIcon {
        svg_content: Some(svg),
        image_data: None,
        prompt: prompt.to_string(),
    }])
}

async fn generate_custom(
    prompt: &str,
    config: &AiConfig,
) -> Result<Vec<GeneratedIcon>, AppError> {
    let endpoint = config
        .endpoint
        .as_ref()
        .ok_or_else(|| AppError::ValidationError("Custom endpoint URL is required".into()))?;

    let client = build_client(config.timeout_secs)?;

    let body = serde_json::json!({
        "prompt": prompt,
        "model": &config.model,
    });

    let resp = client
        .post(endpoint)
        .bearer_auth(&config.api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::BuildError(format!("Custom API request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::BuildError(format!(
            "Custom API error {}: {}",
            status, body
        )));
    }

    let text = resp
        .text()
        .await
        .map_err(|e| AppError::BuildError(format!("Custom API read failed: {}", e)))?;

    let has_svg = text.contains("<svg");
    Ok(vec![GeneratedIcon {
        svg_content: if has_svg { Some(text.clone()) } else { None },
        image_data: if has_svg {
            None
        } else {
            Some(text.into_bytes())
        },
        prompt: prompt.to_string(),
    }])
}

async fn generate_ollama(
    prompt: &str,
    config: &AiConfig,
) -> Result<Vec<GeneratedIcon>, AppError> {
    let base_url = config
        .endpoint
        .as_deref()
        .unwrap_or("http://localhost:11434");
    let url = format!("{}/api/generate", base_url);

    let client = build_client(config.timeout_secs)?;

    let model = if config.model.is_empty() {
        "llava".to_string()
    } else {
        config.model.clone()
    };

    let request = OllamaGenerateRequest {
        model,
        prompt: format!(
            "Generate an SVG icon. Only output the SVG code, nothing else.\n\n{}",
            prompt
        ),
        stream: false,
    };

    let resp = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| AppError::BuildError(format!("Ollama request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::BuildError(format!(
            "Ollama API error {}: {}",
            status, body
        )));
    }

    let data: OllamaGenerateResponse = resp
        .json()
        .await
        .map_err(|e| AppError::BuildError(format!("Ollama parse failed: {}", e)))?;

    let text = data.response;
    let svg = extract_svg(&text);

    Ok(vec![GeneratedIcon {
        svg_content: svg,
        image_data: None,
        prompt: prompt.to_string(),
    }])
}

// ---- Helpers ----

fn build_client(timeout_secs: u64) -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| AppError::BuildError(format!("HTTP client build failed: {}", e)))
}

fn build_prompt(task: AiTask, style: IconStyle, user_prompt: &str) -> String {
    let style_desc = style_to_description(&style);
    match task {
        AiTask::TextToIcon => {
            format!(
                "Create a single icon in {} style. \
                 The icon should be simple, clean, and suitable for use as an app icon or UI element. \
                 Output only the SVG code with a viewBox of '0 0 512 512'.\n\n{}",
                style_desc, user_prompt
            )
        }
        AiTask::SketchToIcon => {
            format!(
                "Convert this sketch into a polished icon in {} style. \
                 Output only the SVG code with a viewBox of '0 0 512 512'.\n\n{}",
                style_desc, user_prompt
            )
        }
        AiTask::StyleTransfer => {
            format!(
                "Transform this icon into {} style while preserving its core shape and meaning. \
                 Output only the SVG code with a viewBox of '0 0 512 512'.\n\n{}",
                style_desc, user_prompt
            )
        }
        AiTask::VaryIcon => {
            format!(
                "Create a variation of this icon in {} style, keeping the same concept but with different visual treatment. \
                 Output only the SVG code with a viewBox of '0 0 512 512'.\n\n{}",
                style_desc, user_prompt
            )
        }
        AiTask::RemoveBackground => user_prompt.to_string(),
    }
}

fn style_to_description(style: &IconStyle) -> &'static str {
    match style {
        IconStyle::Flat => "flat solid-color",
        IconStyle::Outline => "outline stroke-based",
        IconStyle::Duotone => "duotone two-tone",
        IconStyle::Gradient => "gradient-filled",
        IconStyle::ThreeD => "3D isometric",
        IconStyle::Minimal => "minimal line",
        IconStyle::Cartoon => "cartoon illustration",
        IconStyle::PixelArt => "pixel art",
        IconStyle::LineArt => "line art",
        IconStyle::Neon => "neon glow",
    }
}

fn style_to_recraft_style(style: &IconStyle) -> String {
    match style {
        IconStyle::Flat => "flat".into(),
        IconStyle::Outline => "outline".into(),
        IconStyle::Duotone => "duotone".into(),
        IconStyle::Gradient => "gradient".into(),
        IconStyle::ThreeD => "3d".into(),
        IconStyle::Minimal => "minimal".into(),
        IconStyle::Cartoon => "cartoon".into(),
        IconStyle::PixelArt => "pixel_art".into(),
        IconStyle::LineArt => "line_art".into(),
        IconStyle::Neon => "neon".into(),
    }
}

fn extract_svg(text: &str) -> Option<String> {
    if let Some(start) = text.find("<svg") {
        if let Some(end) = text.rfind("</svg>") {
            return Some(text[start..end + 6].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_config_default() {
        let config = AiConfig::default();
        assert!(matches!(config.provider, AiProvider::OpenAi));
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.timeout_secs, 60);
        assert!(config.api_key.is_empty());
    }

    #[test]
    fn test_ai_config_serde_roundtrip() {
        let config = AiConfig {
            provider: AiProvider::Ollama,
            api_key: "test-key".into(),
            model: "llava".into(),
            endpoint: Some("http://localhost:11434".into()),
            timeout_secs: 120,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AiConfig = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed.provider, AiProvider::Ollama));
        assert_eq!(parsed.api_key, "test-key");
        assert_eq!(parsed.model, "llava");
        assert_eq!(parsed.endpoint.unwrap(), "http://localhost:11434");
        assert_eq!(parsed.timeout_secs, 120);
    }

    #[test]
    fn test_generated_icon_serde() {
        let icon = GeneratedIcon {
            svg_content: Some("<svg></svg>".into()),
            image_data: None,
            prompt: "test".into(),
        };
        let json = serde_json::to_string(&icon).unwrap();
        let parsed: GeneratedIcon = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.svg_content.unwrap(), "<svg></svg>");
        assert!(parsed.image_data.is_none());
        assert_eq!(parsed.prompt, "test");
    }

    #[test]
    fn test_build_prompt_text_to_icon() {
        let prompt = build_prompt(
            AiTask::TextToIcon,
            IconStyle::Flat,
            "a home icon",
        );
        assert!(prompt.contains("flat solid-color"));
        assert!(prompt.contains("a home icon"));
        assert!(prompt.contains("viewBox"));
    }

    #[test]
    fn test_extract_svg() {
        let text = "Here is the icon:\n<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 512 512\"><rect/></svg>\nDone.";
        let svg = extract_svg(text).unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_extract_svg_not_found() {
        assert!(extract_svg("no svg here").is_none());
    }

    #[test]
    fn test_style_to_description() {
        assert_eq!(style_to_description(&IconStyle::Flat), "flat solid-color");
        assert_eq!(style_to_description(&IconStyle::Neon), "neon glow");
    }
}
