use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::IconStudioHandler;
use crate::engine::variants;
use crate::engine::weight;
use crate::model::{ThemeRule, ThemeVariant};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateVariantParams {
    #[schemars(description = "Name for the new theme variant")]
    pub name: String,
    #[schemars(description = "Transformation rules (JSON array of ThemeRule)")]
    pub rules: Vec<ThemeRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportVariantsParams {
    #[schemars(description = "List of variant names to export (empty = all)")]
    pub variant_names: Option<Vec<String>>,
    #[schemars(description = "Output directory for exported files")]
    pub output_dir: Option<String>,
}

#[tool_router(router = variant_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "create_variant", description = "Create a new theme variant with transformation rules")]
    async fn create_variant(
        &self,
        Parameters(params): Parameters<CreateVariantParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let variant = ThemeVariant {
            name: params.name,
            base_page_index: project.active_page_index,
            rules: params.rules,
        };
        project.theme_variants.push(variant.clone());
        project.bump_version();
        let json = serde_json::to_string_pretty(&variant)
            .unwrap_or_else(|_| "Variant created".to_string());
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "export_variants", description = "Export all or selected theme variants as SVG")]
    async fn export_variants(
        &self,
        Parameters(params): Parameters<ExportVariantsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let (variants_to_export, project_clone) = {
            let project = self.project.lock().map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
            let filtered: Vec<ThemeVariant> = match params.variant_names {
                Some(ref names) if !names.is_empty() => {
                    project.theme_variants.iter()
                        .filter(|v| names.contains(&v.name))
                        .cloned()
                        .collect()
                }
                _ => project.theme_variants.clone(),
            };
            (filtered, project.clone())
        };

        if variants_to_export.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No variants to export".to_string(),
            )]));
        }

        let mut results = Vec::new();
        for variant in &variants_to_export {
            let derived = variants::generate_variant(&project_clone, &variant.rules);
            match crate::engine::builder::build(&derived) {
                Ok(svg) => results.push(format!("--- {} ---\n{}", variant.name, svg)),
                Err(e) => results.push(format!("--- {} ---\nError: {}", variant.name, e)),
            }
        }

        Ok(CallToolResult::success(vec![Content::text(results.join("\n\n"))]))
    }

    #[tool(name = "list_variants", description = "List all theme variants in the current project")]
    async fn list_variants(
        &self,
    ) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        if project.theme_variants.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No theme variants defined".to_string(),
            )]));
        }
        let mut lines = Vec::new();
        for (i, v) in project.theme_variants.iter().enumerate() {
            lines.push(format!("{}. {} ({} rules)", i, v.name, v.rules.len()));
        }
        Ok(CallToolResult::success(vec![Content::text(lines.join("\n"))]))
    }

    #[tool(name = "list_preset_rules", description = "List available preset variant rule sets")]
    async fn list_preset_rules(
        &self,
    ) -> Result<CallToolResult, ErrorData> {
        let presets = variants::list_preset_rule_sets();
        let json = serde_json::to_string_pretty(&presets)
            .unwrap_or_else(|_| "Failed to serialize".to_string());
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "generate_weight_variants", description = "Generate weight variants (thin, light, regular, medium, bold, fill) from current icon")]
    async fn generate_weight_variants(
        &self,
        Parameters(params): Parameters<GenerateWeightVariantsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let presets: Vec<weight::WeightPreset> = params.weights.iter().filter_map(|w| match w.as_str() {
            "thin" => Some(weight::WeightPreset::Thin),
            "light" => Some(weight::WeightPreset::Light),
            "regular" => Some(weight::WeightPreset::Regular),
            "medium" => Some(weight::WeightPreset::Medium),
            "bold" => Some(weight::WeightPreset::Bold),
            "fill" => Some(weight::WeightPreset::Fill),
            _ => None,
        }).collect();

        let style = weight::detect_icon_style(&project);
        let style_str = match style {
            weight::IconStyleKind::StrokeBased => "StrokeBased",
            weight::IconStyleKind::FillBased => "FillBased",
            weight::IconStyleKind::Mixed => "Mixed",
        };

        let variants = weight::generate_weight_variants(&project, &presets);
        let mut results = Vec::new();
        for v in &variants {
            let weight_str = match &v.weight {
                weight::WeightPreset::Thin => "thin",
                weight::WeightPreset::Light => "light",
                weight::WeightPreset::Regular => "regular",
                weight::WeightPreset::Medium => "medium",
                weight::WeightPreset::Bold => "bold",
                weight::WeightPreset::Fill => "fill",
            };
            match crate::engine::builder::build(&v.project) {
                Ok(svg) => results.push(format!("--- {} ---\n{}", weight_str, svg)),
                Err(e) => results.push(format!("--- {} ---\nError: {}", weight_str, e)),
            }
        }

        let header = format!("Detected style: {}\nGenerated {} variant(s)", style_str, results.len());
        Ok(CallToolResult::success(vec![Content::text(format!("{}\n\n{}", header, results.join("\n\n")))]))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GenerateWeightVariantsParams {
    #[schemars(description = "Weight presets to generate (thin, light, regular, medium, bold, fill)")]
    pub weights: Vec<String>,
}
