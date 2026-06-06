use crate::engine::builder;
use crate::engine::exporter;
use crate::error::AppError;
use crate::model::{ConsistencyIssue, Element, IconProject, IconSet, SetEntry};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_id() -> u64 {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let seq = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    ts.wrapping_add(seq)
}

fn sets_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".iconstudio").join("sets")
}

/// Create a new empty icon set, persisting the index file to disk.
pub fn create_set(name: &str, description: &str) -> Result<IconSet, AppError> {
    let id = format!("set-{}", unique_id());
    let dir = sets_dir().join(&id);
    fs::create_dir_all(&dir).map_err(|e| AppError::ExportError(format!("Failed to create set directory: {}", e)))?;

    let created_at = chrono_now();
    let set = IconSet {
        id,
        name: name.to_string(),
        description: description.to_string(),
        entries: Vec::new(),
        created_at,
    };

    save_set_index(&set)?;
    Ok(set)
}

/// Add the current project as an entry in the set.
/// Copies the project JSON into the set directory and generates a thumbnail.
pub fn add_entry(
    set: &mut IconSet,
    project: &IconProject,
    entry_name: &str,
    tags: Vec<String>,
) -> Result<SetEntry, AppError> {
    let entry_id = format!("entry-{}", unique_id());
    let name = if entry_name.is_empty() {
        format!("icon-{}", set.entries.len() + 1)
    } else {
        entry_name.to_string()
    };

    let set_dir = sets_dir().join(&set.id);
    let thumb_dir = set_dir.join("thumbnails");
    fs::create_dir_all(&thumb_dir)
        .map_err(|e| AppError::ExportError(format!("Failed to create thumbnails dir: {}", e)))?;

    // Save project file
    let project_path = set_dir.join(format!("{}.iconproject.json", entry_id));
    let json = serde_json::to_string_pretty(project)
        .map_err(|e| AppError::ExportError(format!("Failed to serialize project: {}", e)))?;
    fs::write(&project_path, json)
        .map_err(|e| AppError::ExportError(format!("Failed to write project file: {}", e)))?;

    // Generate thumbnail (128x128 SVG data URI)
    let svg_str = builder::build(project)?;
    let thumbnail = generate_thumbnail_svg(&svg_str, 128)?;

    let entry = SetEntry {
        id: entry_id,
        name,
        tags,
        project_path: project_path.to_string_lossy().to_string(),
        thumbnail,
    };

    set.entries.push(entry.clone());
    save_set_index(set)?;

    Ok(entry)
}

