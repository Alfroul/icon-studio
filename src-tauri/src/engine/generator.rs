use crate::colors::{self, PaletteScheme};
use crate::icons;
use crate::model::shapes::ShapeType;
use crate::model::{CommonProps, *};

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next_u32(&mut self) -> u32 {
        // XorShift64
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state >> 32) as u32
    }

    fn next_range(&mut self, min: f64, max: f64) -> f64 {
        min + (self.next_u32() as f64 / u32::MAX as f64) * (max - min)
    }

    fn next_index(&mut self, len: usize) -> usize {
        self.next_u32() as usize % len
    }
}

const SHAPE_TYPES: &[ShapeType] = &[
    ShapeType::Circle,
    ShapeType::RoundedRect,
    ShapeType::Hexagon,
    ShapeType::Star,
    ShapeType::Shield,
    ShapeType::Diamond,
    ShapeType::Triangle,
    ShapeType::Pentagon,
    ShapeType::Octagon,
];

const STYLES: &[&str] = &[
    "minimal",
    "geometric",
    "lettermark",
    "icon",
    "badge",
];

const BASE_COLORS: &[&str] = &[
    "#E74C3C", "#3498DB", "#2ECC71", "#F39C12",
    "#9B59B6", "#1ABC9C", "#E67E22", "#34495E",
    "#E91E63", "#00BCD4", "#FF5722", "#607D8B",
    "#8BC34A", "#FF9800", "#673AB7", "#795548",
];

pub struct GeneratorConfig {
    pub style: Option<String>,
    pub base_color: Option<String>,
    pub text: Option<String>,
    pub icon_name: Option<String>,
    pub size: u32,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            style: None,
            base_color: None,
            text: None,
            icon_name: None,
            size: 512,
        }
    }
}

pub fn generate_random(config: &GeneratorConfig) -> IconProject {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut rng = SimpleRng::new(seed);

    let style = config.style.as_deref().unwrap_or(STYLES[rng.next_index(STYLES.len())]);
    let base_color = config.base_color.as_deref().unwrap_or(BASE_COLORS[rng.next_index(BASE_COLORS.len())]);

    let palette = colors::suggest_palette(
        base_color,
        match rng.next_index(3) {
            0 => PaletteScheme::Complementary,
            1 => PaletteScheme::Analogous,
            _ => PaletteScheme::Triadic,
        },
        4,
    )
    .unwrap_or_else(|_| vec![base_color.to_string(), "#FFFFFF".to_string(), "#000000".to_string()]);

    let size = config.size;
    let mut project = IconProject::default();
    project.canvas.width = size;
    project.canvas.height = size;

    match style {
        "minimal" => generate_minimal(&mut project, &palette, config, &mut rng),
        "geometric" => generate_geometric(&mut project, &palette, config, &mut rng),
        "lettermark" => generate_lettermark(&mut project, &palette, config, &mut rng),
        "icon" => generate_icon_style(&mut project, &palette, config, &mut rng),
        "badge" => generate_badge(&mut project, &palette, config, &mut rng),
        _ => generate_minimal(&mut project, &palette, config, &mut rng),
    }

    project.bump_version();
    project
}

fn generate_minimal(project: &mut IconProject, palette: &[String], config: &GeneratorConfig, rng: &mut SimpleRng) {
    let bg_color = &palette[0];
    project.canvas.background = colors::darken(bg_color, 0.85);

    let center = project.canvas.width as f64 / 2.0;

    let shape_idx = rng.next_index(SHAPE_TYPES.len());
    let shape_size = rng.next_range(0.4, 0.7) * project.canvas.width as f64;

    let id = project.alloc_element_id("shape");
    project.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id,
            x: center - shape_size / 2.0,
            y: center - shape_size / 2.0,
            width: shape_size,
            height: shape_size,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
        shape_type: SHAPE_TYPES[shape_idx].clone(),
        fill: palette[0].clone(),
        stroke: None,
        stroke_width: 0.0,
        border_radius: 0.0,
        stroke_dasharray: None,
        gradient: None,
    }));

    if let Some(ref text) = config.text {
        add_centered_text(project, text, palette, 0.35);
    }
}

