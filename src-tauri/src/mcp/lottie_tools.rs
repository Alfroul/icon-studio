use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::{internal_err, state_err, IconStudioHandler};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportLottieParams {
    #[schemars(description = "Output file path for the Lottie JSON")]
    #[serde(default)]
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportAnimatedGifParams {
    #[schemars(description = "Output file path for the GIF")]
    pub output_path: String,
    #[schemars(description = "Frames per second")]
    #[serde(default = "default_15")]
    pub fps: u32,
    #[schemars(description = "Output width in pixels")]
    #[serde(default = "default_512")]
    pub width: u32,
    #[schemars(description = "Output height in pixels")]
    #[serde(default = "default_512")]
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PreviewAnimationParams {
    #[schemars(description = "Time position in milliseconds (default: 0 = first frame)")]
    #[serde(default)]
    pub time_ms: f64,
}

fn default_15() -> u32 {
    15
}
fn default_512() -> u32 {
    512
}

#[tool_router(router = lottie_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "export_lottie", description = "Export the project's animations as Lottie JSON")]
    async fn export_lottie(
        &self,
        Parameters(params): Parameters<ExportLottieParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let json = {
            let project = self.project.lock().map_err(state_err)?;
            crate::engine::lottie::export_lottie(&project)
                .map_err(|e| internal_err(format!("Lottie export error: {}", e)))?
        };

        if let Some(ref path) = params.output_path {
            crate::engine::utils::validate_file_path(path)
                .map_err(|e| ErrorData::invalid_params(format!("Invalid path: {}", e), None))?;
            if let Some(parent) = Path::new(path).parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| internal_err(format!("Failed to create directory: {}", e)))?;
                }
            }
            std::fs::write(path, &json)
                .map_err(|e| internal_err(format!("Failed to write Lottie JSON: {}", e)))?;
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Lottie JSON exported to '{}'",
                path
            ))]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(json)]))
        }
    }

    #[tool(name = "export_animated_gif", description = "Export the project's animation as an animated GIF")]
    async fn export_animated_gif(
        &self,
        Parameters(params): Parameters<ExportAnimatedGifParams>,
    ) -> Result<CallToolResult, ErrorData> {
        crate::engine::utils::validate_file_path(&params.output_path)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid path: {}", e), None))?;

        if params.fps == 0 || params.fps > 60 {
            return Err(ErrorData::invalid_params("FPS must be between 1 and 60".to_string(), None));
        }
        if params.width == 0 || params.width > 2048 || params.height == 0 || params.height > 2048 {
            return Err(ErrorData::invalid_params("Width and height must be between 1 and 2048".to_string(), None));
        }

        let result = {
            let project = self.project.lock().map_err(state_err)?;
            crate::engine::lottie::export_gif(
                &project,
                params.fps,
                params.width,
                params.height,
                Path::new(&params.output_path),
            )
            .map_err(|e| internal_err(format!("GIF export error: {}", e)))?
        };

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Animated GIF exported to '{}' ({}x{}, {}fps)",
            result.to_string_lossy(),
            params.width,
            params.height,
            params.fps
        ))]))
    }

    #[tool(name = "preview_animation", description = "Preview animation at a specific time, returns PNG frame as base64")]
    async fn preview_animation(
        &self,
        Parameters(params): Parameters<PreviewAnimationParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let png_bytes = {
            let project = self.project.lock().map_err(state_err)?;
            crate::engine::lottie::preview_frame(&project, params.time_ms)
                .map_err(|e| internal_err(format!("Preview render error: {}", e)))?
        };

        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes);
        Ok(CallToolResult::success(vec![
            Content::image(&format!("data:image/png;base64,{}", b64), "image/png"),
            Content::text(format!("Frame at {}ms", params.time_ms)),
        ]))
    }
}
