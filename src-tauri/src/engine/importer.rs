use crate::engine::renderer::get_options;
use crate::error::AppError;
use crate::model::shapes::ShapeType;
use crate::model::*;
use base64::Engine;
use usvg::tiny_skia_path::PathSegment;

/// Context for collecting imported elements with independent ID allocation.
#[allow(dead_code)]
struct ImportContext {
    elements: Vec<Element>,
    next_shape_id: u32,
    next_text_id: u32,
    next_icon_id: u32,
    next_image_id: u32,
    next_path_id: u32,
}

impl ImportContext {
    fn new() -> Self {
        Self {
            elements: Vec::new(),
            next_shape_id: 0,
            next_text_id: 0,
            next_icon_id: 0,
            next_image_id: 0,
            next_path_id: 0,
        }
    }

    fn alloc_id(&mut self, prefix: &str) -> String {
        match prefix {
            "shape" => { self.next_shape_id += 1; format!("shape-{}", self.next_shape_id) }
            "text" => { self.next_text_id += 1; format!("text-{}", self.next_text_id) }
            "image" => { self.next_image_id += 1; format!("image-{}", self.next_image_id) }
            "path" => { self.next_path_id += 1; format!("path-{}", self.next_path_id) }
            _ => { self.next_shape_id += 1; format!("elem-{}", self.next_shape_id) }
        }
    }
}

/// Parses an SVG string and converts it into an `IconProject`.
///
/// Recognizes basic shapes (rect, circle, rounded-rect, hexagon, star, diamond),
/// text, and embedded images. Unsupported SVG features are skipped with a warning.
pub fn import_svg(svg_str: &str) -> Result<IconProject, AppError> {
    let opts = get_options();

    let tree = usvg::Tree::from_str(svg_str, opts)
        .map_err(|e| AppError::RenderError(format!("Failed to parse SVG: {}", e)))?;

    let size = tree.size();
    let canvas = Canvas {
        width: if size.width() >= 1.0 { size.width().round() as u32 } else { 512 },
        height: if size.height() >= 1.0 { size.height().round() as u32 } else { 512 },
        ..Default::default()
    };

    let mut ctx = ImportContext::new();
    walk_group(tree.root(), &mut ctx, 1.0);

    let mut project = IconProject {
        canvas,
        ..Default::default()
    };

    for elem in &mut ctx.elements {
        let prefix = match elem {
            Element::Shape(_) => "shape",
            Element::Text(_) => "text",
            Element::Icon(_) => "icon",
            Element::Image(_) => "image",
            Element::Path(_) => "path",
            Element::Group(_) => "group",
            Element::Symbol(_) => "symbol",
        };
        let new_id = project.alloc_element_id(prefix);
        elem.common_mut().id = new_id;
    }
    project.elements = ctx.elements;

    detect_background(&mut project);

    Ok(project)
}

pub fn import_svg_as_elements(svg_str: &str) -> Result<Vec<Element>, AppError> {
    let opts = get_options();
    let tree = usvg::Tree::from_str(svg_str, opts)
        .map_err(|e| AppError::RenderError(format!("Failed to parse SVG: {}", e)))?;
    let mut ctx = ImportContext::new();
    walk_group(tree.root(), &mut ctx, 1.0);
    Ok(ctx.elements)
}

fn walk_group(group: &usvg::Group, ctx: &mut ImportContext, parent_opacity: f64) {
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => {
                let group_opacity = g.opacity().get() as f64;
                walk_group(g, ctx, parent_opacity * group_opacity);
            }
            usvg::Node::Path(p) => process_path(p, ctx, parent_opacity),
            usvg::Node::Text(t) => process_text(t, ctx, parent_opacity),
            usvg::Node::Image(i) => process_image(i, ctx, parent_opacity),
        }
    }
}

