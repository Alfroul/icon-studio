use crate::model::{Element, IconProject};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WeightPreset {
    Thin,
    Light,
    Regular,
    Medium,
    Bold,
    Fill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IconStyleKind {
    StrokeBased,
    FillBased,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeightVariant {
    pub weight: WeightPreset,
    pub project: IconProject,
}

pub fn detect_icon_style(project: &IconProject) -> IconStyleKind {
    let mut stroke_count = 0usize;
    let mut fill_count = 0usize;
    count_style(project.active_elements(), &mut stroke_count, &mut fill_count);
    if stroke_count == 0 && fill_count == 0 {
        return IconStyleKind::Mixed;
    }
    if stroke_count > fill_count {
        IconStyleKind::StrokeBased
    } else if fill_count > stroke_count {
        IconStyleKind::FillBased
    } else {
        IconStyleKind::Mixed
    }
}

fn count_style(elements: &[Element], stroke_count: &mut usize, fill_count: &mut usize) {
    for elem in elements {
        match elem {
            Element::Shape(e) => {
                if e.stroke_width > 0.0 && e.stroke.is_some() {
                    *stroke_count += 1;
                } else {
                    *fill_count += 1;
                }
            }
            Element::Text(e) => {
                if e.stroke_width > 0.0 && e.stroke.is_some() {
                    *stroke_count += 1;
                } else {
                    *fill_count += 1;
                }
            }
            Element::Icon(e) => {
                if e.stroke_width > 0.0 && e.stroke.is_some() {
                    *stroke_count += 1;
                } else {
                    *fill_count += 1;
                }
            }
            Element::Path(e) => {
                if e.stroke_width > 0.0 {
                    *stroke_count += 1;
                } else {
                    *fill_count += 1;
                }
            }
            Element::Group(g) => {
                count_style(&g.children, stroke_count, fill_count);
            }
            Element::Image(_) | Element::Symbol(_) => {}
        }
    }
}

pub fn scale_stroke_width(elem: &mut Element, factor: f64) {
    match elem {
        Element::Shape(e) => {
            e.stroke_width = (e.stroke_width * factor).clamp(0.1, 20.0);
        }
        Element::Text(e) => {
            e.stroke_width = (e.stroke_width * factor).clamp(0.1, 20.0);
        }
        Element::Icon(e) => {
            e.stroke_width = (e.stroke_width * factor).clamp(0.1, 20.0);
        }
        Element::Path(e) => {
            e.stroke_width = (e.stroke_width * factor).clamp(0.1, 20.0);
        }
        Element::Group(g) => {
            for child in &mut g.children {
                scale_stroke_width(child, factor);
            }
        }
        Element::Image(_) | Element::Symbol(_) => {}
    }
}

pub fn convert_stroke_to_fill(elem: &mut Element) {
    match elem {
        Element::Shape(e) => {
            if let Some(ref stroke_color) = e.stroke {
                e.fill = stroke_color.clone();
                e.stroke = None;
                e.stroke_width = 0.0;
            }
        }
        Element::Text(e) => {
            if let Some(ref stroke_color) = e.stroke {
                e.fill = stroke_color.clone();
                e.stroke = None;
                e.stroke_width = 0.0;
            }
        }
        Element::Icon(e) => {
            if let Some(ref stroke_color) = e.stroke {
                e.fill = stroke_color.clone();
                e.stroke = None;
                e.stroke_width = 0.0;
            }
        }
        Element::Path(e) => {
            if e.stroke_width > 0.0 && !e.stroke.is_empty() {
                e.fill = e.stroke.clone();
                e.stroke = String::new();
                e.stroke_width = 0.0;
            }
        }
        Element::Group(g) => {
            for child in &mut g.children {
                convert_stroke_to_fill(child);
            }
        }
        Element::Image(_) | Element::Symbol(_) => {}
    }
}

fn weight_factor(preset: &WeightPreset) -> f64 {
    match preset {
        WeightPreset::Thin => 0.5,
        WeightPreset::Light => 0.75,
        WeightPreset::Regular => 1.0,
        WeightPreset::Medium => 1.25,
        WeightPreset::Bold => 1.5,
        WeightPreset::Fill => 1.5,
    }
}

pub fn generate_weight_variants(
    project: &IconProject,
    weights: &[WeightPreset],
) -> Vec<WeightVariant> {
    weights
        .iter()
        .map(|preset| {
            let mut clone = project.clone();
            let elements = clone.active_elements_mut();
            if matches!(preset, WeightPreset::Fill) {
                for elem in elements.iter_mut() {
                    scale_stroke_width(elem, 1.5);
                }
                for elem in elements.iter_mut() {
                    convert_stroke_to_fill(elem);
                }
            } else {
                let factor = weight_factor(preset);
                for elem in elements.iter_mut() {
                    scale_stroke_width(elem, factor);
                }
            }
            clone.bump_version();
            WeightVariant {
                weight: preset.clone(),
                project: clone,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn make_stroke_project() -> IconProject {
        let mut p = IconProject::default();
        p.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-1".into(), 50.0, 50.0, 200.0, 200.0),
            shape_type: shapes::ShapeType::Circle,
            fill: "none".into(),
            stroke: Some("#000000".into()),
            stroke_width: 2.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        }));
        p.elements.push(Element::Path(PathElement {
            common: CommonProps::new("path-1".into(), 0.0, 0.0, 100.0, 100.0),
            d: "M0 0L100 100".into(),
            fill: "none".into(),
            stroke: "#000000".into(),
            stroke_width: 2.0,
            stroke_dasharray: None,
            natural_width: 100.0,
            natural_height: 100.0,
            boolean_source: None,
        }));
        p
    }

    fn make_fill_project() -> IconProject {
        let mut p = IconProject::default();
        p.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-1".into(), 50.0, 50.0, 200.0, 200.0),
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".into(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        }));
        p
    }

    #[test]
    fn test_detect_stroke_based() {
        let project = make_stroke_project();
        match detect_icon_style(&project) {
            IconStyleKind::StrokeBased => {}
            other => panic!("Expected StrokeBased, got {:?}", other),
        }
    }

    #[test]
    fn test_detect_fill_based() {
        let project = make_fill_project();
        match detect_icon_style(&project) {
            IconStyleKind::FillBased => {}
            other => panic!("Expected FillBased, got {:?}", other),
        }
    }

    #[test]
    fn test_detect_mixed() {
        let mut project = IconProject::default();
        // One stroke element
        project.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("s1".into(), 0.0, 0.0, 100.0, 100.0),
            shape_type: shapes::ShapeType::Circle,
            fill: "none".into(),
            stroke: Some("#000".into()),
            stroke_width: 2.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        }));
        // One fill element
        project.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("s2".into(), 0.0, 0.0, 100.0, 100.0),
            shape_type: shapes::ShapeType::Circle,
            fill: "#F00".into(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        }));
        match detect_icon_style(&project) {
            IconStyleKind::Mixed => {}
            other => panic!("Expected Mixed, got {:?}", other),
        }
    }

    #[test]
    fn test_scale_stroke_width() {
        let mut elem = Element::Shape(ShapeElement {
            common: CommonProps::new("s1".into(), 0.0, 0.0, 100.0, 100.0),
            shape_type: shapes::ShapeType::Circle,
            fill: "none".into(),
            stroke: Some("#000".into()),
            stroke_width: 2.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        });
        scale_stroke_width(&mut elem, 0.5);
        if let Element::Shape(s) = &elem {
            assert!((s.stroke_width - 1.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected Shape");
        }
    }

    #[test]
    fn test_scale_stroke_clamped() {
        let mut elem = Element::Shape(ShapeElement {
            common: CommonProps::new("s1".into(), 0.0, 0.0, 100.0, 100.0),
            shape_type: shapes::ShapeType::Circle,
            fill: "none".into(),
            stroke: Some("#000".into()),
            stroke_width: 1.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        });
        scale_stroke_width(&mut elem, 100.0);
        if let Element::Shape(s) = &elem {
            assert!((s.stroke_width - 20.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_convert_stroke_to_fill() {
        let mut elem = Element::Shape(ShapeElement {
            common: CommonProps::new("s1".into(), 0.0, 0.0, 100.0, 100.0),
            shape_type: shapes::ShapeType::Circle,
            fill: "none".into(),
            stroke: Some("#333333".into()),
            stroke_width: 2.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        });
        convert_stroke_to_fill(&mut elem);
        if let Element::Shape(s) = &elem {
            assert_eq!(s.fill, "#333333");
            assert!(s.stroke.is_none());
            assert!((s.stroke_width).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_convert_stroke_to_fill_path() {
        let mut elem = Element::Path(PathElement {
            common: CommonProps::new("p1".into(), 0.0, 0.0, 100.0, 100.0),
            d: "M0 0L100 100".into(),
            fill: "none".into(),
            stroke: "#FF0000".into(),
            stroke_width: 2.0,
            stroke_dasharray: None,
            natural_width: 100.0,
            natural_height: 100.0,
            boolean_source: None,
        });
        convert_stroke_to_fill(&mut elem);
        if let Element::Path(p) = &elem {
            assert_eq!(p.fill, "#FF0000");
            assert!(p.stroke.is_empty());
            assert!((p.stroke_width).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_generate_weight_variants_count() {
        let project = make_stroke_project();
        let presets = vec![
            WeightPreset::Thin,
            WeightPreset::Light,
            WeightPreset::Regular,
            WeightPreset::Medium,
            WeightPreset::Bold,
            WeightPreset::Fill,
        ];
        let variants = generate_weight_variants(&project, &presets);
        assert_eq!(variants.len(), 6);
    }

    #[test]
    fn test_regular_preserves_stroke_width() {
        let project = make_stroke_project();
        let variants = generate_weight_variants(&project, &[WeightPreset::Regular]);
        assert_eq!(variants.len(), 1);
        if let Element::Shape(s) = &variants[0].project.elements[0] {
            assert!((s.stroke_width - 2.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_fill_variant_converts() {
        let project = make_stroke_project();
        let variants = generate_weight_variants(&project, &[WeightPreset::Fill]);
        assert_eq!(variants.len(), 1);
        if let Element::Shape(s) = &variants[0].project.elements[0] {
            assert_eq!(s.fill, "#000000");
            assert!(s.stroke.is_none());
        }
    }

    #[test]
    fn test_variants_have_different_svg() {
        let project = make_stroke_project();
        let presets = vec![
            WeightPreset::Thin,
            WeightPreset::Bold,
        ];
        let variants = generate_weight_variants(&project, &presets);
        let svg_thin = crate::engine::builder::build(&variants[0].project).unwrap_or_default();
        let svg_bold = crate::engine::builder::build(&variants[1].project).unwrap_or_default();
        assert_ne!(svg_thin, svg_bold, "Thin and Bold should produce different SVGs");
    }

    #[test]
    fn test_scale_group_children() {
        let mut group = Element::Group(GroupElement {
            common: CommonProps::new("g1".into(), 0.0, 0.0, 200.0, 200.0),
            children: vec![Element::Shape(ShapeElement {
                common: CommonProps::new("s1".into(), 10.0, 10.0, 50.0, 50.0),
                shape_type: shapes::ShapeType::Circle,
                fill: "none".into(),
                stroke: Some("#000".into()),
                stroke_width: 2.0,
                border_radius: 0.0,
                stroke_dasharray: None,
                gradient: None,
            })],
            expanded: false,
        });
        scale_stroke_width(&mut group, 0.5);
        if let Element::Group(g) = &group {
            if let Element::Shape(s) = &g.children[0] {
                assert!((s.stroke_width - 1.0).abs() < f64::EPSILON);
            }
        }
    }

    #[test]
    fn test_convert_group_children() {
        let mut group = Element::Group(GroupElement {
            common: CommonProps::new("g1".into(), 0.0, 0.0, 200.0, 200.0),
            children: vec![Element::Shape(ShapeElement {
                common: CommonProps::new("s1".into(), 10.0, 10.0, 50.0, 50.0),
                shape_type: shapes::ShapeType::Circle,
                fill: "none".into(),
                stroke: Some("#FF0000".into()),
                stroke_width: 2.0,
                border_radius: 0.0,
                stroke_dasharray: None,
                gradient: None,
            })],
            expanded: false,
        });
        convert_stroke_to_fill(&mut group);
        if let Element::Group(g) = &group {
            if let Element::Shape(s) = &g.children[0] {
                assert_eq!(s.fill, "#FF0000");
                assert!(s.stroke.is_none());
            }
        }
    }
}
