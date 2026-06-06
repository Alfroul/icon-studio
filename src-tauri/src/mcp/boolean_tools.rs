use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{internal_err, invalid_params, state_err, IconStudioHandler};
use crate::engine::boolean::{self, BooleanOp};
use crate::model::helpers::find_element_index;
use crate::model::history::AddElementCommand;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct BooleanOperationParams {
    #[schemars(description = "元素 A 的 ID")]
    pub element_a_id: String,
    #[schemars(description = "元素 B 的 ID")]
    pub element_b_id: String,
    #[schemars(description = "运算类型：union（合并）、subtract（减去）、intersect（交集）、exclude（差集）")]
    pub operation: String,
}

#[tool_router(router = boolean_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "boolean_operation", description = "对两个形状/路径元素执行布尔运算（union/subtract/intersect/exclude），生成新的路径元素")]
    async fn boolean_operation_tool(
        &self,
        Parameters(params): Parameters<BooleanOperationParams>,
    ) -> Result<CallToolResult, ErrorData> {
        // 1. Parse operation type
        let op = match params.operation.as_str() {
            "union" => BooleanOp::Union,
            "subtract" => BooleanOp::Subtract,
            "intersect" => BooleanOp::Intersect,
            "exclude" => BooleanOp::Exclude,
            other => {
                return Err(invalid_params(format!(
                    "无效运算类型：{}。可选：union, subtract, intersect, exclude",
                    other
                )))
            }
        };

        // 2. Lock project and find both elements
        let mut project = self.project.lock().map_err(state_err)?;

        // Elements can't be the same
        if params.element_a_id == params.element_b_id {
            return Err(invalid_params("元素 A 和元素 B 不能相同"));
        }

        // Find element A and B by index (need clones since we reference both)
        let idx_a = find_element_index(project.active_elements(), &params.element_a_id)
            .ok_or_else(|| invalid_params(format!("元素 A 未找到：{}", params.element_a_id)))?;
        let idx_b = find_element_index(project.active_elements(), &params.element_b_id)
            .ok_or_else(|| invalid_params(format!("元素 B 未找到：{}", params.element_b_id)))?;

        let elem_a = project.active_elements()[idx_a].clone();
        let elem_b = project.active_elements()[idx_b].clone();

        // 3. Perform boolean operation
        let (mut path_element, _boolean_source) = boolean::perform_boolean(&elem_a, &elem_b, op.clone())
            .map_err(|e| internal_err(format!("布尔运算失败：{}", e)))?;

        // 4. Assign new ID
        let new_id = project.alloc_element_id("path");
        path_element.common.id = new_id.clone();

        // 5. Add to project using history
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history
                .push_and_execute(
                    Box::new(AddElementCommand::new(crate::model::Element::Path(path_element))),
                    &mut project,
                )
                .map_err(internal_err)?;
        }

        project.bump_version();
        drop(project);

        {
            let mut cache = self.cache.lock().map_err(state_err)?;
            cache.invalidate_cache();
        }
        self.emit_change();

        let op_name = match op {
            BooleanOp::Union => "合并",
            BooleanOp::Subtract => "减去",
            BooleanOp::Intersect => "交集",
            BooleanOp::Exclude => "差集",
        };

        Ok(CallToolResult::success(vec![Content::text(format!(
            "布尔运算完成：{}（{} ∘ {}）→ 新元素 '{}'",
            op_name, params.element_a_id, params.element_b_id, new_id
        ))]))
    }
}
