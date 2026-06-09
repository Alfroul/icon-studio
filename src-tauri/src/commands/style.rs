use crate::colors::{self, PaletteScheme};
use crate::engine::soft3d::apply_style_preset as engine_apply_style;
use crate::error::AppError;
use crate::model::helpers::*;
use crate::model::history::{SetGradientCommand, SetShadowCommand, SetFilterCommand, SnapshotCommand};
use crate::model::style_preset::{StyleParams, StylePreset, StyleType};
use crate::model::{Element, Gradient, GradientKind, Shadow};
use crate::model::filter::{FilterType, SvgFilter};
use std::collections::HashMap;
use tauri::State;

use super::canvas::ProjectState;
use super::history::HistoryState;

#[tauri::command]
pub fn set_gradient(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
    gradient_type: String,
    colors: Vec<String>,
    angle: f64,
) -> Result<(), String> {
    let kind = match gradient_type.as_str() {
        "linear" => GradientKind::Linear,
        "radial" => GradientKind::Radial,
        other => return Err(AppError::ValidationError(format!("Invalid gradient type: {other}")).into()),
    };
    let gradient = Gradient {
        gradient_type: kind,
        colors,
        angle,
        stops: Vec::new(),
    };

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element not found: {element_id}")))?;

    // Type guard: Image, Path and Group don't support gradients
    match elem {
        Element::Image(_) | Element::Path(_) | Element::Group(_) => {
            return Err(AppError::ValidationError("Image, Path and Group elements do not support gradients".into()).into());
        }
        _ => {}
    }

    let old_gradient = get_element_gradient(elem).cloned();
    set_element_gradient(elem, gradient.clone());
    project.bump_version();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SetGradientCommand::new(
        element_id,
        old_gradient,
        Some(gradient),
    )));
    Ok(())
}

#[tauri::command]
pub fn clear_gradient(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element not found: {element_id}")))?;
    let old_gradient = get_element_gradient(elem).cloned();
    clear_element_gradient(elem);
    project.bump_version();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SetGradientCommand::new(
        element_id,
        old_gradient,
        None,
    )));
    Ok(())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn set_shadow(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
    color: String,
    blur: f64,
    offset_x: f64,
    offset_y: f64,
    inset: Option<bool>,
) -> Result<(), String> {
    let shadow = Shadow {
        color,
        blur,
        offset_x,
        offset_y,
        inset: inset.unwrap_or(false),
    };

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element not found: {element_id}")))?;
    let old_shadow = get_element_shadow(elem).cloned();
    set_element_shadow(elem, shadow.clone());
    project.bump_version();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SetShadowCommand::new(
        element_id,
        old_shadow,
        Some(shadow),
    )));
    Ok(())
}

#[tauri::command]
pub fn clear_shadow(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element not found: {element_id}")))?;
    let old_shadow = get_element_shadow(elem).cloned();
    clear_element_shadow(elem);
    project.bump_version();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SetShadowCommand::new(
        element_id,
        old_shadow,
        None,
    )));
    Ok(())
}

#[tauri::command]
pub fn set_blend_mode(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
    mode: Option<String>,
) -> Result<(), String> {
    let valid_modes = [
        "normal", "multiply", "screen", "overlay", "darken", "lighten",
        "color-dodge", "color-burn", "hard-light", "soft-light", "difference", "exclusion",
    ];
    let blend_mode = match mode {
        Some(m) if !m.is_empty() => {
            if !valid_modes.contains(&m.as_str()) {
                return Err(AppError::ValidationError(format!("Invalid blend mode: {m}")).into());
            }
            Some(m)
        }
        _ => None,
    };

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let elem = crate::model::helpers::find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element not found: {element_id}")))?;
    let old_blend_mode = elem.common().blend_mode.clone();
    elem.common_mut().blend_mode = blend_mode.clone();
    project.bump_version();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(crate::model::history::SetPropsCommand::new(
        element_id,
        serde_json::json!({ "blend_mode": old_blend_mode }),
        serde_json::json!({ "blend_mode": blend_mode }),
    )));
    Ok(())
}

#[tauri::command]
pub fn suggest_palette(
    base_color: String,
    scheme: String,
    count: usize,
) -> Result<Vec<String>, String> {
    let palette_scheme = match scheme.as_str() {
        "complementary" => PaletteScheme::Complementary,
        "analogous" => PaletteScheme::Analogous,
        "triadic" => PaletteScheme::Triadic,
        "split-complementary" => PaletteScheme::SplitComplementary,
        "monochromatic" => PaletteScheme::Monochromatic,
        other => return Err(AppError::ValidationError(format!("Invalid palette scheme: {other}")).into()),
    };
    colors::suggest_palette(&base_color, palette_scheme, count).map_err(|e| AppError::ValidationError(e).into())
}

#[tauri::command]
pub fn set_filter(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
    filter_type: String,
    params: HashMap<String, f64>,
) -> Result<(), String> {
    let ft = match filter_type.as_str() {
        "noise" => FilterType::Noise,
        "blur" => FilterType::Blur,
        "pixelate" => FilterType::Pixelate,
        "emboss" => FilterType::Emboss,
        "posterize" => FilterType::Posterize,
        "turbulence" => FilterType::Turbulence,
        other => return Err(AppError::ValidationError(format!("Invalid filter type: {other}")).into()),
    };

    let filter = SvgFilter {
        filter_type: ft,
        params,
    };

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element not found: {element_id}")))?;
    let old_filter = get_element_filter(elem).cloned();
    set_element_filter(elem, filter.clone());
    project.bump_version();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SetFilterCommand::new(
        element_id,
        old_filter,
        Some(filter),
    )));
    Ok(())
}

#[tauri::command]
pub fn clear_filter(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let elem = find_element_deep_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Element not found: {element_id}")))?;
    let old_filter = get_element_filter(elem).cloned();
    clear_element_filter(elem);
    project.bump_version();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SetFilterCommand::new(
        element_id,
        old_filter,
        None,
    )));
    Ok(())
}

#[tauri::command]
pub fn apply_style_preset(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
    style_type: String,
    params: Option<StyleParams>,
) -> Result<(), String> {
    let st = match style_type.as_str() {
        "soft-3d" => StyleType::Soft3d,
        "neumorphism" => StyleType::Neumorphism,
        "glassmorphism" => StyleType::Glassmorphism,
        "flat" => StyleType::Flat,
        other => return Err(AppError::ValidationError(format!("Invalid style type: {other}")).into()),
    };
    let preset = StylePreset {
        style_type: st,
        params: params.unwrap_or_default(),
    };

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;

    engine_apply_style(&mut project, &element_id, &preset)
        .map_err(AppError::ValidationError)?;

    project.bump_version();
    let after = project.active_elements().to_vec();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SnapshotCommand::new(
        before,
        before_next_id,
        after,
        project.next_element_id,
    )));
    Ok(())
}
