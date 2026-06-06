use serde::{Deserialize, Serialize};

use super::{CommonProps, Element};

/// A property override applied to a symbol instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolOverride {
    pub property: String,
    pub value: serde_json::Value,
}

/// A reusable symbol definition (master component).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDef {
    pub id: String,
    pub name: String,
    pub source_element: Element,
    #[serde(default)]
    pub overridable_props: Vec<String>,
}

/// An element that is an instance of a symbol definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInstanceElement {
    #[serde(flatten)]
    pub common: CommonProps,
    pub symbol_id: String,
    #[serde(default)]
    pub overrides: Vec<SymbolOverride>,
}

/// Apply a list of overrides to a cloned source element.
/// Unknown property names are silently skipped.
pub fn apply_overrides(element: &mut Element, overrides: &[SymbolOverride]) {
    for ov in overrides {
        apply_single_override(element, ov);
    }
}

fn apply_single_override(element: &mut Element, ov: &SymbolOverride) {
    match ov.property.as_str() {
        "fill" => {
            if let Some(v) = ov.value.as_str() {
                match element {
                    Element::Shape(e) => e.fill = v.to_string(),
                    Element::Text(e) => e.fill = v.to_string(),
                    Element::Icon(e) => e.fill = v.to_string(),
                    Element::Path(e) => e.fill = v.to_string(),
                    _ => {}
                }
            }
        }
        "opacity" => {
            if let Some(v) = ov.value.as_f64() {
                element.common_mut().opacity = v;
            }
        }
        "x" => {
            if let Some(v) = ov.value.as_f64() {
                element.common_mut().x = v;
            }
        }
        "y" => {
            if let Some(v) = ov.value.as_f64() {
                element.common_mut().y = v;
            }
        }
        "width" => {
            if let Some(v) = ov.value.as_f64() {
                element.common_mut().width = v;
            }
        }
        "height" => {
            if let Some(v) = ov.value.as_f64() {
                element.common_mut().height = v;
            }
        }
        "rotation" => {
            if let Some(v) = ov.value.as_f64() {
                element.common_mut().rotation = v;
            }
        }
        "stroke" => {
            if let Some(v) = ov.value.as_str() {
                match element {
                    Element::Shape(e) => e.stroke = Some(v.to_string()),
                    Element::Text(e) => e.stroke = Some(v.to_string()),
                    Element::Icon(e) => e.stroke = Some(v.to_string()),
                    _ => {}
                }
            }
        }
        "content" => {
            if let Some(v) = ov.value.as_str() {
                if let Element::Text(e) = element {
                    e.content = v.to_string();
                }
            }
        }
        _ => {}
    }
}

