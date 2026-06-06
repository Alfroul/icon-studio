use crate::fonts::{self, FontInfo};
use crate::FontDbState;
use tauri::State;

#[tauri::command]
pub fn list_fonts(font_db: State<'_, FontDbState>, keyword: Option<String>) -> Vec<FontInfo> {
    match keyword {
        Some(kw) if !kw.is_empty() => fonts::search_fonts_with(&font_db, &kw),
        _ => fonts::list_system_fonts_with(&font_db),
    }
}
