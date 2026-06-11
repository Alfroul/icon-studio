use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{invalid_params, state_err, IconStudioHandler};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AddOverlayParams {
    #[schemars(description = "Element ID to add overlay to")]
    pub element_id: String,
    #[schemars(description = "Overlay kind: Add/Remove/Check/Info/Warning/Error/Star/Lock/New/Custom")]
    pub kind: String,
    #[schemars(description = "Position: TopLeft/TopRight/BottomLeft/BottomRight")]
    pub position: Option<String>,
    #[schemars(description = "Overlay color (hex)")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RemoveOverlayParams {
    #[schemars(description = "Element ID to remove overlay from")]
    pub element_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct BatchOverlayParams {
    #[schemars(description = "Element IDs to apply overlay to")]
    pub element_ids: Vec<String>,
    #[schemars(description = "Overlay kind")]
    pub kind: String,
    #[schemars(description = "Position")]
    pub position: Option<String>,
}

#[tool_router(router = overlay_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "add_overlay", description = "Add a badge overlay to an element")]
    async fn add_overlay(
        &self,
        Parameters(params): Parameters<AddOverlayParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let elem = project
            .active_elements_mut()
            .iter_mut()
            .find(|e| e.id() == params.element_id)
            .ok_or_else(|| invalid_params(format!("Element '{}' not found", params.element_id)))?;

        use crate::model::{Overlay, OverlayKind, OverlayPosition};
        let kind = match params.kind.as_str() {
            "Add" => OverlayKind::Add,
            "Remove" => OverlayKind::Remove,
            "Check" => OverlayKind::Check,
            "Info" => OverlayKind::Info,
            "Warning" => OverlayKind::Warning,
            "Error" => OverlayKind::Error,
            "Star" => OverlayKind::Star,
            "Lock" => OverlayKind::Lock,
            "New" => OverlayKind::New,
            "Custom" => OverlayKind::Custom,
            _ => return Err(invalid_params(format!("Unknown overlay kind: {}", params.kind))),
        };
        let position = match params.position.as_deref() {
            Some("TopLeft") => OverlayPosition::TopLeft,
            Some("BottomLeft") => OverlayPosition::BottomLeft,
            Some("BottomRight") => OverlayPosition::BottomRight,
            _ => OverlayPosition::TopRight,
        };
        elem.common_mut().overlay = Some(Overlay {
            kind,
            position,
            color: params.color,
            size_ratio: None,
            offset_x: None,
            offset_y: None,
            custom_path: None,
        });
        self.emit_change();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Overlay added to element '{}'", params.element_id
        ))]))
    }

    #[tool(name = "remove_overlay", description = "Remove a badge overlay from an element")]
    async fn remove_overlay(
        &self,
        Parameters(params): Parameters<RemoveOverlayParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let elem = project
            .active_elements_mut()
            .iter_mut()
            .find(|e| e.id() == params.element_id)
            .ok_or_else(|| invalid_params(format!("Element '{}' not found", params.element_id)))?;
        elem.common_mut().overlay = None;
        self.emit_change();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Overlay removed from element '{}'", params.element_id
        ))]))
    }

    #[tool(name = "batch_overlay", description = "Apply a badge overlay to multiple elements at once")]
    async fn batch_overlay(
        &self,
        Parameters(params): Parameters<BatchOverlayParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        use crate::model::{Overlay, OverlayKind, OverlayPosition};
        let kind = match params.kind.as_str() {
            "Add" => OverlayKind::Add,
            "Remove" => OverlayKind::Remove,
            "Check" => OverlayKind::Check,
            "Info" => OverlayKind::Info,
            "Warning" => OverlayKind::Warning,
            "Error" => OverlayKind::Error,
            "Star" => OverlayKind::Star,
            "Lock" => OverlayKind::Lock,
            "New" => OverlayKind::New,
            "Custom" => OverlayKind::Custom,
            _ => return Err(invalid_params(format!("Unknown overlay kind: {}", params.kind))),
        };
        let position = match params.position.as_deref() {
            Some("TopLeft") => OverlayPosition::TopLeft,
            Some("TopRight") => OverlayPosition::TopRight,
            Some("BottomLeft") => OverlayPosition::BottomLeft,
            Some("BottomRight") => OverlayPosition::BottomRight,
            _ => OverlayPosition::BottomRight,
        };
        let overlay = Overlay {
            kind,
            position,
            color: Some("#FF0000".to_string()),
            size_ratio: Some(0.4),
            offset_x: None,
            offset_y: None,
            custom_path: None,
        };

        let mut count = 0u32;
        for eid in &params.element_ids {
            if let Some(elem) = project.active_elements_mut().iter_mut().find(|e| e.id() == eid) {
                elem.common_mut().overlay = Some(overlay.clone());
                count += 1;
            }
        }
        self.emit_change();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Overlay applied to {} element(s)", count
        ))]))
    }
}
