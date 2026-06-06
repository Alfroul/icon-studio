//! Shared layout algorithm — used by both Tauri commands and MCP tools.

use crate::error::AppError;
use crate::model::helpers::{get_element_bounds, set_element_position};

/// Apply a layout to the given elements in-place.
///
/// Returns `Ok(())` on success, or an error for an invalid layout type.
/// The caller is responsible for cloning before / recording history.
pub fn apply_layout(
    elements: &mut [crate::model::Element],
    canvas_width: u32,
    canvas_height: u32,
    layout_type: &str,
    gap: f64,
    padding: f64,
) -> Result<(), AppError> {
    let cw = canvas_width as f64;
    let ch = canvas_height as f64;

    if elements.is_empty() {
        return Ok(());
    }

    match layout_type {
        "center" => {
            let min_x = elements.iter().map(|e| get_element_bounds(e).0).fold(f64::MAX, f64::min);
            let min_y = elements.iter().map(|e| get_element_bounds(e).1).fold(f64::MAX, f64::min);
            let max_x = elements.iter().map(|e| { let (x,_,w,_) = get_element_bounds(e); x + w }).fold(f64::MIN, f64::max);
            let max_y = elements.iter().map(|e| { let (_,y,_,h) = get_element_bounds(e); y + h }).fold(f64::MIN, f64::max);

            let group_cx = (min_x + max_x) / 2.0;
            let group_cy = (min_y + max_y) / 2.0;
            let target_cx = cw / 2.0;
            let target_cy = ch / 2.0;
            let dx = target_cx - group_cx;
            let dy = target_cy - group_cy;

            for elem in elements.iter_mut() {
                let (ex, ey, _, _) = get_element_bounds(elem);
                set_element_position(elem, ex + dx, ey + dy);
            }
        }
        "horizontal" => {
            let mut indices: Vec<usize> = (0..elements.len()).collect();
            indices.sort_by(|&a, &b| {
                let (ax, _, _, _) = get_element_bounds(&elements[a]);
                let (bx, _, _, _) = get_element_bounds(&elements[b]);
                ax.partial_cmp(&bx).unwrap_or(std::cmp::Ordering::Equal)
            });

            let total_width: f64 = indices.iter().map(|&i| get_element_bounds(&elements[i]).2).sum();
            let total_gaps = if indices.len() > 1 { (indices.len() - 1) as f64 * gap } else { 0.0 };
            let start_x = padding + (cw - padding * 2.0 - total_width - total_gaps) / 2.0;

            let mut current_x = start_x;
            for &i in &indices {
                let (_, _, ew, eh) = get_element_bounds(&elements[i]);
                let ny = padding + (ch - padding * 2.0 - eh) / 2.0;
                set_element_position(&mut elements[i], current_x, ny);
                current_x += ew + gap;
            }
        }
        "vertical" => {
            let mut indices: Vec<usize> = (0..elements.len()).collect();
            indices.sort_by(|&a, &b| {
                let (_, ay, _, _) = get_element_bounds(&elements[a]);
                let (_, by, _, _) = get_element_bounds(&elements[b]);
                ay.partial_cmp(&by).unwrap_or(std::cmp::Ordering::Equal)
            });

            let total_height: f64 = indices.iter().map(|&i| get_element_bounds(&elements[i]).3).sum();
            let total_gaps = if indices.len() > 1 { (indices.len() - 1) as f64 * gap } else { 0.0 };
            let start_y = padding + (ch - padding * 2.0 - total_height - total_gaps) / 2.0;

            let mut current_y = start_y;
            for &i in &indices {
                let (_, _, ew, eh) = get_element_bounds(&elements[i]);
                let nx = padding + (cw - padding * 2.0 - ew) / 2.0;
                set_element_position(&mut elements[i], nx, current_y);
                current_y += eh + gap;
            }
        }
        "stack" => {
            for elem in elements.iter_mut() {
                let (_, _, ew, eh) = get_element_bounds(elem);
                let nx = (cw - ew) / 2.0;
                let ny = (ch - eh) / 2.0;
                set_element_position(elem, nx, ny);
            }
        }
        other => {
            return Err(AppError::ValidationError(format!("Invalid layout type: {other}")));
        }
    }

    Ok(())
}
