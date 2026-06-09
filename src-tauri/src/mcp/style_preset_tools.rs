use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::engine::soft3d::apply_style_preset;
use crate::engine::builder;
use crate::engine::renderer;
use base64::Engine;
use crate::model::style_preset::{CustomStylePreset, StyleParams, StylePreset, StyleType};
use crate::model::history::SnapshotCommand;

use super::{invalid_params, state_err, IconStudioHandler};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ApplyStyleParams {
    #[schemars(description = "Element ID to apply the style to")]
    pub element_id: String,
    #[schemars(description = "Style type: soft-3d, neumorphism, glassmorphism, flat")]
    pub style_type: String,
    #[schemars(description = "Style parameters (depth, light_angle, highlight, shadow_softness)")]
    #[serde(default)]
    pub params: Option<StyleParams>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PreviewStyleParams {
    #[schemars(description = "Element ID to preview the style on")]
    pub element_id: String,
    #[schemars(description = "Style type: soft-3d, neumorphism, glassmorphism, flat")]
    pub style_type: String,
    #[schemars(description = "Style parameters")]
    #[serde(default)]
    pub params: Option<StyleParams>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SaveStylePresetParams {
    #[schemars(description = "Name for the saved preset")]
    pub name: String,
    #[schemars(description = "Style type: soft-3d, neumorphism, glassmorphism, flat")]
    pub style_type: String,
    #[schemars(description = "Style parameters")]
    #[serde(default)]
    pub params: Option<StyleParams>,
}

fn parse_style_type(s: &str) -> Result<StyleType, ErrorData> {
    match s {
        "soft-3d" => Ok(StyleType::Soft3d),
        "neumorphism" => Ok(StyleType::Neumorphism),
        "glassmorphism" => Ok(StyleType::Glassmorphism),
        "flat" => Ok(StyleType::Flat),
        other => Err(invalid_params(format!(
            "Invalid style type: {}. Valid: soft-3d, neumorphism, glassmorphism, flat",
            other
        ))),
    }
}

#[tool_router(router = style_preset_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "apply_style", description = "Apply a style preset (soft-3d, neumorphism, glassmorphism, flat) to an element")]
    async fn apply_style(
        &self,
        Parameters(params): Parameters<ApplyStyleParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let style_type = parse_style_type(&params.style_type)?;
        let preset = StylePreset {
            style_type,
            params: params.params.unwrap_or_default(),
        };

        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        apply_style_preset(&mut project, &params.element_id, &preset)
            .map_err(invalid_params)?;

        project.bump_version();
        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(
                before,
                before_next_id,
                after,
                project.next_element_id,
            )));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Applied {} style to '{}'",
            params.style_type, params.element_id
        ))]))
    }

    #[tool(name = "preview_style", description = "Preview a style preset on an element without permanently applying it")]
    async fn preview_style(
        &self,
        Parameters(params): Parameters<PreviewStyleParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let style_type = parse_style_type(&params.style_type)?;
        let preset = StylePreset {
            style_type,
            params: params.params.unwrap_or_default(),
        };

        // Clone project, apply style temporarily, render preview
        let project = self.project.lock().map_err(state_err)?;
        let mut temp_project = project.clone();
        drop(project);

        apply_style_preset(&mut temp_project, &params.element_id, &preset)
            .map_err(invalid_params)?;

        let svg_str = builder::build(&temp_project)
            .map_err(|e| super::internal_err(format!("Build error: {}", e)))?;
        let png_bytes = renderer::render(&svg_str, 256)
            .map_err(|e| super::internal_err(format!("Render error: {}", e)))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

        Ok(CallToolResult::success(vec![Content::image(
            format!("data:image/png;base64,{}", b64),
            "image/png",
        )]))
    }

    #[tool(name = "list_styles", description = "List all available style presets and saved custom presets")]
    async fn list_styles(&self) -> Result<CallToolResult, ErrorData> {
        let mut lines = vec![
            "Built-in style presets:".to_string(),
            "  soft-3d: 3D raised effect (shadow + gradient overlay + bright stroke)".to_string(),
            "    params: depth=5, light_angle=135, highlight=0.3, shadow_softness=8".to_string(),
            "  neumorphism: Soft UI / embossed effect (dual shadows)".to_string(),
            "    params: depth=5, light_angle=135, highlight=0.3, shadow_softness=8".to_string(),
            "  glassmorphism: Frosted glass effect (blur + transparency + white stroke)".to_string(),
            "    params: depth=5, light_angle=135, highlight=0.3, shadow_softness=8".to_string(),
            "  flat: Clear all effects (no parameters)".to_string(),
        ];

        let project = self.project.lock().map_err(state_err)?;
        if !project.custom_style_presets.is_empty() {
            lines.push("Custom presets:".to_string());
            for p in &project.custom_style_presets {
                let st = serde_json::to_string(&p.style_type).unwrap_or_default().trim_matches('"').to_string();
                lines.push(format!(
                    "  {} ({}) — depth={}, light_angle={}, highlight={}, shadow_softness={}",
                    p.name, st, p.params.depth, p.params.light_angle,
                    p.params.highlight, p.params.shadow_softness
                ));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(lines.join("\n"))]))
    }

    #[tool(name = "save_style_preset", description = "Save a custom style preset for reuse")]
    async fn save_style_preset(
        &self,
        Parameters(params): Parameters<SaveStylePresetParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let style_type = parse_style_type(&params.style_type)?;
        let custom = CustomStylePreset {
            name: params.name.clone(),
            style_type,
            params: params.params.unwrap_or_default(),
        };

        let mut project = self.project.lock().map_err(state_err)?;

        // Replace existing preset with same name or append
        if let Some(existing) = project.custom_style_presets.iter_mut().find(|p| p.name == params.name) {
            *existing = custom;
        } else {
            project.custom_style_presets.push(custom);
        }
        project.bump_version();
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Saved style preset '{}'", params.name
        ))]))
    }
}
