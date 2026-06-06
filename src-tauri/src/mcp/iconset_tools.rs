use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{internal_err, invalid_params, state_err, IconStudioHandler};
use crate::engine::iconset;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateIconSetParams {
    #[schemars(description = "Icon set name")]
    pub name: String,
    #[schemars(description = "Description of the icon set")]
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AddToSetParams {
    #[schemars(description = "Icon set ID")]
    pub set_id: String,
    #[schemars(description = "Icon name within the set (auto-generated if empty)")]
    #[serde(default)]
    pub name: String,
    #[schemars(description = "Tags for the icon")]
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListIconSetsParams {}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportSetParams {
    #[schemars(description = "Icon set ID")]
    pub set_id: String,
    #[schemars(description = "Output directory")]
    pub output_dir: String,
    #[schemars(description = "Export format: svg, png, or all")]
    #[serde(default = "default_png")]
    pub format: String,
    #[schemars(description = "Export sizes for PNG (default: 16,32,64,128,256,512)")]
    #[serde(default = "default_sizes")]
    pub sizes: Vec<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CheckSetConsistencyParams {
    #[schemars(description = "Icon set ID")]
    pub set_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct TagIconParams {
    #[schemars(description = "Icon set ID")]
    pub set_id: String,
    #[schemars(description = "Entry ID within the set")]
    pub entry_id: String,
    #[schemars(description = "Tags to set")]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SearchIconsParams {
    #[schemars(description = "Search query (name or tag)")]
    pub query: String,
    #[schemars(description = "Icon set ID to search in (optional, searches all if omitted)")]
    #[serde(default)]
    pub set_id: Option<String>,
    #[schemars(description = "Filter by tags (all must match)")]
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

fn default_png() -> String {
    "png".to_string()
}

fn default_sizes() -> Vec<u32> {
    vec![16, 32, 64, 128, 256, 512]
}

#[tool_router(router = iconset_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "create_icon_set", description = "Create a new icon set collection")]
    async fn create_icon_set(
        &self,
        Parameters(params): Parameters<CreateIconSetParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let set = iconset::create_set(&params.name, &params.description)
            .map_err(internal_err)?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Created icon set '{}' (id: {}) with 0 entries.",
            set.name, set.id
        ))]))
    }

    #[tool(name = "add_to_set", description = "Add the current canvas as an icon in a set")]
    async fn add_to_set(
        &self,
        Parameters(params): Parameters<AddToSetParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut set = iconset::load_set(&params.set_id)
            .map_err(|e| invalid_params(format!("Set '{}' not found: {}", params.set_id, e)))?;

        let project = self.project.lock().map_err(state_err)?;
        let entry = iconset::add_entry(&mut set, &project, &params.name, params.tags)
            .map_err(internal_err)?;
        drop(project);

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added '{}' (id: {}) to set '{}'. Tags: {:?}",
            entry.name, entry.id, set.name, entry.tags
        ))]))
    }

    #[tool(name = "list_icon_sets", description = "List all icon sets with entry counts")]
    async fn list_icon_sets(
        &self,
        Parameters(_params): Parameters<ListIconSetsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let sets = iconset::list_sets().map_err(internal_err)?;

        if sets.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No icon sets. Use create_icon_set to create one.".to_string(),
            )]));
        }

        let mut lines = vec![format!("Icon Sets ({}):", sets.len())];
        for set in &sets {
            let entry_count = set.entries.len();
            lines.push(format!(
                "  {} — {} ({} entries)",
                set.id, set.name, entry_count
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(
            lines.join("\n"),
        )]))
    }

    #[tool(name = "export_set", description = "Export all icons in a set to a directory")]
    async fn export_set(
        &self,
        Parameters(params): Parameters<ExportSetParams>,
    ) -> Result<CallToolResult, ErrorData> {
        crate::engine::utils::validate_file_path(&params.output_dir)
            .map_err(|e| ErrorData::invalid_params(format!("Invalid output directory: {}", e), None))?;

        let files = iconset::export_set(
            &params.set_id,
            &params.format,
            &params.sizes,
            &params.output_dir,
        )
        .map_err(internal_err)?;

        if files.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "Set is empty, no files exported.".to_string(),
            )]));
        }

        let mut lines = vec![format!("Exported {} file(s):", files.len())];
        for f in &files {
            lines.push(format!("  {}", f.display()));
        }

        Ok(CallToolResult::success(vec![Content::text(
            lines.join("\n"),
        )]))
    }

    #[tool(name = "check_set_consistency", description = "Check an icon set for style consistency (stroke width, corner radius, colors)")]
    async fn check_set_consistency(
        &self,
        Parameters(params): Parameters<CheckSetConsistencyParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let report = iconset::check_consistency(&params.set_id)
            .map_err(internal_err)?;

        let mut lines = vec![report.summary];
        if !report.issues.is_empty() {
            lines.push(String::new());
            lines.push("Issues:".to_string());
            for issue in &report.issues {
                lines.push(format!(
                    "  [{}] {} in '{}': expected {}, got {} (element: {})",
                    issue.property,
                    issue.project_path,
                    issue.project_path,
                    issue.expected,
                    issue.actual,
                    issue.element_id
                ));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(
            lines.join("\n"),
        )]))
    }

    #[tool(name = "tag_icon", description = "Set tags on an icon within a set")]
    async fn tag_icon(
        &self,
        Parameters(params): Parameters<TagIconParams>,
    ) -> Result<CallToolResult, ErrorData> {
        iconset::tag_entry(&params.set_id, &params.entry_id, params.tags.clone())
            .map_err(internal_err)?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Updated tags on entry '{}' to {:?}",
            params.entry_id, params.tags
        ))]))
    }

    #[tool(name = "search_icons", description = "Search icons by name or tag across sets")]
    async fn search_icons(
        &self,
        Parameters(params): Parameters<SearchIconsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let results = iconset::search_entries(
            &params.query,
            params.tags.as_ref(),
            params.set_id.as_deref(),
        )
        .map_err(internal_err)?;

        if results.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "No icons found matching '{}'.",
                params.query
            ))]));
        }

        let mut lines = vec![format!("Found {} icon(s):", results.len())];
        for entry in &results {
            lines.push(format!(
                "  {} — {} [{}]",
                entry.id,
                entry.name,
                entry.tags.join(", ")
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(
            lines.join("\n"),
        )]))
    }
}
