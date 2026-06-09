use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::{internal_err, invalid_params, IconStudioHandler};
use crate::engine::pack_importer;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ImportIconPackParams {
    #[schemars(description = "Directory containing SVG files to import")]
    pub dir: String,
    #[schemars(description = "Name for the imported icon pack")]
    pub pack_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListPackIconsParams {
    #[schemars(description = "Pack ID to list icons from")]
    pub pack_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SearchPackIconsParams {
    #[schemars(description = "Pack ID to search within")]
    pub pack_id: String,
    #[schemars(description = "Search query (matches name, category, tags)")]
    pub query: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LoadPackIconParams {
    #[schemars(description = "Pack ID containing the icon")]
    pub pack_id: String,
    #[schemars(description = "Icon name to load")]
    pub icon_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RemoveIconPackParams {
    #[schemars(description = "Pack ID to remove")]
    pub pack_id: String,
}

#[tool_router(router = pack_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "import_icon_pack", description = "Import SVG files from a directory as a new icon pack")]
    async fn import_icon_pack(
        &self,
        Parameters(params): Parameters<ImportIconPackParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let dir = Path::new(&params.dir);
        if !dir.is_dir() {
            return Err(invalid_params(format!(
                "Directory does not exist: {}",
                params.dir
            )));
        }

        let result = pack_importer::import_from_directory(dir, &params.pack_name)
            .map_err(|e| internal_err(format!("Import error: {}", e)))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Imported pack '{}' (id: {}) with {} icons, {} errors",
            result.pack.name, result.pack.id, result.icons_imported, result.errors.len()
        ))]))
    }

    #[tool(name = "list_icon_packs", description = "List all imported icon packs")]
    async fn list_icon_packs(
        &self,
        _params: Parameters<()>,
    ) -> Result<CallToolResult, ErrorData> {
        let packs = pack_importer::list_packs()
            .map_err(|e| internal_err(format!("List packs error: {}", e)))?;

        let summary: Vec<String> = packs
            .iter()
            .map(|p| format!("{} (id: {}, {} icons)", p.name, p.id, p.icon_count))
            .collect();

        Ok(CallToolResult::success(vec![Content::text(if summary.is_empty() {
            "No icon packs found".to_string()
        } else {
            format!("{} pack(s):\n{}", summary.len(), summary.join("\n"))
        })]))
    }

    #[tool(name = "list_pack_icons", description = "List all icons in a specific icon pack")]
    async fn list_pack_icons(
        &self,
        Parameters(params): Parameters<ListPackIconsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let icons = pack_importer::list_pack_icons(&params.pack_id)
            .map_err(|e| internal_err(format!("List icons error: {}", e)))?;

        let names: Vec<String> = icons.iter().map(|i| i.name.clone()).collect();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "{} icon(s) in pack '{}': {}",
            icons.len(),
            params.pack_id,
            names.join(", ")
        ))]))
    }

    #[tool(name = "search_pack_icons", description = "Search icons within a pack by name, category, or tags")]
    async fn search_pack_icons(
        &self,
        Parameters(params): Parameters<SearchPackIconsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let results = pack_importer::search_pack_icons(&params.pack_id, &params.query)
            .map_err(|e| internal_err(format!("Search error: {}", e)))?;

        let names: Vec<String> = results.iter().map(|i| i.name.clone()).collect();

        Ok(CallToolResult::success(vec![Content::text(if results.is_empty() {
            format!("No icons matching '{}' in pack '{}'", params.query, params.pack_id)
        } else {
            format!(
                "Found {} icon(s) matching '{}' in pack '{}': {}",
                results.len(),
                params.query,
                params.pack_id,
                names.join(", ")
            )
        })]))
    }

    #[tool(name = "load_pack_icon", description = "Load SVG content of a specific icon from a pack")]
    async fn load_pack_icon(
        &self,
        Parameters(params): Parameters<LoadPackIconParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let svg = pack_importer::load_pack_icon_svg(&params.pack_id, &params.icon_name)
            .map_err(|e| internal_err(format!("Load icon error: {}", e)))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "SVG for '{}/{}' ({} bytes):\n{}",
            params.pack_id,
            params.icon_name,
            svg.len(),
            svg
        ))]))
    }

    #[tool(name = "remove_icon_pack", description = "Remove an imported icon pack by its ID")]
    async fn remove_icon_pack(
        &self,
        Parameters(params): Parameters<RemoveIconPackParams>,
    ) -> Result<CallToolResult, ErrorData> {
        pack_importer::remove_pack(&params.pack_id)
            .map_err(|e| internal_err(format!("Remove pack error: {}", e)))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Removed pack '{}'",
            params.pack_id
        ))]))
    }
}
