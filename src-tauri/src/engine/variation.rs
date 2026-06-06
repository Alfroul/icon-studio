use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::Element;

use crate::model::IconProject;

/// A single transformation step in a variation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Transform {
    Recolor {
        target: String,
        replacement: String,
    },
    Background {
        color: String,
    },
    CornerRadius {
        value: u32,
    },
    Scale {
        factor: f64,
    },
    Opacity {
        element_type: Option<String>,
        value: f64,
    },
}

/// A named set of transforms producing one variation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Variation {
    pub name: String,
    pub transforms: Vec<Transform>,
}

/// Full configuration for batch variation generation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VariationConfig {
    pub variations: Vec<Variation>,
    pub output_dir: String,
    pub naming: String,
}

/// Generate all variations of a project according to the config.
/// Returns (variation_name, mutated_project) pairs. The original project is never modified.
pub fn generate_variations(
    project: &IconProject,
    config: &VariationConfig,
) -> Vec<(String, IconProject)> {
    config
        .variations
        .iter()
        .map(|variation| {
            let mut clone = project.clone();
            for transform in &variation.transforms {
                apply_transform(&mut clone, transform);
            }
            clone.bump_version();
            (variation.name.clone(), clone)
        })
        .collect()
}

/// Apply a single transform to a project in place.
pub fn apply_transform(project: &mut IconProject, transform: &Transform) {
    match transform {
        Transform::Recolor { target, replacement } => {
            let target_lower = target.to_lowercase();
            for elem in project.active_elements_mut() {
                recolor_element(elem, &target_lower, replacement);
            }
            // Also recolor canvas background if it matches
            if project.active_canvas().background.to_lowercase() == target_lower {
                project.active_canvas_mut().background = replacement.clone();
            }
        }
        Transform::Background { color } => {
            project.active_canvas_mut().background = color.clone();
        }
        Transform::CornerRadius { value } => {
            project.active_canvas_mut().corner_radius = *value;
        }
        Transform::Scale { factor } => {
            if factor.is_nan() || factor.is_infinite() || *factor <= 0.0 || *factor > 1000.0 {
                return;
            }
            let f = *factor;
            let new_w = (project.active_canvas().width as f64 * f).round() as u32;
            let new_h = (project.active_canvas().height as f64 * f).round() as u32;
            if new_w == 0 || new_h == 0 || new_w > 16384 || new_h > 16384 {
                return;
            }
            project.active_canvas_mut().width = new_w;
            project.active_canvas_mut().height = new_h;
            for elem in project.active_elements_mut() {
                scale_element(elem, f);
            }
        }
        Transform::Opacity { element_type, value } => {
            for elem in project.active_elements_mut() {
                let type_matches = match element_type {
                    Some(t) => element_type_str(elem) == t.as_str(),
                    None => true,
                };
                if type_matches {
                    elem.common_mut().opacity = *value;
                }
            }
        }
    }
}

fn element_type_str(elem: &Element) -> &str {
    match elem {
        Element::Shape(_) => "shape",
        Element::Text(_) => "text",
        Element::Icon(_) => "icon",
        Element::Image(_) => "image",
        Element::Path(_) => "path",
        Element::Group(_) => "group",
        Element::Symbol(_) => "symbol",
    }
}

