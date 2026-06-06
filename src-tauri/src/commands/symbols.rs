use crate::commands::canvas::ProjectState;
use crate::model::helpers::*;
use crate::model::symbol::{SymbolDef, SymbolInstanceElement, SymbolOverride, detach_symbol};
use crate::model::{CommonProps, Element};
use tauri::State;

use super::history::HistoryState;

#[tauri::command]
pub fn create_symbol(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
    name: String,
) -> Result<String, String> {
    let mut project = state.lock().map_err(|e| e.to_string())?;
    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;

    let idx = find_element_index(project.active_elements(), &element_id)
        .ok_or_else(|| format!("Element '{}' not found", element_id))?;

    let elem_snapshot = project.active_elements()[idx].clone();
    let elem_common = elem_snapshot.common().clone();
    let instance_id = elem_snapshot.id().to_string();
    let symbol_id = project.alloc_element_id("symbol");

    let def = SymbolDef {
        id: symbol_id.clone(),
        name: name.clone(),
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
        let mut history = history_state.lock().map_err(|e| e.to_string())?;
        history.record(Box::new(crate::model::history::SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
    }
    drop(project);

    Ok(symbol_id)
}

#[tauri::command]
pub fn list_symbols(
    state: State<'_, ProjectState>,
) -> Result<Vec<serde_json::Value>, String> {
    let project = state.lock().map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for (id, def) in &project.symbols {
        let instance_count = count_symbol_instances(project.active_elements(), id);
        result.push(serde_json::json!({
            "id": id,
            "name": def.name,
            "instance_count": instance_count,
            "source_type": type_name(&def.source_element),
        }));
    }
    Ok(result)
}

#[tauri::command]
pub fn update_symbol(
    state: State<'_, ProjectState>,
    symbol_id: String,
    element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| e.to_string())?;
    let elem_snapshot = find_element_deep(project.active_elements(), &element_id)
        .ok_or_else(|| format!("Element '{}' not found", element_id))?
        .clone();
    let def = project.symbols.get_mut(&symbol_id)
        .ok_or_else(|| format!("Symbol '{}' not found", symbol_id))?;
    def.source_element = elem_snapshot;
    project.bump_version();
    Ok(())
}

#[tauri::command]
pub fn detach_symbol_cmd(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    element_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| e.to_string())?;
    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;

    let idx = find_element_index(project.active_elements(), &element_id)
        .ok_or_else(|| format!("Element '{}' not found", element_id))?;

    let instance = match &project.active_elements()[idx] {
        Element::Symbol(inst) => inst.clone(),
        _ => return Err(format!("Element '{}' is not a symbol instance", element_id)),
    };

    let independent = detach_symbol(&instance, &project.symbols)
        .ok_or_else(|| format!("Symbol definition '{}' not found", instance.symbol_id))?;

    project.active_elements_mut()[idx] = independent;
    project.bump_version();

    let after = project.active_elements().to_vec();
    {
        let mut history = history_state.lock().map_err(|e| e.to_string())?;
        history.record(Box::new(crate::model::history::SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
    }
    Ok(())
}

#[tauri::command]
pub fn add_symbol_override(
    state: State<'_, ProjectState>,
    element_id: String,
    property: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| e.to_string())?;
    let elem = find_element_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| format!("Element '{}' not found", element_id))?;
    match elem {
        Element::Symbol(inst) => {
            inst.overrides.retain(|o| o.property != property);
            inst.overrides.push(SymbolOverride {
                property: property.clone(),
                value,
            });
        }
        _ => return Err(format!("Element '{}' is not a symbol instance", element_id)),
    }
    project.bump_version();
    Ok(())
}

#[tauri::command]
pub fn remove_symbol_override(
    state: State<'_, ProjectState>,
    element_id: String,
    property: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| e.to_string())?;
    let elem = find_element_mut(project.active_elements_mut(), &element_id)
        .ok_or_else(|| format!("Element '{}' not found", element_id))?;
    match elem {
        Element::Symbol(inst) => {
            inst.overrides.retain(|o| o.property != property);
        }
        _ => return Err(format!("Element '{}' is not a symbol instance", element_id)),
    }
    project.bump_version();
    Ok(())
}

fn count_symbol_instances(elements: &[Element], symbol_id: &str) -> usize {
    elements.iter().filter(|e| {
        matches!(e, Element::Symbol(inst) if inst.symbol_id == symbol_id)
    }).count()
}

fn type_name(elem: &Element) -> &'static str {
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