/// Detach a symbol instance: produce an independent Element by cloning the
/// source element and applying all overrides. Returns None if the symbol
/// definition cannot be found.
pub fn detach_symbol(
    instance: &SymbolInstanceElement,
    symbols: &std::collections::HashMap<String, SymbolDef>,
) -> Option<Element> {
    let def = symbols.get(&instance.symbol_id)?;
    let mut elem = def.source_element.clone();
    apply_overrides(&mut elem, &instance.overrides);
    elem.common_mut().id = instance.common.id.clone();
    Some(elem)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{shapes::ShapeType, ShapeElement, TextElement};
    use std::collections::HashMap;

    fn make_shape(id: &str, fill: &str, x: f64, y: f64) -> Element {
        Element::Shape(ShapeElement {
            common: CommonProps::new(id.to_string(), x, y, 100.0, 100.0),
            shape_type: ShapeType::Circle,
            fill: fill.to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        })
    }

    fn make_text(id: &str, content: &str, fill: &str) -> Element {
        Element::Text(TextElement {
            common: CommonProps::new(id.to_string(), 50.0, 50.0, 200.0, 50.0),
            content: content.to_string(),
            fill: fill.to_string(),
            font_family: "sans-serif".to_string(),
            font_size: 32.0,
            font_weight: "bold".to_string(),
            letter_spacing: 0.0,
            stroke: None,
            stroke_width: 0.0,
            gradient: None,
        })
    }

    #[test]
    fn test_symbol_def_creation() {
        let source = make_shape("shape-1", "#FF0000", 0.0, 0.0);
        let def = SymbolDef {
            id: "symbol-1".to_string(),
            name: "Red Circle".to_string(),
            source_element: source.clone(),
            overridable_props: vec!["fill".into()],
        };
        assert_eq!(def.id, "symbol-1");
        assert_eq!(def.name, "Red Circle");
        assert_eq!(def.overridable_props.len(), 1);
    }

    #[test]
    fn test_symbol_def_serialization() {
        let source = make_shape("shape-1", "#FF0000", 0.0, 0.0);
        let def = SymbolDef {
            id: "symbol-1".to_string(),
            name: "Red Circle".to_string(),
            source_element: source,
            overridable_props: vec![],
        };
        let json = serde_json::to_string(&def).unwrap();
        assert!(json.contains("\"symbol-1\""));
        assert!(json.contains("\"Red Circle\""));

        let parsed: SymbolDef = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "symbol-1");
        assert_eq!(parsed.name, "Red Circle");
    }

    #[test]
    fn test_symbol_instance_serialization() {
        let inst = SymbolInstanceElement {
            common: CommonProps::new("sym-inst-1".to_string(), 10.0, 20.0, 100.0, 100.0),
            symbol_id: "symbol-1".to_string(),
            overrides: vec![SymbolOverride {
                property: "fill".to_string(),
                value: serde_json::json!("#00FF00"),
            }],
        };
        let json = serde_json::to_string(&inst).unwrap();
        assert!(json.contains("\"type\":\"symbol\""));
        assert!(json.contains("\"symbol_id\":\"symbol-1\""));
        assert!(json.contains("\"overrides\""));
    }

    #[test]
    fn test_apply_override_fill() {
        let mut elem = make_shape("shape-1", "#FF0000", 0.0, 0.0);
        apply_overrides(&mut elem, &[SymbolOverride {
            property: "fill".to_string(),
            value: serde_json::json!("#00FF00"),
        }]);
        match elem {
            Element::Shape(s) => assert_eq!(s.fill, "#00FF00"),
            _ => panic!("Expected Shape"),
        }
    }

    #[test]
    fn test_apply_override_opacity() {
        let mut elem = make_shape("shape-1", "#FF0000", 0.0, 0.0);
        apply_overrides(&mut elem, &[SymbolOverride {
            property: "opacity".to_string(),
            value: serde_json::json!(0.5),
        }]);
        assert!((elem.common().opacity - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_apply_override_position() {
        let mut elem = make_shape("shape-1", "#FF0000", 0.0, 0.0);
        apply_overrides(&mut elem, &[
            SymbolOverride { property: "x".to_string(), value: serde_json::json!(42.0) },
            SymbolOverride { property: "y".to_string(), value: serde_json::json!(84.0) },
        ]);
        assert!((elem.common().x - 42.0).abs() < f64::EPSILON);
        assert!((elem.common().y - 84.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_apply_override_content_text() {
        let mut elem = make_text("text-1", "Hello", "#000000");
        apply_overrides(&mut elem, &[SymbolOverride {
            property: "content".to_string(),
            value: serde_json::json!("World"),
        }]);
        match elem {
            Element::Text(t) => assert_eq!(t.content, "World"),
            _ => panic!("Expected Text"),
        }
    }

    #[test]
    fn test_apply_override_stroke() {
        let mut elem = make_shape("shape-1", "#FF0000", 0.0, 0.0);
        apply_overrides(&mut elem, &[SymbolOverride {
            property: "stroke".to_string(),
            value: serde_json::json!("#000000"),
        }]);
        match elem {
            Element::Shape(s) => assert_eq!(s.stroke, Some("#000000".to_string())),
            _ => panic!("Expected Shape"),
        }
    }

    #[test]
    fn test_apply_override_unknown_property_skipped() {
        let mut elem = make_shape("shape-1", "#FF0000", 0.0, 0.0);
        apply_overrides(&mut elem, &[SymbolOverride {
            property: "nonexistent".to_string(),
            value: serde_json::json!("value"),
        }]);
        // Element should be unchanged
        match elem {
            Element::Shape(s) => assert_eq!(s.fill, "#FF0000"),
            _ => panic!("Expected Shape"),
        }
    }

    #[test]
    fn test_apply_multiple_overrides() {
        let mut elem = make_shape("shape-1", "#FF0000", 0.0, 0.0);
        apply_overrides(&mut elem, &[
            SymbolOverride { property: "fill".to_string(), value: serde_json::json!("#00FF00") },
            SymbolOverride { property: "x".to_string(), value: serde_json::json!(50.0) },
            SymbolOverride { property: "rotation".to_string(), value: serde_json::json!(45.0) },
        ]);
        match &elem {
            Element::Shape(s) => assert_eq!(s.fill, "#00FF00"),
            _ => panic!("Expected Shape"),
        }
        assert!((elem.common().x - 50.0).abs() < f64::EPSILON);
        assert!((elem.common().rotation - 45.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_detach_symbol() {
        let source = make_shape("shape-1", "#FF0000", 0.0, 0.0);
        let def = SymbolDef {
            id: "symbol-1".to_string(),
            name: "Red Circle".to_string(),
            source_element: source,
            overridable_props: vec![],
        };
        let mut symbols = HashMap::new();
        symbols.insert("symbol-1".to_string(), def);

        let inst = SymbolInstanceElement {
            common: CommonProps::new("inst-1".to_string(), 10.0, 20.0, 100.0, 100.0),
            symbol_id: "symbol-1".to_string(),
            overrides: vec![SymbolOverride {
                property: "fill".to_string(),
                value: serde_json::json!("#00FF00"),
            }],
        };

        let independent = detach_symbol(&inst, &symbols).unwrap();
        match &independent {
            Element::Shape(s) => {
                assert_eq!(s.fill, "#00FF00", "override should be applied");
                assert_eq!(s.common.id, "inst-1", "ID should come from instance");
            }
            _ => panic!("Expected Shape"),
        }
    }

    #[test]
    fn test_detach_symbol_missing_def() {
        let inst = SymbolInstanceElement {
            common: CommonProps::new("inst-1".to_string(), 10.0, 20.0, 100.0, 100.0),
            symbol_id: "nonexistent".to_string(),
            overrides: vec![],
        };
        let symbols = HashMap::new();
        assert!(detach_symbol(&inst, &symbols).is_none());
    }

    #[test]
    fn test_backward_compat_no_symbols_field() {
        let json = r##"{"schema_version":"1.0","canvas":{"width":512,"height":512,"background":"#FFFFFF","corner_radius":0},"elements":[],"exports":{"formats":["svg"],"sizes":[16,32]},"templates":{}}"##;
        let project: crate::model::IconProject = serde_json::from_str(json).unwrap();
        assert!(project.symbols.is_empty());
    }

    #[test]
    fn test_backward_compat_with_symbols() {
        let source = make_shape("shape-1", "#FF0000", 0.0, 0.0);
        let def = SymbolDef {
            id: "symbol-1".to_string(),
            name: "Test".to_string(),
            source_element: source,
            overridable_props: vec![],
        };
        let mut project = crate::model::IconProject::default();
        project.symbols.insert("symbol-1".to_string(), def);

        let json = serde_json::to_string(&project).unwrap();
        let parsed: crate::model::IconProject = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.symbols.len(), 1);
        assert!(parsed.symbols.contains_key("symbol-1"));
    }
}
