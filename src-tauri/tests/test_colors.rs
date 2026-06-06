use icon_studio_lib::colors::{suggest_palette, PaletteScheme};

fn parse_hex_to_rgb(hex: &str) -> (f32, f32, f32) {
    let hex = hex.trim_start_matches('#');
    let val = u32::from_str_radix(hex, 16).unwrap();
    let r = ((val >> 16) & 0xFF) as f32 / 255.0;
    let g = ((val >> 8) & 0xFF) as f32 / 255.0;
    let b = (val & 0xFF) as f32 / 255.0;
    (r, g, b)
}

fn rgb_to_hue(r: f32, g: f32, b: f32) -> f32 {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    if delta < f32::EPSILON {
        return 0.0;
    }
    let hue = if (max - r).abs() < f32::EPSILON {
        ((g - b) / delta) % 6.0
    } else if (max - g).abs() < f32::EPSILON {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    (hue * 60.0 + 360.0) % 360.0
}

fn hue_difference(h1: f32, h2: f32) -> f32 {
    let diff = (h1 - h2).abs();
    diff.min(360.0 - diff)
}

#[test]
fn complementary_returns_requested_count() {
    let result = suggest_palette("#FF0000", PaletteScheme::Complementary, 2).unwrap();
    assert_eq!(result.len(), 2, "Should return exactly 2 colors");
}

#[test]
fn complementary_colors_have_approx_180_hue_difference() {
    let result = suggest_palette("#FF0000", PaletteScheme::Complementary, 2).unwrap();
    let (r1, g1, b1) = parse_hex_to_rgb(&result[0]);
    let (r2, g2, b2) = parse_hex_to_rgb(&result[1]);
    let hue1 = rgb_to_hue(r1, g1, b1);
    let hue2 = rgb_to_hue(r2, g2, b2);
    let diff = hue_difference(hue1, hue2);
    assert!(
        (diff - 180.0).abs() < 5.0,
        "Hue difference should be ~180 degrees, got {diff}"
    );
}

#[test]
fn analogous_returns_specified_count() {
    let result = suggest_palette("#4488FF", PaletteScheme::Analogous, 5).unwrap();
    assert_eq!(result.len(), 5, "Should return exactly 5 colors");
}

#[test]
fn all_colors_are_valid_hex_format() {
    let schemes = [
        PaletteScheme::Complementary,
        PaletteScheme::Analogous,
        PaletteScheme::Triadic,
        PaletteScheme::SplitComplementary,
        PaletteScheme::Monochromatic,
    ];
    for scheme in &schemes {
        let result = suggest_palette("#AABBCC", *scheme, 4).unwrap();
        for color in &result {
            assert!(
                color.starts_with('#') && color.len() == 7,
                "Color '{}' should be in #RRGGBB format",
                color
            );
            let hex_part = &color[1..];
            assert!(
                hex_part.chars().all(|c| c.is_ascii_hexdigit()),
                "Color '{}' should have only hex digits after #",
                color
            );
        }
    }
}

#[test]
fn monochromatic_returns_count_colors() {
    let result = suggest_palette("#4488FF", PaletteScheme::Monochromatic, 4).unwrap();
    assert_eq!(result.len(), 4, "Should return exactly 4 colors");
}

#[test]
fn invalid_hex_returns_error() {
    let result = suggest_palette("not-a-color", PaletteScheme::Complementary, 2);
    assert!(result.is_err(), "Invalid hex color should return an error");
}
