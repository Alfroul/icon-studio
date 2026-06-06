use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{invalid_params, state_err, IconStudioHandler};
use crate::model::helpers::*;
use crate::model::history::SnapshotCommand;
use crate::model::symbol::{SymbolDef, SymbolInstanceElement, SymbolOverride, detach_symbol};
use crate::model::{CommonProps, Element};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateSymbolParams {
    #[schemars(description = "要转为符号的元素 ID")]
    pub element_id: String,
    #[schemars(description = "符号名称")]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UpdateSymbolParams {
    #[schemars(description = "符号定义 ID")]
    pub symbol_id: String,
    #[schemars(description = "用于更新主组件的元素 ID")]
    pub element_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DetachSymbolParams {
    #[schemars(description = "要脱离的符号实例元素 ID")]
    pub element_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AddOverrideParams {
    #[schemars(description = "符号实例元素 ID")]
    pub element_id: String,
    #[schemars(description = "要覆盖的属性名（fill/opacity/x/y/width/height/rotation/stroke/content）")]
    pub property: String,
    #[schemars(description = "覆盖值")]
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RemoveOverrideParams {
    #[schemars(description = "符号实例元素 ID")]
    pub element_id: String,
    #[schemars(description = "要移除的覆盖属性名")]
    pub property: String,
}

#[tool_router(router = symbol_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "create_symbol", description = "将指定元素转为符号定义，并在原位置创建一个引用该定义的实例")]
    async fn create_symbol_tool(
        &self,
        Parameters(params): Parameters<CreateSymbolParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        let idx = find_element_index(project.active_elements(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("元素 '{}' 未找到", params.element_id)))?;

        let elem_snapshot = project.active_elements()[idx].clone();
        let elem_common = elem_snapshot.common().clone();
        let instance_id = elem_snapshot.id().to_string();
        let symbol_id = project.alloc_element_id("symbol");

        let def = SymbolDef {
            id: symbol_id.clone(),
            name: params.name.clone(),
            source_element: elem_snapshot,
            overridable_props: vec![
                "fill".into(), "opacity".into(), "x".into(), "y".into(),
                "width".into(), "height".into(), "rotation".into(), "stroke".into(), "content".into(),
            ],
        };

        let instance = SymbolInstanceElement {
            common: CommonProps {
                id: instance_id.clone(),
                ..elem_common
            },
            symbol_id: symbol_id.clone(),
            overrides: vec![],
        };

        project.active_elements_mut()[idx] = Element::Symbol(instance);
        project.symbols.insert(symbol_id.clone(), def);
        project.bump_version();

        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "已创建符号 '{}'（ID: {}），原元素已替换为实例",
            params.name, symbol_id
        ))]))
    }

    #[tool(name = "list_symbols", description = "列出所有符号定义的 ID、名称和实例数量")]
    async fn list_symbols_tool(
        &self,
    ) -> Result<CallToolResult, ErrorData> {
        let project = self.project.lock().map_err(state_err)?;

        if project.symbols.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text("当前项目没有符号定义")]));
        }

        let mut result = Vec::new();
        for (id, def) in &project.symbols {
            let instance_count = count_instances(project.active_elements(), id);
            result.push(serde_json::json!({
                "id": id,
                "name": def.name,
                "instance_count": instance_count,
                "source_type": element_type_name(&def.source_element),
            }));
        }

        drop(project);
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&result).unwrap_or_default())]))
    }

    #[tool(name = "update_symbol", description = "用指定元素的当前状态更新符号定义的主组件")]
    async fn update_symbol_tool(
        &self,
        Parameters(params): Parameters<UpdateSymbolParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        let elem_snapshot = find_element_deep(project.active_elements(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("元素 '{}' 未找到", params.element_id)))?
            .clone();

        let def = project.symbols.get_mut(&params.symbol_id)
            .ok_or_else(|| invalid_params(format!("符号 '{}' 未找到", params.symbol_id)))?;
        def.source_element = elem_snapshot;
        project.bump_version();
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "已更新符号 '{}' 的主组件",
            params.symbol_id
        ))]))
    }

    #[tool(name = "detach_symbol", description = "将符号实例脱离为独立元素")]
    async fn detach_symbol_tool(
        &self,
        Parameters(params): Parameters<DetachSymbolParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        let idx = find_element_index(project.active_elements(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("元素 '{}' 未找到", params.element_id)))?;

        let instance = match &project.active_elements()[idx] {
            Element::Symbol(inst) => inst.clone(),
            _ => return Err(invalid_params(format!("元素 '{}' 不是符号实例", params.element_id))),
        };

        let independent = detach_symbol(&instance, &project.symbols)
            .ok_or_else(|| invalid_params(format!("符号定义 '{}' 未找到", instance.symbol_id)))?;

        project.active_elements_mut()[idx] = independent;
        project.bump_version();

        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "已将符号实例 '{}' 脱离为独立元素",
            params.element_id
        ))]))
    }

    #[tool(name = "add_override", description = "为符号实例添加属性覆盖")]
    async fn add_override_tool(
        &self,
        Parameters(params): Parameters<AddOverrideParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        let elem = find_element_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("元素 '{}' 未找到", params.element_id)))?;

        match elem {
            Element::Symbol(inst) => {
                // Remove existing override for same property
                inst.overrides.retain(|o| o.property != params.property);
                inst.overrides.push(SymbolOverride {
                    property: params.property.clone(),
                    value: params.value.clone(),
                });
            }
            _ => return Err(invalid_params(format!("元素 '{}' 不是符号实例", params.element_id))),
        }
        project.bump_version();
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "已为实例 '{}' 添加覆盖：{} = {}",
            params.element_id, params.property, params.value
        ))]))
    }

    #[tool(name = "remove_override", description = "移除符号实例的指定属性覆盖")]
    async fn remove_override_tool(
        &self,
        Parameters(params): Parameters<RemoveOverrideParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        let elem = find_element_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("元素 '{}' 未找到", params.element_id)))?;

        let removed = match elem {
            Element::Symbol(inst) => {
                let before = inst.overrides.len();
                inst.overrides.retain(|o| o.property != params.property);
                before != inst.overrides.len()
            }
            _ => return Err(invalid_params(format!("元素 '{}' 不是符号实例", params.element_id))),
        };
        project.bump_version();
        drop(project);
        self.emit_change();

        if removed {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "已移除实例 '{}' 的覆盖：{}", params.element_id, params.property
            ))]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "实例 '{}' 没有覆盖：{}", params.element_id, params.property
            ))]))
        }
    }
}

fn count_instances(elements: &[Element], symbol_id: &str) -> usize {
    elements.iter().map(|e| {
        match e {
            Element::Symbol(inst) if inst.symbol_id == symbol_id => 1,
            _ => 0,
        }
    }).sum()
}

fn element_type_name(elem: &Element) -> &'static str {
    match elem {
        Element::Shape(_) => "shape",
        Element::Text(_) => "text",
        Element::Icon(_) => "icon",
        Element::Image(_) => "image",
        Element::Path(_) => "path",
        Element::Group(_) => "group",
        Element::Symbol(_) => "symbol",
    }
}
