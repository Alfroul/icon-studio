pub mod cli;
pub mod engine;
pub mod error;
pub mod icons;
pub mod library;
pub mod model;
pub mod colors;
pub mod fonts;
pub mod services;

pub mod templates;
pub mod commands;
pub mod mcp;

use commands::canvas::ProjectState;
use commands::history::HistoryState;
use commands::export::RenderCacheState;

use std::sync::Arc;
use tauri::{Emitter, Manager};

pub type FontDbState = Arc<fontdb::Database>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(
    project: ProjectState,
    history: HistoryState,
    cache: RenderCacheState,
) {
    let mut font_db = fontdb::Database::new();
    font_db.load_system_fonts();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let project_state = app.state::<ProjectState>();
            let project = project_state.inner().clone();
            let ws_port: u16 = std::env::var("ICONSTUDIO_WS_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(9250);
            services::websocket::spawn(project, ws_port, Some(app.handle().clone()));
            Ok(())
        })
        .manage(project)
        .manage(history)
        .manage(cache)
        .manage(Arc::new(font_db) as FontDbState)
        .invoke_handler(tauri::generate_handler![
            commands::canvas::new_canvas,
            commands::canvas::get_status,
            commands::canvas::get_canvas,
            commands::canvas::save_project,
            commands::canvas::open_project,
            commands::canvas::import_svg_file,
            commands::canvas::get_ws_info,
            commands::elements::add_shape,
            commands::elements::add_text,
            commands::elements::add_icon,
            commands::elements::add_image,
            commands::elements::add_path,
            commands::elements::set_props,
            commands::elements::remove_element,
            commands::elements::list_elements,
            commands::elements::reorder_elements,
            commands::elements::update_canvas,
            commands::elements::list_icons,
            commands::elements::group_elements,
            commands::elements::ungroup,
            commands::elements::add_to_group,
            commands::elements::remove_from_group,
            commands::elements::duplicate_element,
            commands::elements::import_svg_elements,
            commands::elements::list_library_categories,
            commands::elements::list_library_assets,
            commands::elements::add_library_asset,
            commands::export::render_preview,
            commands::export::export_svg,
            commands::export::export_png,
            commands::export::export_ico,
            commands::export::export_android_icons,
            commands::export::export_ios_icons,
            commands::export::export_all,
            commands::style::set_gradient,
            commands::style::clear_gradient,
            commands::style::set_shadow,
            commands::style::clear_shadow,
            commands::style::set_blend_mode,
            commands::style::suggest_palette,
            commands::style::set_filter,
            commands::style::clear_filter,
            commands::style::apply_style_preset,
            commands::fonts::list_fonts,
            commands::layout::set_layout,
            commands::templates::list_builtin_templates,
            commands::templates::apply_builtin_template,
            commands::templates::save_as_template,
            commands::templates::list_user_templates_cmd,
            commands::templates::apply_user_template,
            commands::templates::delete_user_template,
            commands::history::undo,
            commands::history::redo,
            commands::history::can_undo,
            commands::history::can_redo,
            commands::analysis::analyze_colors,
            commands::analysis::check_consistency,
            commands::analysis::find_elements,
            commands::animation::set_animation,
            commands::animation::clear_animation,
            commands::animation::export_webp,
            commands::animation::export_lottie,
            commands::animation::export_animated_gif,
            commands::animation::preview_animation_frame,
            commands::elements::set_clip,
            commands::elements::clear_clip,
            commands::elements::set_mask,
            commands::elements::clear_mask,
            commands::elements::boolean_operation,
            commands::elements::convert_to_path,
            commands::pages::list_pages,
            commands::pages::add_page,
            commands::pages::switch_page,
            commands::pages::delete_page,
            commands::pages::duplicate_page,
            commands::pages::rename_page,
            commands::pages::get_active_page,
            commands::symbols::create_symbol,
            commands::symbols::list_symbols,
            commands::symbols::update_symbol,
            commands::symbols::detach_symbol_cmd,
            commands::symbols::add_symbol_override,
            commands::symbols::remove_symbol_override,
            commands::adaptive::preview_adaptive_icon,
            commands::adaptive::check_adaptive_safe_zone,
            commands::adaptive::set_adaptive_foreground,
            commands::adaptive::set_adaptive_background,
            commands::adaptive::export_adaptive_android,
            commands::brand::list_brand_kits,
            commands::brand::create_brand_kit,
            commands::brand::apply_brand,
            commands::brand::generate_brand_variant,
            commands::brand::export_brand_guide,
            commands::brand::suggest_brand,
            commands::brand::delete_brand_kit,
            commands::brand::update_brand_kit_color,
            commands::iconset::list_icon_sets,
            commands::iconset::create_icon_set,
            commands::iconset::add_to_icon_set,
            commands::iconset::remove_from_icon_set,
            commands::iconset::get_icon_set,
            commands::iconset::export_icon_set,
            commands::iconset::check_icon_set_consistency,
            commands::iconset::tag_icon_entry,
            commands::iconset::search_icons,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if window.emit("close-requested", ()).is_err() {
                    // Frontend unreachable — force exit
                    std::process::exit(0);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