/// Remove an entry from a set and delete its files.
pub fn remove_entry(set_id: &str, entry_id: &str) -> Result<(), AppError> {
    let mut set = load_set(set_id)?;
    let idx = set.entries.iter().position(|e| e.id == entry_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Entry '{}' not found", entry_id)))?;

    let entry = set.entries.remove(idx);

    // Delete project file
    let path = Path::new(&entry.project_path);
    if path.exists() {
        let _ = fs::remove_file(path);
    }

    // Delete thumbnail
    let thumb_path = sets_dir().join(set_id).join("thumbnails").join(format!("{}.svg", entry_id));
    if thumb_path.exists() {
        let _ = fs::remove_file(thumb_path);
    }

    save_set_index(&set)?;
    Ok(())
}

/// List all icon sets found on disk.
pub fn list_sets() -> Result<Vec<IconSet>, AppError> {
    let base = sets_dir();
    if !base.exists() {
        return Ok(Vec::new());
    }

    let mut sets = Vec::new();
    let entries = fs::read_dir(&base)
        .map_err(|e| AppError::ExportError(format!("Failed to read sets directory: {}", e)))?;

    for entry in entries.flatten() {
        let index = entry.path().join("icon-set.json");
        if index.exists() {
            if let Ok(set) = load_set_from_path(&index) {
                sets.push(set);
            }
        }
    }

    sets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(sets)
}

/// Load a single set by ID.
pub fn load_set(set_id: &str) -> Result<IconSet, AppError> {
    let path = sets_dir().join(set_id).join("icon-set.json");
    load_set_from_path(&path)
}

/// Export all icons in a set to a directory.
pub fn export_set(
    set_id: &str,
    format: &str,
    sizes: &[u32],
    output_dir: &str,
) -> Result<Vec<PathBuf>, AppError> {
    let set = load_set(set_id)?;
    if set.entries.is_empty() {
        return Ok(Vec::new());
    }

    let out = PathBuf::from(output_dir).join(sanitize_filename(&set.name));
    fs::create_dir_all(&out)
        .map_err(|e| AppError::ExportError(format!("Failed to create output dir: {}", e)))?;

    let results: Vec<Result<Vec<PathBuf>, AppError>> = set.entries
        .par_iter()
        .map(|entry| {
            let project = load_project_from_path(&entry.project_path)?;
            let svg_str = builder::build(&project)?;
            let entry_name = sanitize_filename(&entry.name);

            let mut files = Vec::new();
            match format {
                "svg" => {
                    let path = out.join(format!("{}.svg", entry_name));
                    fs::write(&path, &svg_str)
                        .map_err(|e| AppError::ExportError(format!("Write failed: {}", e)))?;
                    files.push(path);
                }
                "png" => {
                    for &size in sizes {
                        let path = out.join(format!("{}-{}.png", entry_name, size));
                        let png_bytes = exporter::render_to_png(&svg_str, size)?;
                        fs::write(&path, png_bytes)
                            .map_err(|e| AppError::ExportError(format!("Write failed: {}", e)))?;
                        files.push(path);
                    }
                }
                _ => {
                    // "all" — both SVG and PNG at all sizes
                    let svg_path = out.join(format!("{}.svg", entry_name));
                    fs::write(&svg_path, &svg_str)
                        .map_err(|e| AppError::ExportError(format!("Write failed: {}", e)))?;
                    files.push(svg_path);

                    for &size in sizes {
                        let path = out.join(format!("{}-{}.png", entry_name, size));
                        let png_bytes = exporter::render_to_png(&svg_str, size)?;
                        fs::write(&path, png_bytes)
                            .map_err(|e| AppError::ExportError(format!("Write failed: {}", e)))?;
                        files.push(path);
                    }
                }
            }
            Ok(files)
        })
        .collect();

    let mut all_files = Vec::new();
    for r in results {
        all_files.extend(r?);
    }
    Ok(all_files)
}

/// Consistency report returned by check_consistency.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SetConsistencyReport {
    pub consistent: bool,
    pub issues: Vec<ConsistencyIssue>,
    pub summary: String,
}

/// Check consistency across all icons in a set.
/// Checks stroke_width, border_radius, font_size, opacity, and fill color consistency.
pub fn check_consistency(set_id: &str) -> Result<SetConsistencyReport, AppError> {
    let set = load_set(set_id)?;
    if set.entries.is_empty() {
        return Ok(SetConsistencyReport {
            consistent: true,
            issues: Vec::new(),
            summary: "Empty set, no issues.".to_string(),
        });
    }

    let mut all_stroke_widths: Vec<(String, String, f64)> = Vec::new(); // (entry_name, element_id, value)
    let mut all_border_radii: Vec<(String, String, f64)> = Vec::new();
    let mut all_font_sizes: Vec<(String, String, f64)> = Vec::new();
    let mut all_opacities: Vec<(String, String, f64)> = Vec::new();
    let mut all_fills: Vec<(String, String, String)> = Vec::new();

    for entry in &set.entries {
        if let Ok(project) = load_project_from_path(&entry.project_path) {
            collect_element_stats(
                &project.active_elements(),
                &entry.name,
                &mut all_stroke_widths,
                &mut all_border_radii,
                &mut all_font_sizes,
                &mut all_opacities,
                &mut all_fills,
            );
        }
    }

    let mut issues = Vec::new();

    check_property_consistency("stroke_width", &all_stroke_widths, &mut issues, 0.1);
    check_property_consistency("border_radius", &all_border_radii, &mut issues, 0.1);
    check_property_consistency("font_size", &all_font_sizes, &mut issues, 0.1);
    check_property_consistency("opacity", &all_opacities, &mut issues, 0.1);

    // Color consistency: identify colors that appear only once or rarely
    check_color_consistency(&all_fills, &mut issues);

    let consistent = issues.is_empty();
    let summary = if consistent {
        "All icons are consistent.".to_string()
    } else {
        format!("Found {} consistency issue(s).", issues.len())
    };

    Ok(SetConsistencyReport {
        consistent,
        issues,
        summary,
    })
}

