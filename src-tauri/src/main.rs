#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use icon_studio_lib::model::IconProject;
use icon_studio_lib::model::history::CommandHistory;
use icon_studio_lib::engine::builder::RenderCache;
use icon_studio_lib::commands::canvas::ProjectState;
use icon_studio_lib::commands::history::HistoryState;
use icon_studio_lib::commands::export::RenderCacheState;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--mcp".to_string()) {
        run_mcp_server();
        return;
    }

    if let Some("export" | "analyze" | "batch") = args.get(1).map(|s| s.as_str()) {
        if let Err(e) = icon_studio_lib::cli::run() {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
        return;
    }

    // Create shared state instances — same Arcs are passed to Tauri managed state
    let project: ProjectState = Arc::new(Mutex::new(IconProject::default()));
    let history: HistoryState = Arc::new(Mutex::new(CommandHistory::default()));
    let cache: RenderCacheState = Arc::new(Mutex::new(RenderCache::default()));

    icon_studio_lib::run(project, history, cache);
}

fn run_mcp_server() {
    let project = Arc::new(Mutex::new(IconProject::default()));
    let cache = Arc::new(Mutex::new(RenderCache::default()));
    let history: HistoryState = Arc::new(Mutex::new(CommandHistory::default()));
    let handler = icon_studio_lib::mcp::IconStudioHandler::new(project, cache, history, None);

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        use rmcp::ServiceExt;
        let service = handler
            .serve(rmcp::transport::stdio())
            .await
            .expect("MCP server failed to start");
        service.waiting().await.expect("MCP server error");
    });
}
