use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::IconStudioHandler;
use crate::engine::ai::{self, AiConfig};
use crate::model::{AiProvider, IconStyle};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiGenerateParams {
    #[schemars(description = "Text prompt describing the icon to generate")]
    pub prompt: String,
    #[schemars(description = "Visual style: Flat/Outline/Duotone/Gradient/ThreeD/Minimal/Cartoon/PixelArt/LineArt/Neon")]
    pub style: Option<String>,
    #[schemars(description = "AI provider: OpenAi/Recraft/Custom/Ollama")]
    pub provider: Option<String>,
    #[schemars(description = "API key for the provider")]
    pub api_key: Option<String>,
    #[schemars(description = "Model name")]
    pub model: Option<String>,
    #[schemars(description = "Custom endpoint URL")]
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiGenerateSetParams {
    #[schemars(description = "List of prompts for batch icon generation")]
    pub prompts: Vec<String>,
    #[schemars(description = "Visual style for all icons in the set")]
    pub style: Option<String>,
    #[schemars(description = "AI provider: OpenAi/Recraft/Custom/Ollama")]
    pub provider: Option<String>,
    #[schemars(description = "API key for the provider")]
    pub api_key: Option<String>,
    #[schemars(description = "Model name")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiStyleTransferParams {
    #[schemars(description = "Source SVG content to transfer style from")]
    pub source_svg: String,
    #[schemars(description = "Target style description")]
    pub target_style: String,
    #[schemars(description = "AI provider: OpenAi/Recraft/Custom/Ollama")]
    pub provider: Option<String>,
    #[schemars(description = "API key for the provider")]
    pub api_key: Option<String>,
    #[schemars(description = "Model name")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiRemoveBgParams {
    #[schemars(description = "Base64-encoded image data")]
    pub image_data: String,
    #[schemars(description = "AI provider: OpenAi/Recraft/Custom/Ollama")]
    pub provider: Option<String>,
    #[schemars(description = "API key for the provider")]
    pub api_key: Option<String>,
    #[schemars(description = "Model name")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiVectorizeParams {
    #[schemars(description = "Base64-encoded raster image to vectorize")]
    pub image_data: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiSuggestColorsParams {
    #[schemars(description = "Description of the icon or brand context")]
    pub context: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiCompleteIconParams {
    #[schemars(description = "Partial SVG content to complete")]
    pub partial_svg: String,
    #[schemars(description = "Description of what the complete icon should look like")]
    pub hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AiAnalyzeBrandParams {
    #[schemars(description = "Brand name or URL to analyze")]
    pub brand: String,
}

fn parse_provider(s: &Option<String>) -> AiProvider {
    match s.as_deref().unwrap_or("openAi") {
        "recraft" => AiProvider::Recraft,
        "custom" => AiProvider::Custom,
        "ollama" => AiProvider::Ollama,
        _ => AiProvider::OpenAi,
    }
}

fn parse_style(s: &Option<String>) -> IconStyle {
    match s.as_deref().unwrap_or("flat") {
        "outline" => IconStyle::Outline,
        "duotone" => IconStyle::Duotone,
        "gradient" => IconStyle::Gradient,
        "threeD" => IconStyle::ThreeD,
        "minimal" => IconStyle::Minimal,
        "cartoon" => IconStyle::Cartoon,
        "pixelArt" => IconStyle::PixelArt,
        "lineArt" => IconStyle::LineArt,
        "neon" => IconStyle::Neon,
        _ => IconStyle::Flat,
    }
}

fn build_config(params_provider: &Option<String>, params_key: &Option<String>, params_model: &Option<String>, params_endpoint: &Option<String>) -> Result<AiConfig, ErrorData> {
    let provider = parse_provider(params_provider);
    let api_key = match params_key {
        Some(k) if !k.is_empty() => k.clone(),
        _ => {
            match crate::services::ai_config::load_ai_config() {
                Ok(cfg) => cfg.api_key,
                Err(_) => String::new(),
            }
        }
    };
    let model = match params_model {
        Some(m) if !m.is_empty() => m.clone(),
        _ => {
            match crate::services::ai_config::load_ai_config() {
                Ok(cfg) => cfg.model,
                Err(_) => "gpt-4o".to_string(),
            }
        }
    };
    let endpoint = params_endpoint.clone();
    Ok(AiConfig {
        provider,
        api_key,
        model,
        endpoint,
        ..Default::default()
    })
}

fn missing_key_result(provider: &str) -> Result<CallToolResult, ErrorData> {
    Ok(CallToolResult::success(vec![Content::text(format!(
        "No API key configured for {}. Please set your API key first.",
        provider
    ))]))
}

#[tool_router(router = ai_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "ai_generate", description = "Generate an icon from a text prompt using AI")]
    async fn ai_generate(
        &self,
        Parameters(params): Parameters<AiGenerateParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let config = build_config(&params.provider, &params.api_key, &None, &params.endpoint)
            .map_err(super::internal_err)?;

        if config.api_key.is_empty() && !matches!(config.provider, AiProvider::Ollama) {
            return missing_key_result(&format!("{:?}", config.provider));
        }

        let style = parse_style(&params.style);
        match ai::generate_icon(crate::model::AiTask::TextToIcon, style, &params.prompt, &config).await {
            Ok(icons) => {
                let mut contents = vec![Content::text(format!("Generated {} icon(s)", icons.len()))];
                for (i, icon) in icons.iter().enumerate() {
                    if let Some(ref svg) = icon.svg_content {
                        contents.push(Content::text(format!("Icon {} (SVG):\n{}", i + 1, svg)));
                    } else if let Some(ref data) = icon.image_data {
                        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data);
                        contents.push(Content::text(format!("Icon {} (PNG base64, {} bytes)", i + 1, b64.len())));
                    }
                }
                Ok(CallToolResult::success(contents))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Generation failed: {}", e
            ))])),
        }
    }

    #[tool(name = "ai_generate_set", description = "Generate a batch of icons from multiple prompts")]
    async fn ai_generate_set(
        &self,
        Parameters(params): Parameters<AiGenerateSetParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let config = build_config(&params.provider, &params.api_key, &params.model, &None)
            .map_err(super::internal_err)?;

        if config.api_key.is_empty() && !matches!(config.provider, AiProvider::Ollama) {
            return missing_key_result(&format!("{:?}", config.provider));
        }

        let style = parse_style(&params.style);
        match ai::generate_icon_set(&params.prompts, style, &config).await {
            Ok(icons) => {
                let mut contents = vec![Content::text(format!("Batch generated {} icon(s)", icons.len()))];
                for (i, icon) in icons.iter().enumerate() {
                    if let Some(ref svg) = icon.svg_content {
                        contents.push(Content::text(format!("Icon {} (SVG):\n{}", i + 1, svg)));
                    } else if icon.image_data.is_some() {
                        contents.push(Content::text(format!("Icon {} (PNG)", i + 1)));
                    }
                }
                Ok(CallToolResult::success(contents))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Batch generation failed: {}", e
            ))])),
        }
    }

    #[tool(name = "ai_style_transfer", description = "Transfer the style of one icon to another")]
    async fn ai_style_transfer(
        &self,
        Parameters(params): Parameters<AiStyleTransferParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let config = build_config(&params.provider, &params.api_key, &params.model, &None)
            .map_err(super::internal_err)?;

        if config.api_key.is_empty() && !matches!(config.provider, AiProvider::Ollama) {
            return missing_key_result(&format!("{:?}", config.provider));
        }

        let combined_prompt = format!(
            "Transform this SVG icon to {} style:\n\n{}",
            params.target_style, params.source_svg
        );

        match ai::generate_icon(
            crate::model::AiTask::StyleTransfer,
            IconStyle::Flat,
            &combined_prompt,
            &config,
        )
        .await
        {
            Ok(icons) => {
                let mut contents = vec![Content::text(format!("Style transferred {} icon(s)", icons.len()))];
                for icon in &icons {
                    if let Some(ref svg) = icon.svg_content {
                        contents.push(Content::text(svg.clone()));
                    }
                }
                Ok(CallToolResult::success(contents))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Style transfer failed: {}", e
            ))])),
        }
    }

    #[tool(name = "ai_remove_bg", description = "Remove background from a raster image using AI")]
    async fn ai_remove_bg(
        &self,
        Parameters(params): Parameters<AiRemoveBgParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let config = build_config(&params.provider, &params.api_key, &params.model, &None)
            .map_err(super::internal_err)?;

        if config.api_key.is_empty() {
            return missing_key_result(&format!("{:?}", config.provider));
        }

        let bytes = match base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            params.image_data,
        ) {
            Ok(b) => b,
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Invalid base64: {}", e
                ))]))
            }
        };

        match ai::remove_background(&bytes, &config).await {
            Ok(result) => {
                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result);
                Ok(CallToolResult::success(vec![
                    Content::text("Background removed successfully"),
                    Content::text(format!("data:image/png;base64,{}", b64)),
                ]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Background removal failed: {}", e
            ))])),
        }
    }

    #[tool(name = "ai_vectorize", description = "Convert a raster image to vector SVG by embedding as base64 image element")]
    async fn ai_vectorize(
        &self,
        Parameters(params): Parameters<AiVectorizeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <image href="data:image/png;base64,{}" width="512" height="512"/>
</svg>"#,
            params.image_data
        );
        Ok(CallToolResult::success(vec![
            Content::text("Image embedded as SVG (bitmap-in-SVG wrapper)"),
            Content::text(svg),
        ]))
    }

    #[tool(name = "ai_suggest_colors", description = "Suggest a color palette based on icon or brand context")]
    async fn ai_suggest_colors(
        &self,
        Parameters(params): Parameters<AiSuggestColorsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(super::state_err)?;

        let mut existing_colors: Vec<String> = Vec::new();
        for elem in project.active_elements() {
            match elem {
                crate::model::Element::Shape(s) => existing_colors.push(s.fill.clone()),
                crate::model::Element::Text(t) => existing_colors.push(t.fill.clone()),
                _ => {}
            }
        }

        let mut suggestions = Vec::new();

        if !existing_colors.is_empty() {
            suggestions.push(format!("Based on existing canvas colors ({:?}):", &existing_colors[..3.min(existing_colors.len())]));
        }

        let palettes: &[(&str, &[&str])] = &[
            ("Warm & Vibrant", &["#FF6B6B", "#FFE66D", "#4ECDC4", "#1A535C", "#F7FFF7"]),
            ("Cool & Professional", &["#2B2D42", "#8D99AE", "#EDF2F4", "#EF233C", "#D90429"]),
            ("Nature & Earth", &["#606C38", "#283618", "#FEFAE0", "#DDA15E", "#BC6C25"]),
            ("Bold & Modern", &["#264653", "#2A9D8F", "#E9C46A", "#F4A261", "#E76F51"]),
        ];

        suggestions.push(format!("Context: {}", params.context));
        for (name, colors) in palettes {
            suggestions.push(format!("{}: {}", name, colors.join(", ")));
        }

        Ok(CallToolResult::success(vec![Content::text(suggestions.join("\n"))]))
    }

    #[tool(name = "ai_complete_icon", description = "Complete a partial icon using AI")]
    async fn ai_complete_icon(
        &self,
        Parameters(params): Parameters<AiCompleteIconParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let config = match crate::services::ai_config::load_ai_config() {
            Ok(cfg) => cfg,
            Err(_) => return Ok(CallToolResult::success(vec![Content::text(
                "No AI configuration found. Please configure AI first.".to_string(),
            )])),
        };

        if config.api_key.is_empty() && !matches!(config.provider, AiProvider::Ollama) {
            return missing_key_result(&format!("{:?}", config.provider));
        }

        let prompt = format!(
            "Complete this partial SVG icon based on the hint. Output only the full SVG.\n\nHint: {}\n\nPartial SVG:\n{}",
            params.hint, params.partial_svg
        );

        match ai::generate_icon(
            crate::model::AiTask::VaryIcon,
            IconStyle::Flat,
            &prompt,
            &config,
        )
        .await
        {
            Ok(icons) => {
                let mut contents = vec![Content::text("Icon completion result")];
                for icon in &icons {
                    if let Some(ref svg) = icon.svg_content {
                        contents.push(Content::text(svg.clone()));
                    }
                }
                Ok(CallToolResult::success(contents))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Completion failed: {}", e
            ))])),
        }
    }

    #[tool(name = "ai_analyze_brand", description = "Analyze a brand's visual identity and extract icon guidelines")]
    async fn ai_analyze_brand(
        &self,
        Parameters(params): Parameters<AiAnalyzeBrandParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(super::state_err)?;

        let mut analysis = vec![format!("Brand analysis for: {}", params.brand)];

        if !project.brand_kits.is_empty() {
            analysis.push("Found brand kits in project:".into());
            for kit in &project.brand_kits {
                analysis.push(format!("  - {} ({} colors)", kit.name, kit.colors.len()));
            }
        }

        let canvas_colors: Vec<String> = project
            .active_elements()
            .iter()
            .filter_map(|e| match e {
                crate::model::Element::Shape(s) => Some(s.fill.clone()),
                crate::model::Element::Text(t) => Some(t.fill.clone()),
                _ => None,
            })
            .collect();

        if !canvas_colors.is_empty() {
            analysis.push(format!("Current canvas colors: {:?}", canvas_colors));
        }

        analysis.push(format!(
            "Suggested icon style guidelines for '{}':",
            params.brand
        ));
        analysis.push("  - Use consistent stroke width (1.5-2px for outline, none for flat)".into());
        analysis.push("  - Maintain 2px corner radius for rounded style".into());
        analysis.push("  - Limit palette to 2-3 primary colors".into());

        Ok(CallToolResult::success(vec![Content::text(analysis.join("\n"))]))
    }
}
