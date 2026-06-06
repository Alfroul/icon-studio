use palette::{FromColor, Hsl, Srgb};

// ---------------------------------------------------------------------------
// Unified hex color parsing
// ---------------------------------------------------------------------------

/// Parsed hex color with optional alpha.
///
/// Supports `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA` formats.
#[derive(Debug, Clone, Copy)]
pub struct ParsedColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8, // 255 = fully opaque
}

impl ParsedColor {
    /// Parse `#RGB`, `#RGBA`, `#RRGGBB`, or `#RRGGBBAA` hex string.
    /// Returns `None` for invalid inputs.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let h = hex.trim_start_matches('#');
        match h.len() {
            3 => {
                let r = u8::from_str_radix(&h[0..1], 16).ok()?;
                let g = u8::from_str_radix(&h[1..2], 16).ok()?;
                let b = u8::from_str_radix(&h[2..3], 16).ok()?;
                Some(ParsedColor {
                    r: r * 17,
                    g: g * 17,
                    b: b * 17,
                    a: 255,
                })
            }
            4 => {
                let r = u8::from_str_radix(&h[0..1], 16).ok()?;
                let g = u8::from_str_radix(&h[1..2], 16).ok()?;
                let b = u8::from_str_radix(&h[2..3], 16).ok()?;
                let a = u8::from_str_radix(&h[3..4], 16).ok()?;
                Some(ParsedColor {
                    r: r * 17,
                    g: g * 17,
                    b: b * 17,
                    a: a * 17,
                })
            }
            6 => {
                let val = u32::from_str_radix(h, 16).ok()?;
                Some(ParsedColor {
                    r: ((val >> 16) & 0xFF) as u8,
                    g: ((val >> 8) & 0xFF) as u8,
                    b: (val & 0xFF) as u8,
                    a: 255,
                })
            }
            8 => {
                let rgb = u32::from_str_radix(&h[0..6], 16).ok()?;
                let a = u8::from_str_radix(&h[6..8], 16).ok()?;
                Some(ParsedColor {
                    r: ((rgb >> 16) & 0xFF) as u8,
                    g: ((rgb >> 8) & 0xFF) as u8,
                    b: (rgb & 0xFF) as u8,
                    a,
                })
            }
            _ => None,
        }
    }

    /// Convert to `"#RRGGBB"` uppercase string (no alpha).
    pub fn to_rgb_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Get alpha as `0.0`–`1.0` float.
    pub fn alpha_f64(&self) -> f64 {
        self.a as f64 / 255.0
    }
}

/// Normalize a hex color string to uppercase `#RRGGBB` form.
/// Non-hex inputs are returned as-is (unchanged).
pub fn normalize_hex(hex: &str) -> String {
    let h = hex.trim_start_matches('#');
    if h.is_empty() || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return hex.to_string();
    }
    let expanded = match h.len() {
        3 => {
            // #RGB → #RRGGBB
            let chars: Vec<char> = h.chars().collect();
            format!("{}{}{}{}{}{}",
                chars[0], chars[0], chars[1], chars[1], chars[2], chars[2])
        }
        4 => {
            // #RGBA → #RRGGBBAA
            let chars: Vec<char> = h.chars().collect();
            format!("{}{}{}{}{}{}{}{}",
                chars[0], chars[0], chars[1], chars[1], chars[2], chars[2], chars[3], chars[3])
        }
        _ => h.to_string(),
    };
    format!("#{}", expanded.to_uppercase())
}

/// Darken a hex color by a factor (0.0 = black, 1.0 = unchanged).
/// Returns `"#RRGGBB"` string. Invalid inputs are returned as-is.
pub fn darken(hex: &str, factor: f64) -> String {
    match ParsedColor::from_hex(hex) {
        Some(c) => {
            let r = (c.r as f64 * factor) as u8;
            let g = (c.g as f64 * factor) as u8;
            let b = (c.b as f64 * factor) as u8;
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        }
        None => hex.to_string(),
    }
}

/// Split a hex color (possibly with alpha) into a `"#RRGGBB"` color string
/// and an alpha `f64` (0.0–1.0). Invalid inputs return `(hex, 1.0)`.
pub fn split_hex_color_with_alpha(hex: &str) -> (String, f64) {
    match ParsedColor::from_hex(hex) {
        Some(c) => (c.to_rgb_hex(), c.alpha_f64()),
        None => (hex.to_string(), 1.0),
    }
}

// ---------------------------------------------------------------------------
// Palette suggestions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub enum PaletteScheme {
    Complementary,
    Analogous,
    Triadic,
    SplitComplementary,
    Monochromatic,
}

fn parse_hex(hex: &str) -> Result<Srgb, String> {
    let c = ParsedColor::from_hex(hex)
        .ok_or_else(|| format!("Invalid hex color: {}", hex))?;
    Ok(Srgb::new(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0))
}

fn to_hex(color: Srgb) -> String {
    let r = (color.red.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (color.green.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (color.blue.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

pub fn suggest_palette(base_color: &str, scheme: PaletteScheme, count: usize) -> Result<Vec<String>, String> {
    let rgb = parse_hex(base_color)?;
    let hsl: Hsl = Hsl::from_color(rgb);
    let hue_deg = hsl.hue.into_degrees();

    let colors: Vec<Srgb> = match scheme {
        PaletteScheme::Complementary => {
            let mut result = vec![rgb];
            for i in 1..count {
                let t = i as f32 / (count - 1).max(1) as f32;
                let hue = (hue_deg + t * 180.0).rem_euclid(360.0);
                let interp = Hsl::new(hue, hsl.saturation, hsl.lightness);
                result.push(Srgb::from_color(interp));
            }
            result
        }
        PaletteScheme::Analogous => {
            let mut result = Vec::with_capacity(count);
            for i in 0..count {
                let t = if count > 1 { i as f32 / (count - 1) as f32 } else { 0.5 };
                let hue = (hue_deg - 30.0 + t * 60.0).rem_euclid(360.0);
                result.push(Srgb::from_color(Hsl::new(hue, hsl.saturation, hsl.lightness)));
            }
            result
        }
        PaletteScheme::Triadic => {
            let mut result = Vec::with_capacity(count);
            for i in 0..count {
                let offset = (i as f32 / 3.0) * 120.0;
                let hue = (hue_deg + offset).rem_euclid(360.0);
                result.push(Srgb::from_color(Hsl::new(hue, hsl.saturation, hsl.lightness)));
            }
            result
        }
        PaletteScheme::SplitComplementary => {
            let mut result = Vec::with_capacity(count);
            let angles = [0.0_f32, 150.0, 210.0];
            for i in 0..count {
                let base_angle = angles[i % 3];
                let extra_offset = (i as f32 / 3.0).floor() * 10.0;
                let hue = (hue_deg + base_angle + extra_offset).rem_euclid(360.0);
                result.push(Srgb::from_color(Hsl::new(hue, hsl.saturation, hsl.lightness)));
            }
            result
        }
        PaletteScheme::Monochromatic => {
            let mut result = Vec::with_capacity(count);
            for i in 0..count {
                let t = if count > 1 { i as f32 / (count - 1) as f32 } else { 0.5 };
                let lightness = 0.15 + t * 0.7;
                result.push(Srgb::from_color(Hsl::new(hsl.hue, hsl.saturation, lightness)));
            }
            result
        }
    };

    Ok(colors.into_iter().map(to_hex).collect())
}