/// Update tags on an entry.
pub fn tag_entry(set_id: &str, entry_id: &str, tags: Vec<String>) -> Result<(), AppError> {
    let mut set = load_set(set_id)?;
    let entry = set.entries.iter_mut()
        .find(|e| e.id == entry_id)
        .ok_or_else(|| AppError::NotFoundError(format!("Entry '{}' not found", entry_id)))?;
    entry.tags = tags;
    save_set_index(&set)?;
    Ok(())
}

/// Search entries by name or tags, optionally within a specific set.
pub fn search_entries(query: &str, tags: Option<&Vec<String>>, set_id: Option<&str>) -> Result<Vec<SetEntry>, AppError> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    let sets: Vec<IconSet> = if let Some(sid) = set_id {
        vec![load_set(sid)?]
    } else {
        list_sets()?
    };

    for set in &sets {
        for entry in &set.entries {
            // If tags filter is provided, entry must match all required tags
            let tag_filter_ok = tags.map_or(true, |required_tags| {
                required_tags.iter().all(|t| {
                    entry.tags.iter().any(|et| et.to_lowercase() == t.to_lowercase())
                })
            });

            if !tag_filter_ok {
                continue;
            }

            // If query is non-empty, match by name or tag text
            if query_lower.is_empty() {
                results.push(entry.clone());
            } else {
                let name_match = entry.name.to_lowercase().contains(&query_lower);
                let tag_text_match = entry.tags.iter().any(|t| t.to_lowercase().contains(&query_lower));
                if name_match || tag_text_match {
                    results.push(entry.clone());
                }
            }
        }
    }

    Ok(results)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple ISO-ish format without chrono dependency
    format!("{}", now)
}

fn save_set_index(set: &IconSet) -> Result<(), AppError> {
    let dir = sets_dir().join(&set.id);
    fs::create_dir_all(&dir)
        .map_err(|e| AppError::ExportError(format!("Failed to create set dir: {}", e)))?;

    let path = dir.join("icon-set.json");
    let json = serde_json::to_string_pretty(set)
        .map_err(|e| AppError::ExportError(format!("Failed to serialize set index: {}", e)))?;
    fs::write(&path, json)
        .map_err(|e| AppError::ExportError(format!("Failed to write set index: {}", e)))?;
    Ok(())
}

fn load_set_from_path(path: &Path) -> Result<IconSet, AppError> {
    let json = fs::read_to_string(path)
        .map_err(|e| AppError::ExportError(format!("Failed to read set index: {}", e)))?;
    serde_json::from_str(&json)
        .map_err(|e| AppError::ExportError(format!("Failed to parse set index: {}", e)))
}

fn load_project_from_path(path: &str) -> Result<IconProject, AppError> {
    let json = fs::read_to_string(path)
        .map_err(|e| AppError::ExportError(format!("Failed to read project '{}': {}", path, e)))?;
    serde_json::from_str(&json)
        .map_err(|e| AppError::ExportError(format!("Failed to parse project '{}': {}", path, e)))
}

