use crate::engine::theme_presets;
use crate::error::AppError;
use crate::model::group::GroupElement;
use crate::model::helpers::*;
use crate::model::history::{AddElementCommand, CanvasCommand, RemoveElementCommand, ReorderCommand, SetPropsCommand, SnapshotCommand};
use crate::model::{CommonProps, Element, IconElement, ImageElement, PathElement, ShapeElement, TextElement};

use tauri::State;

/// Recursively reassign IDs for an element and all its children (for groups).
fn reassign_ids(elem: &mut Element, project: &mut crate::model::IconProject) {
    let new_id = project.alloc_element_id(match elem {
        Element::Shape(_) => "shape",
        Element::Text(_) => "text",
        Element::Icon(_) => "icon",
        Element::Image(_) => "image",
        Element::Path(_) => "path",
        Element::Group(_) => "group",
        Element::Symbol(_) => "symbol",
    });
    elem.common_mut().id = new_id;

    if let Element::Group(g) = elem {
        for child in &mut g.children {
            reassign_ids(child, project);
        }
    }
}

use super::canvas::ProjectState;
use super::export::RenderCacheState;
use super::history::HistoryState;

#[tauri::command]
pub fn add_shape(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    shape_type: String,
    fill: String,
    size: f64,
    x: f64,
    y: f64,
) -> Result<ShapeElement, String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let id = project.alloc_element_id("shape");
    let st = parse_shape_type(&shape_type)?;
    if size.is_nan() || size <= 0.0 {
        return Err(AppError::ValidationError("size must be a positive number".into()).into());
    }
    let element = ShapeElement {
        common: CommonProps {
            id,
            x,
            y,
            width: size,
            height: size,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
        shape_type: st,
        fill,
        stroke: None,
        stroke_width: 0.0,
        border_radius: 0.0,
        stroke_dasharray: None,
        gradient: None,
    };
    let result = element.clone();
    let cmd = AddElementCommand::new(Element::Shape(element));
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.push_and_execute(Box::new(cmd), &mut project)?;
    Ok(result)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn add_text(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    content: String,
    font_family: String,
    font_size: f64,
    fill: String,
    x: f64,
    y: f64,
) -> Result<TextElement, String> {
    if content.trim().is_empty() {
        return Err(AppError::ValidationError("content must not be empty".into()).into());
    }
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let id = project.alloc_element_id("text");
    if font_size.is_nan() || font_size <= 0.0 {
        return Err(AppError::ValidationError("font_size must be a positive number".into()).into());
    }
    let width = crate::engine::text_measure::measure_text_width(&content, &font_family, font_size as f32, 400) as f64;
    let height = font_size * 1.2;
    let element = TextElement {
        common: CommonProps {
            id,
            x,
            y,
            width,
            height,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
        content,
        fill,
        font_family,
        font_size,
        font_weight: "normal".to_string(),
        letter_spacing: 0.0,
        stroke: None,
        stroke_width: 0.0,
        gradient: None,
    };
    let result = element.clone();
    let cmd = AddElementCommand::new(Element::Text(element));
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.push_and_execute(Box::new(cmd), &mut project)?;
    Ok(result)
}

#[tauri::command]
pub fn add_icon(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    icon_name: String,
    fill: String,
    size: f64,
    x: f64,
    y: f64,
) -> Result<IconElement, String> {
    if icon_name.trim().is_empty() {
        return Err(AppError::ValidationError("icon_name must not be empty".into()).into());
    }
    if crate::icons::get_icon_path(&icon_name).is_none() {
        return Err(AppError::NotFoundError(format!("Icon '{}' not found in Lucide library", icon_name)).into());
    }
    if size.is_nan() || size <= 0.0 {
        return Err(AppError::ValidationError("size must be a positive number".into()).into());
    }
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let id = project.alloc_element_id("icon");
    let element = IconElement {
        common: CommonProps {
            id,
            x,
            y,
            width: size,
            height: size,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
        name: icon_name,
        fill,
        stroke: None,
        stroke_width: 0.0,
        gradient: None,
    };
    let result = element.clone();
    let cmd = AddElementCommand::new(Element::Icon(element));
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.push_and_execute(Box::new(cmd), &mut project)?;
    Ok(result)
}

#[tauri::command]
pub fn add_image(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    file_path: String,
    width: f64,
    height: f64,
    x: f64,
    y: f64,
) -> Result<ImageElement, String> {
    let (data, _mime) = crate::services::elements::detect_mime_and_encode(&file_path)?;

    if width.is_nan() || height.is_nan() || width <= 0.0 || height <= 0.0 {
        return Err(AppError::ValidationError("width and height must be positive numbers".into()).into());
    }

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let id = project.alloc_element_id("image");
    let element = ImageElement {
        common: CommonProps {
            id,
            x,
            y,
            width,
            height,
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
    let result = element.clone();
    let cmd = AddElementCommand::new(Element::Image(element));
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.push_and_execute(Box::new(cmd), &mut project)?;
    Ok(result)
}

#[tauri::command]
pub fn add_path(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    d: String,
    stroke: String,
    stroke_width: f64,
    fill: Option<String>,
) -> Result<PathElement, String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let id = project.alloc_element_id("path");
    let mut element = PathElement {
        common: CommonProps {
            id,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
        d,
        fill: fill.unwrap_or_else(|| "none".to_string()),
        stroke,
        stroke_width,
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
    let result = element.clone();
    let cmd = AddElementCommand::new(Element::Path(element));
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.push_and_execute(Box::new(cmd), &mut project)?;
    Ok(result)
}

#[tauri::command]
pub fn set_props(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
    props: serde_json::Value,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element '{}' not found", element_id)))?;

    let old_props = serde_json::to_value(&*elem).map_err(AppError::from)?;
    let mut merged = old_props.clone();
    if let (serde_json::Value::Object(ref mut cur), serde_json::Value::Object(ref incoming)) =
        (&mut merged, &props)
    {
        for (key, value) in incoming {
            if key == "type" || key == "id" {
                continue;
            }
            cur.insert(key.clone(), value.clone());
        }
    }
    let new_props = merged;

    let will_recalc_text_dims = props.as_object().is_some_and(|obj| {
        obj.keys().any(|k| matches!(k.as_str(), "content" | "font_family" | "font_size" | "font_weight" | "letter_spacing"))
    });

    if let Some(obj) = new_props.as_object() {
        let skip_dim_validation = will_recalc_text_dims;
        if let Some(width) = obj.get("width").and_then(|v| v.as_f64()) {
            if !skip_dim_validation && (width.is_nan() || width <= 0.0) {
                return Err(AppError::ValidationError("width must be a positive number".into()).into());
            }
        }
        if let Some(height) = obj.get("height").and_then(|v| v.as_f64()) {
            if !skip_dim_validation && (height.is_nan() || height <= 0.0) {
                return Err(AppError::ValidationError("height must be a positive number".into()).into());
            }
        }
        if let Some(opacity) = obj.get("opacity").and_then(|v| v.as_f64()) {
            if opacity.is_nan() || !(0.0..=1.0).contains(&opacity) {
                return Err(AppError::ValidationError("opacity must be between 0.0 and 1.0".into()).into());
            }
        }
        if let Some(font_size) = obj.get("font_size").and_then(|v| v.as_f64()) {
            if font_size.is_nan() || font_size <= 0.0 {
                return Err(AppError::ValidationError("font_size must be a positive number".into()).into());
            }
        }
        if let Some(rotation) = obj.get("rotation").and_then(|v| v.as_f64()) {
            if !rotation.is_finite() {
                return Err(AppError::ValidationError("rotation must be a finite number".into()).into());
            }
        }
        if let Some(x) = obj.get("x").and_then(|v| v.as_f64()) {
            if !x.is_finite() {
                return Err(AppError::ValidationError("x must be a finite number".into()).into());
            }
        }
        if let Some(y) = obj.get("y").and_then(|v| v.as_f64()) {
            if !y.is_finite() {
                return Err(AppError::ValidationError("y must be a finite number".into()).into());
            }
        }
        if let Some(stroke_width) = obj.get("stroke_width").and_then(|v| v.as_f64()) {
            if stroke_width.is_nan() || stroke_width < 0.0 {
                return Err(AppError::ValidationError("stroke_width must be non-negative".into()).into());
            }
        }
    }

    let cmd = SetPropsCommand::new(element_id.clone(), old_props, new_props);
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.push_and_execute(Box::new(cmd), &mut project)?;

    Ok(())
}

#[tauri::command]
pub fn remove_element(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
) -> Result<bool, String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    // First, verify the element exists (deep search) before any mutations
    let found = find_element_deep_mut(project.active_elements_mut(), &element_id);
    if found.is_none() {
        return Err(AppError::NotFoundError(format!("Element '{}' not found", element_id)).into());
    }

    // Now safe to clean up references
    crate::services::elements::cleanup_clip_mask_refs(project.active_elements_mut(), &element_id);

    // Remove element — try top-level first, then deep into groups
    let elements = project.active_elements_mut();
    if let Some(idx) = find_element_index(elements, &element_id) {
        let element = elements[idx].clone();
        let cmd = RemoveElementCommand::new(element, idx);
        let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        history.push_and_execute(Box::new(cmd), &mut project)?;
    } else {
        // Element is inside a group — remove recursively
        fn remove_from_group(elements: &mut [Element], target_id: &str) -> bool {
            for elem in elements.iter_mut() {
                if let Element::Group(g) = elem {
                    if let Some(idx) = g.children.iter().position(|c| c.id() == target_id) {
                        g.children.remove(idx);
                        if !g.children.is_empty() {
                            let (bx, by, bw, bh) = calc_group_bounds(&g.children);
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
        remove_from_group(project.active_elements_mut(), &element_id);
        project.bump_version();
    }

    Ok(true)
}

#[tauri::command]
pub fn list_elements(state: State<'_, ProjectState>) -> Result<Vec<serde_json::Value>, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    project
        .active_elements()
        .iter()
        .map(|e| serde_json::to_value(e).map_err(|err| AppError::SerdeError(err).to_string()))
        .collect()
}

#[tauri::command]
pub fn reorder_elements(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
    new_index: usize,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let old_index = find_element_index(project.active_elements(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element '{}' not found", element_id)))?;

    if new_index >= project.active_elements().len() {
        return Err(AppError::ValidationError(format!(
            "Index {} out of bounds ({} elements)",
            new_index,
            project.active_elements().len()
        )).into());
    }

    let cmd = ReorderCommand::new(element_id, old_index, new_index);
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.push_and_execute(Box::new(cmd), &mut project)?;
    Ok(())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn update_canvas(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    width: Option<u32>,
    height: Option<u32>,
    background: Option<String>,
    corner_radius: Option<u32>,
    background_gradient: Option<crate::model::Gradient>,
    clear_background_gradient: Option<bool>,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let old_canvas = project.active_canvas().clone();
    if let Some(w) = width {
        if w == 0 || w > 8192 {
            return Err(AppError::ValidationError("Width must be 1-8192".into()).into());
        }
        project.active_canvas_mut().width = w;
    }
    if let Some(h) = height {
        if h == 0 || h > 8192 {
            return Err(AppError::ValidationError("Height must be 1-8192".into()).into());
        }
        project.active_canvas_mut().height = h;
    }
    if let Some(bg) = background {
        project.active_canvas_mut().background = bg;
    }
    if let Some(cr) = corner_radius {
        if cr > 50 {
            return Err(AppError::ValidationError("Corner radius must be 0-50%".into()).into());
        }
        project.active_canvas_mut().corner_radius = cr;
    }
    if let Some(clear) = clear_background_gradient {
        if clear {
            project.active_canvas_mut().background_gradient = None;
        }
    }
    if let Some(grad) = background_gradient {
        project.active_canvas_mut().background_gradient = Some(grad);
    }
    project.bump_version();
    let new_canvas = project.active_canvas().clone();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(CanvasCommand::new(old_canvas, new_canvas)));
    Ok(())
}

#[tauri::command]
pub fn list_icons(keyword: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    let icons = match keyword {
        Some(kw) => crate::icons::search_icons(&kw),
        None => crate::icons::list_all_icons(),
    };
    let result = icons
        .into_iter()
        .map(|info| {
            let path = crate::icons::get_icon_path(&info.name).unwrap_or("");
            serde_json::json!({
                "name": info.name,
                "tags": info.tags,
                "path": path,
            })
        })
        .collect();
    Ok(result)
}

#[tauri::command]
pub fn group_elements(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_ids: Vec<String>,
) -> Result<(), String> {
    if element_ids.len() < 2 {
        return Err(AppError::ValidationError("至少需要 2 个元素才能编组".into()).into());
    }

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;

    let mut children = Vec::new();
    let mut first_index = None;
    for eid in &element_ids {
        let idx = find_element_index(project.active_elements(), eid)
            .ok_or_else(|| AppError::NotFoundError(format!("元素 '{}' 未找到", eid)))?;
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
            id: group_id,
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
            overlay: None,
        },
        children,
        expanded: true,
    };

    let insert_at = first_index.unwrap_or(0);
    let ids_set: Vec<String> = element_ids;
    let elements = project.active_elements_mut();
    elements.retain(|e| !ids_set.contains(&e.id().to_string()));
    let insert_at = insert_at.min(elements.len());
    elements.insert(insert_at, Element::Group(group));
    project.bump_version();

    let after = project.active_elements().to_vec();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
    Ok(())
}

#[tauri::command]
pub fn ungroup(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    group_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;

    let idx = find_element_index(project.active_elements(), &group_id)
        .ok_or_else(|| AppError::NotFoundError(format!("组 '{}' 未找到", group_id)))?;

    let group = match &project.active_elements()[idx] {
        Element::Group(g) => g.clone(),
        _ => return Err(AppError::ValidationError(format!("元素 '{}' 不是组", group_id)).into()),
    };

    let mut released = Vec::new();
    for mut child in group.children {
        let c = child.common_mut();
        c.x += group.common.x;
        c.y += group.common.y;
        released.push(child);
    }

    let elements = project.active_elements_mut();
    elements.remove(idx);
    for (i, elem) in released.into_iter().enumerate() {
        elements.insert(idx + i, elem);
    }
    project.bump_version();

    let after = project.active_elements().to_vec();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
    Ok(())
}

#[tauri::command]
pub fn add_to_group(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    group_id: String,
    element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;

    let group_idx = find_element_index(project.active_elements(), &group_id)
        .ok_or_else(|| AppError::NotFoundError(format!("组 '{}' 未找到", group_id)))?;

    let elem_idx = find_element_index(project.active_elements(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("元素 '{}' 未找到", element_id)))?;

    if group_idx == elem_idx {
        return Err(AppError::ValidationError("不能将组添加到自身".into()).into());
    }

    let mut elem = project.active_elements()[elem_idx].clone();
    let (gx, gy) = {
        let g = match &project.active_elements()[group_idx] {
            Element::Group(g) => g,
            _ => return Err(AppError::ValidationError(format!("元素 '{}' 不是组", group_id)).into()),
        };
        (g.common.x, g.common.y)
    };

    let c = elem.common_mut();
    c.x -= gx;
    c.y -= gy;

    let elements = project.active_elements_mut();
    elements.remove(elem_idx);
    let group_idx = if elem_idx < group_idx { group_idx - 1 } else { group_idx };
    if let Element::Group(g) = &mut elements[group_idx] {
        g.children.push(elem);
        let (bx, by, bw, bh) = calc_group_bounds(&g.children);
        g.common.x = bx;
        g.common.y = by;
        g.common.width = bw;
        g.common.height = bh;
    }
    project.bump_version();

    let after = project.active_elements().to_vec();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
    Ok(())
}

#[tauri::command]
pub fn remove_from_group(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    group_id: String,
    element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;

    let group_idx = find_element_index(project.active_elements(), &group_id)
        .ok_or_else(|| AppError::NotFoundError(format!("组 '{}' 未找到", group_id)))?;

    let (gx, gy, child_count, found) = {
        let g = match &project.active_elements()[group_idx] {
            Element::Group(g) => g,
            _ => return Err(AppError::ValidationError(format!("元素 '{}' 不是组", group_id)).into()),
        };
        let found = g.children.iter().position(|c| c.id() == element_id)
            .ok_or_else(|| AppError::NotFoundError(format!("子元素 '{}' 不在组 '{}' 中", element_id, group_id)))?;
        (g.common.x, g.common.y, g.children.len(), found)
    };

    let mut child_elem = {
        let elements = project.active_elements_mut();
        let g = match &mut elements[group_idx] {
            Element::Group(g) => g,
            _ => unreachable!(),
        };
        g.children.remove(found)
    };

    let c = child_elem.common_mut();
    c.x += gx;
    c.y += gy;

    let elements = project.active_elements_mut();
    if child_count == 1 {
        elements.remove(group_idx);
        elements.insert(group_idx, child_elem);
    } else {
        elements.insert(group_idx + 1, child_elem);
        if let Element::Group(g) = &mut elements[group_idx] {
            let (bx, by, bw, bh) = calc_group_bounds(&g.children);
            g.common.x = bx;
            g.common.y = by;
            g.common.width = bw;
            g.common.height = bh;
        }
    }
    project.bump_version();

    let after = project.active_elements().to_vec();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
    Ok(())
}

#[tauri::command]
pub fn duplicate_element(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
) -> Result<String, String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let idx = find_element_index(project.active_elements(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element '{}' not found", element_id)))?;

    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;

    let mut clone = project.active_elements()[idx].clone();
    let (x, y, ..) = get_element_bounds(&clone);
    set_element_position(&mut clone, x + 20.0, y + 20.0);

    reassign_ids(&mut clone, &mut project);

    let new_id = clone.id().to_string();
    project.active_elements_mut().insert(idx + 1, clone);
    project.bump_version();

    let after = project.active_elements().to_vec();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));

    Ok(new_id)
}

struct SvgBounds {
    min_x: f64,
    min_y: f64,
    size: f64,
}

fn calculate_svg_bounds(elements: &[Element]) -> SvgBounds {
    if elements.is_empty() {
        return SvgBounds { min_x: 0.0, min_y: 0.0, size: 24.0 };
    }
    let min_x = elements.iter().map(|e| e.common().x).fold(f64::MAX, f64::min);
    let min_y = elements.iter().map(|e| e.common().y).fold(f64::MAX, f64::min);
    let max_x = elements
        .iter()
        .map(|e| e.common().x + e.common().width)
        .fold(f64::MIN, f64::max);
    let max_y = elements
        .iter()
        .map(|e| e.common().y + e.common().height)
        .fold(f64::MIN, f64::max);
    SvgBounds {
        min_x,
        min_y,
        size: (max_x - min_x).max(max_y - min_y).max(1.0),
    }
}

#[tauri::command]
pub fn import_svg_elements(
    project: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    cache_state: State<'_, RenderCacheState>,
    path: String,
    target_x: Option<f64>,
    target_y: Option<f64>,
    target_size: Option<f64>,
) -> Result<Vec<String>, String> {
    let svg_str = std::fs::read_to_string(&path)
        .map_err(|e| AppError::RenderError(format!("读取文件失败：{}", e)))?;

    let mut elements = crate::engine::importer::import_svg_as_elements(&svg_str)
        .map_err(|e| AppError::RenderError(format!("解析 SVG 失败：{}", e)))?;

    if elements.is_empty() {
        return Ok(Vec::new());
    }

    let mut project = project.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let canvas_size = project.active_canvas().width.min(project.active_canvas().height) as f64;
    let bounds = calculate_svg_bounds(&elements);
    let target_sz = target_size.unwrap_or(canvas_size * 0.6);
    let scale = target_sz / bounds.size;
    let tx = target_x.unwrap_or((canvas_size - target_sz) / 2.0);
    let ty = target_y.unwrap_or((canvas_size - target_sz) / 2.0);

    let mut ids = Vec::new();
    for elem in &mut elements {
        let prefix = match elem {
            Element::Shape(_) => "shape",
            Element::Text(_) => "text",
            Element::Icon(_) => "icon",
            Element::Image(_) => "image",
            Element::Path(_) => "path",
            Element::Group(_) => "group",
            Element::Symbol(_) => "symbol",
        };

        let new_id = project.alloc_element_id(prefix);
        elem.common_mut().id = new_id.clone();
        ids.push(new_id);

        let c = elem.common_mut();
        c.x = (c.x - bounds.min_x) * scale + tx;
        c.y = (c.y - bounds.min_y) * scale + ty;
        c.width *= scale;
        c.height *= scale;
    }

    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.begin_batch("Import SVG elements").map_err(|e| e.to_string())?;
    for e in elements {
        history.push_and_execute(
            Box::new(AddElementCommand::new(e)),
            &mut project,
        ).map_err(|e| e.to_string())?;
    }
    history.commit_batch().map_err(|e| e.to_string())?;

    project.bump_version();
    drop(project);

    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();

    Ok(ids)
}

#[tauri::command]
pub fn list_library_categories() -> Result<Vec<String>, String> {
    Ok(crate::library::list_categories())
}

#[tauri::command]
pub fn list_library_assets(
    category: Option<String>,
    keyword: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let assets = crate::library::list_assets(
        category.as_deref(),
        keyword.as_deref(),
    );
    Ok(assets.iter().map(|a| {
        serde_json::json!({
            "name": a.name,
            "label": a.label,
            "category": a.category,
            "tags": a.tags,
            "svg": a.svg,
        })
    }).collect())
}

#[tauri::command]
pub fn add_library_asset(
    project_state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    cache_state: State<'_, RenderCacheState>,
    asset_name: String,
    target_x: Option<f64>,
    target_y: Option<f64>,
    target_size: Option<f64>,
) -> Result<Vec<String>, String> {
    let svg_str = crate::library::get_asset_svg(&asset_name)
        .ok_or_else(|| AppError::NotFoundError(format!("Library asset '{}' not found", asset_name)))?;

    let mut elements = crate::engine::importer::import_svg_as_elements(svg_str)
        .map_err(|e| AppError::RenderError(format!("解析素材 SVG 失败：{}", e)))?;

    if elements.is_empty() {
        return Ok(Vec::new());
    }

    let mut project = project_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let canvas_size = project.active_canvas().width.min(project.active_canvas().height) as f64;
    let bounds = calculate_svg_bounds(&elements);
    let target_sz = target_size.unwrap_or(canvas_size * 0.6);
    let scale = target_sz / bounds.size;
    let tx = target_x.unwrap_or((canvas_size - target_sz) / 2.0);
    let ty = target_y.unwrap_or((canvas_size - target_sz) / 2.0);

    let mut ids = Vec::new();
    for elem in &mut elements {
        let prefix = match elem {
            Element::Shape(_) => "shape",
            Element::Text(_) => "text",
            Element::Icon(_) => "icon",
            Element::Image(_) => "image",
            Element::Path(_) => "path",
            Element::Group(_) => "group",
            Element::Symbol(_) => "symbol",
        };

        let new_id = project.alloc_element_id(prefix);
        elem.common_mut().id = new_id.clone();
        ids.push(new_id);

        let c = elem.common_mut();
        c.x = (c.x - bounds.min_x) * scale + tx;
        c.y = (c.y - bounds.min_y) * scale + ty;
        c.width *= scale;
        c.height *= scale;
    }

    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.begin_batch(&format!("Add library asset '{}'", asset_name)).map_err(|e| e.to_string())?;
    for e in elements {
        history.push_and_execute(
            Box::new(AddElementCommand::new(e)),
            &mut project,
        ).map_err(|e| e.to_string())?;
    }
    history.commit_batch().map_err(|e| e.to_string())?;

    project.bump_version();
    drop(project);

    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();

    Ok(ids)
}

#[tauri::command]
pub fn set_clip(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
    clip_element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    crate::services::elements::validate_clip_mask_reference(
        project.active_elements(), &element_id, &clip_element_id,
    ).map_err(|e| e.to_string())?;

    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;

    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element '{}' not found", element_id)))?;
    elem.common_mut().clip_element_id = Some(clip_element_id);

    project.bump_version();
    let after = project.active_elements().to_vec();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
    Ok(())
}

#[tauri::command]
pub fn clear_clip(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;

    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element '{}' not found", element_id)))?;
    elem.common_mut().clip_element_id = None;

    project.bump_version();
    let after = project.active_elements().to_vec();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
    Ok(())
}

#[tauri::command]
pub fn set_mask(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
    mask_element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    crate::services::elements::validate_clip_mask_reference(
        project.active_elements(), &element_id, &mask_element_id,
    ).map_err(|e| e.to_string())?;

    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;

    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element '{}' not found", element_id)))?;
    elem.common_mut().mask_element_id = Some(mask_element_id);

    project.bump_version();
    let after = project.active_elements().to_vec();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
    Ok(())
}

#[tauri::command]
pub fn clear_mask(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;

    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element '{}' not found", element_id)))?;
    elem.common_mut().mask_element_id = None;

    project.bump_version();
    let after = project.active_elements().to_vec();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
    Ok(())
}

#[tauri::command]
pub fn boolean_operation(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    cache_state: State<'_, RenderCacheState>,
    element_a_id: String,
    element_b_id: String,
    operation: String,
) -> Result<String, String> {
    use crate::engine::boolean::{self, BooleanOp};

    let op = match operation.as_str() {
        "union" => BooleanOp::Union,
        "subtract" => BooleanOp::Subtract,
        "intersect" => BooleanOp::Intersect,
        "exclude" => BooleanOp::Exclude,
        other => return Err(format!("Invalid operation: {}. Use union, subtract, intersect, or exclude", other)),
    };

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    if element_a_id == element_b_id {
        return Err("Elements A and B must be different".into());
    }

    let idx_a = find_element_index(project.active_elements(), &element_a_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element '{}' not found", element_a_id)))?;
    let idx_b = find_element_index(project.active_elements(), &element_b_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element '{}' not found", element_b_id)))?;

    let elem_a = project.active_elements()[idx_a].clone();
    let elem_b = project.active_elements()[idx_b].clone();

    let (mut path_element, _boolean_source) = boolean::perform_boolean(&elem_a, &elem_b, op)
        .map_err(|e| e.to_string())?;

    let new_id = project.alloc_element_id("path");
    path_element.common.id = new_id.clone();

    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.push_and_execute(
        Box::new(AddElementCommand::new(Element::Path(path_element))),
        &mut project,
    ).map_err(|e| e.to_string())?;

    project.bump_version();
    drop(project);

    {
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        cache.invalidate_cache();
    }

    Ok(new_id)
}

#[tauri::command]
pub fn convert_to_path(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    cache_state: State<'_, RenderCacheState>,
    element_id: String,
) -> Result<String, String> {
    use crate::engine::boolean::shape_to_path_d;

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    let idx = find_element_index(project.active_elements(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element '{}' not found", element_id)))?;

    let elem = &project.active_elements()[idx];

    let (shape_type, x, y, w, h, border_radius) = match elem {
        Element::Shape(s) => (&s.shape_type, s.common.x, s.common.y, s.common.width, s.common.height, s.border_radius),
        _ => return Err(AppError::ValidationError("Element is not a shape".into()).into()),
    };

    let d = shape_to_path_d(shape_type, x, y, w, h, border_radius);

    let shape = match elem { Element::Shape(s) => s, _ => unreachable!() };
    let new_id = element_id.clone();

    let mut path_element = PathElement {
        common: CommonProps {
            id: new_id.clone(),
            x: 0.0,
            y: 0.0,
            width: w,
            height: h,
            opacity: shape.common.opacity,
            rotation: shape.common.rotation,
            shadows: shape.common.shadows.clone(),
            animation: shape.common.animation.clone(),
            blend_mode: shape.common.blend_mode.clone(),
            clip_element_id: shape.common.clip_element_id.clone(),
            mask_element_id: shape.common.mask_element_id.clone(),
            locked: shape.common.locked,
            visible: shape.common.visible,
            svg_filter: shape.common.svg_filter.clone(),
            overlay: shape.common.overlay.clone(),
        },
        d,
        fill: shape.fill.clone(),
        stroke: shape.stroke.clone().unwrap_or_else(|| "none".to_string()),
        stroke_width: shape.stroke_width,
        stroke_dasharray: shape.stroke_dasharray.clone(),
        natural_width: 0.0,
        natural_height: 0.0,
        boolean_source: None,
    };

    // Recompute natural dimensions
    {
        let mut wrapper = Element::Path(path_element.clone());
        crate::model::helpers::recompute_path_natural_dims(&mut wrapper);
        if let Element::Path(pe) = wrapper {
            path_element = pe;
        }
    }

    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;

    project.active_elements_mut()[idx] = Element::Path(path_element);
    project.bump_version();

    let after = project.active_elements().to_vec();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));

    drop(project);
    {
        let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        cache.invalidate_cache();
    }

    Ok(new_id)
}

#[tauri::command]
pub fn fit_element_to_canvas(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    let canvas_w = project.active_canvas().width as f64;
    let canvas_h = project.active_canvas().height as f64;

    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element '{}' not found", element_id)))?;

    let (ew, eh) = (elem.common().width, elem.common().height);
    let (x, y, w, h) = theme_presets::fit_to_canvas(canvas_w, canvas_h, ew, eh);

    let old_props = serde_json::to_value(&*elem).map_err(AppError::from)?;
    let mut new_props = old_props.clone();
    if let serde_json::Value::Object(ref mut map) = new_props {
        map.insert("x".into(), serde_json::json!(x));
        map.insert("y".into(), serde_json::json!(y));
        map.insert("width".into(), serde_json::json!(w));
        map.insert("height".into(), serde_json::json!(h));
    }

    let _ = elem;

    let cmd = SetPropsCommand::new(element_id, old_props, new_props);
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.push_and_execute(Box::new(cmd), &mut project)?;
    Ok(())
}

#[tauri::command]
pub fn add_image_from_data(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    image_data: String,
    width: f64,
    height: f64,
) -> Result<ImageElement, String> {
    if width.is_nan() || height.is_nan() || width <= 0.0 || height <= 0.0 {
        return Err(AppError::ValidationError("width and height must be positive numbers".into()).into());
    }

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    let canvas_w = project.active_canvas().width as f64;
    let canvas_h = project.active_canvas().height as f64;
    let (x, y, fit_w, fit_h) = theme_presets::fit_to_canvas(canvas_w, canvas_h, width, height);

    let id = project.alloc_element_id("image");
    let element = ImageElement {
        common: CommonProps {
            id,
            x,
            y,
            width: fit_w,
            height: fit_h,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
            animation: None,
            blend_mode: None,
            clip_element_id: None,
            mask_element_id: None,
            locked: false,
            visible: true,
            svg_filter: None,
            overlay: None,
        },
        data: image_data,
    };
    let result = element.clone();
    let cmd = AddElementCommand::new(Element::Image(element));
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.push_and_execute(Box::new(cmd), &mut project)?;
    Ok(result)
}

#[tauri::command]
pub fn add_path_from_svg(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    cache_state: State<'_, RenderCacheState>,
    svg_content: String,
) -> Result<Vec<String>, String> {
    let mut elements = crate::engine::importer::import_svg_as_elements(&svg_content)
        .map_err(|e| AppError::RenderError(format!("Failed to parse SVG: {}", e)))?;

    if elements.is_empty() {
        return Ok(Vec::new());
    }

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let canvas_w = project.active_canvas().width as f64;
    let canvas_h = project.active_canvas().height as f64;
    let bounds = calculate_svg_bounds(&elements);
    let (target_x, target_y, target_sz_w, target_sz_h) = theme_presets::fit_to_canvas(
        canvas_w, canvas_h, bounds.size, bounds.size,
    );

    let scale_x = target_sz_w / bounds.size.max(1.0);
    let scale_y = target_sz_h / bounds.size.max(1.0);

    let mut ids = Vec::new();
    for elem in &mut elements {
        let prefix = match elem {
            Element::Shape(_) => "shape",
            Element::Text(_) => "text",
            Element::Icon(_) => "icon",
            Element::Image(_) => "image",
            Element::Path(_) => "path",
            Element::Group(_) => "group",
            Element::Symbol(_) => "symbol",
        };

        let new_id = project.alloc_element_id(prefix);
        elem.common_mut().id = new_id.clone();
        ids.push(new_id);

        let c = elem.common_mut();
        c.x = (c.x - bounds.min_x) * scale_x + target_x;
        c.y = (c.y - bounds.min_y) * scale_y + target_y;
        c.width *= scale_x;
        c.height *= scale_y;
    }

    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.begin_batch("Import dropped SVG").map_err(|e| e.to_string())?;
    for e in elements {
        history.push_and_execute(
            Box::new(AddElementCommand::new(e)),
            &mut project,
        ).map_err(|e| e.to_string())?;
    }
    history.commit_batch().map_err(|e| e.to_string())?;

    project.bump_version();
    drop(project);

    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();

    Ok(ids)
}

// ---- Overlay Commands ----

#[tauri::command]
pub fn set_element_overlay(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    element_id: String,
    overlay: crate::model::Overlay,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| format!("Element '{}' not found", element_id))?;
    elem.common_mut().overlay = Some(overlay);

    project.bump_version();
    drop(project);

    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();
    Ok(())
}

#[tauri::command]
pub fn remove_element_overlay(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| format!("Element '{}' not found", element_id))?;
    elem.common_mut().overlay = None;

    project.bump_version();
    drop(project);

    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();
    Ok(())
}

#[tauri::command]
pub fn batch_apply_overlay(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    element_ids: Vec<String>,
    overlay: crate::model::Overlay,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    for eid in &element_ids {
        if let Some(elem) = find_element_deep_mut(project.active_elements_mut(), eid) {
            elem.common_mut().overlay = Some(overlay.clone());
        }
    }

    project.bump_version();
    drop(project);

    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();
    Ok(())
}

#[tauri::command]
pub fn batch_remove_overlay(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    element_ids: Vec<String>,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    for eid in &element_ids {
        if let Some(elem) = find_element_deep_mut(project.active_elements_mut(), eid) {
            elem.common_mut().overlay = None;
        }
    }

    project.bump_version();
    drop(project);

    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();
    Ok(())
}
