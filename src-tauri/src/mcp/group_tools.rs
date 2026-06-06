use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{invalid_params, state_err, IconStudioHandler};
use crate::model::group::GroupElement;
use crate::model::helpers::*;
use crate::model::history::SnapshotCommand;
use crate::model::{CommonProps, Element};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GroupElementsParams {
    #[schemars(description = "要编组的元素 ID 列表")]
    pub element_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UngroupParams {
    #[schemars(description = "要解散的组元素 ID")]
    pub group_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AddToGroupParams {
    #[schemars(description = "目标组元素 ID")]
    pub group_id: String,
    #[schemars(description = "要添加进组的元素 ID")]
    pub element_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RemoveFromGroupParams {
    #[schemars(description = "组元素 ID")]
    pub group_id: String,
    #[schemars(description = "要从组中移除的子元素 ID")]
    pub element_id: String,
}

#[tool_router(router = group_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "group_elements", description = "将指定元素列表编为一个组")]
    async fn group_elements_tool(
        &self,
        Parameters(params): Parameters<GroupElementsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if params.element_ids.len() < 2 {
            return Err(invalid_params("至少需要 2 个元素才能编组"));
        }

        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        let mut children = Vec::new();
        let mut first_index = None;
        for eid in &params.element_ids {
            let idx = find_element_index(project.active_elements(), eid)
                .ok_or_else(|| invalid_params(format!("元素 '{}' 未找到", eid)))?;
            if first_index.is_none() || idx < first_index.unwrap_or(usize::MAX) {
                first_index = Some(idx);
            }
            children.push(project.active_elements()[idx].clone());
        }

        let (min_x, min_y, gw, gh) = calc_group_bounds(&children);

        for child in &mut children {
            let c = child.common_mut();
            c.x -= min_x;
            c.y -= min_y;
        }

        let group_id = project.alloc_element_id("group");
        let group = GroupElement {
            common: CommonProps {
                id: group_id.clone(),
                x: min_x,
                y: min_y,
                width: gw,
                height: gh,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            children,
            expanded: true,
        };

        let insert_at = first_index.unwrap_or(0);
        let ids_set: Vec<String> = params.element_ids.clone();
        project.active_elements_mut().retain(|e| !ids_set.contains(&e.id().to_string()));
        let insert_at = insert_at.min(project.active_elements().len());
        project.active_elements_mut().insert(insert_at, Element::Group(group));
        project.bump_version();

        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "已将 {} 个元素编组为 '{}'（位置：{:.0},{:.0}，大小：{:.0}x{:.0}）",
            params.element_ids.len(), group_id, min_x, min_y, gw, gh
        ))]))
    }

    #[tool(name = "ungroup", description = "解散组，将子元素恢复为独立元素")]
    async fn ungroup_tool(
        &self,
        Parameters(params): Parameters<UngroupParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        let idx = find_element_index(project.active_elements(), &params.group_id)
            .ok_or_else(|| invalid_params(format!("组 '{}' 未找到", params.group_id)))?;

        let group = match &project.active_elements()[idx] {
            Element::Group(g) => g.clone(),
            _ => return Err(invalid_params(format!("元素 '{}' 不是组", params.group_id))),
        };

        let child_count = group.children.len();
        let mut released = Vec::new();
        for mut child in group.children {
            let c = child.common_mut();
            c.x += group.common.x;
            c.y += group.common.y;
            released.push(child);
        }

        project.active_elements_mut().remove(idx);
        for (i, elem) in released.into_iter().enumerate() {
            project.active_elements_mut().insert(idx + i, elem);
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
            "已解散组 '{}'，恢复了 {} 个子元素",
            params.group_id, child_count
        ))]))
    }

    #[tool(name = "add_to_group", description = "向现有组添加元素")]
    async fn add_to_group_tool(
        &self,
        Parameters(params): Parameters<AddToGroupParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        let group_idx = find_element_index(project.active_elements(), &params.group_id)
            .ok_or_else(|| invalid_params(format!("组 '{}' 未找到", params.group_id)))?;

        let elem_idx = find_element_index(project.active_elements(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("元素 '{}' 未找到", params.element_id)))?;

        if group_idx == elem_idx {
            return Err(invalid_params("不能将组添加到自身"));
        }

        let mut elem = project.active_elements()[elem_idx].clone();
        let (gx, gy) = {
            let g = match &project.active_elements()[group_idx] {
                Element::Group(g) => g,
                _ => return Err(invalid_params(format!("元素 '{}' 不是组", params.group_id))),
            };
            (g.common.x, g.common.y)
        };

        let c = elem.common_mut();
        c.x -= gx;
        c.y -= gy;

        let removed_id = params.element_id.clone();
        project.active_elements_mut().remove(elem_idx);
        let group_idx = if elem_idx < group_idx { group_idx - 1 } else { group_idx };
        if let Element::Group(g) = &mut project.active_elements_mut()[group_idx] {
            let old_gx = g.common.x;
            let old_gy = g.common.y;
            g.children.push(elem);
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
        project.bump_version();

        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "已将元素 '{}' 添加到组 '{}'", removed_id, params.group_id
        ))]))
    }

    #[tool(name = "remove_from_group", description = "从组中移除元素，恢复为独立元素")]
    async fn remove_from_group_tool(
        &self,
        Parameters(params): Parameters<RemoveFromGroupParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        let group_idx = find_element_index(project.active_elements(), &params.group_id)
            .ok_or_else(|| invalid_params(format!("组 '{}' 未找到", params.group_id)))?;

        let (gx, gy, child_count, found) = {
            let g = match &project.active_elements()[group_idx] {
                Element::Group(g) => g,
                _ => return Err(invalid_params(format!("元素 '{}' 不是组", params.group_id))),
            };
            let found = g.children.iter().position(|c| c.id() == params.element_id)
                .ok_or_else(|| invalid_params(format!("子元素 '{}' 不在组 '{}' 中", params.element_id, params.group_id)))?;
            (g.common.x, g.common.y, g.children.len(), found)
        };

        let mut child_elem = {
            let g = match &mut project.active_elements_mut()[group_idx] {
                Element::Group(g) => g,
                _ => unreachable!(),
            };
            g.children.remove(found)
        };

        let c = child_elem.common_mut();
        c.x += gx;
        c.y += gy;

        if child_count == 1 {
            project.active_elements_mut().remove(group_idx);
            project.active_elements_mut().insert(group_idx, child_elem);
        } else {
            project.active_elements_mut().insert(group_idx + 1, child_elem);
            if let Element::Group(g) = &mut project.active_elements_mut()[group_idx] {
                let (bx, by, bw, bh) = calc_group_bounds(&g.children);
                let dx = bx - gx;
                let dy = by - gy;
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
            "已将元素 '{}' 从组 '{}' 中移除", params.element_id, params.group_id
        ))]))
    }
}
