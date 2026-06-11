use crate::engine::variants;
use crate::engine::weight::{self, WeightPreset};
use crate::error::AppError;
use crate::model::{ThemeRule, ThemeVariant};
use serde::{Deserialize, Serialize};
use tauri::State;

use super::canvas::ProjectState;
use super::export::RenderCacheState;

#[tauri::command]
pub fn create_variant(
    state: State<'_, ProjectState>,
    name: String,
    rules: Vec<ThemeRule>,
) -> Result<ThemeVariant, String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let variant = ThemeVariant {
        name,
        base_page_index: project.active_page_index,
        rules,
    };
    project.theme_variants.push(variant.clone());
    project.bump_version();
    Ok(variant)
}

#[tauri::command]
pub fn delete_variant(
    state: State<'_, ProjectState>,
    index: usize,
) -> Result<(), String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    if index >= project.theme_variants.len() {
        return Err(format!("Variant index {} out of range", index));
    }
    project.theme_variants.remove(index);
    project.bump_version();
    Ok(())
}

#[tauri::command]
pub fn preview_variant(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    index: usize,
) -> Result<String, String> {
    let (variant, project_clone) = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        if index >= project.theme_variants.len() {
            return Err(format!("Variant index {} out of range", index));
        }
        let variant = project.theme_variants[index].clone();
        (variant, project.clone())
    };

    let derived = variants::generate_variant(&project_clone, &variant.rules);
    let mut cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    cache.build(&derived).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_variant(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    index: usize,
    format: String,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let variant = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        if index >= project.theme_variants.len() {
            return Err(format!("Variant index {} out of range", index));
        }
        project.theme_variants[index].clone()
    };

    export_variant_inner(&state, &cache_state, &variant, &format, &output_dir)
}

#[tauri::command]
pub fn export_all_variants(
    state: State<'_, ProjectState>,
    cache_state: State<'_, RenderCacheState>,
    format: String,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let variants_list: Vec<ThemeVariant> = {
        let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        project.theme_variants.clone()
    };

    let mut all_paths = Vec::new();
    for variant in &variants_list {
        let paths = export_variant_inner(&state, &cache_state, variant, &format, &output_dir)?;
        all_paths.extend(paths);
    }
    Ok(all_paths)
}

fn export_variant_inner(
    state: &State<'_, ProjectState>,
    cache_state: &State<'_, RenderCacheState>,
    variant: &ThemeVariant,
    format: &str,
    output_dir: &str,
) -> Result<Vec<String>, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let derived = variants::generate_variant(&project, &variant.rules);
    drop(project);

    let svg_str = {
        let _cache = cache_state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
        crate::engine::builder::build(&derived)?
    };

    let safe_name = variant.name.to_lowercase().replace(' ', "-");
    let file_base = format!("icon.{}", safe_name);

    match format {
        "svg" => {
            let path = format!("{}/{}.svg", output_dir.trim_end_matches('/'), file_base);
            crate::services::export::write_svg_to_file(&svg_str, &path)?;
            Ok(vec![path])
        }
        "png" => {
            crate::services::export::write_pngs_to_dir(
                &svg_str,
                &[16, 32, 64, 128, 256, 512],
                output_dir,
            )
            .map(|paths| {
                paths.into_iter().map(|p| {
                    // Rename to include variant name
                    p.replace("icon-", &format!("{}-", file_base))
                }).collect()
            })
            .map_err(|e| e.to_string())
        }
        _ => {
            let path = format!("{}/{}.svg", output_dir.trim_end_matches('/'), file_base);
            crate::services::export::write_svg_to_file(&svg_str, &path)?;
            Ok(vec![path])
        }
    }
}

#[tauri::command]
pub fn list_variants(state: State<'_, ProjectState>) -> Result<Vec<ThemeVariant>, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    Ok(project.theme_variants.clone())
}

#[tauri::command]
pub fn list_preset_rules() -> Vec<variants::PresetRuleSet> {
    variants::list_preset_rule_sets()
}

#[tauri::command]
pub fn generate_all_presets(
    state: State<'_, ProjectState>,
) -> Result<Vec<ThemeVariant>, String> {
    let mut project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let presets = variants::list_preset_rule_sets();
    let mut created = Vec::new();
    for preset in &presets {
        let variant = ThemeVariant {
            name: preset.name.clone(),
            base_page_index: project.active_page_index,
            rules: preset.rules.clone(),
        };
        project.theme_variants.push(variant.clone());
        created.push(variant);
    }
    project.bump_version();
    Ok(created)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeightVariantResult {
    pub weight: String,
    pub svg: String,
}

#[tauri::command]
pub fn generate_weight_variants(
    state: State<'_, ProjectState>,
    weights: Vec<String>,
) -> Result<Vec<WeightVariantResult>, String> {
    let project = state.lock().map_err(|e| AppError::LockError(e.to_string()))?;
    let presets: Vec<WeightPreset> = weights.iter().filter_map(|w| match w.as_str() {
        "thin" => Some(WeightPreset::Thin),
        "light" => Some(WeightPreset::Light),
        "regular" => Some(WeightPreset::Regular),
        "medium" => Some(WeightPreset::Medium),
        "bold" => Some(WeightPreset::Bold),
        "fill" => Some(WeightPreset::Fill),
        _ => None,
    }).collect();

    let variants = weight::generate_weight_variants(&project, &presets);
    let results: Vec<WeightVariantResult> = variants.into_iter().map(|v| {
        let weight_str = match &v.weight {
            WeightPreset::Thin => "thin",
            WeightPreset::Light => "light",
            WeightPreset::Regular => "regular",
            WeightPreset::Medium => "medium",
            WeightPreset::Bold => "bold",
            WeightPreset::Fill => "fill",
        };
        let svg = crate::engine::builder::build(&v.project).unwrap_or_default();
        WeightVariantResult {
            weight: weight_str.to_string(),
            svg,
        }
    }).collect();

    Ok(results)
}
