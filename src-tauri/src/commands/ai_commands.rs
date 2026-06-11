use crate::engine::ai::{self, AiConfig};
use crate::model::{AiProvider, AiTask, IconStyle};
use crate::services::ai_config;
use tauri::State;

use super::canvas::ProjectState;

fn ai_config_from_parts(
    provider: AiProvider,
    api_key: String,
    model: String,
    endpoint: Option<String>,
) -> AiConfig {
    AiConfig {
        provider,
        api_key,
        model,
        endpoint,
        ..Default::default()
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn generate_icon(
    _state: State<'_, ProjectState>,
    task: AiTask,
    style: IconStyle,
    prompt: String,
    provider: AiProvider,
    api_key: String,
    model: String,
    endpoint: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let config = ai_config_from_parts(provider, api_key, model, endpoint);
    let icons = ai::generate_icon(task, style, &prompt, &config)
        .await
        .map_err(|e| e.to_string())?;

    Ok(icons.into_iter().map(|i| serde_json::to_value(i).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn generate_icon_set(
    _state: State<'_, ProjectState>,
    prompts: Vec<String>,
    style: IconStyle,
    provider: AiProvider,
    api_key: String,
    model: String,
) -> Result<Vec<serde_json::Value>, String> {
    let config = ai_config_from_parts(provider, api_key, model, None);
    let icons = ai::generate_icon_set(&prompts, style, &config)
        .await
        .map_err(|e| e.to_string())?;

    Ok(icons.into_iter().map(|i| serde_json::to_value(i).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn ai_remove_background(
    _state: State<'_, ProjectState>,
    image_data: String,
    provider: AiProvider,
    api_key: String,
    model: String,
) -> Result<String, String> {
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, image_data)
        .map_err(|e| format!("Invalid base64 image data: {}", e))?;

    let config = ai_config_from_parts(provider, api_key, model, None);

    let result = ai::remove_background(&bytes, &config)
        .await
        .map_err(|e| e.to_string())?;

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        result,
    ))
}

#[tauri::command]
pub fn configure_ai(
    provider: AiProvider,
    api_key: String,
    model: String,
    endpoint: Option<String>,
) -> Result<(), String> {
    let config = ai_config_from_parts(provider, api_key, model, endpoint);
    ai_config::save_ai_config(&config)
}

#[tauri::command]
pub fn get_ai_config() -> Result<AiConfig, String> {
    ai_config::load_ai_config()
}
