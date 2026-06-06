mod core;
mod elements_tools;
mod style_tools;
mod export_tools;
mod analysis_tools;
mod canvas_tools;
mod group_tools;
mod animation_tools;
mod boolean_tools;
mod filter_tools;
mod page_tools;
mod symbol_tools;
mod style_preset_tools;
mod adaptive_tools;
mod brand_tools;
mod iconset_tools;
mod lottie_tools;

pub use core::*;
pub use elements_tools::*;
pub use style_tools::*;
pub use export_tools::*;
pub use analysis_tools::*;
pub use canvas_tools::*;
pub use group_tools::*;
pub use animation_tools::*;
pub use boolean_tools::*;
pub use filter_tools::*;
pub use page_tools::*;
pub use symbol_tools::*;
pub use style_preset_tools::*;
pub use adaptive_tools::*;
pub use brand_tools::*;
pub use iconset_tools::*;
pub use lottie_tools::*;

use rmcp::{
    ServerHandler,
    handler::server::tool::ToolRouter,
    model::{ErrorData, Implementation, ServerCapabilities, ServerInfo},
    tool_handler,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

use crate::engine::builder::RenderCache;
use crate::model::IconProject;
use crate::model::history::CommandHistory;

// ---------------------------------------------------------------------------
// Shared param struct (used by both core and extended tools)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RemoveElementParams {
    #[schemars(description = "Element ID to remove")]
    pub element_id: String,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub struct IconStudioHandler {
    pub project: Arc<Mutex<IconProject>>,
    pub cache: Arc<Mutex<RenderCache>>,
    pub history: Arc<Mutex<CommandHistory>>,
    pub core_router: ToolRouter<Self>,
    pub full_router: ToolRouter<Self>,
    pub full_mode: AtomicBool,
    pub app_handle: Option<AppHandle>,
}

impl IconStudioHandler {
    pub fn new(project: Arc<Mutex<IconProject>>, cache: Arc<Mutex<RenderCache>>, history: Arc<Mutex<CommandHistory>>, app_handle: Option<AppHandle>) -> Self {
        let core = Self::core_router();
        let mut full = Self::core_router();
        full.merge(Self::elements_router());
        full.merge(Self::style_router());
        full.merge(Self::export_router());
        full.merge(Self::analysis_router());
        full.merge(Self::canvas_router());
        full.merge(Self::group_router());
        full.merge(Self::animation_router());
        full.merge(Self::boolean_router());
        full.merge(Self::filter_router());
        full.merge(Self::page_router());
        full.merge(Self::symbol_router());
        full.merge(Self::style_preset_router());
        full.merge(Self::adaptive_router());
        full.merge(Self::brand_router());
        full.merge(Self::iconset_router());
        full.merge(Self::lottie_router());
        Self {
            project,
            cache,
            history,
            core_router: core,
            full_router: full,
            full_mode: AtomicBool::new(true),  // Always full mode
            app_handle,
        }
    }

    pub fn emit_change(&self) {
        if let Some(ref handle) = self.app_handle {
            if let Err(e) = handle.emit("project-changed", ()) {
                eprintln!("Warning: failed to emit project-changed event: {}", e);
            }
        }
    }

    fn active_router(&self) -> &ToolRouter<Self> {
        if self.full_mode.load(Ordering::SeqCst) {
            &self.full_router
        } else {
            &self.core_router
        }
    }

    pub fn is_full_mode(&self) -> bool {
        self.full_mode.load(Ordering::SeqCst)
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// System internal error — build/render/IO failures, unexpected conditions.
pub(crate) fn internal_err(msg: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(msg.to_string(), None)
}

/// User input validation failure — invalid parameters, out-of-range values.
pub(crate) fn invalid_params(msg: impl std::fmt::Display) -> ErrorData {
    ErrorData::invalid_params(msg.to_string(), None)
}

/// State lock error — Mutex poisoned, concurrent access failure.
pub(crate) fn state_err(e: impl std::fmt::Display) -> ErrorData {
        ErrorData::internal_error(format!("Internal state error: {}", e), None)
}

/// Element not found — the requested element ID does not exist.
#[allow(dead_code)]
pub(crate) fn not_found_err(msg: impl std::fmt::Display) -> ErrorData {
    ErrorData::invalid_params(msg.to_string(), None)
}

// ---------------------------------------------------------------------------
// ServerHandler
// ---------------------------------------------------------------------------

#[tool_handler(router = self.active_router().clone())]
impl ServerHandler for IconStudioHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder().enable_tools().build(),
        )
        .with_server_info(
            Implementation::new("iconstudio", env!("CARGO_PKG_VERSION"))
                .with_title("IconStudio MCP Server"),
        )
        .with_instructions(
            "IconStudio MCP server for creating and editing icons, logos, and app icons. Workflow: icon_new (create canvas) → add_shape/add_text/add_icon/add_image (add elements) → set_props (modify properties) → set_gradient/set_shadow (styles) → export_svg/export_png/export_ico/export_all (export). All 62 tools are available."
        )
    }
}
