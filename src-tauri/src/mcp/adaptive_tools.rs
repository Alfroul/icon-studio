use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{internal_err, state_err, IconStudioHandler};
use crate::engine::adaptive;
use crate::model::{AdaptiveConfig, AdaptiveShape};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PreviewAdaptiveParams {
    #[schemars(description = "Adaptive shape: circle, squircle, rounded-rect, pill, square")]
    pub shape: String,
    #[schemars(description = "Preview size in pixels")]
    #[serde(default = "default_512")]
    pub size: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetForegroundLayerParams {
    #[schemars(description = "Element IDs to mark as foreground layer")]
    pub element_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetBackgroundLayerParams {
    #[schemars(description = "Element IDs to mark as background layer")]
    pub element_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CheckSafeZoneParams {
    #[schemars(description = "Safe zone margin percentage (default 34 = Android standard 66% safe zone)")]
    #[serde(default = "default_34")]
    pub margin_percent: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportAdaptiveAndroidParams {
    #[schemars(description = "Output directory for exported files")]
    pub output_dir: String,
}

fn default_512() -> u32 { 512 }
fn default_34() -> f64 { 34.0 }

fn parse_shape(s: &str) -> Result<AdaptiveShape, ErrorData> {
    match s {
        "circle" => Ok(AdaptiveShape::Circle),
        "squircle" => Ok(AdaptiveShape::Squircle),
        "rounded-rect" => Ok(AdaptiveShape::RoundedRect),
        "pill" => Ok(AdaptiveShape::Pill),
        "square" => Ok(AdaptiveShape::Square),
        _ => Err(ErrorData::invalid_params(
            format!("Invalid shape '{}'. Must be one of: circle, squircle, rounded-rect, pill, square", s),
            None,
        )),
    }
}

#[tool_router(router = adaptive_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "preview_adaptive", description = "Preview how the icon looks in different adaptive shapes (circle, squircle, rounded-rect, pill, square). Returns base64 PNG.")]
    async fn preview_adaptive(
        &self,
        Parameters(params): Parameters<PreviewAdaptiveParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let shape = parse_shape(&params.shape)?;
        let project = self.project.lock().map_err(state_err)?;
        let b64 = adaptive::render_adaptive_base64(&project, shape, params.size)
            .map_err(internal_err)?;
        Ok(CallToolResult::success(vec![
            Content::image(
                format!("data:image/png;base64,{}", b64),
                "image/png",
            ),
        ]))
    }

    #[tool(name = "set_foreground_layer", description = "Mark elements as the foreground layer for adaptive icons. Unmarked elements are not automatically assigned to background.")]
    async fn set_foreground_layer(
        &self,
        Parameters(params): Parameters<SetForegroundLayerParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let count;
        {
            let mut project = self.project.lock().map_err(state_err)?;
            let config = project.adaptive.get_or_insert_with(|| AdaptiveConfig {
                foreground_ids: Vec::new(),
                background_ids: Vec::new(),
            });
            config.foreground_ids = params.element_ids;
            count = config.foreground_ids.len();
            project.bump_version();
        }
        self.emit_change();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Set {} element(s) as foreground layer", count
        ))]))
    }

    #[tool(name = "set_background_layer", description = "Mark elements as the background layer for adaptive icons.")]
    async fn set_background_layer(
        &self,
        Parameters(params): Parameters<SetBackgroundLayerParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let count;
        {
            let mut project = self.project.lock().map_err(state_err)?;
            let config = project.adaptive.get_or_insert_with(|| AdaptiveConfig {
                foreground_ids: Vec::new(),
                background_ids: Vec::new(),
            });
            config.background_ids = params.element_ids;
            count = config.background_ids.len();
            project.bump_version();
        }
        self.emit_change();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Set {} element(s) as background layer", count
        ))]))
    }

    #[tool(name = "check_safe_zone", description = "Check which elements fall outside the adaptive icon safe zone. Default margin=34 (Android standard 66% safe zone).")]
    async fn check_safe_zone(
        &self,
        Parameters(params): Parameters<CheckSafeZoneParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;
        let result = adaptive::check_safe_zone(&project, params.margin_percent);
        let json = serde_json::to_string_pretty(&result)
            .map_err(internal_err)?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "export_adaptive_android", description = "Export adaptive icon assets for Android (foreground + background PNGs + mipmap directory structure).")]
    async fn export_adaptive_android(
        &self,
        Parameters(params): Parameters<ExportAdaptiveAndroidParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;
        let result = adaptive::export_adaptive_layers(&project, &params.output_dir)
            .map_err(internal_err)?;
        let file_list = result.files.join("\n");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Exported {} files:\n{}", result.files.len(), file_list
        ))]))
    }
}
