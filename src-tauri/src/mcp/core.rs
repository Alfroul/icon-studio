//! Core MCP tools — 12 essential tools + load_extended_tools (13 total in core router).
//!
//! Core tools provide the minimum viable icon creation workflow:
//! create canvas → add elements → modify → style → export.

use base64::Engine;
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{internal_err, invalid_params, state_err, IconStudioHandler, RemoveElementParams};
use crate::engine::renderer;
use crate::engine::text_measure;
use crate::icons;
use crate::model::helpers::*;
use crate::model::history::{AddElementCommand, SetGradientCommand, SnapshotCommand};
use crate::model::{Canvas, CommonProps, Element, ExportConfig, Gradient, GradientKind, IconElement, ShapeElement, TextElement};

// ---------------------------------------------------------------------------
// Core param structs
// ---------------------------------------------------------------------------

fn default_background() -> String {
    "#FFFFFF".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct IconNewParams {
    #[schemars(description = "Canvas width in pixels")]
    pub width: u32,
    #[schemars(description = "Canvas height in pixels")]
    pub height: u32,
    #[schemars(description = "Background color (hex, e.g. #FFFFFF) or 'transparent'")]
    #[serde(default = "default_background")]
    pub background: String,
}

fn default_512() -> u32 {
    512
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct IconPreviewParams {
    #[schemars(description = "Preview size in pixels")]
    #[serde(default = "default_512")]
    pub size: u32,
}

fn default_100() -> f64 {
    100.0
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AddShapeParams {
    #[schemars(description = "Shape type: circle, rect, rounded-rect, hexagon, star, shield, diamond")]
    pub shape_type: String,
    #[schemars(description = "Fill color (hex)")]
    pub fill: String,
    #[schemars(description = "Stroke color (hex, optional)")]
    pub stroke: Option<String>,
    #[schemars(description = "Shape size (width=height=size)")]
    #[serde(default = "default_100")]
    pub size: f64,
    #[schemars(description = "X position (top-left origin)")]
    #[serde(default)]
    pub x: f64,
    #[schemars(description = "Y position (top-left origin)")]
    #[serde(default)]
    pub y: f64,
}

fn default_font() -> String {
    "sans-serif".to_string()
}
fn default_font_size() -> f64 {
    24.0
}
fn default_black() -> String {
    "#000000".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AddTextParams {
    #[schemars(description = "Text content")]
    pub content: String,
    #[schemars(description = "Font family name")]
    #[serde(default = "default_font")]
    pub font_family: String,
    #[schemars(description = "Font size in pixels")]
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    #[schemars(description = "Fill color (hex)")]
    #[serde(default = "default_black")]
    pub fill: String,
    #[schemars(description = "X position (top-left origin)")]
    #[serde(default)]
    pub x: f64,
    #[schemars(description = "Y position (top-left origin)")]
    #[serde(default)]
    pub y: f64,
}

fn default_64() -> f64 {
    64.0
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AddIconParams {
    #[schemars(description = "Lucide icon name (e.g. 'heart', 'star')")]
    pub name: String,
    #[schemars(description = "Fill color (hex)")]
    #[serde(default = "default_black")]
    pub fill: String,
    #[schemars(description = "Icon size in pixels")]
    #[serde(default = "default_64")]
    pub size: f64,
    #[schemars(description = "X position (top-left origin)")]
    #[serde(default)]
    pub x: f64,
    #[schemars(description = "Y position (top-left origin)")]
    #[serde(default)]
    pub y: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetPropsParams {
    #[schemars(description = "Element ID to update")]
    pub element_id: String,
    #[schemars(description = "JSON object of properties to update. Valid keys depend on element type.\n\nCommon (all): x, y, width, height, opacity (0-1), rotation (degrees)\nShape: fill, stroke, stroke_width, border_radius\nText: content, fill, font_family, font_size, font_weight, letter_spacing\nIcon: fill, stroke, stroke_width\nPath: d, fill, stroke, stroke_width\nImage: (common only)\n\nUnknown keys are silently ignored.")]
    pub props: serde_json::Value,
}

fn default_10() -> f64 {
    10.0
}
fn default_20() -> f64 {
    20.0
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetLayoutParams {
    #[schemars(description = "Layout type: center (center each), horizontal, vertical, stack (stack centered)")]
    pub layout_type: String,
    #[schemars(description = "Gap between elements in pixels")]
    #[serde(default = "default_10")]
    pub gap: f64,
    #[schemars(description = "Padding from canvas edge in pixels")]
    #[serde(default = "default_20")]
    pub padding: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetGradientParams {
    #[schemars(description = "Element ID")]
    pub element_id: String,
    #[schemars(description = "Gradient type: linear or radial")]
    pub gradient_type: String,
    #[schemars(description = "Gradient color stops (hex colors, minimum 2)")]
    pub colors: Vec<String>,
    #[schemars(description = "Gradient angle in degrees (linear only)")]
    #[serde(default)]
    pub angle: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportSvgParams {
    #[schemars(description = "Output file path. Omit to return SVG text content")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ExportPngParams {
    #[schemars(description = "Export sizes in pixels (e.g. [16, 32, 64, 128, 256, 512])")]
    pub sizes: Vec<u32>,
    #[schemars(description = "PNG output directory")]
    pub output_dir: String,
}

// ---------------------------------------------------------------------------
// Core tool router — 12 core tools + load_extended_tools
// ---------------------------------------------------------------------------

#[tool_router(router = core_router, vis = "pub")]
impl IconStudioHandler {
    // -- Canvas Management --

    #[tool(name = "icon_new", description = "Create a new canvas with specified dimensions and background. Clears any existing project.")]
    async fn icon_new(
        &self,
        Parameters(params): Parameters<IconNewParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let bg = params.background.clone();
        if params.width == 0 || params.height == 0 || params.width > 8192 || params.height > 8192 {
            return Err(invalid_params("Invalid canvas dimensions: width and height must be 1-8192"));
        }
        let mut project = self.project.lock().map_err(state_err)?;
        project.canvas = Canvas {
            width: params.width,
            height: params.height,
            background: params.background,
            corner_radius: 0,
            background_gradient: None,
        };
        project.elements.clear();
        project.pages.clear();
        project.active_page_index = 0;
        project.next_element_id = 1;
        project.exports = ExportConfig::default();
        project.bump_version();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.clear();
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Created new canvas: {}x{}, background '{}'",
            params.width, params.height, bg
        ))]))
    }

    #[tool(name = "icon_status", description = "Get current project status including canvas dimensions, element count, and element ID list")]
    async fn icon_status(&self) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;
        let canvas = project.active_canvas();
        let w = canvas.width;
        let h = canvas.height;
        let bg = canvas.background.clone();
        let elements = project.active_elements();
        let count = elements.len();
        let corner = canvas.corner_radius;

        let page_info = if !project.pages.is_empty() {
            format!(" (page {}/{})", project.active_page_index_clamped() + 1, project.pages.len())
        } else {
            String::new()
        };

        let mut elem_lines = Vec::new();
        for elem in elements {
            let t = match elem {
                Element::Shape(_) => "shape",
                Element::Text(_) => "text",
                Element::Icon(_) => "icon",
                Element::Image(_) => "image",
                Element::Path(_) => "path",
                Element::Group(_) => "group",
                Element::Symbol(_) => "symbol",
            };
            elem_lines.push(format!("  {} [{}]", elem.id(), t));
        }
        let elem_summary = if elem_lines.is_empty() {
            "  (none)".to_string()
        } else {
            elem_lines.join("\n")
        };
        drop(project);

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Canvas: {}x{}, background: {}, corner_radius: {}%{}\nElements ({}):\n{}",
            w, h, bg, corner, page_info, count, elem_summary
        ))]))
    }

    #[tool(name = "icon_preview", description = "Preview current icon as base64-encoded PNG image")]
    async fn icon_preview(
        &self,
        Parameters(params): Parameters<IconPreviewParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let size = if params.size == 0 { 512 } else { params.size };
        if size > 8192 {
            return Err(invalid_params(format!("Preview size {} exceeds maximum 8192", size)));
        }

        let svg_str = {
            let project = self.project.lock().map_err(state_err)?;
            let mut cache = self.cache.lock().map_err(state_err)?;
            crate::services::export::build_svg(&project, &mut cache).map_err(|e| internal_err(format!("Build error: {}", e)))?
        };

        let png_bytes = renderer::render(&svg_str, size)
            .map_err(|e| internal_err(format!("Render error: {}", e)))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

        Ok(CallToolResult::success(vec![Content::image(
            format!("data:image/png;base64,{}", b64),
            "image/png",
        )]))
    }

    // -- Element Adding --

    #[tool(name = "add_shape", description = "Add a geometric shape element to the canvas. Shapes: circle, rect, rounded-rect, hexagon, star, shield, diamond")]
    async fn add_shape(
        &self,
        Parameters(params): Parameters<AddShapeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if params.size.is_nan() || params.size <= 0.0 {
            return Err(invalid_params("size must be positive"));
        }
        let mut project = self.project.lock().map_err(state_err)?;
        let id = project.alloc_element_id("shape");
        let st = parse_shape_type(&params.shape_type)
            .map_err(invalid_params)?;
        let stroke_width = if params.stroke.as_ref().is_none_or(|s| s.is_empty() || s == "none") { 0.0 } else { 2.0 };
        let element = ShapeElement {
            common: CommonProps {
                id: id.clone(),
                x: params.x,
                y: params.y,
                width: params.size,
                height: params.size,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            shape_type: st,
            fill: params.fill,
            stroke: params.stroke,
            stroke_width,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        };
        let cmd = AddElementCommand::new(Element::Shape(element));
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.push_and_execute(Box::new(cmd), &mut project).map_err(internal_err)?;
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added shape '{}' (id: {})",
            params.shape_type, id
        ))]))
    }

    #[tool(name = "add_text", description = "Add a text element to the canvas")]
    async fn add_text(
        &self,
        Parameters(params): Parameters<AddTextParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if params.content.trim().is_empty() {
            return Err(invalid_params("Text content cannot be empty"));
        }
        if params.font_size.is_nan() || params.font_size <= 0.0 {
            return Err(invalid_params("font_size must be positive"));
        }
        let mut project = self.project.lock().map_err(state_err)?;
        let id = project.alloc_element_id("text");
        let width = text_measure::measure_text_width(
            &params.content, &params.font_family, params.font_size as f32, 400,
        ) as f64;
        let height = params.font_size * 1.2;
        let element = TextElement {
            common: CommonProps {
                id: id.clone(),
                x: params.x,
                y: params.y,
                width,
                height,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            content: params.content.clone(),
            fill: params.fill,
            font_family: params.font_family,
            font_size: params.font_size,
            font_weight: "normal".to_string(),
            letter_spacing: 0.0,
            stroke: None,
            stroke_width: 0.0,
            gradient: None,
        };
        let cmd = AddElementCommand::new(Element::Text(element));
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.push_and_execute(Box::new(cmd), &mut project).map_err(internal_err)?;
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added text '{}' (id: {})",
            params.content, id
        ))]))
    }

    #[tool(name = "add_icon", description = "Add a Lucide icon element to the canvas. Use list_icons to browse available icons.")]
    async fn add_icon(
        &self,
        Parameters(params): Parameters<AddIconParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if icons::get_icon_path(&params.name).is_none() {
            return Err(invalid_params(format!("Icon '{}' not found in Lucide library. Use list_icons to browse.", params.name)));
        }
        if params.size.is_nan() || params.size <= 0.0 {
            return Err(invalid_params("size must be positive"));
        }
        let mut project = self.project.lock().map_err(state_err)?;
        let id = project.alloc_element_id("icon");
        let element = IconElement {
            common: CommonProps {
                id: id.clone(),
                x: params.x,
                y: params.y,
                width: params.size,
                height: params.size,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            name: params.name.clone(),
            fill: params.fill,
            stroke: None,
            stroke_width: 0.0,
            gradient: None,
        };
        let cmd = AddElementCommand::new(Element::Icon(element));
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.push_and_execute(Box::new(cmd), &mut project).map_err(internal_err)?;
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added icon '{}' (id: {})",
            params.name, id
        ))]))
    }

    // -- Element Modification --

    #[tool(name = "set_props", description = "Update properties of an existing element. Pass a JSON object with the properties to change. See parameter docs for valid keys per element type.")]
    async fn set_props(
        &self,
        Parameters(params): Parameters<SetPropsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element '{}' not found. Use list_elements to see all IDs.", params.element_id)))?;

        let mut existing =
            serde_json::to_value(&*elem).map_err(|e| internal_err(format!("Serialization error: {}", e)))?;
        let existing_obj = existing
            .as_object_mut()
            .ok_or_else(|| internal_err("Element is not a JSON object"))?;

        if let Some(incoming) = params.props.as_object() {
            for (key, value) in incoming {
                if key == "type" || key == "id" {
                    continue;
                }
                if let Some(num) = value.as_f64() {
                    match key.as_str() {
                        "width" | "height" if num.is_nan() || num <= 0.0 => {
                            return Err(invalid_params(format!("{} must be positive", key)));
                        }
                        "opacity" if num.is_nan() || !(0.0..=1.0).contains(&num) => {
                            return Err(invalid_params("opacity must be between 0.0 and 1.0"));
                        }
                        "rotation" | "x" | "y" if num.is_nan() => {
                            return Err(invalid_params(format!("{} cannot be NaN", key)));
                        }
                        "font_size" if num.is_nan() || num <= 0.0 => {
                            return Err(invalid_params("font_size must be positive"));
                        }
                        "stroke_width" if num.is_nan() || num < 0.0 => {
                            return Err(invalid_params("stroke_width cannot be negative"));
                        }
                        _ => {}
                    }
                }
                existing_obj.insert(key.clone(), value.clone());
            }
        }

        let updated: Element =
            serde_json::from_value(existing).map_err(|e| internal_err(format!("Deserialization failed: {}", e)))?;
        *elem = updated;

        if let Some(elem) = find_element_deep_mut(project.active_elements_mut(), &params.element_id) {
            if let Element::Text(ref mut t) = elem {
                let text_props = ["content", "font_family", "font_size", "font_weight", "letter_spacing"];
                let needs_recalc = params.props.as_object()
                    .map(|obj| obj.keys().any(|k| text_props.contains(&k.as_str())))
                    .unwrap_or(false);
                if needs_recalc {
                    let weight_u16 = match t.font_weight.as_str() {
                        "light" => 300,
                        "normal" => 400,
                        "medium" => 500,
                        "semibold" => 600,
                        "bold" => 700,
                        other => other.parse::<u16>().unwrap_or(400),
                    };
                    t.common.width = text_measure::measure_text_width(
                        &t.content, &t.font_family, t.font_size as f32, weight_u16,
                    ) as f64;
                    t.common.height = t.font_size * 1.2;
                }
            }

            let needs_path_recalc = params.props.as_object()
                .map(|obj| obj.contains_key("d"))
                .unwrap_or(false);
            if needs_path_recalc {
                crate::model::helpers::recompute_path_natural_dims(elem);
            }
        }
        project.bump_version();

        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Updated element '{}'",
            params.element_id
        ))]))
    }

    #[tool(name = "remove_element", description = "Remove an element by ID")]
    async fn remove_element(
        &self,
        Parameters(params): Parameters<RemoveElementParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let target_id = params.element_id.clone();
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;
        let elements = project.active_elements_mut();

        // Try top-level first
        if let Some(idx) = elements.iter().position(|e| element_id(e) == target_id) {
            elements.remove(idx);
        } else {
            fn remove_from_group(elements: &mut [Element], target_id: &str) -> bool {
                for elem in elements.iter_mut() {
                    if let Element::Group(g) = elem {
                        if let Some(idx) = g.children.iter().position(|c| element_id(c) == target_id) {
                            let old_gx = g.common.x;
                            let old_gy = g.common.y;
                            g.children.remove(idx);
                            if !g.children.is_empty() {
                                let (bx, by, bw, bh) = calc_group_bounds(&g.children);
                                let dx = bx - old_gx;
                                let dy = by - old_gy;
                                if dx != 0.0 || dy != 0.0 {
                                    for child in &mut g.children {
                                        let c = child.common_mut();
                                        c.x -= dx;
                                        c.y -= dy;
                                    }
                                }
                                g.common.x = bx;
                                g.common.y = by;
                                g.common.width = bw;
                                g.common.height = bh;
                            }
                            return true;
                        }
                        if remove_from_group(&mut g.children, target_id) {
                            return true;
                        }
                    }
                }
                false
            }
            if !remove_from_group(elements, &target_id) {
                return Err(invalid_params(format!("Element '{}' not found", params.element_id)));
            }
        }

        crate::services::elements::cleanup_clip_mask_refs(project.active_elements_mut(), &target_id);
        project.bump_version();
        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Removed element '{}'",
            params.element_id
        ))]))
    }

    #[tool(name = "set_layout", description = "Apply layout to arrange all elements. Types: center, horizontal, vertical, stack")]
    async fn set_layout(
        &self,
        Parameters(params): Parameters<SetLayoutParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        if project.active_elements().is_empty() {
            drop(project);
            return Ok(CallToolResult::success(vec![Content::text(
                "No elements to layout.".to_string(),
            )]));
        }

        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;
        let cw = project.active_canvas().width;
        let ch = project.active_canvas().height;

        crate::services::layout::apply_layout(
            project.active_elements_mut(),
            cw,
            ch,
            &params.layout_type,
            params.gap,
            params.padding,
        ).map_err(|e| invalid_params(e.to_string()))?;

        let count = project.active_elements().len();
        project.bump_version();
        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Applied '{}' layout to {} elements",
            params.layout_type, count
        ))]))
    }

    // -- Style --

    #[tool(name = "set_gradient", description = "Set gradient fill on an element. Types: linear, radial")]
    async fn set_gradient(
        &self,
        Parameters(params): Parameters<SetGradientParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if params.colors.len() < 2 {
            return Err(invalid_params("Gradient requires at least 2 colors"));
        }

        let kind = match params.gradient_type.as_str() {
            "linear" => GradientKind::Linear,
            "radial" => GradientKind::Radial,
            other => return Err(invalid_params(format!("Invalid gradient type: {}. Use 'linear' or 'radial'.", other))),
        };
        let gradient = Gradient {
            gradient_type: kind,
            colors: params.colors.clone(),
            angle: params.angle,
            stops: Vec::new(),
        };

        let mut project = self.project.lock().map_err(state_err)?;
        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;
        match elem {
            Element::Image(_) | Element::Path(_) | Element::Group(_) => return Err(invalid_params("This element type does not support gradients")),
            _ => {}
        }
        let old_gradient = get_element_gradient(elem).cloned();
        let new_gradient = gradient.clone();
        set_element_gradient(elem, gradient);
        project.bump_version();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SetGradientCommand::new(
                params.element_id.clone(),
                old_gradient,
                Some(new_gradient),
            )));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Set {} gradient on '{}'",
            params.element_id, params.gradient_type
        ))]))
    }

    // -- Export --

    #[tool(name = "export_svg", description = "Export current icon as SVG file")]
    async fn export_svg(
        &self,
        Parameters(params): Parameters<ExportSvgParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let svg_str = {
            let project = self.project.lock().map_err(state_err)?;
            let mut cache = self.cache.lock().map_err(state_err)?;
            crate::services::export::build_svg(&project, &mut cache)
                .map_err(|e| internal_err(format!("Build error: {}", e)))?
        };

        match params.path {
            Some(p) => {
                crate::services::export::write_svg_to_file(&svg_str, &p)
                    .map_err(|e| internal_err(format!("SVG export failed: {}", e)))?;
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "SVG exported to '{}'",
                    p
                ))]))
            }
            None => Ok(CallToolResult::success(vec![Content::text(svg_str)])),
        }
    }

    #[tool(name = "export_png", description = "Export current icon as PNG files at specified sizes")]
    async fn export_png(
        &self,
        Parameters(params): Parameters<ExportPngParams>,
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

        let paths = crate::services::export::write_pngs_to_dir(&svg_str, &params.sizes, &params.output_dir)
            .map_err(|e| internal_err(format!("PNG export error: {}", e)))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Exported {} PNG files: {}",
            paths.len(),
            paths.join(", ")
        ))]))
    }

    // -- Dynamic tool loading --

    #[tool(name = "load_extended_tools", description = "Load extended tool set. All tools are available by default; this is kept for backward compatibility.")]
    async fn load_extended_tools(&self) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![Content::text(
            "All 62 tools are already available.".to_string(),
        )]))
    }
}
