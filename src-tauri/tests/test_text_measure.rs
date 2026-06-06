use icon_studio_lib::engine::text_measure::measure_text_width;

#[test]
fn test_estimate_fallback_precision() {
    let w = measure_text_width("Hello", "sans-serif", 24.0, 400);
    assert!(w > 0.0, "Width should be positive (exact or fallback)");
}

#[test]
fn test_chinese_text() {
    let w = measure_text_width("图标", "Microsoft YaHei", 24.0, 400);
    assert!(w > 0.0, "Chinese text width should be positive");
}

#[test]
fn test_empty_string() {
    let w = measure_text_width("", "sans-serif", 16.0, 400);
    assert!(w >= 0.0, "Empty string width should be >= 0");
}

#[test]
fn test_long_string() {
    let long = "A".repeat(1000);
    let w = measure_text_width(&long, "sans-serif", 12.0, 400);
    assert!(w > 100.0, "Long string should have substantial width");
}

#[test]
fn test_nonexistent_font_falls_back() {
    let w = measure_text_width("Test", "ThisFontDoesNotExist12345", 20.0, 400);
    assert!(w > 0.0, "Should fall back to estimation for missing fonts");
}

#[test]
fn test_cache_consistency() {
    let w1 = measure_text_width("CacheCheck", "Arial", 18.0, 500);
    let w2 = measure_text_width("CacheCheck", "Arial", 18.0, 500);
    assert!((w1 - w2).abs() < f32::EPSILON, "Cached result should be identical");
}

#[test]
fn test_different_sizes_produce_different_widths() {
    let w_small = measure_text_width("Size", "sans-serif", 10.0, 400);
    let w_large = measure_text_width("Size", "sans-serif", 40.0, 400);
    assert!(w_large > w_small, "Larger font size should produce wider text");
}

#[test]
fn test_different_texts_produce_different_widths() {
    let w_short = measure_text_width("Hi", "sans-serif", 16.0, 400);
    let w_long = measure_text_width("Hello World!", "sans-serif", 16.0, 400);
    assert!(w_long > w_short, "Longer text should produce wider result");
}
