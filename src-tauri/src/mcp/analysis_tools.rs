use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{internal_err, invalid_params, state_err, IconStudioHandler};
use crate::engine::analyzer;

fn default_5() -> usize {
    5
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SuggestPaletteParams {
    #[schemars(description = "Base color (hex, e.g. #FF5733)")]
    pub base_color: String,
    #[schemars(description = "Color scheme: complementary, analogous, triadic, split-complementary, monochromatic")]
    pub scheme: String,
    #[schemars(description = "Number of suggested colors")]
    #[serde(default = "default_5")]
    pub count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListFontsParams {
    #[schemars(description = "Keyword to filter fonts by name (optional)")]
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListIconsParams {
    #[schemars(description = "Keyword to filter icons by name or tags (optional)")]
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FixConsistencyParams {
    #[schemars(description = "Element IDs to fix")]
    pub element_ids: Vec<String>,
}

#[tool_router(router = analysis_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "analyze_colors", description = "Analyze color usage: identify primary, secondary, and accent colors")]
    async fn analyze_colors_tool(
        &self,
    ) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;
        let result = analyzer::analyze_colors(&project);
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| internal_err(format!("Serialization error: {}", e)))?;
        drop(project);

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "analyze_consistency", description = "Full consistency report: border radius, stroke width, font size, opacity, fill style, proportions, visual center drift")]
    async fn analyze_consistency_tool(
        &self,
    ) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;
        let result = analyzer::check_consistency(&project);
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| internal_err(format!("Serialization error: {}", e)))?;
        drop(project);

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "check_consistency", description = "Check design consistency (border radius, stroke width, font size, opacity, fill style, proportions, visual center drift)")]
    async fn check_consistency_tool(
        &self,
    ) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;
        let result = analyzer::check_consistency(&project);
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| internal_err(format!("Serialization error: {}", e)))?;
        drop(project);

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "fix_consistency_issues", description = "Fix consistency issues for specified element IDs by setting deviated properties to mode values")]
    async fn fix_consistency_issues_tool(
        &self,
        Parameters(params): Parameters<FixConsistencyParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;
        let fixed = analyzer::fix_consistency_issues(&project, &params.element_ids)
            .map_err(|e| internal_err(format!("Fix error: {}", e)))?;
        drop(project);

        let mut guard = self.project.lock().map_err(state_err)?;
        *guard = fixed;
        drop(guard);

        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(
            "Consistency issues fixed successfully.".to_string(),
        )]))
    }

    #[tool(name = "suggest_palette", description = "Suggest color palette based on base color and color scheme")]
    async fn suggest_palette(
        &self,
        Parameters(params): Parameters<SuggestPaletteParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let palette_scheme = match params.scheme.as_str() {
            "complementary" => crate::colors::PaletteScheme::Complementary,
            "analogous" => crate::colors::PaletteScheme::Analogous,
            "triadic" => crate::colors::PaletteScheme::Triadic,
            "split-complementary" => crate::colors::PaletteScheme::SplitComplementary,
            "monochromatic" => crate::colors::PaletteScheme::Monochromatic,
            other => return Err(invalid_params(format!("Invalid palette scheme: {}", other))),
        };

        let colors = crate::colors::suggest_palette(&params.base_color, palette_scheme, params.count)
            .map_err(|e| internal_err(format!("Palette error: {}", e)))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Palette ({}): {}",
            params.scheme,
            colors.join(", ")
        ))]))
    }

    #[tool(name = "list_fonts", description = "List available system fonts, optionally filter by keyword")]
    async fn list_fonts(
        &self,
        Parameters(params): Parameters<ListFontsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let fonts = match &params.keyword {
            Some(kw) if !kw.is_empty() => crate::fonts::search_fonts(kw),
            _ => crate::fonts::list_system_fonts(),
        };

        let mut lines = Vec::with_capacity(fonts.len().min(50));
        for f in fonts.iter().take(50) {
            lines.push(format!("{} ({} weight={})", f.name, f.style, f.weight));
        }

        let total = fonts.len();
        if total > 50 {
            lines.push(format!("... and {} more", total - 50));
        }

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Fonts ({} total):\n{}",
            total,
            lines.join("\n")
        ))]))
    }

    #[tool(name = "list_icons", description = "List available Lucide icons, optionally filter by keyword")]
    async fn list_icons(
        &self,
        Parameters(params): Parameters<ListIconsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let icon_list = match &params.keyword {
            Some(kw) if !kw.is_empty() => crate::icons::search_icons(kw),
            _ => crate::icons::list_all_icons(),
        };

        let total = icon_list.len();
        let names: Vec<String> = icon_list.iter().take(100).map(|i| i.name.clone()).collect();
        let mut text = format!("Icons ({} total): {}", total, names.join(", "));
        if total > 100 {
            text.push_str(&format!("\n... and {} more", total - 100));
        }
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}
