use crate::model::helpers::find_element_deep_mut;
use crate::model::style_preset::{StyleParams, StyleType, StylePreset};
use crate::model::filter::{FilterType, SvgFilter};
use crate::model::{Element, Gradient, GradientKind, IconProject, Shadow};
use std::collections::HashMap;

/// Apply a style preset to an element, expanding it into atomic effects
/// (shadows, gradients, filters, strokes).
pub fn apply_style_preset(
    project: &mut IconProject,
    element_id: &str,
    preset: &StylePreset,
) -> Result<(), String> {
    let elem = find_element_deep_mut(project.active_elements_mut(), element_id)
        .ok_or_else(|| format!("Element not found: {}", element_id))?;

    match preset.style_type {
        StyleType::Soft3d => apply_soft_3d(elem, &preset.params),
        StyleType::Neumorphism => apply_neumorphism(elem, &preset.params),
        StyleType::Glassmorphism => apply_glassmorphism(elem, &preset.params),
        StyleType::Flat => apply_flat(elem),
    }
}

fn apply_soft_3d(elem: &mut Element, params: &StyleParams) -> Result<(), String> {
    // 1. Offset shadow: depth controls offset, shadow_softness controls blur
    let shadow = Shadow {
        color: "#00000040".to_string(),
        blur: params.shadow_softness,
        offset_x: params.depth * 0.5,
        offset_y: params.depth,
        inset: false,
    };
    elem.common_mut().shadows = vec![shadow];

    // 2. Gradient overlay: light_angle as angle, highlight as bright-end intensity
    let gradient = Gradient {
        gradient_type: GradientKind::Linear,
        colors: vec![
            format!("rgba(255,255,255,{:.2})", params.highlight),
            "rgba(255,255,255,0)".to_string(),
        ],
        angle: params.light_angle,
        stops: vec![],
    };
    match elem {
        Element::Shape(e) => e.gradient = Some(gradient),
        Element::Text(e) => e.gradient = Some(gradient),
        Element::Icon(e) => e.gradient = Some(gradient),
        _ => {}
    }

    // 3. Bright stroke: width=1, opacity 0.2-0.4 based on highlight
    let stroke_alpha = 0.2 + params.highlight * 0.2;
    let stroke_color = format!("rgba(255,255,255,{:.1})", stroke_alpha);
    match elem {
        Element::Shape(e) => {
            e.stroke = Some(stroke_color);
            e.stroke_width = 1.0;
        }
        Element::Text(e) => {
            e.stroke = Some(stroke_color);
            e.stroke_width = 1.0;
        }
        Element::Icon(e) => {
            e.stroke = Some(stroke_color);
            e.stroke_width = 1.0;
        }
        Element::Path(e) => {
            e.stroke = stroke_color;
            e.stroke_width = 1.0;
        }
        _ => {}
    }

    Ok(())
}

fn apply_neumorphism(elem: &mut Element, params: &StyleParams) -> Result<(), String> {
    // Dual shadows: light (top-left) + dark (bottom-right)
    let light_shadow = Shadow {
        color: "#FFFFFF40".to_string(),
        blur: params.shadow_softness,
        offset_x: -params.depth,
        offset_y: -params.depth,
        inset: false,
    };
    let dark_shadow = Shadow {
        color: "#00000030".to_string(),
        blur: params.shadow_softness,
        offset_x: params.depth,
        offset_y: params.depth,
        inset: false,
    };
    elem.common_mut().shadows = vec![light_shadow, dark_shadow];
    Ok(())
}

fn apply_glassmorphism(elem: &mut Element, params: &StyleParams) -> Result<(), String> {
    // 1. Blur filter
    let mut filter_params = HashMap::new();
    filter_params.insert("stdDeviation".to_string(), params.shadow_softness);
    elem.common_mut().svg_filter = Some(SvgFilter {
        filter_type: FilterType::Blur,
        params: filter_params,
    });

    // 2. Semi-transparent fill
    elem.common_mut().opacity = 0.2 + params.highlight * 0.2;

    // 3. White thin stroke
    match elem {
        Element::Shape(e) => {
            e.stroke = Some("#FFFFFF80".to_string());
            e.stroke_width = 1.0;
        }
        Element::Text(e) => {
            e.stroke = Some("#FFFFFF80".to_string());
            e.stroke_width = 1.0;
        }
        Element::Icon(e) => {
            e.stroke = Some("#FFFFFF80".to_string());
            e.stroke_width = 1.0;
        }
        Element::Path(e) => {
            e.stroke = "#FFFFFF80".to_string();
            e.stroke_width = 1.0;
        }
        _ => {}
    }

    Ok(())
}

