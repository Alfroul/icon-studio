use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{invalid_params, state_err, IconStudioHandler, RemoveElementParams};
use crate::model::helpers::*;
use crate::model::history::{CanvasCommand, SetGradientCommand, SetShadowCommand, SnapshotCommand};
use crate::model::{Element, Gradient, GradientKind, Shadow};

fn default_shadow_color() -> String {
    "#00000040".to_string()
}
fn default_8() -> f64 {
    8.0
}
fn default_4() -> f64 {
    4.0
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetShadowParams {
    #[schemars(description = "Element ID")]
    pub element_id: String,
    #[schemars(description = "Shadow color with alpha (hex, e.g. #00000040)")]
    #[serde(default = "default_shadow_color")]
    pub color: String,
    #[schemars(description = "Blur radius in pixels")]
    #[serde(default = "default_8")]
    pub blur: f64,
    #[schemars(description = "Horizontal offset")]
    #[serde(default)]
    pub offset_x: f64,
    #[schemars(description = "Vertical offset")]
    #[serde(default = "default_4")]
    pub offset_y: f64,
    #[schemars(description = "Inset shadow (true = inner shadow)")]
    #[serde(default)]
    pub inset: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetCanvasGradientParams {
    #[schemars(description = "Gradient type: linear or radial")]
    pub gradient_type: String,
    #[schemars(description = "Gradient color stops (hex colors)")]
    pub colors: Vec<String>,
    #[schemars(description = "Angle in degrees (linear only)")]
    #[serde(default)]
    pub angle: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetBorderRadiusParams {
    #[schemars(description = "Shape element ID")]
    pub element_id: String,
    #[schemars(description = "Border radius in pixels")]
    pub radius: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetStrokeDasharrayParams {
    #[schemars(description = "Element ID")]
    pub element_id: String,
    #[schemars(description = "Dash pattern (e.g. '5,3'). Empty string clears dashes.")]
    pub dasharray: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AddShadowParams {
    #[schemars(description = "Element ID")]
    pub element_id: String,
    #[schemars(description = "Shadow color with alpha (hex, e.g. #00000040)")]
    #[serde(default = "default_shadow_color")]
    pub color: String,
    #[schemars(description = "Blur radius in pixels")]
    #[serde(default = "default_8")]
    pub blur: f64,
    #[schemars(description = "Horizontal offset")]
    #[serde(default)]
    pub offset_x: f64,
    #[schemars(description = "Vertical offset")]
    #[serde(default = "default_4")]
    pub offset_y: f64,
    #[schemars(description = "Inset shadow (true = inner shadow)")]
    #[serde(default)]
    pub inset: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RemoveShadowParams {
    #[schemars(description = "Element ID")]
    pub element_id: String,
    #[schemars(description = "Shadow index to remove (0-based)")]
    pub index: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetBlendModeParams {
    #[schemars(description = "Element ID")]
    pub element_id: String,
    #[schemars(description = "Blend mode: normal, multiply, screen, overlay, darken, lighten, color-dodge, color-burn, hard-light, soft-light, difference, exclusion. Empty string clears.")]
    pub mode: String,
}

#[tool_router(router = style_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "set_shadow", description = "Set drop shadow on element (replaces existing shadows)")]
    async fn set_shadow(
        &self,
        Parameters(params): Parameters<SetShadowParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if params.blur.is_nan() || params.blur < 0.0 {
            return Err(invalid_params("blur must be non-negative"));
        }
        if params.offset_x.is_nan() || params.offset_y.is_nan() {
            return Err(invalid_params("offset cannot be NaN"));
        }

        let shadow = Shadow {
            color: params.color.clone(),
            blur: params.blur,
            offset_x: params.offset_x,
            offset_y: params.offset_y,
            inset: params.inset,
        };

        let mut project = self.project.lock().map_err(state_err)?;
        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;
        let old_shadow = get_element_shadow(elem).cloned();
        let new_shadow = shadow.clone();
        set_element_shadow(elem, shadow);
        project.bump_version();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SetShadowCommand::new(
                params.element_id.clone(),
                old_shadow,
                Some(new_shadow),
            )));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Set shadow on '{}'",
            params.element_id
        ))]))
    }

    #[tool(name = "clear_gradient", description = "Remove gradient from element")]
    async fn clear_gradient(
        &self,
        Parameters(params): Parameters<RemoveElementParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;
        let old_gradient = get_element_gradient(elem).cloned();
        clear_element_gradient(elem);
        project.bump_version();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SetGradientCommand::new(
                params.element_id.clone(),
                old_gradient,
                None,
            )));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Cleared gradient on '{}'",
            params.element_id
        ))]))
    }

    #[tool(name = "clear_shadow", description = "Remove all shadows from element")]
    async fn clear_shadow(
        &self,
        Parameters(params): Parameters<RemoveElementParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;
        let old_shadow = get_element_shadow(elem).cloned();
        clear_element_shadow(elem);
        project.bump_version();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SetShadowCommand::new(
                params.element_id.clone(),
                old_shadow,
                None,
            )));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Cleared shadow on '{}'",
            params.element_id
        ))]))
    }

    #[tool(name = "set_canvas_gradient", description = "Set canvas background gradient (replaces solid background color)")]
    async fn set_canvas_gradient(
        &self,
        Parameters(params): Parameters<SetCanvasGradientParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if params.colors.len() < 2 {
            return Err(invalid_params("Gradient requires at least 2 colors"));
        }

        let kind = match params.gradient_type.as_str() {
            "linear" => GradientKind::Linear,
            "radial" => GradientKind::Radial,
            other => return Err(invalid_params(format!("Invalid gradient type: {}", other))),
        };
        let gradient = Gradient {
            gradient_type: kind,
            colors: params.colors.clone(),
            angle: params.angle,
            stops: Vec::new(),
        };

        let mut project = self.project.lock().map_err(state_err)?;
        let old_canvas = project.active_canvas().clone();
        project.active_canvas_mut().background_gradient = Some(gradient);
        project.bump_version();
        let new_canvas = project.active_canvas().clone();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(CanvasCommand::new(old_canvas, new_canvas)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Set canvas {} gradient background", params.gradient_type
        ))]))
    }

    #[tool(name = "clear_canvas_gradient", description = "Clear canvas background gradient, restore solid color")]
    async fn clear_canvas_gradient(&self) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let old_canvas = project.active_canvas().clone();
        project.active_canvas_mut().background_gradient = None;
        project.bump_version();
        let new_canvas = project.active_canvas().clone();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(CanvasCommand::new(old_canvas, new_canvas)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(
            "Cleared canvas gradient background".to_string(),
        )]))
    }

    #[tool(name = "set_border_radius", description = "Set border radius on a shape element (pixels)")]
    async fn set_border_radius(
        &self,
        Parameters(params): Parameters<SetBorderRadiusParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;
        if params.radius < 0.0 {
            return Err(invalid_params("Border radius cannot be negative"));
        }
        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;

        match elem {
            Element::Shape(s) => {
                s.border_radius = params.radius;
            }
            _ => {
                drop(project);
                return Err(invalid_params("border_radius only works on shape elements"));
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
            "Set border radius {}px on '{}'", params.element_id, params.radius
        ))]))
    }

    #[tool(name = "set_stroke_dasharray", description = "Set stroke dash pattern (e.g. '5,3'). Empty string clears.")]
    async fn set_stroke_dasharray(
        &self,
        Parameters(params): Parameters<SetStrokeDasharrayParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;
        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;

        if !params.dasharray.is_empty() {
            let parts: Vec<&str> = params.dasharray.split(&[',', ' '][..]).filter(|s| !s.is_empty()).collect();
            for part in parts {
                match part.parse::<f64>() {
                    Ok(v) if v > 0.0 => {}
                    _ => return Err(invalid_params(format!(
                        "Invalid dash format '{}': must be comma-separated positive numbers (e.g. '5,3')", params.dasharray
                    ))),
                }
            }
        }

        let da = if params.dasharray.is_empty() {
            None
        } else {
            Some(params.dasharray.clone())
        };

        match elem {
            Element::Shape(s) => s.stroke_dasharray = da,
            Element::Path(p) => p.stroke_dasharray = da,
            _ => {
                drop(project);
                return Err(invalid_params("stroke_dasharray only works on shape and path elements"));
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

        Ok(CallToolResult::success(vec![Content::text(if params.dasharray.is_empty() {
            format!("Cleared dash style on '{}'", params.element_id)
        } else {
            format!("Set dash '{}' on '{}'", params.dasharray, params.element_id)
        })]))
    }

    #[tool(name = "add_shadow", description = "Add an additional shadow to element (multi-shadow support)")]
    async fn add_shadow(
        &self,
        Parameters(params): Parameters<AddShadowParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if params.blur.is_nan() || params.blur < 0.0 {
            return Err(invalid_params("blur must be non-negative"));
        }
        if params.offset_x.is_nan() || params.offset_y.is_nan() {
            return Err(invalid_params("offset cannot be NaN"));
        }

        let shadow = Shadow {
            color: params.color.clone(),
            blur: params.blur,
            offset_x: params.offset_x,
            offset_y: params.offset_y,
            inset: params.inset,
        };

        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;
        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;
        add_element_shadow(elem, shadow);
        project.bump_version();
        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added shadow to '{}'", params.element_id
        ))]))
    }

    #[tool(name = "remove_shadow", description = "Remove shadow at specified index from element")]
    async fn remove_shadow(
        &self,
        Parameters(params): Parameters<RemoveShadowParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;
        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;
        if !remove_element_shadow(elem, params.index) {
            return Err(invalid_params(format!(
                "Index {} out of range (current shadows: {})",
                params.index, elem.common().shadows.len()
            )));
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
            "Removed shadow {} from '{}'", params.element_id, params.index
        ))]))
    }

    #[tool(name = "set_blend_mode", description = "Set element blend mode")]
    async fn set_blend_mode(
        &self,
        Parameters(params): Parameters<SetBlendModeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let valid_modes = [
            "normal", "multiply", "screen", "overlay", "darken", "lighten",
            "color-dodge", "color-burn", "hard-light", "soft-light", "difference", "exclusion",
        ];
        let mode = if params.mode.is_empty() {
            None
        } else if valid_modes.contains(&params.mode.as_str()) {
            Some(params.mode.clone())
        } else {
            return Err(invalid_params(format!("Invalid blend mode: {}. Valid: {}", params.mode, valid_modes.join(", "))));
        };

        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;
        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;
        elem.common_mut().blend_mode = mode;
        project.bump_version();
        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(if params.mode.is_empty() {
            format!("Cleared blend mode on '{}'", params.element_id)
        } else {
            format!("Set blend mode {} on '{}'", params.mode, params.element_id)
        })]))
    }
}