fn recolor_element(elem: &mut Element, target_lower: &str, replacement: &str) {
    match elem {
        Element::Shape(e) => {
            if e.fill.to_lowercase() == *target_lower {
                e.fill = replacement.to_string();
            }
            if let Some(ref stroke) = e.stroke {
                if stroke.to_lowercase() == *target_lower {
                    e.stroke = Some(replacement.to_string());
                }
            }
            if let Some(ref grad) = e.gradient {
                let mut new_grad = grad.clone();
                let changed: Vec<String> = new_grad
                    .colors
                    .iter()
                    .map(|c| {
                        if c.to_lowercase() == *target_lower {
                            replacement.to_string()
                        } else {
                            c.clone()
                        }
                    })
                    .collect();
                new_grad.colors = changed;
                e.gradient = Some(new_grad);
            }
        }
        Element::Text(e) => {
            if e.fill.to_lowercase() == *target_lower {
                e.fill = replacement.to_string();
            }
            if let Some(ref stroke) = e.stroke {
                if stroke.to_lowercase() == *target_lower {
                    e.stroke = Some(replacement.to_string());
                }
            }
            if let Some(ref grad) = e.gradient {
                let mut new_grad = grad.clone();
                let changed: Vec<String> = new_grad
                    .colors
                    .iter()
                    .map(|c| {
                        if c.to_lowercase() == *target_lower {
                            replacement.to_string()
                        } else {
                            c.clone()
                        }
                    })
                    .collect();
                new_grad.colors = changed;
                e.gradient = Some(new_grad);
            }
        }
        Element::Icon(e) => {
            if e.fill.to_lowercase() == *target_lower {
                e.fill = replacement.to_string();
            }
            if let Some(ref stroke) = e.stroke {
                if stroke.to_lowercase() == *target_lower {
                    e.stroke = Some(replacement.to_string());
                }
            }
            if let Some(ref grad) = e.gradient {
                let mut new_grad = grad.clone();
                let changed: Vec<String> = new_grad
                    .colors
                    .iter()
                    .map(|c| {
                        if c.to_lowercase() == *target_lower {
                            replacement.to_string()
                        } else {
                            c.clone()
                        }
                    })
                    .collect();
                new_grad.colors = changed;
                e.gradient = Some(new_grad);
            }
        }
        Element::Path(e) => {
            if e.fill.to_lowercase() == *target_lower {
                e.fill = replacement.to_string();
            }
            if e.stroke.to_lowercase() == *target_lower {
                e.stroke = replacement.to_string();
            }
        }
        Element::Group(g) => {
            for child in &mut g.children {
                recolor_element(child, target_lower, replacement);
            }
        }
        Element::Image(_) => {}
        Element::Symbol(_) => {}
    }
}

fn scale_element(elem: &mut Element, factor: f64) {
    let c = elem.common_mut();
    c.x *= factor;
    c.y *= factor;
    c.width *= factor;
    c.height *= factor;
    // Recursively scale group children
    if let Element::Group(g) = elem {
        for child in &mut g.children {
            scale_element(child, factor);
        }
    }
}

// ---------------------------------------------------------------------------
// Preset variation templates
// ---------------------------------------------------------------------------

/// Dark mode: flip background to dark, lighten text/icon fills.
pub fn dark_mode_variation() -> VariationConfig {
    VariationConfig {
        variations: vec![Variation {
            name: "dark".into(),
            transforms: vec![
                Transform::Background {
                    color: "#1A1A2E".into(),
                },
                Transform::Recolor {
                    target: "#000000".into(),
                    replacement: "#FFFFFF".into(),
                },
                Transform::Recolor {
                    target: "#333333".into(),
                    replacement: "#E0E0E0".into(),
                },
                Transform::Recolor {
                    target: "#FFFFFF".into(),
                    replacement: "#1A1A2E".into(),
                },
            ],
        }],
        output_dir: "./variations".into(),
        naming: "{name}-{variation}".into(),
    }
}

/// Multiple size variants.
pub fn size_variations(sizes: &[u32]) -> VariationConfig {
    let base: u32 = 512;
    let variations: Vec<Variation> = sizes
        .iter()
        .map(|&s| {
            let factor = s as f64 / base as f64;
            Variation {
                name: format!("{}x{}", s, s),
                transforms: vec![Transform::Scale { factor }],
            }
        })
        .collect();

    VariationConfig {
        variations,
        output_dir: "./variations".into(),
        naming: "{name}-{variation}".into(),
    }
}