fn apply_flat(elem: &mut Element) -> Result<(), String> {
    // Clear all effects
    elem.common_mut().shadows.clear();
    elem.common_mut().svg_filter = None;
    elem.common_mut().blend_mode = None;
    elem.common_mut().opacity = 1.0;

    // Clear gradient
    match elem {
        Element::Shape(e) => e.gradient = None,
        Element::Text(e) => e.gradient = None,
        Element::Icon(e) => e.gradient = None,
        _ => {}
    }

    // Clear stroke
    match elem {
        Element::Shape(e) => {
            e.stroke = None;
            e.stroke_width = 0.0;
        }
        Element::Text(e) => {
            e.stroke = None;
            e.stroke_width = 0.0;
        }
        Element::Icon(e) => {
            e.stroke = None;
            e.stroke_width = 0.0;
        }
        Element::Path(e) => {
            e.stroke = String::new();
            e.stroke_width = 0.0;
        }
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CommonProps, ShapeElement};
    use crate::model::shapes::ShapeType;

    fn make_test_project() -> IconProject {
        let mut p = IconProject::default();
        p.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-1".to_string(), 50.0, 50.0, 200.0, 200.0),
            shape_type: ShapeType::Rect,
            fill: "#4488FF".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 16.0,
            stroke_dasharray: None,
            gradient: None,
        }));
        p
    }

    #[test]
    fn test_soft_3d_applies_shadow_gradient_stroke() {
        let mut project = make_test_project();
        let preset = StylePreset {
            style_type: StyleType::Soft3d,
            params: StyleParams::default(),
        };
        apply_style_preset(&mut project, "shape-1", &preset).unwrap();

        let elem = &project.elements[0];
        // Shadow
        assert_eq!(elem.common().shadows.len(), 1);
        assert_eq!(elem.common().shadows[0].blur, 8.0);
        assert_eq!(elem.common().shadows[0].offset_x, 2.5);

        // Gradient
        if let Element::Shape(s) = elem {
            assert!(s.gradient.is_some());
            let grad = s.gradient.as_ref().unwrap();
            assert!(matches!(grad.gradient_type, GradientKind::Linear));
            assert_eq!(grad.angle, 135.0);

            // Stroke
            assert!(s.stroke.is_some());
            assert_eq!(s.stroke_width, 1.0);
        } else {
            panic!("Expected Shape element");
        }
    }

    #[test]
    fn test_neumorphism_applies_dual_shadows() {
        let mut project = make_test_project();
        let preset = StylePreset {
            style_type: StyleType::Neumorphism,
            params: StyleParams::default(),
        };
        apply_style_preset(&mut project, "shape-1", &preset).unwrap();

        let elem = &project.elements[0];
        assert_eq!(elem.common().shadows.len(), 2);
        // Light shadow (top-left)
        assert_eq!(elem.common().shadows[0].offset_x, -5.0);
        assert_eq!(elem.common().shadows[0].offset_y, -5.0);
        // Dark shadow (bottom-right)
        assert_eq!(elem.common().shadows[1].offset_x, 5.0);
        assert_eq!(elem.common().shadows[1].offset_y, 5.0);
    }

    #[test]
    fn test_glassmorphism_applies_blur_transparency_stroke() {
        let mut project = make_test_project();
        let preset = StylePreset {
            style_type: StyleType::Glassmorphism,
            params: StyleParams::default(),
        };
        apply_style_preset(&mut project, "shape-1", &preset).unwrap();

        let elem = &project.elements[0];
        // Blur filter
        assert!(elem.common().svg_filter.is_some());
        let filter = elem.common().svg_filter.as_ref().unwrap();
        assert!(matches!(filter.filter_type, FilterType::Blur));

        // Semi-transparent
        assert!(elem.common().opacity < 1.0);

        // White stroke
        if let Element::Shape(s) = elem {
            assert!(s.stroke.is_some());
            assert!(s.stroke.as_ref().unwrap().contains("FFFFFF"));
        }
    }

    #[test]
    fn test_flat_clears_all_effects() {
        let mut project = make_test_project();
        // First apply Soft 3D
        let soft3d = StylePreset {
            style_type: StyleType::Soft3d,
            params: StyleParams::default(),
        };
        apply_style_preset(&mut project, "shape-1", &soft3d).unwrap();

        // Then flatten
        let flat = StylePreset {
            style_type: StyleType::Flat,
            params: StyleParams::default(),
        };
        apply_style_preset(&mut project, "shape-1", &flat).unwrap();

        let elem = &project.elements[0];
        assert!(elem.common().shadows.is_empty());
        assert!(elem.common().svg_filter.is_none());
        assert!(elem.common().blend_mode.is_none());
        assert_eq!(elem.common().opacity, 1.0);

        if let Element::Shape(s) = elem {
            assert!(s.gradient.is_none());
            assert!(s.stroke.is_none());
        }
    }

    #[test]
    fn test_nonexistent_element_returns_error() {
        let mut project = make_test_project();
        let preset = StylePreset {
            style_type: StyleType::Soft3d,
            params: StyleParams::default(),
        };
        let result = apply_style_preset(&mut project, "shape-999", &preset);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
