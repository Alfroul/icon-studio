use tauri::{AppHandle, Emitter};

pub fn emit_project_changed(handle: &AppHandle) {
    if let Err(e) = handle.emit("project-changed", ()) {
        eprintln!("Warning: failed to emit project-changed event: {}", e);
    }
}
