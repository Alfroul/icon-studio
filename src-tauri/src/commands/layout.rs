use crate::error::AppError;
pub use crate::model::helpers::{element_id, get_element_bounds, set_element_position};
use crate::model::history::SnapshotCommand;
use serde::{Deserialize, Serialize};
use tauri::State;

use super::canvas::ProjectState;
use super::history::HistoryState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub x: f64,
    pub y: f64,
}

#[tauri::command]
pub fn set_layout(
    state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
    layout_type: String,
    gap: f64,
    padding: f64,
) -> Result<Vec<Position>, String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;

    let old_positions: Vec<Position> = project.active_elements().iter().map(|e| {
        let (x, y, _, _) = get_element_bounds(e);
        Position { id: element_id(e).to_string(), x, y }
    }).collect();

    if project.active_elements().is_empty() {
        return Ok(old_positions);
    }

    let before = project.active_elements().to_vec();
    let before_next_id = project.next_element_id;
    let cw = project.active_canvas().width;
    let ch = project.active_canvas().height;

    crate::services::layout::apply_layout(
        project.active_elements_mut(),
        cw,
        ch,
        &layout_type,
        gap,
        padding,
    )?;

    project.bump_version();
    let mut history = history_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    history.record(Box::new(SnapshotCommand::new(before, before_next_id, project.active_elements().to_vec(), project.next_element_id)));

    Ok(old_positions)
}
