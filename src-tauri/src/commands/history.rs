use crate::commands::canvas::ProjectState;
use crate::model::history::CommandHistory;
use std::sync::{Arc, Mutex};
use tauri::State;

pub type HistoryState = Arc<Mutex<CommandHistory>>;

#[tauri::command]
pub fn undo(
    project_state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
) -> Result<bool, String> {
    let mut project = project_state.lock().map_err(|e| e.to_string())?;
    let mut history = history_state.lock().map_err(|e| e.to_string())?;
    history.undo(&mut project)
}

#[tauri::command]
pub fn redo(
    project_state: State<'_, ProjectState>,
    history_state: State<'_, HistoryState>,
) -> Result<bool, String> {
    let mut project = project_state.lock().map_err(|e| e.to_string())?;
    let mut history = history_state.lock().map_err(|e| e.to_string())?;
    history.redo(&mut project)
}

#[tauri::command]
pub fn can_undo(history_state: State<'_, HistoryState>) -> Result<bool, String> {
    let history = history_state.lock().map_err(|e| e.to_string())?;
    Ok(history.can_undo())
}

#[tauri::command]
pub fn can_redo(history_state: State<'_, HistoryState>) -> Result<bool, String> {
    let history = history_state.lock().map_err(|e| e.to_string())?;
    Ok(history.can_redo())
}

#[tauri::command]
pub fn clear_history(history_state: State<'_, HistoryState>) -> Result<(), String> {
    let mut history = history_state.lock().map_err(|e| e.to_string())?;
    history.clear();
    Ok(())
}