fn generate_geometric(project: &mut IconProject, palette: &[String], _config: &GeneratorConfig, rng: &mut SimpleRng) {
    project.canvas.background = palette[0].clone();

    let count = 2 + rng.next_index(4);
    let canvas_size = project.canvas.width as f64;

    for i in 0..count {
        let shape_idx = rng.next_index(SHAPE_TYPES.len());
        let shape_size = rng.next_range(0.15, 0.45) * canvas_size;
        let pos_x = rng.next_range(0.05, 0.8) * canvas_size;
        let pos_y = rng.next_range(0.05, 0.8) * canvas_size;
        let rotation = rng.next_range(0.0, 360.0);

        let id = project.alloc_element_id("shape");
        project.elements.push(Element::Shape(ShapeElement {
            common: CommonProps {
                id,
                x: pos_x,
                y: pos_y,
                width: shape_size,
                height: shape_size,
                opacity: rng.next_range(0.6, 1.0),
                rotation,
                shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
            shape_type: SHAPE_TYPES[shape_idx].clone(),
            fill: palette[if palette.len() > 1 { 1 + i % (palette.len() - 1) } else { 0 }].clone(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_minimal_style() {
        let config = GeneratorConfig {
            style: Some("minimal".to_string()),
            base_color: Some("#3498DB".to_string()),
            text: None,
            icon_name: None,
            size: 512,
        };
        let project = generate_random(&config);
        assert_eq!(project.canvas.width, 512);
        assert_eq!(project.canvas.height, 512);
        assert!(!project.elements.is_empty());
    }

    #[test]
    fn test_generate_geometric_style() {
        let config = GeneratorConfig {
            style: Some("geometric".to_string()),
            base_color: Some("#E74C3C".to_string()),
            text: None,
            icon_name: None,
            size: 512,
        };
        let project = generate_random(&config);
        assert!(!project.elements.is_empty());
        assert!(project.elements.len() >= 2);
    }

    #[test]
    fn test_generate_lettermark_style() {
        let config = GeneratorConfig {
            style: Some("lettermark".to_string()),
            base_color: Some("#2ECC71".to_string()),
            text: Some("A".to_string()),
            icon_name: None,
            size: 512,
        };
        let project = generate_random(&config);
        assert!(!project.elements.is_empty());
        let has_text = project.elements.iter().any(|e| matches!(e, Element::Text(_)));
        assert!(has_text);
    }

    #[test]
    fn test_generate_icon_style() {
        let config = GeneratorConfig {
            style: Some("icon".to_string()),
            base_color: Some("#9B59B6".to_string()),
            text: None,
            icon_name: Some("heart".to_string()),
            size: 512,
        };
        let project = generate_random(&config);
        assert!(!project.elements.is_empty());
        let has_icon = project.elements.iter().any(|e| matches!(e, Element::Icon(_)));
        assert!(has_icon);
    }

    #[test]
    fn test_generate_badge_style() {
        let config = GeneratorConfig {
            style: Some("badge".to_string()),
            base_color: Some("#F39C12".to_string()),
            text: Some("VIP".to_string()),
            icon_name: None,
            size: 512,
        };
        let project = generate_random(&config);
        assert!(!project.elements.is_empty());
        assert_eq!(project.canvas.background, "transparent");
    }

    #[test]
    fn test_generate_custom_size() {
        let config = GeneratorConfig {
            style: Some("minimal".to_string()),
            size: 256,
            ..Default::default()
        };
        let project = generate_random(&config);
        assert_eq!(project.canvas.width, 256);
        assert_eq!(project.canvas.height, 256);
    }

    #[test]
    fn test_generate_has_bumped_version() {
        let config = GeneratorConfig::default();
        let project = generate_random(&config);
        assert!(project.version > 0);
    }

    #[test]
    fn test_generate_unknown_style_falls_back_to_minimal() {
        let config = GeneratorConfig {
            style: Some("nonexistent".to_string()),
            ..Default::default()
        };
        let project = generate_random(&config);
        assert!(!project.elements.is_empty());
    }
}

fn generate_lettermark(project: &mut IconProject, palette: &[String], config: &GeneratorConfig, rng: &mut SimpleRng) {
    project.canvas.background = palette[0].clone();

    let letter = config.text.as_deref().unwrap_or("A").chars().next().unwrap_or('A');
    let text = letter.to_uppercase().to_string();

    let canvas_size = project.canvas.width as f64;
    let bg_shape_size = canvas_size * rng.next_range(0.7, 0.9);

    let shape_idx = rng.next_index(2);
    let shape_type = if shape_idx == 0 { ShapeType::Circle } else { ShapeType::RoundedRect };

    let id = project.alloc_element_id("shape");
    project.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id,
            x: (canvas_size - bg_shape_size) / 2.0,
            y: (canvas_size - bg_shape_size) / 2.0,
            width: bg_shape_size,
            height: bg_shape_size,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
        shape_type,
        fill: "#FFFFFF".to_string(),
        stroke: None,
        stroke_width: 0.0,
        border_radius: 0.0,
        stroke_dasharray: None,
        gradient: None,
    }));

    add_centered_text(project, &text, &palette[1..], 0.4);
}

fn generate_icon_style(project: &mut IconProject, palette: &[String], config: &GeneratorConfig, rng: &mut SimpleRng) {
    let bg_color = &palette[0];
    project.canvas.background = bg_color.clone();

    let canvas_size = project.canvas.width as f64;
    let bg_shape_size = canvas_size * rng.next_range(0.75, 0.9);

    let shape_type = if rng.next_index(2) == 0 {
        ShapeType::Circle
    } else {
        ShapeType::RoundedRect
    };

    let id = project.alloc_element_id("shape");
    project.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id,
            x: (canvas_size - bg_shape_size) / 2.0,
            y: (canvas_size - bg_shape_size) / 2.0,
            width: bg_shape_size,
            height: bg_shape_size,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
        shape_type,
        fill: "#FFFFFF".to_string(),
        stroke: None,
        stroke_width: 0.0,
        border_radius: 0.0,
        stroke_dasharray: None,
        gradient: None,
    }));

    let icon_name_owned;
    let icon_name = if let Some(ref name) = config.icon_name {
        name.as_str()
    } else {
        let all = icons::list_all_icons();
        icon_name_owned = if all.is_empty() {
            "heart".to_string()
        } else {
            all[rng.next_index(all.len())].name.clone()
        };
        icon_name_owned.as_str()
    };

    let icon_size = canvas_size * rng.next_range(0.3, 0.5);
    let id = project.alloc_element_id("icon");
    project.elements.push(Element::Icon(IconElement {
        common: CommonProps {
            id,
            x: (canvas_size - icon_size) / 2.0,
            y: (canvas_size - icon_size) / 2.0,
            width: icon_size,
            height: icon_size,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
        name: icon_name.to_string(),
        fill: palette[0].clone(),
        stroke: None,
        stroke_width: 0.0,
        gradient: None,
    }));
}

fn generate_badge(project: &mut IconProject, palette: &[String], config: &GeneratorConfig, rng: &mut SimpleRng) {
    project.canvas.background = "transparent".to_string();

    let canvas_size = project.canvas.width as f64;
    let shield_size = canvas_size * rng.next_range(0.8, 0.95);

    let id = project.alloc_element_id("shape");
    project.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id,
            x: (canvas_size - shield_size) / 2.0,
            y: (canvas_size - shield_size) / 2.0,
            width: shield_size,
            height: shield_size,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
        shape_type: ShapeType::Shield,
        fill: palette[0].clone(),
        stroke: None,
        stroke_width: 0.0,
        border_radius: 0.0,
        stroke_dasharray: None,
        gradient: None,
    }));

    if let Some(ref text) = config.text {
        add_centered_text(project, text, &["#FFFFFF".to_string()], 0.25);
    }
}

fn add_centered_text(project: &mut IconProject, text: &str, palette: &[String], size_ratio: f64) {
    let canvas_size = project.canvas.width as f64;
    let font_size = canvas_size * size_ratio;
    let text_color = palette.get(1).cloned().unwrap_or_else(|| "#FFFFFF".to_string());

    let width = crate::engine::text_measure::measure_text_width(text, "sans-serif", font_size as f32, 700) as f64;
    let height = font_size * 1.2;

    let id = project.alloc_element_id("text");
    project.elements.push(Element::Text(TextElement {
        common: CommonProps {
            id,
            x: (canvas_size - width) / 2.0,
            y: (canvas_size - height) / 2.0,
            width,
            height,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
        content: text.to_string(),
        fill: text_color,
        font_family: "sans-serif".to_string(),
        font_size,
        font_weight: "bold".to_string(),
        letter_spacing: 0.0,
        stroke: None,
        stroke_width: 0.0,
        gradient: None,
    }));
}

