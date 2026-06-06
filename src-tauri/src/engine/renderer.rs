use crate::error::AppError;
use std::sync::OnceLock;

/// usvg::Options owns its internal fontdb::Database (unavoidable in usvg 0.44).
/// This is initialized once and shared across all render calls.
/// For font enumeration, the shared FontDbState (Arc<fontdb::Database>) in
/// Tauri managed state is used instead — see crate::FontDbState.
static USVG_OPTIONS: OnceLock<usvg::Options<'static>> = OnceLock::new();

pub fn get_options() -> &'static usvg::Options<'static> {
    USVG_OPTIONS.get_or_init(|| {
        let mut opt = usvg::Options::default();
        opt.fontdb_mut().load_system_fonts();
        opt
    })
}

/// Parse an SVG string into a usvg::Tree, reusable for multiple render passes.
pub fn parse_svg(svg_str: &str) -> Result<usvg::Tree, AppError> {
    let opt = get_options();
    usvg::Tree::from_data(svg_str.as_bytes(), opt)
        .map_err(|e| AppError::RenderError(format!("SVG parse error: {e}")))
}

/// Render a pre-parsed SVG tree to PNG bytes at the given size.
/// Use this in batch exports to avoid re-parsing the SVG per size.
pub fn render_from_tree(tree: &usvg::Tree, size: u32) -> Result<Vec<u8>, AppError> {
    validate_size(size)?;
    rasterize(tree, size)
}

pub fn render(svg_str: &str, size: u32) -> Result<Vec<u8>, AppError> {
    validate_size(size)?;
    let tree = parse_svg(svg_str)?;
    rasterize(&tree, size)
}

fn validate_size(size: u32) -> Result<(), AppError> {
    if size == 0 {
        return Err(AppError::RenderError(
            "Size must be greater than 0".to_string(),
        ));
    }

    if size > 8192 {
        return Err(AppError::RenderError(format!(
            "Size {size} exceeds maximum of 8192"
        )));
    }

    Ok(())
}

fn rasterize(tree: &usvg::Tree, size: u32) -> Result<Vec<u8>, AppError> {
    let svg_size = tree.size();
    if svg_size.width() <= 0.0 || svg_size.height() <= 0.0 {
        let pixmap = resvg::tiny_skia::Pixmap::new(size, size)
            .ok_or_else(|| AppError::RenderError("Failed to create pixmap".to_string()))?;
        return pixmap
            .encode_png()
            .map_err(|e| AppError::RenderError(format!("PNG encode error: {e}")));
    }
    let scale_x = size as f32 / svg_size.width();
    let scale_y = size as f32 / svg_size.height();
    let scale = scale_x.min(scale_y);
    let offset_x = (size as f32 - svg_size.width() * scale) / 2.0;
    let offset_y = (size as f32 - svg_size.height() * scale) / 2.0;
    let transform = resvg::tiny_skia::Transform {
        sx: scale, ky: 0.0, kx: 0.0, sy: scale, tx: offset_x, ty: offset_y,
    };

    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)
        .ok_or_else(|| AppError::RenderError("Failed to create pixmap".to_string()))?;

    // resvg::render() returns () in v0.44 — no failure signal to check.
    // If the tree is valid, rendering always writes pixels into the pixmap.
    resvg::render(tree, transform, &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|e| AppError::RenderError(format!("PNG encode error: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    static VALID_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="#FF0000"/></svg>"##;

    #[test]
    fn test_render_produces_png_bytes() {
        let png = render(VALID_SVG, 64).unwrap();
        assert!(!png.is_empty());
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_render_from_tree_produces_png() {
        let tree = parse_svg(VALID_SVG).unwrap();
        let png = render_from_tree(&tree, 64).unwrap();
        assert!(!png.is_empty());
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_parse_svg_valid() {
        let result = parse_svg(VALID_SVG);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_svg_invalid() {
        let result = parse_svg("this is not SVG");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_size_rejects_zero() {
        let result = render(VALID_SVG, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("greater than 0"));
    }

    #[test]
    fn test_validate_size_rejects_too_large() {
        let result = render(VALID_SVG, 8193);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_validate_size_accepts_valid() {
        let result = render(VALID_SVG, 256);
        assert!(result.is_ok());
    }
}
