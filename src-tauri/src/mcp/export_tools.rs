use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::{internal_err, invalid_params, state_err, IconStudioHandler};
use crate::engine::codegen::{self, CodeExportOptions, CodeFormat};
use crate::engine::exporter;
use crate::engine::tokens::{self, TokenFormat};
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
    #[schemars(description = "Snap coordinates to pixel grid for small sizes (≤32px)")]
    pub pixel_snap: Option<bool>,
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
    #[schemars(description = "Snap coordinates to pixel grid for small sizes (≤32px)")]
    pub pixel_snap: Option<bool>,
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

        let svg_str = if params.pixel_snap.unwrap_or(false) {
            exporter::snap_to_pixel_grid(&svg_str, 0.5)
                .map_err(|e| internal_err(format!("Pixel snap error: {}", e)))?
        } else {
            svg_str
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
        let result = exporter::export_favicon_package(
            &svg_str,
            dir,
            &params.app_name,
            params.theme_color.as_deref(),
            params.background_color.as_deref(),
        )
        .map_err(|e| internal_err(format!("Favicon export error: {}", e)))?;

        let file_names: Vec<String> = result
            .paths
            .iter()
            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
            .collect();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Exported favicon package ({} files):\n{}\n\nHTML snippet:\n{}",
            result.paths.len(),
            file_names.join("\n"),
            result.html_snippet
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

    #[tool(name = "export_code", description = "Export current icon as framework component code (ReactTs, VueTs, SwiftUI, Flutter, Xaml, VectorDrawable, SvgSymbol, SvgMinified, Cpp, Svelte)")]
    async fn export_code(
        &self,
        Parameters(params): Parameters<ExportCodeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let svg_str = {
            let project = self.project.lock().map_err(state_err)?;
            let mut cache = self.cache.lock().map_err(state_err)?;
            crate::services::export::build_svg(&project, &mut cache)
                .map_err(|e| internal_err(format!("Build error: {}", e)))?
        };

        let format = match params.format.as_str() {
            "reactTs" => CodeFormat::ReactTs,
            "vueTs" => CodeFormat::VueTs,
            "swiftUI" => CodeFormat::SwiftUI,
            "flutter" => CodeFormat::Flutter,
            "xaml" => CodeFormat::Xaml,
            "vectorDrawable" => CodeFormat::VectorDrawable,
            "svgSymbol" => CodeFormat::SvgSymbol,
            "svgMinified" => CodeFormat::SvgMinified,
            "cpp" => CodeFormat::Cpp,
            "svelte" => CodeFormat::Svelte,
            other => return Err(invalid_params(format!(
                "Unknown format '{}'. Valid: reactTs, vueTs, swiftUI, flutter, xaml, vectorDrawable, svgSymbol, svgMinified, cpp, svelte", other
            ))),
        };

        let options = CodeExportOptions {
            component_name: params.component_name,
            format,
            size: params.size,
            parametrize_fill: params.parametrize_fill,
        };

        let result = codegen::export_code(&svg_str, &options)
            .map_err(|e| internal_err(format!("Code export error: {}", e)))?;

        Ok(CallToolResult::success(vec![
            Content::text(format!("Generated {} ({} bytes)\n\n{}", result.filename, result.code.len(), result.code)),
        ]))
    }

    #[tool(name = "export_tokens", description = "Export design tokens from current project (CSS Variables, JSON DTCG, SCSS, Tailwind Config)")]
    async fn export_tokens(
        &self,
        Parameters(params): Parameters<ExportTokensParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let format = match params.format.as_str() {
            "cssVariables" => TokenFormat::CssVariables,
            "jsonDtcg" => TokenFormat::JsonDtcg,
            "scssVariables" => TokenFormat::ScssVariables,
            "tailwindConfig" => TokenFormat::TailwindConfig,
            other => return Err(invalid_params(format!(
                "Unknown format '{}'. Valid: cssVariables, jsonDtcg, scssVariables, tailwindConfig", other
            ))),
        };

        let result = {
            let project = self.project.lock().map_err(state_err)?;
            let tokens = tokens::extract_tokens(&project);
            tokens::format_tokens(&tokens, format)
        };

        Ok(CallToolResult::success(vec![
            Content::text(format!("{} ({} bytes)\n\n{}", result.filename, result.content.len(), result.content)),
        ]))
    }

    #[tool(name = "export_pwa_icons", description = "Export PWA manifest icons (192x192, 512x512)")]
    async fn export_pwa_icons(
        &self,
        Parameters(params): Parameters<ExportPwaIconsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let svg_str = {
            let project = self.project.lock().map_err(state_err)?;
            let mut cache = self.cache.lock().map_err(state_err)?;
            crate::services::export::build_svg(&project, &mut cache)
                .map_err(|e| internal_err(format!("Build error: {}", e)))?
        };

        let theme = params.theme_color.as_deref().unwrap_or("#FFFFFF");
        let bg = params.background_color.as_deref().unwrap_or("#FFFFFF");
        let paths = exporter::export_pwa_icons(
            &svg_str,
            Path::new(&params.output_dir),
            &params.app_name,
            theme,
            bg,
        )
        .map_err(|e| internal_err(format!("PWA export error: {}", e)))?;

        let file_names: Vec<String> = paths
            .iter()
            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
            .collect();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Exported PWA icons ({} files):\n{}",
            paths.len(),
            file_names.join("\n")
        ))]))
    }

    #[tool(name = "export_all_platforms", description = "Export icons for all platforms (iOS, Android, PWA, Favicon)")]
    async fn export_all_platforms(
        &self,
        Parameters(params): Parameters<ExportAllPlatformsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let svg_str = {
            let project = self.project.lock().map_err(state_err)?;
            let mut cache = self.cache.lock().map_err(state_err)?;
            crate::services::export::build_svg(&project, &mut cache)
                .map_err(|e| internal_err(format!("Build error: {}", e)))?
        };

        let theme = params.theme_color.as_deref().unwrap_or("#FFFFFF");
        let bg = params.background_color.as_deref().unwrap_or("#FFFFFF");
        let result = exporter::export_all_platforms(
            &svg_str,
            Path::new(&params.output_dir),
            &params.app_name,
            theme,
            bg,
        )
        .map_err(|e| internal_err(format!("All-platforms export error: {}", e)))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Exported all platforms:\n  iOS: {} files\n  Android: {} files\n  PWA: {} files\n  Favicon: {} files",
            result.ios_paths.len(),
            result.android_paths.len(),
            result.pwa_paths.len(),
            result.favicon_paths.len()
        ))]))
    }

    #[tool(name = "export_sprite_sheet", description = "Export multiple icons as a sprite sheet image")]
    async fn export_sprite_sheet(
        &self,
        Parameters(params): Parameters<ExportSpriteSheetParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let svg_str = {
            let project = self.project.lock().map_err(state_err)?;
            let mut cache = self.cache.lock().map_err(state_err)?;
            crate::services::export::build_svg(&project, &mut cache)
                .map_err(|e| internal_err(format!("Build error: {}", e)))?
        };

        let padding = params.padding.unwrap_or(0);
        let svgs: &[(&str, &str)] = &[("icon", &svg_str)];
        let result = exporter::export_sprite_sheet(
            svgs,
            params.columns,
            params.icon_size,
            padding,
            Path::new(&params.output_path),
        )
        .map_err(|e| internal_err(format!("Sprite sheet export error: {}", e)))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Sprite sheet exported: {}x{}, {} icons\nImage: {}\nicons.json: {}",
            result.total_width,
            result.total_height,
            result.icons.len(),
            result.image_path.to_string_lossy(),
            result.icons.len()
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

fn default_size() -> u32 { 24 }
fn default_parametrize_fill() -> bool { true }

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportCodeParams {
    #[schemars(description = "Component name in PascalCase (e.g. HomeIcon)")]
    pub component_name: String,
    #[schemars(description = "Target framework: reactTs, vueTs, swiftUI, flutter, xaml, vectorDrawable, svgSymbol, svgMinified, cpp, svelte")]
    pub format: String,
    #[schemars(description = "Default size in pixels")]
    #[serde(default = "default_size")]
    pub size: u32,
    #[schemars(description = "Replace fill colors with currentColor")]
    #[serde(default = "default_parametrize_fill")]
    pub parametrize_fill: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportTokensParams {
    #[schemars(description = "Token format: cssVariables, jsonDtcg, scssVariables, tailwindConfig")]
    pub format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportPwaIconsParams {
    #[schemars(description = "Output directory for PWA icons")]
    pub output_dir: String,
    #[schemars(description = "App name")]
    #[serde(default = "default_app_name")]
    pub app_name: String,
    #[schemars(description = "Theme color (hex)")]
    pub theme_color: Option<String>,
    #[schemars(description = "Background color (hex)")]
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportAllPlatformsParams {
    #[schemars(description = "Output directory")]
    pub output_dir: String,
    #[schemars(description = "App name")]
    #[serde(default = "default_app_name")]
    pub app_name: String,
    #[schemars(description = "Theme color (hex)")]
    pub theme_color: Option<String>,
    #[schemars(description = "Background color (hex)")]
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportSpriteSheetParams {
    #[schemars(description = "Output image path")]
    pub output_path: String,
    #[schemars(description = "Number of columns")]
    #[serde(default = "default_columns")]
    pub columns: u32,
    #[schemars(description = "Icon size in pixels")]
    #[serde(default = "default_size")]
    pub icon_size: u32,
    #[schemars(description = "Padding between icons")]
    pub padding: Option<u32>,
}

fn default_columns() -> u32 { 4 }
