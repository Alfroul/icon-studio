use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{internal_err, invalid_params, state_err, IconStudioHandler};
use crate::engine::brand;
use crate::model::BrandKit;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateBrandKitParams {
    #[schemars(description = "Brand kit name")]
    pub name: String,
    #[schemars(description = "Primary color hex (e.g. '#FF6B35')")]
    pub primary: String,
    #[schemars(description = "Secondary color hex (optional, auto-derived as complementary)")]
    pub secondary: Option<String>,
    #[schemars(description = "Accent color hex (optional, auto-derived as triadic)")]
    pub accent: Option<String>,
    #[schemars(description = "Neutral color hex (optional, auto-derived as desaturated)")]
    pub neutral: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ApplyBrandParams {
    #[schemars(description = "Brand kit ID to apply")]
    pub kit_id: String,
    #[schemars(description = "Application mode: 'closest' (default) or 'exact'")]
    #[serde(default = "default_closest")]
    pub mode: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GenerateVariantParams {
    #[schemars(description = "Brand kit ID")]
    pub kit_id: String,
    #[schemars(description = "Variant type: dark, light, high-contrast")]
    pub variant_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportBrandGuideParams {
    #[schemars(description = "Brand kit ID")]
    pub kit_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SuggestBrandParams {
    #[schemars(description = "Text description of the brand (e.g. 'tech startup', '自然环保')")]
    pub description: String,
}

fn default_closest() -> String {
    "closest".to_string()
}

fn find_kit<'a>(project: &'a crate::model::IconProject, kit_id: &str) -> Option<&'a BrandKit> {
    project.brand_kits.iter().find(|k| k.id == kit_id)
}

fn format_kit_preview(kit: &BrandKit) -> String {
    let mut lines = vec![format!("**{}** (id: {})", kit.name, kit.id)];
    for (role, hex) in &kit.colors {
        let role_name = serde_json::to_string(role)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();
        lines.push(format!("  {} {}", hex, role_name));
    }
    lines.join("\n")
}

#[tool_router(router = brand_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "create_brand_kit", description = "Create a new brand kit with semantic color roles. Auto-derives secondary/accent/neutral if not provided.")]
    async fn create_brand_kit(
        &self,
        Parameters(params): Parameters<CreateBrandKitParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let kit = brand::create_brand_kit(
            &params.name,
            &params.primary,
            params.secondary.as_deref(),
            params.accent.as_deref(),
            params.neutral.as_deref(),
        )
        .map_err(internal_err)?;

        let preview = format_kit_preview(&kit);

        let mut project = self.project.lock().map_err(state_err)?;
        project.brand_kits.push(kit);
        project.bump_version();
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Created brand kit:\n{}",
            preview
        ))]))
    }

    #[tool(name = "apply_brand", description = "Apply a brand kit's colors to the current project canvas elements")]
    async fn apply_brand(
        &self,
        Parameters(params): Parameters<ApplyBrandParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let kit = {
            let project = self.project.lock().map_err(state_err)?;
            find_kit(&project, &params.kit_id)
                .cloned()
                .ok_or_else(|| invalid_params(format!("Brand kit '{}' not found", params.kit_id)))?
        };

        let mut project = self.project.lock().map_err(state_err)?;
        brand::apply_brand(&mut project, &kit, &params.mode).map_err(internal_err)?;
        project.bump_version();
        drop(project);

        let mut cache = self.cache.lock().map_err(state_err)?;
        cache.invalidate_cache();
        drop(cache);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Applied brand kit '{}' (mode: {})",
            kit.name, params.mode
        ))]))
    }

    #[tool(name = "generate_variant", description = "Generate a brand variant (dark, light, high-contrast) from an existing kit")]
    async fn generate_variant(
        &self,
        Parameters(params): Parameters<GenerateVariantParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let kit = {
            let project = self.project.lock().map_err(state_err)?;
            find_kit(&project, &params.kit_id)
                .cloned()
                .ok_or_else(|| invalid_params(format!("Brand kit '{}' not found", params.kit_id)))?
        };

        let variant = brand::generate_variant(&kit, &params.variant_type).map_err(internal_err)?;
        let preview = format_kit_preview(&variant);

        let mut project = self.project.lock().map_err(state_err)?;
        project.brand_kits.push(variant);
        project.bump_version();
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Generated {} variant:\n{}",
            params.variant_type, preview
        ))]))
    }

    #[tool(name = "export_brand_guide", description = "Export a brand guide document (Markdown) with color values and usage guidelines")]
    async fn export_brand_guide(
        &self,
        Parameters(params): Parameters<ExportBrandGuideParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;
        let kit = find_kit(&project, &params.kit_id)
            .ok_or_else(|| invalid_params(format!("Brand kit '{}' not found", params.kit_id)))?;
        let guide = brand::export_brand_guide(kit);
        drop(project);

        Ok(CallToolResult::success(vec![Content::text(guide)]))
    }

    #[tool(name = "list_brand_kits", description = "List all saved brand kits with name and primary color")]
    async fn list_brand_kits(&self) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;

        if project.brand_kits.is_empty() {
            drop(project);
            return Ok(CallToolResult::success(vec![Content::text(
                "No brand kits. Use create_brand_kit to create one.".to_string(),
            )]));
        }

        let mut lines = vec![format!("Brand Kits ({}):", project.brand_kits.len())];
        for kit in &project.brand_kits {
            let primary = kit
                .colors
                .get(&crate::model::BrandColorRole::Primary)
                .map(|s| s.as_str())
                .unwrap_or("?");
            lines.push(format!("  {} — {} {}", kit.id, primary, kit.name));
        }
        drop(project);

        Ok(CallToolResult::success(vec![Content::text(
            lines.join("\n"),
        )]))
    }

    #[tool(name = "suggest_brand_from_description", description = "Suggest a brand color palette from a text description (keywords like 'tech/科技', 'nature/自然', 'luxury/高端', etc.)")]
    async fn suggest_brand_from_description(
        &self,
        Parameters(params): Parameters<SuggestBrandParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let kit =
            brand::suggest_brand_from_description(&params.description).map_err(internal_err)?;
        let preview = format_kit_preview(&kit);

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Suggested brand kit:\n{}",
            preview
        ))]))
    }
}
