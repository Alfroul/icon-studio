use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{internal_err, invalid_params, state_err, IconStudioHandler};
use crate::engine::analyzer::{self, ElementFilter};
use crate::model::helpers::*;
use crate::model::history::{AddElementCommand, SnapshotCommand};
use crate::model::{CommonProps, Element, ImageElement, PathElement};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AddImageParams {
    #[schemars(description = "Image file path")]
    pub path: String,
    #[schemars(description = "Image element width in pixels")]
    pub width: f64,
    #[schemars(description = "Image element height in pixels")]
    pub height: f64,
    #[schemars(description = "X position")]
    #[serde(default)]
    pub x: f64,
    #[schemars(description = "Y position")]
    #[serde(default)]
    pub y: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FindElementsParams {
    #[schemars(description = "Filter by element type: shape, text, icon, image, path, group")]
    pub element_type: Option<String>,
    #[schemars(description = "Exact fill color match (case-insensitive)")]
    pub fill: Option<String>,
    #[schemars(description = "Minimum width")]
    pub min_width: Option<f64>,
    #[schemars(description = "Maximum width")]
    pub max_width: Option<f64>,
}

fn default_20_offset() -> f64 {
    20.0
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DuplicateElementParams {
    #[schemars(description = "Element ID to duplicate")]
    pub element_id: String,
    #[schemars(description = "X offset for duplicate")]
    #[serde(default = "default_20_offset")]
    pub dx: f64,
    #[schemars(description = "Y offset for duplicate")]
    #[serde(default = "default_20_offset")]
    pub dy: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ReorderElementParams {
    #[schemars(description = "Element ID")]
    pub element_id: String,
    #[schemars(description = "Direction: front (top), back (bottom), up (one layer up), down (one layer down)")]
    pub direction: String,
}

fn default_none() -> String {
    "none".to_string()
}
fn default_black() -> String {
    "#000000".to_string()
}

fn default_100() -> f64 {
    100.0
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AddPathParams {
    #[schemars(description = "SVG path data (d attribute)")]
    pub d: String,
    #[schemars(description = "Fill color (hex or 'none')")]
    #[serde(default = "default_none")]
    pub fill: String,
    #[schemars(description = "Stroke color (hex)")]
    #[serde(default = "default_black")]
    pub stroke: String,
    #[schemars(description = "Stroke width")]
    #[serde(default)]
    pub stroke_width: f64,
    #[schemars(description = "X position")]
    #[serde(default)]
    pub x: f64,
    #[schemars(description = "Y position")]
    #[serde(default)]
    pub y: f64,
    #[schemars(description = "Width")]
    #[serde(default = "default_100")]
    pub width: f64,
    #[schemars(description = "Height")]
    #[serde(default = "default_100")]
    pub height: f64,
}

#[tool_router(router = elements_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "add_image", description = "Add an image element from file path")]
    async fn add_image(
        &self,
        Parameters(params): Parameters<AddImageParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if params.width.is_nan() || params.height.is_nan() || params.width <= 0.0 || params.height <= 0.0 {
            return Err(invalid_params("Image width and height must be positive"));
        }
        let (data, _mime) = crate::services::elements::detect_mime_and_encode(&params.path)
            .map_err(|e| internal_err(format!("Failed to read image: {}", e)))?;

        let mut project = self.project.lock().map_err(state_err)?;
        let id = project.alloc_element_id("image");
        let element = ImageElement {
            common: CommonProps {
                id: id.clone(),
                x: params.x,
                y: params.y,
                width: params.width,
                height: params.height,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
            data,
        };
        let cmd = AddElementCommand::new(Element::Image(element));
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.push_and_execute(Box::new(cmd), &mut project).map_err(internal_err)?;
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added image from '{}' (id: {})",
            params.path, id
        ))]))
    }

    #[tool(name = "list_elements", description = "List all elements with ID, type, position, size, and style summary")]
    async fn list_elements(&self) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;
        let elements = project.active_elements();
        let mut summaries = Vec::new();
        for element in elements {
            let line = match element {
                Element::Shape(s) => {
                    format!(
                        "[{}] shape: {:?} at ({:.0},{:.0}) {:.0}x{:.0} fill={}",
                        s.common.id, s.shape_type, s.common.x, s.common.y, s.common.width, s.common.height, s.fill
                    )
                }
                Element::Text(t) => {
                    format!(
                        "[{}] text: \"{}\" {} {:.0}px at ({:.0},{:.0})",
                        t.common.id, t.content, t.font_family, t.font_size, t.common.x, t.common.y
                    )
                }
                Element::Icon(i) => {
                    format!(
                        "[{}] icon: \"{}\" at ({:.0},{:.0}) {:.0}x{:.0}",
                        i.common.id, i.name, i.common.x, i.common.y, i.common.width, i.common.height
                    )
                }
                Element::Image(img) => {
                    format!(
                        "[{}] image at ({:.0},{:.0}) {:.0}x{:.0}",
                        img.common.id, img.common.x, img.common.y, img.common.width, img.common.height
                    )
                }
                Element::Path(p) => {
                    format!(
                        "[{}] path at ({:.0},{:.0}) stroke={}",
                        p.common.id, p.common.x, p.common.y, p.stroke
                    )
                }
                Element::Group(g) => {
                    format!(
                        "[{}] group: {} children at ({:.0},{:.0}) {:.0}x{:.0}",
                        g.common.id, g.children.len(), g.common.x, g.common.y, g.common.width, g.common.height
                    )
                }
                Element::Symbol(s) => {
                    format!(
                        "[{}] symbol: ref={} at ({:.0},{:.0})",
                        s.common.id, s.symbol_id, s.common.x, s.common.y
                    )
                }
            };
            summaries.push(line);
        }
        let count = summaries.len();
        drop(project);

        if summaries.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No elements in project.".to_string(),
            )]));
        }

        let mut text = format!("Elements ({}):\n", count);
        for s in &summaries {
            text.push_str(s);
            text.push('\n');
        }
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(name = "find_elements", description = "Find elements by type, fill color, or size range")]
    async fn find_elements_tool(
        &self,
        Parameters(params): Parameters<FindElementsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;
        let filter = ElementFilter {
            element_type: params.element_type,
            fill: params.fill,
            min_width: params.min_width,
            max_width: params.max_width,
        };
        let result = analyzer::find_elements(&project, &filter);
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| internal_err(format!("Serialization error: {}", e)))?;
        drop(project);

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "duplicate_element", description = "Duplicate an element with optional offset. Returns new element ID.")]
    async fn duplicate_element(
        &self,
        Parameters(params): Parameters<DuplicateElementParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        let found = find_element_deep(project.active_elements(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;

        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        let mut clone = found.clone();

        let (x, y, ..) = get_element_bounds(&clone);
        set_element_position(&mut clone, x + params.dx, y + params.dy);

        fn reassign_ids(elem: &mut Element, project: &mut crate::model::IconProject) {
            let prefix = match elem {
                Element::Shape(_) => "shape",
                Element::Text(_) => "text",
                Element::Icon(_) => "icon",
                Element::Image(_) => "image",
                Element::Path(_) => "path",
                Element::Group(_) => "group",
                Element::Symbol(_) => "symbol",
            };
            let new_child_id = project.alloc_element_id(prefix);
            elem.common_mut().id = new_child_id;
            if let Element::Group(g) = elem {
                for child in &mut g.children {
                    reassign_ids(child, project);
                }
            }
        }

        reassign_ids(&mut clone, &mut project);
        let new_id = clone.id().to_string();

        insert_element_after(project.active_elements_mut(), &params.element_id, clone);
        project.bump_version();

        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Duplicated element, new ID: {}", new_id
        ))]))
    }

    #[tool(name = "reorder_element", description = "Change element layer order. Direction: front, back, up, down")]
    async fn reorder_element(
        &self,
        Parameters(params): Parameters<ReorderElementParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        let (parent, old_idx) = find_element_parent_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;

        let len = parent.len();
        let new_idx = match params.direction.as_str() {
            "front" => len - 1,
            "back" => 0,
            "up" => (old_idx + 1).min(len - 1),
            "down" => old_idx.saturating_sub(1),
            other => return Err(invalid_params(format!("Invalid direction: {}. Valid: front, back, up, down", other))),
        };

        if old_idx == new_idx {
            drop(project);
            return Ok(CallToolResult::success(vec![Content::text(
                "Element already at target position".to_string(),
            )]));
        }

        let elem = parent.remove(old_idx);
        parent.insert(new_idx, elem);
        project.bump_version();
        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Moved '{}' to position {} ({})", params.element_id, new_idx, params.direction
        ))]))
    }

    #[tool(name = "add_path", description = "Add a custom SVG path element")]
    async fn add_path_tool(
        &self,
        Parameters(params): Parameters<AddPathParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if params.width.is_nan() || params.width <= 0.0 || params.height.is_nan() || params.height <= 0.0 {
            return Err(invalid_params("width and height must be positive"));
        }
        if params.stroke_width.is_nan() || params.stroke_width < 0.0 {
            return Err(invalid_params("stroke_width must be non-negative"));
        }
        if params.d.is_empty() {
            return Err(invalid_params("Path data (d) cannot be empty"));
        }

        let mut project = self.project.lock().map_err(state_err)?;
        let id = project.alloc_element_id("path");
        let mut element = PathElement {
            common: CommonProps {
                id: id.clone(),
                x: params.x,
                y: params.y,
                width: params.width,
                height: params.height,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
            d: params.d,
            fill: params.fill,
            stroke: params.stroke,
            stroke_width: params.stroke_width,
            stroke_dasharray: None,
            natural_width: 0.0,
            natural_height: 0.0,
            boolean_source: None,
        };
        {
            let mut wrapper = Element::Path(element.clone());
            crate::model::helpers::recompute_path_natural_dims(&mut wrapper);
            if let Element::Path(pe) = wrapper {
                element = pe;
            }
        }
        let cmd = AddElementCommand::new(Element::Path(element));
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.push_and_execute(Box::new(cmd), &mut project).map_err(internal_err)?;
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added path element (id: {})", id
        ))]))
    }
}
