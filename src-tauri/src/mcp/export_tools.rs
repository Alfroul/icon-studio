use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::{internal_err, invalid_params, state_err, IconStudioHandler};
use crate::engine::exporter;
use crate::engine::variation::{self, VariationConfig};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportWebpParams {
    #[schemars(description = "Export size in pixels")]
    pub size: u32,
    #[schemars(description = "WebP output file path")]
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportIcoParams {
    #[schemars(description = "ICO sizes to include")]
    pub sizes: Vec<u32>,
    #[schemars(description = "ICO output file path")]
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportAllParams {
    #[schemars(description = "Output directory for all exported files")]
    pub output_dir: String,
    #[schemars(description = "Export formats (e.g. [\"svg\",\"png\",\"ico\"]). Default: all")]
    pub formats: Option<Vec<String>>,
}

fn default_app_name() -> String {
    "My App".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportFaviconParams {
    #[schemars(description = "Output directory (generates favicon.ico, multi-size PNGs, SVG, apple-touch-icon, site.webmanifest)")]
    pub output_dir: String,
    #[schemars(description = "App name (written to site.webmanifest)")]
    #[serde(default = "default_app_name")]
    pub app_name: String,
    #[schemars(description = "Theme color (hex, for site.webmanifest)")]
    pub theme_color: Option<String>,
    #[schemars(description = "Background color (hex, for site.webmanifest)")]
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SaveProjectParams {
    #[schemars(description = "Output file path. Default: 'project.iconproject.json'")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct OpenProjectParams {
    #[schemars(description = ".iconproject.json or .svg file path")]
    pub path: String,
}

#[tool_router(router = export_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "export_ico", description = "Export current icon as ICO file")]
    async fn export_ico(
        &self,
        Parameters(params): Parameters<ExportIcoParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let svg_str = {
            let project = self.project.lock().map_err(state_err)?;
            let mut cache = self.cache.lock().map_err(state_err)?;
            crate::services::export::build_svg(&project, &mut cache)
                .map_err(|e| internal_err(format!("Build error: {}", e)))?
        };

        if params.sizes.is_empty() {
            return Err(invalid_params("At least one export size required"));
        }
        for &size in &params.sizes {
            if size == 0 || size > 8192 {
                return Err(invalid_params(format!("Export size {} invalid, range: 1-8192", size)));
            }
        }

        let result = crate::services::export::write_ico_to_file(&svg_str, &params.sizes, &params.path)
            .map_err(|e| internal_err(format!("ICO export error: {}", e)))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "ICO exported to '{}'",
            result
        ))]))
    }

    #[tool(name = "export_all", description = "Batch export in multiple formats")]
    async fn export_all(
        &self,
        Parameters(params): Parameters<ExportAllParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let formats = params
            .formats
            .unwrap_or_else(|| vec!["svg".into(), "png".into(), "ico".into()]);

        let svg_str = {
            let project = self.project.lock().map_err(state_err)?;
            let mut cache = self.cache.lock().map_err(state_err)?;
            crate::services::export::build_svg(&project, &mut cache)
                .map_err(|e| internal_err(format!("Build error: {}", e)))?
        };

        let all_paths = crate::services::export::export_all_formats(
            &svg_str, &params.output_dir, &formats, &[16, 32, 64, 128, 256, 512],
        ).map_err(|e| internal_err(format!("Batch export error: {}", e)))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Exported {} files: {}",
            all_paths.len(),
            all_paths.join(", ")
        ))]))
    }

    #[tool(name = "export_favicon_package", description = "Export complete favicon package (ICO, PNGs, SVG, webmanifest)")]
    async fn export_favicon_package(
        &self,
        Parameters(params): Parameters<ExportFaviconParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let svg_str = {
            let project = self.project.lock().map_err(state_err)?;
            let mut cache = self.cache.lock().map_err(state_err)?;
            crate::services::export::build_svg(&project, &mut cache)
                .map_err(|e| internal_err(format!("Build error: {}", e)))?
        };

        crate::engine::utils::validate_file_path(&params.output_dir)
            .map_err(invalid_params)?;
        let dir = Path::new(&params.output_dir);
        let paths = exporter::export_favicon_package(
            &svg_str,
            dir,
            &params.app_name,
            params.theme_color.as_deref(),
            params.background_color.as_deref(),
        )
        .map_err(|e| internal_err(format!("Favicon export error: {}", e)))?;

        let file_names: Vec<String> = paths
            .iter()
            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
            .collect();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Exported favicon package ({} files):\n{}",
            paths.len(),
            file_names.join("\n")
        ))]))
    }

    #[tool(name = "export_webp", description = "Export current icon as WebP file")]
    async fn export_webp(
        &self,
        Parameters(params): Parameters<ExportWebpParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let svg_str = {
            let project = self.project.lock().map_err(state_err)?;
            let mut cache = self.cache.lock().map_err(state_err)?;
            crate::services::export::build_svg(&project, &mut cache)
                .map_err(|e| internal_err(format!("Build error: {}", e)))?
        };

        if params.size == 0 || params.size > 8192 {
            return Err(invalid_params(format!("Export size {} invalid, range: 1-8192", params.size)));
        }

        crate::engine::utils::validate_file_path(&params.path)
            .map_err(invalid_params)?;

        let result = crate::engine::exporter::export_webp(&svg_str, params.size, Path::new(&params.path))
            .map_err(|e| internal_err(format!("WebP export error: {}", e)))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "WebP exported to '{}'",
            result.to_string_lossy()
        ))]))
    }

    #[tool(name = "save_project", description = "Save current project as .iconproject.json")]
    async fn save_project(
        &self,
        Parameters(params): Parameters<SaveProjectParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let (json, file_path) = {
            let project = self.project.lock().map_err(state_err)?;
            let json = serde_json::to_string_pretty(&*project)
                .map_err(|e| internal_err(format!("Serialization error: {}", e)))?;
            let fp = match params.path {
                Some(ref p) => {
                    crate::engine::utils::validate_file_path(p)
                        .map_err(|e| invalid_params(format!("Invalid file path: {}", e)))?;
                    if p.ends_with(".iconproject.json") {
                        p.clone()
                    } else if p.ends_with(".json") {
                        format!("{}.iconproject.json", &p[..p.len() - 5])
                    } else {
                        format!("{}.iconproject.json", p)
                    }
                }
                None => "project.iconproject.json".to_string(),
            };
            (json, fp)
        };

        if let Some(parent) = Path::new(&file_path).parent() {
            if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)
                    .map_err(|e| internal_err(format!("Failed to create directory: {}", e)))?;
            }
        }

        std::fs::write(&file_path, json)
            .map_err(|e| internal_err(format!("Failed to write project: {}", e)))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Project saved to '{}'",
            file_path
        ))]))
    }

    #[tool(name = "open_project", description = "Open project from .iconproject.json or .svg file")]
    async fn open_project(
        &self,
        Parameters(params): Parameters<OpenProjectParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let loaded = crate::services::project::load_project_auto(&params.path)
            .map_err(|e| internal_err(format!("Failed to open project '{}': {}", params.path, e)))?;

        let (w, h, count) = {
            let mut project = self.project.lock().map_err(state_err)?;
            *project = loaded;
            project.recalc_next_element_id();
            project.bump_version();
            let w = project.active_canvas().width;
            let h = project.active_canvas().height;
            let count = project.active_elements().len();
            (w, h, count)
        };

        {
            let mut cache = self.cache.lock().map_err(state_err)?;
            cache.invalidate_cache();
        }
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.clear();
        }
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Opened project: {}x{}, {} elements",
            w, h, count
        ))]))
    }

    #[tool(name = "generate_variations", description = "Generate design variations from current project using transforms (recolor, background, corner_radius, scale, opacity)")]
    async fn generate_variations(
        &self,
        Parameters(params): Parameters<GenerateVariationsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let config: VariationConfig = serde_json::from_value(serde_json::to_value(&params.config).map_err(|e| invalid_params(format!("Invalid config: {}", e)))?)
            .map_err(|e| invalid_params(format!("Invalid config: {}", e)))?;

        if config.variations.is_empty() {
            return Err(invalid_params("At least one variation required"));
        }

        let svg_map = {
            let project = self.project.lock().map_err(state_err)?;
            let variations = variation::generate_variations(&project, &config);
            let mut results = Vec::new();
            for (name, var_project) in &variations {
                let svg = crate::engine::builder::build(var_project)
                    .map_err(|e| internal_err(format!("Build error for '{}': {}", name, e)))?;
                results.push((name.clone(), svg));
            }
            results
        };

        let output_dir = Path::new(&config.output_dir);
        std::fs::create_dir_all(output_dir)
            .map_err(|e| internal_err(format!("Failed to create output dir: {}", e)))?;

        let stem = "icon";
        let naming = config.naming.clone();
        let mut exported_paths = Vec::new();

        for (var_name, svg) in &svg_map {
            let out_name = naming
                .replace("{name}", stem)
                .replace("{variation}", var_name);

            let path = output_dir.join(format!("{}.svg", out_name));
            std::fs::write(&path, svg)
                .map_err(|e| internal_err(format!("Write error: {}", e)))?;
            exported_paths.push(path.to_string_lossy().into_owned());

            let png_bytes = exporter::render_to_png(svg, 512)
                .map_err(|e| internal_err(format!("Render error for '{}': {}", var_name, e)))?;
            let png_path = output_dir.join(format!("{}_512.png", out_name));
            std::fs::write(&png_path, &png_bytes)
                .map_err(|e| internal_err(format!("Write error: {}", e)))?;
            exported_paths.push(png_path.to_string_lossy().into_owned());
        }

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Generated {} variations ({} files):\n{}",
            svg_map.len(),
            exported_paths.len(),
            exported_paths.join("\n")
        ))]))
    }

    #[tool(name = "preset_variations", description = "Generate variations using a preset template (dark_mode, sizes_16_32_64, sizes_128_256_512)")]
    async fn preset_variations(
        &self,
        Parameters(params): Parameters<PresetVariationsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let config = match params.preset.as_str() {
            "dark_mode" => variation::dark_mode_variation(),
            "sizes_16_32_64" => variation::size_variations(&[16, 32, 64]),
            "sizes_128_256_512" => variation::size_variations(&[128, 256, 512]),
            other => {
                return Err(invalid_params(format!(
                    "Unknown preset '{}'. Available: dark_mode, sizes_16_32_64, sizes_128_256_512",
                    other
                )));
            }
        };

        let output_dir = params.output_dir.unwrap_or_else(|| "./variations".into());
        let mut config = config;
        config.output_dir = output_dir;

        let svg_map = {
            let project = self.project.lock().map_err(state_err)?;
            let variations = variation::generate_variations(&project, &config);
            let mut results = Vec::new();
            for (name, var_project) in &variations {
                let svg = crate::engine::builder::build(var_project)
                    .map_err(|e| internal_err(format!("Build error for '{}': {}", name, e)))?;
                results.push((name.clone(), svg));
            }
            results
        };

        let dir = Path::new(&config.output_dir);
        std::fs::create_dir_all(dir)
            .map_err(|e| internal_err(format!("Failed to create dir: {}", e)))?;

        let mut exported = Vec::new();
        for (var_name, svg) in &svg_map {
            let path = dir.join(format!("{}-{}.svg", "icon", var_name));
            std::fs::write(&path, svg)
                .map_err(|e| internal_err(format!("Write error: {}", e)))?;
            exported.push(path.to_string_lossy().into_owned());
        }

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Preset '{}' generated {} variations:\n{}",
            params.preset,
            svg_map.len(),
            exported.join("\n")
        ))]))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GenerateVariationsParams {
    #[schemars(description = "Variation configuration with transforms")]
    pub config: VariationConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PresetVariationsParams {
    #[schemars(description = "Preset name: dark_mode, sizes_16_32_64, sizes_128_256_512")]
    pub preset: String,
    #[schemars(description = "Output directory (default: ./variations)")]
    pub output_dir: Option<String>,
}
