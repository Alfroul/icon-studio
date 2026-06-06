use crate::model::shapes::ShapeType;
use crate::model::{
    Canvas, CommonProps, Element, ExportConfig, IconElement, IconProject, ShapeElement, TextElement,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateMeta {
    pub name: String,
    pub description: String,
    pub is_builtin: bool,
}

/// Returns the 8 built-in template definitions
pub fn built_in_templates() -> Vec<(TemplateMeta, IconProject)> {
    vec![
        // 1. App Icon - Square
        (
            TemplateMeta {
                name: "App Icon - Square".into(),
                description: "Rounded square background with centered icon".into(),
                is_builtin: true,
            },
            app_icon_square(),
        ),
        // 2. App Icon - Circle
        (
            TemplateMeta {
                name: "App Icon - Circle".into(),
                description: "Circle background with centered icon".into(),
                is_builtin: true,
            },
            app_icon_circle(),
        ),
        // 3. Letter Logo
        (
            TemplateMeta {
                name: "Letter Logo".into(),
                description: "Hexagon background with centered letter".into(),
                is_builtin: true,
            },
            letter_logo(),
        ),
        // 4. Dual Letter Logo
        (
            TemplateMeta {
                name: "Dual Letter Logo".into(),
                description: "Two contrasting letters, no background".into(),
                is_builtin: true,
            },
            dual_letter_logo(),
        ),
        // 5. Text Logo
        (
            TemplateMeta {
                name: "Text Logo".into(),
                description: "Bold text centered on transparent canvas".into(),
                is_builtin: true,
            },
            text_logo(),
        ),
        // 6. Pure Icon
        (
            TemplateMeta {
                name: "Pure Icon".into(),
                description: "Single icon on transparent background".into(),
                is_builtin: true,
            },
            pure_icon(),
        ),
        // 7. Shield Badge
        (
            TemplateMeta {
                name: "Shield Badge".into(),
                description: "Shield shape with centered text".into(),
                is_builtin: true,
            },
            shield_badge(),
        ),
        // 8. Minimal Square
        (
            TemplateMeta {
                name: "Minimal Square".into(),
                description: "Dark square with centered initials".into(),
                is_builtin: true,
            },
            minimal_square(),
        ),
    ]
}


fn default_canvas() -> Canvas {
    Canvas {
        width: 512,
        height: 512,
        background: "transparent".into(),
        corner_radius: 0,
        background_gradient: None,
    }
}

fn default_exports() -> ExportConfig {
    ExportConfig::default()
}

/// 1. App Icon - Square: rounded-rect bg #4F46E5, cornerRadius=20%, centered star icon
fn app_icon_square() -> IconProject {
    IconProject {
        schema_version: "1.0".into(),
        canvas: Canvas {
            width: 512,
            height: 512,
            background: "transparent".into(),
            corner_radius: 20,
            background_gradient: None,
        },
        elements: vec![
            Element::Shape(ShapeElement {
                common: CommonProps {
                    id: "shape-1".into(),
                    x: 0.0,
                    y: 0.0,
                    width: 512.0,
                    height: 512.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
                shape_type: ShapeType::RoundedRect,
                fill: "#4F46E5".into(),
                stroke: None,
                stroke_width: 0.0,
                border_radius: 0.0,
                stroke_dasharray: None,
                gradient: None,
            }),
            Element::Icon(IconElement {
                common: CommonProps {
                    id: "icon-1".into(),
                    x: 156.0,
                    y: 156.0,
                    width: 200.0,
                    height: 200.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
                name: "star".into(),
                fill: "#FFFFFF".into(),
                stroke: None,
                stroke_width: 0.0,
                gradient: None,
            }),
        ],
        exports: default_exports(),
        templates: HashMap::new(),
        pages: vec![],
        active_page_index: 0,
        next_element_id: 3,
        version: 0,
        symbols: HashMap::new(),
        adaptive: None,
        brand_kits: Vec::new(),
        custom_style_presets: Vec::new(),
    }
}

/// 2. App Icon - Circle: circle bg #059669, centered heart icon
fn app_icon_circle() -> IconProject {
    let size = 400.0;
    let offset = (512.0 - size) / 2.0;
    let icon_size = 200.0;
    let icon_offset = (512.0 - icon_size) / 2.0;
    IconProject {
        schema_version: "1.0".into(),
        canvas: default_canvas(),
        elements: vec![
            Element::Shape(ShapeElement {
                common: CommonProps {
                    id: "shape-1".into(),
                    x: offset,
                    y: offset,
                    width: size,
                    height: size,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
                shape_type: ShapeType::Circle,
                fill: "#059669".into(),
                stroke: None,
                stroke_width: 0.0,
                border_radius: 0.0,
                stroke_dasharray: None,
                gradient: None,
            }),
            Element::Icon(IconElement {
                common: CommonProps {
                    id: "icon-1".into(),
                    x: icon_offset,
                    y: icon_offset,
                    width: icon_size,
                    height: icon_size,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
                name: "heart".into(),
                fill: "#FFFFFF".into(),
                stroke: None,
                stroke_width: 0.0,
                gradient: None,
            }),
        ],
        exports: default_exports(),
        templates: HashMap::new(),
        pages: vec![],
        active_page_index: 0,
        next_element_id: 3,
        version: 0,
        symbols: HashMap::new(),
        adaptive: None,
        brand_kits: Vec::new(),
        custom_style_presets: Vec::new(),
    }
}

/// 3. Letter Logo: hexagon bg #7C3AED (400 at center), centered "A"
fn letter_logo() -> IconProject {
    let bg_size = 400.0;
    let bg_offset = (512.0 - bg_size) / 2.0;
    IconProject {
        schema_version: "1.0".into(),
        canvas: default_canvas(),
        elements: vec![
            Element::Shape(ShapeElement {
                common: CommonProps {
                    id: "shape-1".into(),
                    x: bg_offset,
                    y: bg_offset,
                    width: bg_size,
                    height: bg_size,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
                shape_type: ShapeType::Hexagon,
                fill: "#7C3AED".into(),
                stroke: None,
                stroke_width: 0.0,
                border_radius: 0.0,
                stroke_dasharray: None,
                gradient: None,
            }),
            Element::Text(TextElement {
                common: CommonProps {
                    id: "text-1".into(),
                    x: 156.0,
                    y: 156.0,
                    width: 200.0,
                    height: 200.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
                content: "A".into(),
                fill: "#FFFFFF".into(),
                font_family: "sans-serif".into(),
                font_size: 200.0,
                font_weight: "bold".into(),
                letter_spacing: 0.0,
                stroke: None,
                stroke_width: 0.0,
                gradient: None,
            }),
        ],
        exports: default_exports(),
        templates: HashMap::new(),
        pages: vec![],
        active_page_index: 0,
        next_element_id: 3,
        version: 0,
        symbols: HashMap::new(),
        adaptive: None,
        brand_kits: Vec::new(),
        custom_style_presets: Vec::new(),
    }
}

/// 4. Dual Letter Logo: "A" blue at left, "B" red at right
fn dual_letter_logo() -> IconProject {
    let elem_w = 200.0;
    let elem_h = 200.0;
    let gap = 12.0;
    let total_w = elem_w * 2.0 + gap;
    let start_x = (512.0 - total_w) / 2.0;
    let y = (512.0 - elem_h) / 2.0;

    IconProject {
        schema_version: "1.0".into(),
        canvas: default_canvas(),
        elements: vec![
            Element::Text(TextElement {
                common: CommonProps {
                    id: "text-1".into(),
                    x: start_x,
                    y,
                    width: elem_w,
                    height: elem_h,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
                content: "A".into(),
                fill: "#3B82F6".into(),
                font_family: "sans-serif".into(),
                font_size: 120.0,
                font_weight: "bold".into(),
                letter_spacing: 0.0,
                stroke: None,
                stroke_width: 0.0,
                gradient: None,
            }),
            Element::Text(TextElement {
                common: CommonProps {
                    id: "text-2".into(),
                    x: start_x + elem_w + gap,
                    y,
                    width: elem_w,
                    height: elem_h,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
                content: "B".into(),
                fill: "#EF4444".into(),
                font_family: "sans-serif".into(),
                font_size: 120.0,
                font_weight: "bold".into(),
                letter_spacing: 0.0,
                stroke: None,
                stroke_width: 0.0,
                gradient: None,
            }),
        ],
        exports: default_exports(),
        templates: HashMap::new(),
        pages: vec![],
        active_page_index: 0,
        next_element_id: 3,
        version: 0,
        symbols: HashMap::new(),
        adaptive: None,
        brand_kits: Vec::new(),
        custom_style_presets: Vec::new(),
    }
}

/// 5. Text Logo: "STUDIO" centered
fn text_logo() -> IconProject {
    IconProject {
        schema_version: "1.0".into(),
        canvas: default_canvas(),
        elements: vec![Element::Text(TextElement {
            common: CommonProps {
                id: "text-1".into(),
                x: 56.0,
                y: 206.0,
                width: 400.0,
                height: 100.0,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            content: "STUDIO".into(),
            fill: "#1E1E1E".into(),
            font_family: "sans-serif".into(),
            font_size: 80.0,
            font_weight: "bold".into(),
            letter_spacing: 0.0,
            stroke: None,
            stroke_width: 0.0,
            gradient: None,
        })],
        exports: default_exports(),
        templates: HashMap::new(),
        pages: vec![],
        active_page_index: 0,
        next_element_id: 2,
        version: 0,
        symbols: HashMap::new(),
        adaptive: None,
        brand_kits: Vec::new(),
        custom_style_presets: Vec::new(),
    }
}

/// 6. Pure Icon: centered "zap" in amber
fn pure_icon() -> IconProject {
    let icon_size = 300.0;
    let offset = (512.0 - icon_size) / 2.0;
    IconProject {
        schema_version: "1.0".into(),
        canvas: default_canvas(),
        elements: vec![Element::Icon(IconElement {
            common: CommonProps {
                id: "icon-1".into(),
                x: offset,
                y: offset,
                width: icon_size,
                height: icon_size,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            name: "zap".into(),
            fill: "#F59E0B".into(),
            stroke: None,
            stroke_width: 0.0,
            gradient: None,
        })],
        exports: default_exports(),
        templates: HashMap::new(),
        pages: vec![],
        active_page_index: 0,
        next_element_id: 2,
        version: 0,
        symbols: HashMap::new(),
        adaptive: None,
        brand_kits: Vec::new(),
        custom_style_presets: Vec::new(),
    }
}

/// 7. Shield Badge: shield #DC2626, centered "VIP"
fn shield_badge() -> IconProject {
    let bg_size = 400.0;
    let bg_offset = (512.0 - bg_size) / 2.0;
    let text_w = 200.0;
    let text_h = 100.0;
    let text_offset_x = (512.0 - text_w) / 2.0;
    let text_offset_y = (512.0 - text_h) / 2.0;

    IconProject {
        schema_version: "1.0".into(),
        canvas: default_canvas(),
        elements: vec![
            Element::Shape(ShapeElement {
                common: CommonProps {
                    id: "shape-1".into(),
                    x: bg_offset,
                    y: bg_offset,
                    width: bg_size,
                    height: bg_size,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
                shape_type: ShapeType::Shield,
                fill: "#DC2626".into(),
                stroke: None,
                stroke_width: 0.0,
                border_radius: 0.0,
                stroke_dasharray: None,
                gradient: None,
            }),
            Element::Text(TextElement {
                common: CommonProps {
                    id: "text-1".into(),
                    x: text_offset_x,
                    y: text_offset_y,
                    width: text_w,
                    height: text_h,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
                content: "VIP".into(),
                fill: "#FFFFFF".into(),
                font_family: "sans-serif".into(),
                font_size: 60.0,
                font_weight: "bold".into(),
                letter_spacing: 0.0,
                stroke: None,
                stroke_width: 0.0,
                gradient: None,
            }),
        ],
        exports: default_exports(),
        templates: HashMap::new(),
        pages: vec![],
        active_page_index: 0,
        next_element_id: 3,
        version: 0,
        symbols: HashMap::new(),
        adaptive: None,
        brand_kits: Vec::new(),
        custom_style_presets: Vec::new(),
    }
}

/// 8. Minimal Square: dark rect full-canvas, centered "IS"
fn minimal_square() -> IconProject {
    let text_w = 200.0;
    let text_h = 200.0;
    let text_offset_x = (512.0 - text_w) / 2.0;
    let text_offset_y = (512.0 - text_h) / 2.0;

    IconProject {
        schema_version: "1.0".into(),
        canvas: default_canvas(),
        elements: vec![
            Element::Shape(ShapeElement {
                common: CommonProps {
                    id: "shape-1".into(),
                    x: 0.0,
                    y: 0.0,
                    width: 512.0,
                    height: 512.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
                shape_type: ShapeType::Rect,
                fill: "#111827".into(),
                stroke: None,
                stroke_width: 0.0,
                border_radius: 0.0,
                stroke_dasharray: None,
                gradient: None,
            }),
            Element::Text(TextElement {
                common: CommonProps {
                    id: "text-1".into(),
                    x: text_offset_x,
                    y: text_offset_y,
                    width: text_w,
                    height: text_h,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
                content: "IS".into(),
                fill: "#FFFFFF".into(),
                font_family: "sans-serif".into(),
                font_size: 140.0,
                font_weight: "bold".into(),
                letter_spacing: 0.0,
                stroke: None,
                stroke_width: 0.0,
                gradient: None,
            }),
        ],
        exports: default_exports(),
        templates: HashMap::new(),
        pages: vec![],
        active_page_index: 0,
        next_element_id: 3,
        version: 0,
        symbols: HashMap::new(),
        adaptive: None,
        brand_kits: Vec::new(),
        custom_style_presets: Vec::new(),
    }
}


fn template_dir() -> PathBuf {
    let base = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join("iconstudio").join("templates")
}

pub fn save_template(name: &str, project: &IconProject) -> Result<(), String> {
    validate_template_name(name)?;
    let dir = template_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let path = dir.join(format!("{}.json", name));
    let tpl = project.as_template();
    let json = serde_json::to_string_pretty(&tpl).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn load_template(name: &str) -> Result<IconProject, String> {
    validate_template_name(name)?;
    let path = template_dir().join(format!("{}.json", name));
    let content = fs::read_to_string(&path).map_err(|e| format!("Template '{}' not found: {}", name, e))?;
    let mut project: IconProject = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    project.recalc_next_element_id();
    Ok(project)
}

pub fn list_user_templates() -> Vec<TemplateMeta> {
    let dir = template_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let mut results = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    results.push(TemplateMeta {
                        name: name.to_string(),
                        description: "User template".into(),
                        is_builtin: false,
                    });
                }
            }
        }
    }
    results
}

pub fn delete_template(name: &str) -> Result<(), String> {
    validate_template_name(name)?;
    let path = template_dir().join(format!("{}.json", name));
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())
    } else {
        Err(format!("Template '{}' not found", name))
    }
}

fn validate_template_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Template name cannot be empty".into());
    }
    if name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.contains(char::is_control)
        || name.contains(':')
        || name.contains('*')
        || name.contains('?')
        || name.contains('"')
        || name.contains('<')
        || name.contains('>')
        || name.contains('|')
    {
        return Err(format!("Invalid template name: '{}'", name));
    }
    Ok(())
}
