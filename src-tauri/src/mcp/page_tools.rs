use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{invalid_params, state_err, IconStudioHandler};
use crate::model::Page;

// ---------------------------------------------------------------------------
// Param structs
// ---------------------------------------------------------------------------

fn default_512_opt() -> Option<u32> { Some(512) }

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AddPageParams {
    #[schemars(description = "Page name")]
    pub name: String,
    #[schemars(description = "Canvas width in pixels")]
    #[serde(default = "default_512_opt")]
    pub width: Option<u32>,
    #[schemars(description = "Canvas height in pixels")]
    #[serde(default = "default_512_opt")]
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SwitchPageParams {
    #[schemars(description = "Page ID to switch to")]
    pub page_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DeletePageParams {
    #[schemars(description = "Page ID to delete")]
    pub page_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DuplicatePageParams {
    #[schemars(description = "Page ID to duplicate")]
    pub page_id: String,
    #[schemars(description = "Name for the duplicated page")]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RenamePageParams {
    #[schemars(description = "Page ID to rename")]
    pub page_id: String,
    #[schemars(description = "New name for the page")]
    pub name: String,
}

// ---------------------------------------------------------------------------
// Page management tools
// ---------------------------------------------------------------------------

#[tool_router(router = page_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "list_pages", description = "List all pages in the project with ID, name, and element count")]
    async fn list_pages(&self) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;

        if project.pages.is_empty() {
            drop(project);
            return Ok(CallToolResult::success(vec![Content::text(
                "Project has a single canvas (no pages). Use add_page to create pages.".to_string(),
            )]));
        }

        let mut lines = Vec::new();
        for (i, page) in project.pages.iter().enumerate() {
            let active = if i == project.active_page_index_clamped() { " *" } else { "" };
            lines.push(format!(
                "  [{}] {} ({}x{}, {} elements){}",
                page.id, page.name, page.canvas.width, page.canvas.height, page.elements.len(), active
            ));
        }
        drop(project);

        let mut text = format!("Pages ({}):\n", lines.len());
        text.push_str(&lines.join("\n"));
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(name = "add_page", description = "Add a new page to the project and switch to it")]
    async fn add_page(
        &self,
        Parameters(params): Parameters<AddPageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let width = params.width.unwrap_or(512);
        let height = params.height.unwrap_or(512);
        if width == 0 || height == 0 || width > 8192 || height > 8192 {
            return Err(invalid_params("Invalid canvas dimensions: width and height must be 1-8192"));
        }

        let mut project = self.project.lock().map_err(state_err)?;

        // If pages is empty, migrate current canvas+elements to first page
        if project.pages.is_empty() {
            let first_page = Page::from_project(&project);
            project.pages.push(first_page);
        }

        let new_page = Page::new(&params.name, width, height);
        let page_id = new_page.id.clone();
        project.pages.push(new_page);
        project.active_page_index = project.pages.len() - 1;
        project.bump_version();
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added page '{}' (id: {})", params.name, page_id
        ))]))
    }

    #[tool(name = "switch_page", description = "Switch to a different page by ID")]
    async fn switch_page(
        &self,
        Parameters(params): Parameters<SwitchPageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        if project.pages.is_empty() {
            return Err(invalid_params("Project has no pages. Use add_page to create pages first."));
        }

        let idx = project.pages.iter().position(|p| p.id == params.page_id)
            .ok_or_else(|| invalid_params(format!("Page '{}' not found", params.page_id)))?;

        if idx == project.active_page_index_clamped() {
            drop(project);
            return Ok(CallToolResult::success(vec![Content::text(
                format!("Already on page '{}'", params.page_id),
            )]));
        }

        project.active_page_index = idx;
        project.bump_version();
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Switched to page '{}' (index: {})", params.page_id, idx
        ))]))
    }

    #[tool(name = "delete_page", description = "Delete a page by ID. Must keep at least one page.")]
    async fn delete_page(
        &self,
        Parameters(params): Parameters<DeletePageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        if project.pages.is_empty() {
            return Err(invalid_params("Project has no pages to delete."));
        }

        if project.pages.len() <= 1 {
            return Err(invalid_params("Cannot delete the last page. Must keep at least one page."));
        }

        let idx = project.pages.iter().position(|p| p.id == params.page_id)
            .ok_or_else(|| invalid_params(format!("Page '{}' not found", params.page_id)))?;

        project.pages.remove(idx);

        // Adjust active_page_index
        if project.active_page_index >= project.pages.len() {
            project.active_page_index = project.pages.len() - 1;
        } else if idx < project.active_page_index {
            project.active_page_index -= 1;
        } else if idx == project.active_page_index {
            project.active_page_index = project.active_page_index.min(project.pages.len() - 1);
        }

        project.bump_version();
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Deleted page '{}'", params.page_id
        ))]))
    }

    #[tool(name = "duplicate_page", description = "Duplicate a page with a new name")]
    async fn duplicate_page(
        &self,
        Parameters(params): Parameters<DuplicatePageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        let idx = project.pages.iter().position(|p| p.id == params.page_id)
            .ok_or_else(|| invalid_params(format!("Page '{}' not found", params.page_id)))?;

        let mut clone = project.pages[idx].clone();
        clone.id = format!("page-{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis());
        clone.name = params.name.clone();

        let new_id = clone.id.clone();
        project.pages.push(clone);
        project.bump_version();
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Duplicated page as '{}' (id: {})", params.name, new_id
        ))]))
    }

    #[tool(name = "rename_page", description = "Rename a page")]
    async fn rename_page(
        &self,
        Parameters(params): Parameters<RenamePageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        let page = project.pages.iter_mut().find(|p| p.id == params.page_id)
            .ok_or_else(|| invalid_params(format!("Page '{}' not found", params.page_id)))?;

        page.name = params.name.clone();
        project.bump_version();
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Renamed page to '{}'", params.name
        ))]))
    }
}