/// Multiple color palette variants.
pub fn color_palette_variations(palettes: &[(String, String)]) -> VariationConfig {
    let variations: Vec<Variation> = palettes
        .iter()
        .map(|(name, color)| Variation {
            name: name.clone(),
            transforms: vec![
                Transform::Background {
                    color: color.clone(),
                },
            ],
        })
        .collect();

    VariationConfig {
        variations,
        output_dir: "./variations".into(),
        naming: "{name}-{variation}".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn make_test_project() -> IconProject {
        let mut p = IconProject::default();
        p.elements.push(Element::Shape(ShapeElement {
            common: CommonProps {
                id: "shape-1".into(),
                x: 50.0,
                y: 50.0,
                width: 200.0,
                height: 200.0,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
                mask_element_id: None,
                locked: false,
                visible: true,
                svg_filter: None,
            },
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF5733".into(),
            stroke: Some("#000000".into()),
            stroke_width: 2.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        }));
        p.elements.push(Element::Text(TextElement {
            common: CommonProps {
                id: "text-1".into(),
                x: 100.0,
                y: 300.0,
                width: 200.0,
                height: 40.0,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
                mask_element_id: None,
                locked: false,
                visible: true,
                svg_filter: None,
            },
            content: "Hello".into(),
            fill: "#FFFFFF".into(),
            font_family: "Arial".into(),
            font_size: 32.0,
            font_weight: "bold".into(),
            letter_spacing: 0.0,
            stroke: None,
            stroke_width: 0.0,
            gradient: None,
        }));
        p
    }

    #[test]
    fn test_recolor_transform() {
        let mut project = make_test_project();
        apply_transform(&mut project, &Transform::Recolor {
            target: "#FF5733".into(),
            replacement: "#00FF00".into(),
        });

        if let Element::Shape(s) = &project.elements[0] {
            assert_eq!(s.fill, "#00FF00");
        } else {
            panic!("Expected shape element");
        }
    }

    #[test]
    fn test_recolor_case_insensitive() {
        let mut project = make_test_project();
        apply_transform(&mut project, &Transform::Recolor {
            target: "#ff5733".into(),
            replacement: "#00FF00".into(),
        });

        if let Element::Shape(s) = &project.elements[0] {
            assert_eq!(s.fill, "#00FF00");
        } else {
            panic!("Expected shape element");
        }
    }

    #[test]
    fn test_recolor_stroke() {
        let mut project = make_test_project();
        apply_transform(&mut project, &Transform::Recolor {
            target: "#000000".into(),
            replacement: "#111111".into(),
        });

        if let Element::Shape(s) = &project.elements[0] {
            assert_eq!(s.stroke.as_deref(), Some("#111111"));
        } else {
            panic!("Expected shape element");
        }
    }

    #[test]
    fn test_background_transform() {
        let mut project = make_test_project();
        apply_transform(&mut project, &Transform::Background {
            color: "#2C3E50".into(),
        });
        assert_eq!(project.canvas.background, "#2C3E50");
    }

    #[test]
    fn test_corner_radius_transform() {
        let mut project = make_test_project();
        apply_transform(&mut project, &Transform::CornerRadius { value: 20 });
        assert_eq!(project.canvas.corner_radius, 20);
    }

    #[test]
    fn test_scale_transform() {
        let mut project = make_test_project();
        let orig_w = project.canvas.width;
        let orig_h = project.canvas.height;
        let orig_x = project.elements[0].common().x;
        let orig_ew = project.elements[0].common().width;

        apply_transform(&mut project, &Transform::Scale { factor: 0.5 });

        assert_eq!(project.canvas.width, (orig_w as f64 * 0.5).round() as u32);
        assert_eq!(project.canvas.height, (orig_h as f64 * 0.5).round() as u32);
        assert!((project.elements[0].common().x - orig_x * 0.5).abs() < f64::EPSILON);
        assert!((project.elements[0].common().width - orig_ew * 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_opacity_transform_all() {
        let mut project = make_test_project();
        apply_transform(&mut project, &Transform::Opacity {
            element_type: None,
            value: 0.5,
        });
        assert_eq!(project.elements[0].common().opacity, 0.5);
        assert_eq!(project.elements[1].common().opacity, 0.5);
    }

    #[test]
    fn test_opacity_transform_by_type() {
        let mut project = make_test_project();
        apply_transform(&mut project, &Transform::Opacity {
            element_type: Some("text".into()),
            value: 0.3,
        });
        assert_eq!(project.elements[0].common().opacity, 1.0); // shape unchanged
        assert_eq!(project.elements[1].common().opacity, 0.3); // text changed
    }

    #[test]
    fn test_multiple_transforms_sequential() {
        let mut project = make_test_project();
        apply_transform(&mut project, &Transform::Background {
            color: "#000000".into(),
        });
        apply_transform(&mut project, &Transform::CornerRadius { value: 15 });
        apply_transform(&mut project, &Transform::Opacity {
            element_type: None,
            value: 0.8,
        });

        assert_eq!(project.canvas.background, "#000000");
        assert_eq!(project.canvas.corner_radius, 15);
        assert_eq!(project.elements[0].common().opacity, 0.8);
    }

    #[test]
    fn test_generate_variations_does_not_modify_original() {
        let project = make_test_project();
        let original_bg = project.canvas.background.clone();
        let original_opacity = project.elements[0].common().opacity;

        let config = VariationConfig {
            variations: vec![Variation {
                name: "test".into(),
                transforms: vec![
                    Transform::Background {
                        color: "#111111".into(),
                    },
                    Transform::Opacity {
                        element_type: None,
                        value: 0.5,
                    },
                ],
            }],
            output_dir: "./out".into(),
            naming: "{name}-{variation}".into(),
        };

        let results = generate_variations(&project, &config);

        // Original is untouched
        assert_eq!(project.canvas.background, original_bg);
        assert_eq!(project.elements[0].common().opacity, original_opacity);

        // Variation is mutated
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "test");
        assert_eq!(results[0].1.canvas.background, "#111111");
        assert_eq!(results[0].1.elements[0].common().opacity, 0.5);
    }

    #[test]
    fn test_variation_config_json_deserialize() {
        let json = r##"{
            "variations": [
                {
                    "name": "dark",
                    "transforms": [
                        { "type": "background", "color": "#1A1A2E" },
                        { "type": "opacity", "element_type": "shape", "value": 0.8 }
                    ]
                },
                {
                    "name": "small",
                    "transforms": [
                        { "type": "scale", "factor": 0.5 }
                    ]
                }
            ],
            "output_dir": "./variations",
            "naming": "{name}-{variation}"
        }"##;

        let config: VariationConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.variations.len(), 2);
        assert_eq!(config.variations[0].name, "dark");
        assert_eq!(config.variations[1].name, "small");
        assert_eq!(config.output_dir, "./variations");

        match &config.variations[0].transforms[0] {
            Transform::Background { color } => assert_eq!(color, "#1A1A2E"),
            _ => panic!("Expected Background transform"),
        }
        match &config.variations[0].transforms[1] {
            Transform::Opacity { element_type, value } => {
                assert_eq!(element_type.as_deref(), Some("shape"));
                assert!((value - 0.8).abs() < f64::EPSILON);
            }
            _ => panic!("Expected Opacity transform"),
        }
    }

    #[test]
    fn test_recolor_includes_canvas_background() {
        let mut project = make_test_project();
        project.canvas.background = "#FF5733".into();

        apply_transform(&mut project, &Transform::Recolor {
            target: "#FF5733".into(),
            replacement: "#00FF00".into(),
        });

        assert_eq!(project.canvas.background, "#00FF00");
    }

    #[test]
    fn test_scale_group_children() {
        let mut project = make_test_project();
        let group = GroupElement {
            common: CommonProps {
                id: "group-1".into(),
                x: 100.0,
                y: 100.0,
                width: 200.0,
                height: 200.0,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
                mask_element_id: None,
                locked: false,
                visible: true,
                svg_filter: None,
            },
            children: vec![Element::Shape(ShapeElement {
                common: CommonProps {
                    id: "shape-2".into(),
                    x: 10.0,
                    y: 10.0,
                    width: 50.0,
                    height: 50.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                    animation: None,
                    blend_mode: None,
                    clip_element_id: None,
                    mask_element_id: None,
                    locked: false,
                    visible: true,
                    svg_filter: None,
                },
                shape_type: shapes::ShapeType::Circle,
                fill: "#FF0000".into(),
                stroke: None,
                stroke_width: 0.0,
                border_radius: 0.0,
                stroke_dasharray: None,
                gradient: None,
            })],
            expanded: false,
        };
        project.elements.push(Element::Group(group));

        apply_transform(&mut project, &Transform::Scale { factor: 2.0 });

        if let Element::Group(g) = &project.elements[2] {
            let child = &g.children[0];
            assert!((child.common().x - 20.0).abs() < f64::EPSILON);
            assert!((child.common().width - 100.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected group element");
        }
    }

    #[test]
    fn test_dark_mode_preset() {
        let preset = dark_mode_variation();
        assert_eq!(preset.variations.len(), 1);
        assert_eq!(preset.variations[0].name, "dark");
    }

    #[test]
    fn test_size_variations_preset() {
        let preset = size_variations(&[16, 32, 64]);
        assert_eq!(preset.variations.len(), 3);
        assert_eq!(preset.variations[0].name, "16x16");
        assert_eq!(preset.variations[2].name, "64x64");
    }

    #[test]
    fn test_color_palette_preset() {
        let preset = color_palette_variations(&[
            ("blue".into(), "#3B82F6".into()),
            ("green".into(), "#10B981".into()),
        ]);
        assert_eq!(preset.variations.len(), 2);
        assert_eq!(preset.variations[0].name, "blue");
        assert_eq!(preset.variations[1].name, "green");
    }
}
