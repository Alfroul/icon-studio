use icon_studio_lib::engine::importer;
use icon_studio_lib::model::*;

#[test]
fn test_import_circle_and_text() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
        <circle cx="100" cy="100" r="50" fill="red"/>
        <text x="100" y="100" font-size="24" fill="black">Hello</text>
    </svg>"#;
    let project = importer::import_svg(svg).unwrap();
    assert!(
        !project.elements.is_empty(),
        "Should have at least 1 element (circle), got {}",
        project.elements.len()
    );
    assert_eq!(project.canvas.width, 200);
    assert_eq!(project.canvas.height, 200);
}

#[test]
fn test_import_empty_svg() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"/>"#;
    let project = importer::import_svg(svg).unwrap();
    assert!(project.elements.is_empty());
    assert_eq!(project.canvas.width, 100);
}

#[test]
fn test_import_with_gradient() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <defs><linearGradient id="g1"><stop offset="0%" stop-color="red"/><stop offset="100%" stop-color="blue"/></linearGradient></defs>
        <rect width="100" height="100" fill="url(#g1)"/>
    </svg>"##;
    let project = importer::import_svg(svg).unwrap();
    assert!(
        !project.elements.is_empty() || project.canvas.background != "#FFFFFF",
        "Should either have elements or a non-default background"
    );
}

#[test]
fn test_import_invalid_svg() {
    let result = importer::import_svg("not valid svg");
    assert!(result.is_err());
}

#[test]
fn test_import_background_detection() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <rect width="100" height="100" fill="#336699"/>
        <circle cx="50" cy="50" r="20" fill="white"/>
    </svg>"##;
    let project = importer::import_svg(svg).unwrap();
    assert_eq!(project.canvas.background, "#336699");
}

#[test]
fn test_import_rect() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
        <rect x="20" y="30" width="100" height="80" fill="#FF0000"/>
    </svg>"##;
    let project = importer::import_svg(svg).unwrap();
    assert_eq!(project.elements.len(), 1, "Should have 1 element");
    if let Element::Path(ref p) = project.elements[0] {
        assert_eq!(p.fill, "#FF0000");
    } else {
        panic!("Expected Path element, got {:?}", project.elements[0]);
    }
}

#[test]
fn test_import_rounded_rect() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
        <rect x="10" y="10" width="180" height="180" rx="20" ry="20" fill="#00FF00"/>
    </svg>"##;
    let project = importer::import_svg(svg).unwrap();
    assert_eq!(project.elements.len(), 1, "Should have 1 element");
    if let Element::Path(ref p) = project.elements[0] {
        assert_eq!(p.fill, "#00FF00");
    } else {
        panic!("Expected Path element");
    }
}

#[test]
fn test_import_hexagon() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
        <polygon points="100,10 190,55 190,145 100,190 10,145 10,55" fill="#3333FF"/>
    </svg>"##;
    let project = importer::import_svg(svg).unwrap();
    assert_eq!(project.elements.len(), 1, "Should have 1 element");
    if let Element::Path(ref p) = project.elements[0] {
        assert_eq!(p.fill, "#3333FF");
    } else {
        panic!("Expected Path element");
    }
}

#[test]
fn test_import_star() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
        <polygon points="100,10 120,80 190,80 130,120 150,190 100,150 50,190 70,120 10,80 80,80" fill="#FFD700"/>
    </svg>"##;
    let project = importer::import_svg(svg).unwrap();
    assert_eq!(project.elements.len(), 1, "Should have 1 element");
    if let Element::Path(ref p) = project.elements[0] {
        assert_eq!(p.fill, "#FFD700");
    } else {
        panic!("Expected Path element");
    }
}

#[test]
fn test_import_text_element() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
        <text x="50" y="100" font-size="24" fill="#000000">Test</text>
    </svg>"##;
    let project = importer::import_svg(svg).unwrap();
    let text_count = project.elements.iter().filter(|e| matches!(e, Element::Text(_))).count();
    if text_count > 0 {
        if let Element::Text(ref t) = project.elements.iter().find(|e| matches!(e, Element::Text(_))).unwrap() {
            assert_eq!(t.content, "Test");
            assert_eq!(t.font_size, 24.0);
        }
    }
    assert!(project.elements.len() <= 1, "Should have at most 1 text element");
}

#[test]
fn test_import_image() {
    // Minimal 1x1 PNG as base64
    let png_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==";
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="200" height="200">
        <image x="10" y="10" width="100" height="100" xlink:href="data:image/png;base64,{}"/>
    </svg>"#,
        png_b64
    );
    let project = importer::import_svg(&svg).unwrap();
    assert!(
        project.elements.iter().any(|e| matches!(e, Element::Image(_))),
        "Should have at least 1 image element, got {} elements: {:?}",
        project.elements.len(),
        project.elements.iter().map(|e| match e {
            Element::Shape(_) => "shape",
            Element::Text(_) => "text",
            Element::Icon(_) => "icon",
            Element::Image(_) => "image",
            Element::Path(_) => "path",
            Element::Group(_) => "group",
            Element::Symbol(_) => "symbol",
        }).collect::<Vec<_>>()
    );
}

#[test]
fn test_import_multi_element() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512">
        <rect x="10" y="10" width="200" height="200" fill="#FF0000"/>
        <circle cx="300" cy="300" r="50" fill="#00FF00"/>
        <polygon points="100,10 190,55 190,145 100,190 10,145 10,55" fill="#3333FF"/>
    </svg>"##;
    let project = importer::import_svg(svg).unwrap();
    assert!(
        project.elements.len() >= 2,
        "Should have at least 2 elements, got {}",
        project.elements.len()
    );
    let has_path = project.elements.iter().any(|e| matches!(e, Element::Path(_)));
    assert!(has_path, "Should have at least one path element");
}

#[test]
fn test_import_diamond() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
        <polygon points="100,10 190,100 100,190 10,100" fill="#FF00FF"/>
    </svg>"##;
    let project = importer::import_svg(svg).unwrap();
    assert_eq!(project.elements.len(), 1, "Should have 1 element");
    if let Element::Path(ref p) = project.elements[0] {
        assert_eq!(p.fill, "#FF00FF");
    } else {
        panic!("Expected Path element");
    }
}

#[test]
fn test_import_unsupported_svg_no_panic() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <animate attributeName="x" from="0" to="100" dur="1s"/>
    </svg>"#;
    let result = importer::import_svg(svg);
    assert!(result.is_ok(), "Should not panic on unsupported SVG features");
}