fn generate_thumbnail_svg(svg_str: &str, size: u32) -> Result<String, AppError> {
    // Generate a small PNG thumbnail, then encode as data URI
    let png_bytes = exporter::render_to_png(svg_str, size)?;
    let b64 = base64_encode(&png_bytes);
    Ok(format!("data:image/png;base64,{}", b64))
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

fn collect_element_stats(
    elements: &[Element],
    entry_name: &str,
    stroke_widths: &mut Vec<(String, String, f64)>,
    border_radii: &mut Vec<(String, String, f64)>,
    font_sizes: &mut Vec<(String, String, f64)>,
    opacities: &mut Vec<(String, String, f64)>,
    fills: &mut Vec<(String, String, String)>,
) {
    for elem in elements {
        let id = elem.id().to_string();
        let common = elem.common();

        opacities.push((entry_name.to_string(), id.clone(), common.opacity));

        match elem {
            Element::Shape(s) => {
                if s.stroke_width > 0.0 {
                    stroke_widths.push((entry_name.to_string(), id.clone(), s.stroke_width));
                }
                if s.border_radius > 0.0 {
                    border_radii.push((entry_name.to_string(), id.clone(), s.border_radius));
                }
                fills.push((entry_name.to_string(), id.clone(), s.fill.to_uppercase()));
            }
            Element::Text(t) => {
                font_sizes.push((entry_name.to_string(), id.clone(), t.font_size));
                if t.stroke_width > 0.0 {
                    stroke_widths.push((entry_name.to_string(), id.clone(), t.stroke_width));
                }
                fills.push((entry_name.to_string(), id.clone(), t.fill.to_uppercase()));
            }
            Element::Icon(i) => {
                if i.stroke_width > 0.0 {
                    stroke_widths.push((entry_name.to_string(), id.clone(), i.stroke_width));
                }
                fills.push((entry_name.to_string(), id.clone(), i.fill.to_uppercase()));
            }
            Element::Path(p) => {
                if p.stroke_width > 0.0 {
                    stroke_widths.push((entry_name.to_string(), id.clone(), p.stroke_width));
                }
                fills.push((entry_name.to_string(), id.clone(), p.fill.to_uppercase()));
            }
            Element::Group(g) => {
                // Recurse into group children
                collect_element_stats(
                    &g.children,
                    entry_name,
                    stroke_widths,
                    border_radii,
                    font_sizes,
                    opacities,
                    fills,
                );
            }
            _ => {}
        }
    }
}

/// Find the mode (most frequent value) of a numeric property.
fn find_mode(values: &[(String, String, f64)]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut freq: HashMap<u64, usize> = HashMap::new();
    let mut val_map: HashMap<u64, f64> = HashMap::new();
    for (_, _, v) in values {
        // Quantize to 2 decimal places for grouping
        let key = (v * 100.0).round() as u64;
        *freq.entry(key).or_insert(0) += 1;
        val_map.entry(key).or_insert(*v);
    }
    freq.into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(key, _)| val_map[&key])
}

fn check_property_consistency(
    property: &str,
    values: &[(String, String, f64)],
    issues: &mut Vec<ConsistencyIssue>,
    threshold: f64,
) {
    if values.len() < 2 {
        return;
    }
    let mode = match find_mode(values) {
        Some(m) => m,
        None => return,
    };
    if mode == 0.0 {
        return; // Skip zero as reference
    }

    for (entry_name, element_id, v) in values {
        let deviation = (v - mode).abs() / mode;
        if deviation > threshold {
            issues.push(ConsistencyIssue {
                property: property.to_string(),
                expected: format!("{:.2}", mode),
                actual: format!("{:.2}", v),
                element_id: element_id.clone(),
                project_path: entry_name.clone(),
            });
        }
    }
}

fn check_color_consistency(
    fills: &[(String, String, String)],
    issues: &mut Vec<ConsistencyIssue>,
) {
    if fills.len() < 3 {
        return;
    }

    // Count color frequencies
    let mut freq: HashMap<String, usize> = HashMap::new();
    for (_, _, color) in fills {
        *freq.entry(color.clone()).or_insert(0) += 1;
    }

    // Colors used only once might be outliers
    let singletons: Vec<&str> = freq.iter()
        .filter(|(_, &count)| count == 1)
        .map(|(c, _)| c.as_str())
        .collect();

    if !singletons.is_empty() && freq.len() > 3 {
        // More than 3 unique colors with singletons — flag
        for (entry_name, element_id, color) in fills {
            if singletons.contains(&color.as_str()) {
                issues.push(ConsistencyIssue {
                    property: "fill".to_string(),
                    expected: "common palette colors".to_string(),
                    actual: color.clone(),
                    element_id: element_id.clone(),
                    project_path: entry_name.clone(),
                });
            }
        }
    }
}