fn process_path(path: &usvg::Path, ctx: &mut ImportContext, parent_opacity: f64) {
    if !path.is_visible() {
        return;
    }

    let abs_bbox = path.abs_bounding_box();
    let data = path.data();

    // Compute the bounding box of raw path data in its local coordinate system.
    // usvg may pre-apply some transforms (e.g. scale) to path data while keeping
    // others (e.g. translate) separate.  Normalising by the LOCAL bbox origin and
    // using the ABS bbox for x/y/width/height lets the builder's scale factor
    // (width/natural_width) compensate for any baked-in transforms.
    let local = local_path_bbox(data);

    let id = ctx.alloc_id("path");
    let element = PathElement {
        common: CommonProps {
            id,
            x: abs_bbox.x() as f64,
            y: abs_bbox.y() as f64,
            width: abs_bbox.width() as f64,
            height: abs_bbox.height() as f64,
            opacity: parent_opacity * path.fill().map(|f| f.opacity().get() as f64).unwrap_or(1.0),
            rotation: extract_rotation_from_transform(path.abs_transform()),
            shadows: vec![],
            animation: None,
            blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
        d: normalised_d(data, local.min_x, local.min_y),
        fill: path.fill().map(extract_fill_color).unwrap_or_else(|| "none".to_string()),
        stroke: path.stroke().and_then(extract_stroke_color).unwrap_or_else(|| "none".to_string()),
        stroke_width: path.stroke().map(|s| s.width().get() as f64).unwrap_or(0.0),
        stroke_dasharray: None,
        natural_width: local.width,
        natural_height: local.height,
        boolean_source: None,
    };
    ctx.elements.push(Element::Path(element));
}

struct LocalBBox {
    min_x: f64,
    min_y: f64,
    width: f64,
    height: f64,
}

fn local_path_bbox(data: &usvg::tiny_skia_path::Path) -> LocalBBox {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    let mut has_points = false;

    for seg in data.segments() {
        let points: Vec<usvg::tiny_skia_path::Point> = match seg {
            PathSegment::MoveTo(p) => { has_points = true; vec![p] }
            PathSegment::LineTo(p) => { has_points = true; vec![p] }
            PathSegment::CubicTo(p1, p2, p3) => { has_points = true; vec![p1, p2, p3] }
            PathSegment::QuadTo(p1, p2) => { has_points = true; vec![p1, p2] }
            PathSegment::Close => vec![],
        };
        for p in &points {
            let x = p.x as f64;
            let y = p.y as f64;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }

    if !has_points {
        return LocalBBox {
            min_x: 0.0,
            min_y: 0.0,
            width: 0.0,
            height: 0.0,
        };
    }

    LocalBBox {
        min_x,
        min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}

fn process_text(text: &usvg::Text, ctx: &mut ImportContext, parent_opacity: f64) {
    // Skip if all spans are invisible (usvg::Text has no is_visible(), check spans instead)
    let all_invisible = text.chunks().iter().all(|chunk| {
        chunk.spans().iter().all(|span| !span.is_visible())
    });
    if all_invisible {
        return;
    }

    let bbox = text.abs_bounding_box();

    for chunk in text.chunks() {
        let content = chunk.text().to_string();
        if content.is_empty() {
            continue;
        }

        let (font_family, font_size, font_weight) = chunk
            .spans()
            .first()
            .map(|span| {
                let font = span.font();
                let family = font
                    .families()
                    .first()
                    .map(|f| f.to_string().trim_matches('"').to_string())
                    .unwrap_or_else(|| "sans-serif".to_string());
                let size = span.font_size().get() as f64;
                let weight = match font.weight() {
                    w if w <= 300 => "300",
                    w if w <= 400 => "normal",
                    w if w <= 600 => "500",
                    _ => "bold",
                };
                (family, size, weight.to_string())
            })
            .unwrap_or_else(|| ("sans-serif".to_string(), 16.0, "normal".to_string()));

        let fill = chunk
            .spans()
            .first()
            .and_then(|s| s.fill())
            .map(extract_fill_color)
            .unwrap_or_else(|| "#000000".to_string());

        let gradient = chunk
            .spans()
            .first()
            .and_then(|s| s.fill())
            .and_then(|f| extract_gradient(f.paint()));

        let letter_spacing = chunk
            .spans()
            .first()
            .map(|s| s.letter_spacing() as f64)
            .unwrap_or(0.0);

        let fill_opacity = chunk
            .spans()
            .first()
            .and_then(|s| s.fill())
            .map(|f| f.opacity().get() as f64)
            .unwrap_or(1.0);
        let opacity = parent_opacity * fill_opacity;

        let rotation = extract_rotation_from_transform(text.abs_transform());

        let id = ctx.alloc_id("text");

        let element = TextElement {
            common: CommonProps {
                id,
                x: bbox.x() as f64,
                y: bbox.y() as f64,
                width: bbox.width() as f64,
                height: bbox.height() as f64,
                opacity,
                rotation,
                shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            content,
            fill,
            font_family,
            font_size,
            font_weight,
            letter_spacing,
            stroke: None,
            stroke_width: 0.0,
            gradient,
        };

        ctx.elements.push(Element::Text(element));
    }
}

fn process_image(image: &usvg::Image, ctx: &mut ImportContext, parent_opacity: f64) {
    if !image.is_visible() {
        return;
    }

    let bbox = image.abs_bounding_box();
    let data = match encode_image_data(image.kind()) {
        Some(d) => d,
        None => return,
    };

    let id = ctx.alloc_id("image");

    let element = ImageElement {
        common: CommonProps {
            id,
            x: bbox.x() as f64,
            y: bbox.y() as f64,
            width: bbox.width() as f64,
            height: bbox.height() as f64,
            opacity: parent_opacity,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
        data,
    };

    ctx.elements.push(Element::Image(element));
}

/// Classifies a normalised usvg path into a `ShapeType`.
///
/// usvg converts all SVG basic shapes (`<rect>`, `<circle>`, `<polygon>`, etc.)
/// into raw `<path>` data. Classification is done by counting segment types:
///
/// | line_tos | cubic_tos | Shape       |
/// |----------|-----------|-------------|
/// | 0        | 4         | Circle      |
/// | 3        | 0         | Rect/Diamond|
/// | 4        | 4         | RoundedRect |
/// | 5        | 0         | Hexagon     |
/// | 9        | 0         | Star        |
#[allow(dead_code)]
fn classify_path(path: &usvg::tiny_skia_path::Path) -> Option<ShapeType> {
    let segments: Vec<PathSegment> = path.segments().collect();

    let mut move_tos = 0u32;
    let mut line_tos = 0u32;
    let mut cubic_tos = 0u32;

    for seg in &segments {
        match seg {
            PathSegment::MoveTo(_) => move_tos += 1,
            PathSegment::LineTo(_) => line_tos += 1,
            PathSegment::CubicTo(_, _, _) => cubic_tos += 1,
            PathSegment::QuadTo(_, _) | PathSegment::Close => {}
        }
    }

    if move_tos != 1 {
        return None;
    }

    if line_tos == 0 && cubic_tos == 4 {
        return Some(ShapeType::Circle);
    }

    if line_tos == 3 && cubic_tos == 0 {
        return if is_axis_aligned_rect(&segments) {
            Some(ShapeType::Rect)
        } else {
            Some(ShapeType::Diamond)
        };
    }

    if line_tos == 4 && cubic_tos == 4 {
        return Some(ShapeType::RoundedRect);
    }

    if line_tos == 5 && cubic_tos == 0 {
        return Some(ShapeType::Hexagon);
    }

    if line_tos == 9 && cubic_tos == 0 {
        return Some(ShapeType::Star);
    }

    if line_tos >= 2 && (2..=6).contains(&cubic_tos) && is_shield_shape(&segments) {
        return Some(ShapeType::Shield);
    }

    if line_tos == 2 && cubic_tos == 0 {
        return Some(ShapeType::Triangle);
    }

    if line_tos == 4 && cubic_tos == 0 {
        return Some(ShapeType::Pentagon);
    }

    if line_tos == 7 && cubic_tos == 0 {
        return Some(ShapeType::Octagon);
    }

    if line_tos == 0 && cubic_tos == 2 {
        return Some(ShapeType::Heart);
    }

    None
}

#[allow(dead_code)]
fn is_axis_aligned_rect(segments: &[PathSegment]) -> bool {
    let mut points: Vec<usvg::tiny_skia_path::Point> = Vec::new();
    for seg in segments {
        match seg {
            PathSegment::MoveTo(p) | PathSegment::LineTo(p) => points.push(*p),
            PathSegment::Close => {}
            _ => return false,
        }
    }

    if points.len() < 3 {
        return false;
    }

    let tolerance = 0.5_f32;
    for i in 0..points.len() {
        let p1 = points[i];
        let p2 = points[(i + 1) % points.len()];
        let dx = (p1.x - p2.x).abs();
        let dy = (p1.y - p2.y).abs();
        if dx > tolerance && dy > tolerance {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
fn is_shield_shape(segments: &[PathSegment]) -> bool {
    let points: Vec<(f32, f32)> = segments
        .iter()
        .filter_map(|seg| match seg {
            PathSegment::MoveTo(p) => Some((p.x, p.y)),
            PathSegment::LineTo(p) => Some((p.x, p.y)),
            PathSegment::CubicTo(_, _, p) => Some((p.x, p.y)),
            _ => None,
        })
        .collect();

    if points.len() < 4 {
        return false;
    }

    let xs: Vec<f32> = points.iter().map(|p| p.0).collect();
    let ys: Vec<f32> = points.iter().map(|p| p.1).collect();

    let min_x = xs.iter().cloned().fold(f32::MAX, f32::min);
    let max_x = xs.iter().cloned().fold(f32::MIN, f32::max);
    let min_y = ys.iter().cloned().fold(f32::MAX, f32::min);
    let max_y = ys.iter().cloned().fold(f32::MIN, f32::max);

    let center_x = (min_x + max_x) / 2.0;
    let width = max_x - min_x;
    let height = max_y - min_y;

    if width < 1.0 || height < 1.0 {
        return false;
    }

    let max_y_point = ys
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| points[i]);

    if let Some((tip_x, tip_y)) = max_y_point {
        let x_center_ratio = (tip_x - center_x).abs() / width;
        let y_bottom_ratio = (tip_y - min_y) / height;
        x_center_ratio < 0.15 && y_bottom_ratio > 0.80
    } else {
        false
    }
}

fn extract_rotation_from_transform(t: usvg::Transform) -> f64 {
    let sx = t.sx as f64;
    let ky = t.ky as f64;

    // Isolate rotation from scale: normalize by column vector magnitude
    let scale_x = (sx * sx + ky * ky).sqrt();
    if scale_x.abs() < f64::EPSILON {
        return 0.0;
    }

    f64::atan2(ky / scale_x, sx / scale_x).to_degrees()
}

fn color_to_hex(color: usvg::Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.red, color.green, color.blue)
}

fn extract_fill_color(fill: &usvg::Fill) -> String {
    match fill.paint() {
        usvg::Paint::Color(c) => color_to_hex(*c),
        usvg::Paint::LinearGradient(lg) => lg
            .stops()
            .first()
            .map(|s| color_to_hex(s.color()))
            .unwrap_or_else(|| "#000000".to_string()),
        usvg::Paint::RadialGradient(rg) => rg
            .stops()
            .first()
            .map(|s| color_to_hex(s.color()))
            .unwrap_or_else(|| "#000000".to_string()),
        usvg::Paint::Pattern(_) => "#000000".to_string(),
    }
}

fn extract_stroke_color(stroke: &usvg::Stroke) -> Option<String> {
    match stroke.paint() {
        usvg::Paint::Color(c) => Some(color_to_hex(*c)),
        _ => None,
    }
}

fn extract_gradient(paint: &usvg::Paint) -> Option<Gradient> {
    match paint {
        usvg::Paint::LinearGradient(lg) => {
            let colors: Vec<String> = lg.stops().iter().map(|s| color_to_hex(s.color())).collect();
            let stops: Vec<f64> = lg.stops().iter().map(|s| s.offset().get() as f64).collect();
            let angle = (lg.y2() - lg.y1()).atan2(lg.x2() - lg.x1()).to_degrees() as f64;

            Some(Gradient {
                gradient_type: GradientKind::Linear,
                colors,
                angle,
                stops,
            })
        }
        usvg::Paint::RadialGradient(rg) => {
            let colors: Vec<String> = rg.stops().iter().map(|s| color_to_hex(s.color())).collect();
            let stops: Vec<f64> = rg.stops().iter().map(|s| s.offset().get() as f64).collect();

            Some(Gradient {
                gradient_type: GradientKind::Radial,
                colors,
                angle: 0.0,
                stops,
            })
        }
        _ => None,
    }
}

fn encode_image_data(kind: &usvg::ImageKind) -> Option<String> {
    match kind {
        usvg::ImageKind::PNG(data) => {
            let b64 = base64::engine::general_purpose::STANDARD.encode(data.as_ref());
            Some(format!("data:image/png;base64,{}", b64))
        }
        usvg::ImageKind::JPEG(data) => {
            let b64 = base64::engine::general_purpose::STANDARD.encode(data.as_ref());
            Some(format!("data:image/jpeg;base64,{}", b64))
        }
        usvg::ImageKind::GIF(data) => {
            let b64 = base64::engine::general_purpose::STANDARD.encode(data.as_ref());
            Some(format!("data:image/gif;base64,{}", b64))
        }
        usvg::ImageKind::WEBP(data) => {
            let b64 = base64::engine::general_purpose::STANDARD.encode(data.as_ref());
            Some(format!("data:image/webp;base64,{}", b64))
        }
        usvg::ImageKind::SVG(tree) => {
            let sz = tree.size();
            let w = sz.width().round() as u32;
            let h = sz.height().round() as u32;
            if w == 0 || h == 0 {
                return None;
            }
            let transform = resvg::tiny_skia::Transform::identity();
            let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)?;
            resvg::render(tree, transform, &mut pixmap.as_mut());
            let png_bytes = pixmap.encode_png().ok()?;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
            Some(format!("data:image/png;base64,{}", b64))
        }
    }
}

fn detect_background(project: &mut IconProject) {
    if project.elements.is_empty() {
        return;
    }

    let cw = project.canvas.width as f64;
    let ch = project.canvas.height as f64;
    let tol = 2.0;

    let common = project.elements[0].common();
    if common.x.abs() < tol
        && common.y.abs() < tol
        && (common.width - cw).abs() < tol
        && (common.height - ch).abs() < tol
    {
        let bg = match &project.elements[0] {
            Element::Shape(s) => s.fill.clone(),
            Element::Path(p) => p.fill.clone(),
            _ => return,
        };
        if bg == "none" || bg.is_empty() {
            return;
        }
        project.elements.remove(0);
        project.canvas.background = bg;
    }
}

/// Build a `d` attribute string with all coordinates shifted so the path
/// starts at (0, 0) relative to its bounding-box origin.
fn normalised_d(data: &usvg::tiny_skia_path::Path, ox: f64, oy: f64) -> String {
    data.segments()
        .map(|seg| match seg {
            PathSegment::MoveTo(p) => format!("M {:.2} {:.2}", p.x as f64 - ox, p.y as f64 - oy),
            PathSegment::LineTo(p) => format!("L {:.2} {:.2}", p.x as f64 - ox, p.y as f64 - oy),
            PathSegment::CubicTo(p1, p2, p3) => format!(
                "C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
                p1.x as f64 - ox, p1.y as f64 - oy,
                p2.x as f64 - ox, p2.y as f64 - oy,
                p3.x as f64 - ox, p3.y as f64 - oy,
            ),
            PathSegment::QuadTo(p1, p2) => format!(
                "Q {:.2} {:.2} {:.2} {:.2}",
                p1.x as f64 - ox, p1.y as f64 - oy,
                p2.x as f64 - ox, p2.y as f64 - oy,
            ),
            PathSegment::Close => "Z".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}
