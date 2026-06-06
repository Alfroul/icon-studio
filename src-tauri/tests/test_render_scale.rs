use icon_studio_lib::engine::{builder, renderer};
use icon_studio_lib::model::*;
use icon_studio_lib::model::shapes::ShapeType;

fn decode_pixel(png_bytes: &[u8], size: u32, x: usize, y: usize) -> (u8, u8, u8, u8) {
    let pixmap = resvg::tiny_skia::Pixmap::decode_png(png_bytes).unwrap();
    assert_eq!(pixmap.width(), size);
    assert_eq!(pixmap.height(), size);
    let idx = (y * size as usize + x) * 4;
    let data = pixmap.data();
    (data[idx], data[idx + 1], data[idx + 2], data[idx + 3])
}

fn assert_pixel(svg: &str, size: u32, x: usize, y: usize, expected_rgb: (u8, u8, u8)) {
    let png_bytes = renderer::render(svg, size).unwrap();
    let (r, g, b, a) = decode_pixel(&png_bytes, size, x, y);
    eprintln!("  {}px pixel({},{}) = rgba({},{},{},{})", size, x, y, r, g, b, a);
    assert!(a > 200, "pixel at ({},{}) {}px should be opaque, got a={}", x, y, size, a);
    assert_eq!(r, expected_rgb.0, "red at ({},{}) {}px", x, y, size);
    assert_eq!(g, expected_rgb.1, "green at ({},{}) {}px", x, y, size);
    assert_eq!(b, expected_rgb.2, "blue at ({},{}) {}px", x, y, size);
}

fn app_icon_project_1024() -> IconProject {
    let mut project = IconProject::default();
    project.canvas.width = 1024;
    project.canvas.height = 1024;
    project.canvas.background = "#6C63FF".into();
    project.canvas.corner_radius = 20;
    project.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id: "shape-1".into(),
            x: 312.0, y: 312.0, width: 400.0, height: 400.0,
            opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
        shape_type: ShapeType::Circle,
        fill: "#FFFFFF".into(), stroke: None, stroke_width: 0.0,
        border_radius: 0.0, stroke_dasharray: None,
        gradient: None,
    }));
    project
}

#[test]
fn test_simple_rect_scales_correctly() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512"><rect width="512" height="512" fill="#3366FF"/></svg>"##;
    for size in [16u32, 32, 64, 128, 256, 512, 1024] {
        assert_pixel(svg, size, size as usize / 2, size as usize / 2, (0x33, 0x66, 0xFF));
    }
}

#[test]
fn test_clippath_rounded_rect_scales_correctly() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024"><defs><clipPath id="app-icon-clip"><rect width="1024" height="1024" rx="204.80" ry="204.80"/></clipPath></defs><g clip-path="url(#app-icon-clip)"><rect width="1024" height="1024" fill="#6C63FF"/><circle cx="512" cy="512" r="300" fill="#FFFFFF"/></g></svg>"##;
    for size in [64u32, 128, 256, 512, 1024] {
        assert_pixel(svg, size, size as usize / 2, size as usize / 2, (0xFF, 0xFF, 0xFF));
    }
}

#[test]
fn test_full_app_icon_scales_correctly() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024"><defs><clipPath id="app-icon-clip"><rect width="1024" height="1024" rx="204.80" ry="204.80"/></clipPath><linearGradient id="gradient-0" gradientTransform="rotate(135, 0.5, 0.5)"><stop offset="0.0000%" stop-color="#667EEA"/><stop offset="100.0000%" stop-color="#764BA2"/></linearGradient><filter id="shadow-1"><feDropShadow dx="0" dy="8" stdDeviation="16" flood-color="#000000" flood-opacity="0.3000"/></filter></defs><g clip-path="url(#app-icon-clip)"><rect width="1024" height="1024" fill="url(#gradient-0)"/><g transform="translate(256.00, 256.00) scale(21.3333, 21.3333)" filter="url(#shadow-1)" fill="#FFFFFF"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></g></g></svg>"##;
    for size in [64u32, 128, 256, 512, 1024] {
        let png_bytes = renderer::render(svg, size).unwrap();
        let pixmap = resvg::tiny_skia::Pixmap::decode_png(&png_bytes).unwrap();
        let c = size as usize / 2;
        let idx = (c * size as usize + c) * 4;
        let data = pixmap.data();
        let a = data[idx + 3];
        eprintln!("  {}px center alpha={}", size, a);
        assert!(a > 100, "center at {}px should have content, got a={}", size, a);
    }
}

