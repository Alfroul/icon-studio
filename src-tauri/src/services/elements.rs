//! Shared image/MIME helpers — used by both Tauri commands and MCP tools.

use crate::error::AppError;
use crate::model::Element;
use base64::Engine;
use std::path::Path;

/// Read an image file, detect its MIME type from extension, and return
/// the base64-encoded `data:` URI string along with the detected MIME type.
pub fn detect_mime_and_encode(file_path: &str) -> Result<(String, String), AppError> {
    crate::engine::utils::validate_file_path(file_path)?;
    let bytes = std::fs::read(file_path)?;
    if bytes.len() > 10 * 1024 * 1024 {
        return Err(AppError::ValidationError("Image file too large (max 10MB)".into()));
    }
    let extension = Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let mime = match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        _ => "image/png",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data = format!("data:{};base64,{}", mime, b64);
    Ok((data, mime.to_string()))
}

const MAX_REFERENCE_DEPTH: usize = 10;

pub fn validate_clip_mask_reference(
    elements: &[Element],
    element_id: &str,
    ref_element_id: &str,
) -> Result<(), AppError> {
    if element_id == ref_element_id {
        return Err(AppError::ValidationError("Self-reference not allowed for clip/mask".into()));
    }

    if !elements.iter().any(|e| e.id() == ref_element_id) {
        return Err(AppError::ValidationError(format!(
            "Reference element '{}' not found", ref_element_id
        )));
    }

    check_no_circular(elements, ref_element_id, element_id, 0)
}

fn check_no_circular(
    elements: &[Element],
    from_id: &str,
    original_id: &str,
    depth: usize,
) -> Result<(), AppError> {
    if depth > MAX_REFERENCE_DEPTH {
        return Err(AppError::ValidationError("Clip/mask reference chain too deep".into()));
    }

    let target = match elements.iter().find(|e| e.id() == from_id) {
        Some(e) => e,
        None => return Ok(()),
    };

    let common = target.common();
    if let Some(ref clip_id) = common.clip_element_id {
        if clip_id == original_id {
            return Err(AppError::ValidationError("Circular clip/mask reference detected".into()));
        }
        check_no_circular(elements, clip_id, original_id, depth + 1)?;
    }
    if let Some(ref mask_id) = common.mask_element_id {
        if mask_id == original_id {
            return Err(AppError::ValidationError("Circular clip/mask reference detected".into()));
        }
        check_no_circular(elements, mask_id, original_id, depth + 1)?;
    }

    Ok(())
}

pub fn cleanup_clip_mask_refs(elements: &mut [Element], deleted_id: &str) {
    for elem in elements.iter_mut() {
        let common = elem.common_mut();
        if let Some(ref clip_id) = common.clip_element_id {
            if clip_id == deleted_id {
                common.clip_element_id = None;
            }
        }
        if let Some(ref mask_id) = common.mask_element_id {
            if mask_id == deleted_id {
                common.mask_element_id = None;
            }
        }
        if let Element::Group(g) = elem {
            cleanup_clip_mask_refs(&mut g.children, deleted_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CommonProps, ShapeElement, shapes::ShapeType};

    fn make_shape(id: &str) -> Element {
        Element::Shape(ShapeElement {
            common: CommonProps {
                id: id.to_string(),
                x: 0.0, y: 0.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
                mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
            },
            shape_type: ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        })
    }

    #[test]
    fn test_validate_clip_no_self_reference() {
        let elems = vec![make_shape("shape-1")];
        let result = validate_clip_mask_reference(&elems, "shape-1", "shape-1");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Self-reference"));
    }

    #[test]
    fn test_validate_clip_no_circular() {
        let mut a = make_shape("shape-1");
        a.common_mut().clip_element_id = Some("shape-2".to_string());
        let b = make_shape("shape-2");

        let result = validate_clip_mask_reference(&vec![a.clone(), b], "shape-2", "shape-1");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular"));
    }

    #[test]
    fn test_validate_clip_element_exists() {
        let elems = vec![make_shape("shape-1")];
        let result = validate_clip_mask_reference(&elems, "shape-1", "nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_validate_clip_ok() {
        let a = make_shape("shape-1");
        let b = make_shape("shape-2");
        let result = validate_clip_mask_reference(&vec![a, b], "shape-1", "shape-2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_cleanup_clip_mask_refs() {
        let mut a = make_shape("shape-1");
        a.common_mut().clip_element_id = Some("shape-2".to_string());
        let mut b = make_shape("shape-2");
        b.common_mut().mask_element_id = Some("shape-1".to_string());
        let c = make_shape("shape-3");

        let mut elems = vec![a, b, c];
        cleanup_clip_mask_refs(&mut elems, "shape-2");

        assert!(elems[0].common().clip_element_id.is_none(), "clip ref to shape-2 should be cleared");
        assert!(elems[1].common().mask_element_id.is_some(), "mask ref to shape-1 should remain");
    }
}
