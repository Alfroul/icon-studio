use crate::engine;
use crate::error::AppError;
use crate::model::{BrandColorRole, BrandKit};

use super::canvas::ProjectState;
use super::export::RenderCacheState;

use tauri::{AppHandle, Emitter, State};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct BrandKitInfo {
    pub id: String,
    pub name: String,
    pub colors: std::collections::HashMap<String, String>,
    pub variant_count: usize,
}

fn kit_to_info(kit: &BrandKit) -> BrandKitInfo {
    let mut colors = std::collections::HashMap::new();
    for (role, hex) in &kit.colors {
        let role_name = serde_json::to_string(role)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();
        colors.insert(role_name, hex.clone());
    }
    BrandKitInfo {
        id: kit.id.clone(),
        name: kit.name.clone(),
        colors,
        variant_count: kit.variants.len(),
    }
}

#[tauri::command]
pub fn list_brand_kits(state: State<'_, ProjectState>) -> Result<Vec<BrandKitInfo>, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    Ok(project.brand_kits.iter().map(kit_to_info).collect())
}

#[tauri::command]
pub fn create_brand_kit(
    app_handle: AppHandle,
    state: State<'_, ProjectState>,
    name: String,
    primary: String,
    secondary: Option<String>,
    accent: Option<String>,
    neutral: Option<String>,
) -> Result<BrandKitInfo, String> {
    let kit = engine::brand::create_brand_kit(
        &name,
        &primary,
        secondary.as_deref(),
        accent.as_deref(),
        neutral.as_deref(),
    )
    .map_err(AppError::BuildError)?;

    let info = kit_to_info(&kit);

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    project.brand_kits.push(kit);
    project.bump_version();
    drop(project);

    let _ = app_handle.emit("project-changed", ());
    Ok(info)
}

#[tauri::command]
pub fn apply_brand(
    app_handle: AppHandle,
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    kit_id: String,
    mode: Option<String>,
) -> Result<(), String> {
    let mode = mode.unwrap_or_else(|| "closest".to_string());

    let kit = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        project
            .brand_kits
            .iter()
            .find(|k| k.id == kit_id)
            .cloned()
            .ok_or_else(|| AppError::NotFoundError(format!("Brand kit '{}' not found", kit_id)))?
    };

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    engine::brand::apply_brand(&mut project, &kit, &mode)
        .map_err(AppError::BuildError)?;
    project.bump_version();
    drop(project);

    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.invalidate_cache();
    drop(cache);

    let _ = app_handle.emit("project-changed", ());
    Ok(())
}

#[tauri::command]
pub fn generate_brand_variant(
    app_handle: AppHandle,
    state: State<'_, ProjectState>,
    kit_id: String,
    variant_type: String,
) -> Result<BrandKitInfo, String> {
    let kit = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        project
            .brand_kits
            .iter()
            .find(|k| k.id == kit_id)
            .cloned()
            .ok_or_else(|| AppError::NotFoundError(format!("Brand kit '{}' not found", kit_id)))?
    };

    let variant = engine::brand::generate_variant(&kit, &variant_type)
        .map_err(AppError::BuildError)?;
    let info = kit_to_info(&variant);

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    project.brand_kits.push(variant);
    project.bump_version();
    drop(project);

    let _ = app_handle.emit("project-changed", ());
    Ok(info)
}

#[tauri::command]
pub fn export_brand_guide(
    state: State<'_, ProjectState>,
    kit_id: String,
) -> Result<String, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let kit = project
        .brand_kits
        .iter()
        .find(|k| k.id == kit_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Brand kit '{}' not found", kit_id)))?;
    Ok(engine::brand::export_brand_guide(kit))
}

#[tauri::command]
pub fn suggest_brand(
    description: String,
) -> Result<BrandKitInfo, String> {
    let kit = engine::brand::suggest_brand_from_description(&description)
        .map_err(AppError::BuildError)?;
    Ok(kit_to_info(&kit))
}

#[tauri::command]
pub fn delete_brand_kit(
    app_handle: AppHandle,
    state: State<'_, ProjectState>,
    kit_id: String,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let idx = project
        .brand_kits
        .iter()
        .position(|k| k.id == kit_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Brand kit '{}' not found", kit_id)))?;
    project.brand_kits.remove(idx);
    project.bump_version();
    drop(project);

    let _ = app_handle.emit("project-changed", ());
    Ok(())
}

#[tauri::command]
pub fn update_brand_kit_color(
    app_handle: AppHandle,
    state: State<'_, ProjectState>,
    kit_id: String,
    role: String,
    color: String,
) -> Result<(), String> {
    let role_enum: BrandColorRole =
        serde_json::from_str(&format!("\"{}\"", role)).map_err(|_| {
            AppError::ValidationError(format!("Invalid color role: {}", role))
        })?;

    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let kit = project
        .brand_kits
        .iter_mut()
        .find(|k| k.id == kit_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Brand kit '{}' not found", kit_id)))?;
    kit.colors.insert(role_enum, color);
    project.bump_version();
    drop(project);

    let _ = app_handle.emit("project-changed", ());
    Ok(())
}