#[test]
fn test_real_builder_1024_app_icon_all_sizes() {
    let project = app_icon_project_1024();
    let svg = builder::build(&project).expect("build");
    eprintln!("SVG:\n{}", svg);
    assert!(svg.contains("clipPath"), "app icon should have clipPath");
    assert!(svg.contains("viewBox"), "SVG should have viewBox");

    for size in [16u32, 32, 64, 128, 256, 512, 1024] {
        let png_bytes = renderer::render(&svg, size).unwrap();
        let pixmap = resvg::tiny_skia::Pixmap::decode_png(&png_bytes).unwrap();
        assert_eq!(pixmap.width(), size);
        assert_eq!(pixmap.height(), size);
        let c = size as usize / 2;
        let idx = (c * size as usize + c) * 4;
        let data = pixmap.data();
        let (r, g, b, a) = (data[idx], data[idx+1], data[idx+2], data[idx+3]);
        eprintln!("  {}px center: rgba({},{},{},{})", size, r, g, b, a);
        assert!(a > 200, "center at {}px should be opaque, got a={}", size, a);
    }
}

#[test]
fn test_real_builder_512_app_icon_all_sizes() {
    let mut project = IconProject::default();
    project.canvas.background = "#1a1a2e".into();
    project.canvas.corner_radius = 15;
    project.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id: "shape-1".into(),
            x: 106.0, y: 106.0, width: 300.0, height: 300.0,
            opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
        shape_type: ShapeType::Star,
        fill: "#FFD700".into(), stroke: None, stroke_width: 0.0,
        border_radius: 0.0, stroke_dasharray: None,
        gradient: None,
    }));

    let svg = builder::build(&project).expect("build");
    for size in [16u32, 32, 64, 128, 256, 512, 1024] {
        let png_bytes = renderer::render(&svg, size).unwrap();
        let pixmap = resvg::tiny_skia::Pixmap::decode_png(&png_bytes).unwrap();
        let c = size as usize / 2;
        let idx = (c * size as usize + c) * 4;
        let data = pixmap.data();
        let a = data[idx + 3];
        eprintln!("  {}px center alpha={}", size, a);
        assert!(a > 200, "center at {}px should be opaque (background), got a={}", size, a);
    }
}

#[test]
fn test_real_builder_gradient_shadow_all_sizes() {
    let mut project = IconProject::default();
    project.canvas.width = 1024;
    project.canvas.height = 1024;
    project.canvas.background = "#0f0f23".into();
    project.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id: "shape-1".into(),
            x: 262.0, y: 262.0, width: 500.0, height: 500.0,
            opacity: 1.0, rotation: 0.0,
            shadows: vec![Shadow {
                color: "#00000040".into(),
                blur: 16.0,
                offset_x: 0.0,
                offset_y: 8.0,
                inset: false,
            }],
            animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
        shape_type: ShapeType::RoundedRect,
        fill: "#667EEA".into(), stroke: None, stroke_width: 0.0,
        border_radius: 40.0, stroke_dasharray: None,
        gradient: Some(Gradient {
            gradient_type: GradientKind::Linear,
            colors: vec!["#667EEA".into(), "#764BA2".into()],
            angle: 135.0,
            stops: vec![],
        }),
    }));

    let svg = builder::build(&project).expect("build");
    assert!(svg.contains("linearGradient"));
    assert!(svg.contains("feDropShadow"));

    for size in [64u32, 128, 256, 512, 1024] {
        let png_bytes = renderer::render(&svg, size).unwrap();
        let pixmap = resvg::tiny_skia::Pixmap::decode_png(&png_bytes).unwrap();
        let c = size as usize / 2;
        let idx = (c * size as usize + c) * 4;
        let data = pixmap.data();
        let a = data[idx + 3];
        eprintln!("  {}px center alpha={}", size, a);
        assert!(a > 200, "center at {}px should be opaque (gradient shape), got a={}", size, a);
    }
}
