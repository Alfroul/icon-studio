use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{invalid_params, state_err, IconStudioHandler, RemoveElementParams};
use crate::model::helpers::*;
use crate::model::history::SnapshotCommand;
use crate::model::{Animation, AnimationType};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetAnimationParams {
    #[schemars(description = "元素 ID")]
    pub element_id: String,
    #[schemars(description = "动画类型：rotate、scale、fade、translate、path")]
    pub animation_type: String,
    #[schemars(description = "持续时间（秒）")]
    #[serde(default = "default_duration")]
    pub duration: f64,
    #[schemars(description = "延迟（秒）")]
    #[serde(default)]
    pub delay: f64,
    #[schemars(description = "是否循环")]
    #[serde(default = "default_true")]
    pub repeat: bool,
    #[schemars(description = "缓动函数：ease-in-out / linear / ease / ease-in / ease-out")]
    #[serde(default = "default_easing")]
    pub easing: String,
    #[schemars(description = "类型专属参数 JSON（如 {\"min_scale\":0.8}、{\"dx\":10,\"dy\":0}、{\"path\":\"M0,0 L10,0\"}）")]
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

fn default_duration() -> f64 { 2.0 }
fn default_true() -> bool { true }
fn default_easing() -> String { "ease-in-out".to_string() }

fn collect_animations(elements: &[crate::model::Element], result: &mut Vec<String>) {
    use crate::model::Element;
    for elem in elements {
        if let Some(ref anim) = elem.common().animation {
            let type_str = match anim.animation_type {
                AnimationType::Rotate => "rotate",
                AnimationType::Scale => "scale",
                AnimationType::Fade => "fade",
                AnimationType::Translate => "translate",
                AnimationType::Path => "path",
            };
            result.push(format!(
                "{}: {} ({}s, {})",
                elem.id(),
                type_str,
                anim.duration,
                if anim.repeat { "循环" } else { "单次" }
            ));
        }
        if let Element::Group(g) = elem {
            collect_animations(&g.children, result);
        }
    }
}

#[tool_router(router = animation_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "set_animation", description = "为元素设置 SVG 动画。支持：rotate（旋转）、scale（缩放/脉冲）、fade（渐变）、translate（平移）、path（路径动画）")]
    async fn set_animation(
        &self,
        Parameters(params): Parameters<SetAnimationParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let anim_type = match params.animation_type.as_str() {
            "rotate" => AnimationType::Rotate,
            "scale" => AnimationType::Scale,
            "fade" => AnimationType::Fade,
            "translate" => AnimationType::Translate,
            "path" => AnimationType::Path,
            other => return Err(invalid_params(format!("无效动画类型：{}。有效：rotate、scale、fade、translate、path", other))),
        };

        if params.duration.is_nan() || params.duration <= 0.0 {
            return Err(invalid_params("duration 必须为正数"));
        }
        if params.delay.is_nan() || params.delay < 0.0 {
            return Err(invalid_params("delay 不能为负数"));
        }

        let animation = Animation {
            animation_type: anim_type,
            duration: params.duration,
            delay: params.delay,
            repeat: params.repeat,
            easing: params.easing.clone(),
            params: params.params.unwrap_or(serde_json::Value::Null),
        };

        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("元素未找到：{}", params.element_id)))?;
        elem.common_mut().animation = Some(animation);
        project.bump_version();

        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "已为 '{}' 设置 {} 动画（{}秒，{}）",
            params.element_id, params.animation_type, params.duration,
            if params.repeat { "循环" } else { "单次" }
        ))]))
    }

    #[tool(name = "clear_animation", description = "清除元素的动画")]
    async fn clear_animation(
        &self,
        Parameters(params): Parameters<RemoveElementParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("元素未找到：{}", params.element_id)))?;
        elem.common_mut().animation = None;
        project.bump_version();

        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "已清除 '{}' 的动画",
            params.element_id
        ))]))
    }

    #[tool(name = "list_animations", description = "列出当前所有带动画的元素")]
    async fn list_animations(&self) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;
        let mut result = Vec::new();
        collect_animations(project.active_elements(), &mut result);
        drop(project);

        if result.is_empty() {
            Ok(CallToolResult::success(vec![Content::text("没有元素设置了动画".to_string())]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(result.join("\n"))]))
        }
    }
}
