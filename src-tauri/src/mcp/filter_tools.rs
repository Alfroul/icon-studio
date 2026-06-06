use std::collections::HashMap;

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{invalid_params, state_err, IconStudioHandler, RemoveElementParams};
use crate::model::filter::{FilterType, SvgFilter};
use crate::model::helpers::{find_element_deep_mut, get_element_filter};
use crate::model::history::SetFilterCommand;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetFilterParams {
    #[schemars(description = "Element ID to apply filter to")]
    pub element_id: String,
    #[schemars(description = "Filter type: noise, blur, pixelate, emboss, posterize, turbulence")]
    pub filter_type: String,
    #[schemars(description = "Filter parameters (varies by type):\n- noise: baseFrequency (0.001-0.1), numOctaves (1-5)\n- blur: stdDeviation (0-20)\n- pixelate: size (2-20)\n- emboss: strength (0.5-3.0)\n- posterize: steps (2-10)\n- turbulence: baseFrequency (0.001-0.1), numOctaves (1-5)")]
    #[serde(default)]
    pub params: HashMap<String, f64>,
}

#[tool_router(router = filter_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "set_filter", description = "Apply an SVG filter effect to an element")]
    async fn set_filter(
        &self,
        Parameters(params): Parameters<SetFilterParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let filter_type = match params.filter_type.as_str() {
            "noise" => FilterType::Noise,
            "blur" => FilterType::Blur,
            "pixelate" => FilterType::Pixelate,
            "emboss" => FilterType::Emboss,
            "posterize" => FilterType::Posterize,
            "turbulence" => FilterType::Turbulence,
            other => return Err(invalid_params(format!("Unknown filter type: '{}'. Valid types: noise, blur, pixelate, emboss, posterize, turbulence", other))),
        };

        let filter = SvgFilter {
            filter_type,
            params: params.params,
        };

        let mut project = self.project.lock().map_err(state_err)?;
        let old_filter = {
            let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
                .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;
            let old = get_element_filter(elem).cloned();
            elem.common_mut().svg_filter = Some(filter.clone());
            old
        };
        project.bump_version();

        let mut history = self.history.lock().map_err(state_err)?;
        history.record(Box::new(SetFilterCommand::new(
            params.element_id.clone(),
            old_filter,
            Some(filter),
        )));

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Filter '{}' applied to element '{}'",
            params.filter_type, params.element_id
        ))]))
    }

    #[tool(name = "clear_filter", description = "Remove SVG filter from an element")]
    async fn clear_filter(
        &self,
        Parameters(params): Parameters<RemoveElementParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let old_filter = {
            let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
                .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;
            let old = get_element_filter(elem).cloned();
            elem.common_mut().svg_filter = None;
            old
        };
        project.bump_version();

        let mut history = self.history.lock().map_err(state_err)?;
        history.record(Box::new(SetFilterCommand::new(
            params.element_id.clone(),
            old_filter,
            None,
        )));

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Filter cleared from element '{}'",
            params.element_id
        ))]))
    }
}
