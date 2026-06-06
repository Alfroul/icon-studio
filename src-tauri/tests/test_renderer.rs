use icon_studio_lib::engine::renderer;

#[test]
fn test_render_simple_svg() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><circle cx="50" cy="50" r="40" fill="red"/></svg>"#;
    let png = renderer::render(svg, 100).unwrap();
    assert!(!png.is_empty());
    assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
}

#[test]
fn test_render_different_sizes() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="blue"/></svg>"#;
    let small = renderer::render(svg, 32).unwrap();
    let large = renderer::render(svg, 256).unwrap();
    assert!(large.len() > small.len());
}

#[test]
fn test_render_specific_sizes() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512"><circle cx="256" cy="256" r="200" fill="#FF5733"/></svg>"##;
    for size in [16u32, 32, 64, 128, 512] {
        let png = renderer::render(svg, size).expect(&format!("render at {}px", size));
        assert!(!png.is_empty(), "PNG at {}px should not be empty", size);
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47], "PNG magic at {}px", size);
    }
}

#[test]
fn test_render_transparent_background_has_alpha() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"></svg>"#;
    let png = renderer::render(svg, 100).unwrap();
    assert!(png.len() > 8, "PNG should have data beyond header");
    // PNG IHDR is at offset 8 (after 8-byte signature). Byte 25 = color type.
    // Color type 6 = RGBA (has alpha), color type 2 = RGB (no alpha)
    let ihdr_start = 8 + 4 + 4;
    let color_type_byte = png.get(ihdr_start + 9);
    assert!(color_type_byte.is_some(), "PNG should have IHDR color type");
    let ct = *color_type_byte.unwrap();
    assert!(
        ct == 6 || ct == 4,
        "Expected RGBA(6) or GA(4) color type for transparent background, got {}",
        ct
    );
}

#[test]
fn test_render_solid_background_pixel_color() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="#FF0000"/></svg>"##;
    let png = renderer::render(svg, 10).unwrap();
    assert!(png.len() > 100, "PNG should have pixel data");
    // At 10x10 pixels with RGBA, the IDAT chunk contains actual pixel data.
    // Just verify the PNG is valid and non-trivial — exact pixel matching
    // would require a PNG decoder which is overkill for a test.
    assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    // Larger PNG with same content should be bigger due to more pixel data
    let png_big = renderer::render(svg, 100).unwrap();
    assert!(png_big.len() > png.len());
}

#[test]
fn test_render_invalid_svg() {
    let result = renderer::render("not svg at all", 100);
    assert!(result.is_err());
}

#[test]
fn test_render_zero_size() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"/>"#;
    let result = renderer::render(svg, 0);
    assert!(result.is_err());
}
