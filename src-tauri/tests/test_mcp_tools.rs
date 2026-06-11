use std::sync::{Arc, Mutex};
use base64::Engine;
use icon_studio_lib::model::history::{AddElementCommand, CommandHistory, SetPropsCommand};
use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::*;
use icon_studio_lib::mcp::IconStudioHandler;
use icon_studio_lib::mcp::*;
use icon_studio_lib::engine::builder::RenderCache;

fn new_handler() -> IconStudioHandler {
    IconStudioHandler::new(Arc::new(Mutex::new(IconProject::default())), Arc::new(Mutex::new(RenderCache::default())), Arc::new(Mutex::new(CommandHistory::default())), None)
}

fn add_shape_to_project(project: &Arc<Mutex<IconProject>>, shape_type: &str, fill: &str, size: f64, x: f64, y: f64) -> String {
    let mut p = project.lock().unwrap();
    let id = p.alloc_element_id("shape");
    let st = match shape_type {
        "circle" => ShapeType::Circle,
        "rect" => ShapeType::Rect,
        "rounded-rect" => ShapeType::RoundedRect,
        "hexagon" => ShapeType::Hexagon,
        "star" => ShapeType::Star,
        "shield" => ShapeType::Shield,
        "diamond" => ShapeType::Diamond,
        _ => ShapeType::Circle,
    };
    p.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id: id.clone(),
            x,
            y,
            width: size,
            height: size,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        overlay: None,
        },
        shape_type: st,
        fill: fill.to_string(),
        stroke: None,
        stroke_width: 0.0,
        border_radius: 0.0,
        stroke_dasharray: None,
        gradient: None,
    }));
    id
}

#[tokio::test]
async fn test_icon_new() {
    let handler = new_handler();
    let params = IconNewParams {
        width: 256,
        height: 256,
        background: "#FF0000".to_string(),
    };

    {
        let mut project = handler.project.lock().unwrap();
        project.canvas = Canvas {
            width: params.width,
            height: params.height,
            background: params.background.clone(),
            corner_radius: 0,
            background_gradient: None,
        };
        project.elements.clear();
        project.next_element_id = 1;
    }

    let project = handler.project.lock().unwrap();
    assert_eq!(project.canvas.width, 256);
    assert_eq!(project.canvas.height, 256);
    assert_eq!(project.canvas.background, "#FF0000");
    assert_eq!(project.elements.len(), 0);
}

#[tokio::test]
async fn test_add_shape_and_list() {
    let handler = new_handler();

    add_shape_to_project(&handler.project, "circle", "#FF0000", 100.0, 10.0, 20.0);

    {
        let project = handler.project.lock().unwrap();
        assert_eq!(project.elements.len(), 1);
        if let Element::Shape(ref s) = project.elements[0] {
            assert!(matches!(s.shape_type, ShapeType::Circle));
            assert_eq!(s.fill, "#FF0000");
            assert_eq!(s.common.width, 100.0);
            assert_eq!(s.common.x, 10.0);
            assert_eq!(s.common.y, 20.0);
        } else {
            panic!("Expected Shape element");
        }
    }

    {
        let mut project = handler.project.lock().unwrap();
        let id = project.alloc_element_id("text");
        project.elements.push(Element::Text(TextElement {
            common: CommonProps {
                id,
                x: 50.0,
                y: 50.0,
                width: 72.0,
                height: 28.8,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
                blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        overlay: None,
        },
            content: "Hello".to_string(),
            fill: "#000000".to_string(),
            font_family: "Microsoft YaHei".to_string(),
            font_size: 24.0,
            font_weight: "normal".to_string(),
            letter_spacing: 0.0,
            stroke: None,
            stroke_width: 0.0,
            gradient: None,
        }));
    }

    let project = handler.project.lock().unwrap();
    assert_eq!(project.elements.len(), 2);
}

#[tokio::test]
async fn test_set_props() {
    let handler = new_handler();
    let elem_id = add_shape_to_project(&handler.project, "circle", "#FF0000", 100.0, 10.0, 20.0);

    {
        let mut project = handler.project.lock().unwrap();
        let idx = project.elements.iter().position(|e| match e {
            Element::Shape(s) => s.common.id == elem_id,
            _ => false,
        }).expect("Element not found");

        let mut existing = serde_json::to_value(&project.elements[idx]).unwrap();
        if let Some(obj) = existing.as_object_mut() {
            obj.insert("fill".to_string(), serde_json::Value::String("#00FF00".to_string()));
        }
        let updated: Element = serde_json::from_value(existing).unwrap();
        project.elements[idx] = updated;
    }

    let project = handler.project.lock().unwrap();
    if let Element::Shape(ref s) = project.elements[0] {
        assert_eq!(s.fill, "#00FF00");
    } else {
        panic!("Expected Shape element");
    }
}

#[tokio::test]
async fn test_remove_element() {
    let handler = new_handler();
    let id1 = add_shape_to_project(&handler.project, "circle", "#FF0000", 100.0, 0.0, 0.0);
    let id2 = add_shape_to_project(&handler.project, "rect", "#0000FF", 50.0, 100.0, 100.0);

    {
        let project = handler.project.lock().unwrap();
        assert_eq!(project.elements.len(), 2);
    }

    {
        let mut project = handler.project.lock().unwrap();
        project.elements.retain(|e| match e {
            Element::Shape(s) => s.common.id != id1,
            Element::Text(t) => t.common.id != id1,
            Element::Icon(i) => i.common.id != id1,
            Element::Image(img) => img.common.id != id1,
            Element::Path(p) => p.common.id != id1,
            Element::Group(g) => g.common.id != id1,
            Element::Symbol(s) => s.common.id != id1,
        });
    }

    let project = handler.project.lock().unwrap();
    assert_eq!(project.elements.len(), 1);
    if let Element::Shape(ref s) = project.elements[0] {
        assert_eq!(s.common.id, id2);
    }
}

#[tokio::test]
async fn test_export_svg() {
    let handler = new_handler();
    add_shape_to_project(&handler.project, "circle", "#FF0000", 100.0, 50.0, 50.0);

    let dir = tempfile::tempdir().unwrap();
    let svg_path = dir.path().join("test_icon.svg");

    let svg_str = {
        let project = handler.project.lock().unwrap();
        icon_studio_lib::engine::builder::build(&project).unwrap()
    };
    std::fs::write(&svg_path, &svg_str).unwrap();

    assert!(svg_path.exists());
    let content = std::fs::read_to_string(&svg_path).unwrap();
    assert!(content.contains("<svg"));
    assert!(content.contains("</svg>"));
}

#[tokio::test]
async fn test_suggest_palette() {
    let colors = icon_studio_lib::colors::suggest_palette(
        "#FF0000",
        icon_studio_lib::colors::PaletteScheme::Complementary,
        5,
    )
    .unwrap();

    assert_eq!(colors.len(), 5);
    for c in &colors {
        assert!(c.starts_with('#'), "Color should start with #: {}", c);
        assert_eq!(c.len(), 7, "Color should be 7 chars (#RRGGBB): {}", c);
    }
}

#[tokio::test]
async fn test_set_gradient() {
    let handler = new_handler();
    let elem_id = add_shape_to_project(&handler.project, "circle", "#FF0000", 100.0, 50.0, 50.0);

    {
        let mut project = handler.project.lock().unwrap();
        let elem = project.elements.iter_mut().find(|e| match e {
            Element::Shape(s) => s.common.id == elem_id,
            _ => false,
        }).expect("Element not found");

        if let Element::Shape(ref mut s) = elem {
            s.gradient = Some(Gradient {
                gradient_type: GradientKind::Linear,
                colors: vec!["#FF0000".to_string(), "#00FF00".to_string()],
                angle: 45.0,
                stops: Vec::new(),
            });
        }
    }

    let project = handler.project.lock().unwrap();
    if let Element::Shape(ref s) = project.elements[0] {
        let g = s.gradient.as_ref().expect("Gradient should be set");
        assert!(matches!(g.gradient_type, GradientKind::Linear));
        assert_eq!(g.colors.len(), 2);
        assert_eq!(g.colors[0], "#FF0000");
        assert_eq!(g.colors[1], "#00FF00");
        assert_eq!(g.angle, 45.0);
    }
}

#[tokio::test]
async fn test_set_shadow() {
    let handler = new_handler();
    let elem_id = add_shape_to_project(&handler.project, "circle", "#FF0000", 100.0, 50.0, 50.0);

    {
        let mut project = handler.project.lock().unwrap();
        let elem = project.elements.iter_mut().find(|e| match e {
            Element::Shape(s) => s.common.id == elem_id,
            _ => false,
        }).expect("Element not found");

        if let Element::Shape(ref mut s) = elem {
            s.common.shadows = vec![Shadow {
                color: "#00000040".to_string(),
                blur: 8.0,
                offset_x: 0.0,
                offset_y: 4.0,
                inset: false,
            }];
        }
    }

    let project = handler.project.lock().unwrap();
    if let Element::Shape(ref s) = project.elements[0] {
        let sh = s.common.shadows.first().expect("Shadow should be set");
        assert_eq!(sh.color, "#00000040");
        assert_eq!(sh.blur, 8.0);
        assert_eq!(sh.offset_x, 0.0);
        assert_eq!(sh.offset_y, 4.0);
    }
}

#[tokio::test]
async fn test_icon_preview() {
    let handler = new_handler();
    add_shape_to_project(&handler.project, "circle", "#FF0000", 100.0, 50.0, 50.0);

    let svg_str = {
        let project = handler.project.lock().unwrap();
        icon_studio_lib::engine::builder::build(&project).unwrap()
    };

    let png_bytes = icon_studio_lib::engine::renderer::render(&svg_str, 64).unwrap();
    assert!(!png_bytes.is_empty());

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
    assert!(!b64.is_empty());
    let data_uri = format!("data:image/png;base64,{}", b64);
    assert!(data_uri.starts_with("data:image/png;base64,"));
}

#[tokio::test]
async fn test_set_layout() {
    let handler = new_handler();
    add_shape_to_project(&handler.project, "circle", "#FF0000", 100.0, 0.0, 0.0);
    add_shape_to_project(&handler.project, "rect", "#0000FF", 50.0, 0.0, 0.0);
    add_shape_to_project(&handler.project, "star", "#00FF00", 80.0, 0.0, 0.0);

    {
        let mut project = handler.project.lock().unwrap();
        let cw = project.canvas.width as f64;
        let ch = project.canvas.height as f64;
        let gap = 10.0;
        let padding = 20.0;

        let mut indices: Vec<usize> = (0..project.elements.len()).collect();
        indices.sort_by(|&a, &b| {
            let (ax, _, _, _) = match &project.elements[a] {
                Element::Shape(s) => (s.common.x, s.common.y, s.common.width, s.common.height),
                Element::Text(t) => (t.common.x, t.common.y, t.common.width, t.common.height),
                Element::Icon(i) => (i.common.x, i.common.y, i.common.width, i.common.height),
                Element::Image(img) => (img.common.x, img.common.y, img.common.width, img.common.height),
                Element::Path(p) => (p.common.x, p.common.y, p.common.width, p.common.height),
                Element::Group(g) => (g.common.x, g.common.y, g.common.width, g.common.height),
                Element::Symbol(s) => (s.common.x, s.common.y, s.common.width, s.common.height),
            };
            let (bx, _, _, _) = match &project.elements[b] {
                Element::Shape(s) => (s.common.x, s.common.y, s.common.width, s.common.height),
                Element::Text(t) => (t.common.x, t.common.y, t.common.width, t.common.height),
                Element::Icon(i) => (i.common.x, i.common.y, i.common.width, i.common.height),
                Element::Image(img) => (img.common.x, img.common.y, img.common.width, img.common.height),
                Element::Path(p) => (p.common.x, p.common.y, p.common.width, p.common.height),
                Element::Group(g) => (g.common.x, g.common.y, g.common.width, g.common.height),
                Element::Symbol(s) => (s.common.x, s.common.y, s.common.width, s.common.height),
            };
            ax.partial_cmp(&bx).unwrap_or(std::cmp::Ordering::Equal)
        });

        let total_width: f64 = indices.iter().map(|&i| {
            match &project.elements[i] {
                Element::Shape(s) => s.common.width,
                Element::Text(t) => t.common.width,
                Element::Icon(i_) => i_.common.width,
                Element::Image(img) => img.common.width,
                Element::Path(p) => p.common.width,
                Element::Group(g) => g.common.width,
                Element::Symbol(s) => s.common.width,
            }
        }).sum();

        let total_gaps = if indices.len() > 1 { (indices.len() - 1) as f64 * gap } else { 0.0 };
        let start_x = padding + (cw - padding * 2.0 - total_width - total_gaps) / 2.0;

        let mut current_x = start_x;
        for &i in &indices {
            let (ew, eh) = match &project.elements[i] {
                Element::Shape(s) => (s.common.width, s.common.height),
                Element::Text(t) => (t.common.width, t.common.height),
                Element::Icon(i_) => (i_.common.width, i_.common.height),
                Element::Image(img) => (img.common.width, img.common.height),
                Element::Path(p) => (p.common.width, p.common.height),
                Element::Group(g) => (g.common.width, g.common.height),
                Element::Symbol(s) => (s.common.width, s.common.height),
            };
            let ny = padding + (ch - padding * 2.0 - eh) / 2.0;
            match &mut project.elements[i] {
                Element::Shape(ref mut s) => { s.common.x = current_x; s.common.y = ny; }
                Element::Text(ref mut t) => { t.common.x = current_x; t.common.y = ny; }
                Element::Icon(ref mut i_) => { i_.common.x = current_x; i_.common.y = ny; }
                Element::Image(ref mut img) => { img.common.x = current_x; img.common.y = ny; }
                Element::Path(ref mut p) => { p.common.x = current_x; p.common.y = ny; }
                Element::Group(ref mut g) => { g.common.x = current_x; g.common.y = ny; }
                Element::Symbol(ref mut s) => { s.common.x = current_x; s.common.y = ny; }
            }
            current_x += ew + gap;
        }
    }

    let project = handler.project.lock().unwrap();
    assert_eq!(project.elements.len(), 3);

    for elem in &project.elements {
        let (x, _y, _w, _h) = match elem {
            Element::Shape(s) => (s.common.x, s.common.y, s.common.width, s.common.height),
            _ => panic!("Expected Shape"),
        };
        assert!(x >= 0.0, "Element x should be non-negative after layout");
    }
}

#[tokio::test]
async fn test_undo_redo_via_command_history() {
    let handler = new_handler();
    let elem_id = add_shape_to_project(&handler.project, "circle", "#FF0000", 100.0, 10.0, 20.0);

    let mut history = CommandHistory::new(50);
    {
        let mut project = handler.project.lock().unwrap();

        let old_props = serde_json::to_value(&project.elements[0]).unwrap();
        let mut new_props = old_props.clone();
        if let Some(obj) = new_props.as_object_mut() {
            obj.insert("fill".to_string(), serde_json::Value::String("#00FF00".to_string()));
        }

        let cmd = Box::new(SetPropsCommand::new(
            elem_id.clone(),
            old_props,
            new_props,
        ));
        history.push_and_execute(cmd, &mut project).unwrap();
    }

    {
        let project = handler.project.lock().unwrap();
        if let Element::Shape(ref s) = project.elements[0] {
            assert_eq!(s.fill, "#00FF00", "Fill should be green after set_props");
        }
    }

    {
        let mut project = handler.project.lock().unwrap();
        history.undo(&mut project).unwrap();
    }

    {
        let project = handler.project.lock().unwrap();
        if let Element::Shape(ref s) = project.elements[0] {
            assert_eq!(s.fill, "#FF0000", "Fill should be red after undo");
        }
    }

    {
        let mut project = handler.project.lock().unwrap();
        history.redo(&mut project).unwrap();
    }

    {
        let project = handler.project.lock().unwrap();
        if let Element::Shape(ref s) = project.elements[0] {
            assert_eq!(s.fill, "#00FF00", "Fill should be green after redo");
        }
    }
}

#[tokio::test]
async fn test_batch_undo_via_command_history() {
    let handler = new_handler();
    let mut history = CommandHistory::new(50);

    {
        let mut project = handler.project.lock().unwrap();
        history.begin_batch("add 3 shapes").unwrap();

        for i in 0..3 {
            let id = project.alloc_element_id("shape");
            let elem = Element::Shape(ShapeElement {
                common: CommonProps {
                    id,
                    x: i as f64 * 100.0,
                    y: 0.0,
                    width: 80.0,
                    height: 80.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        overlay: None,
        },
                shape_type: ShapeType::Circle,
                fill: format!("#FF{:02X}00", i * 80),
                stroke: None,
                stroke_width: 0.0,
                border_radius: 0.0,
                stroke_dasharray: None,
                gradient: None,
            });
            let cmd = Box::new(AddElementCommand::new(elem));
            history.push_and_execute(cmd, &mut project).unwrap();
        }

        history.commit_batch().unwrap();
    }

    {
        let project = handler.project.lock().unwrap();
        assert_eq!(project.elements.len(), 3, "Should have 3 elements after batch");
    }

    {
        let mut project = handler.project.lock().unwrap();
        history.undo(&mut project).unwrap();
    }

    {
        let project = handler.project.lock().unwrap();
        assert_eq!(project.elements.len(), 0, "All 3 elements should be undone in one undo");
    }

    {
        let mut project = handler.project.lock().unwrap();
        history.redo(&mut project).unwrap();
    }

    {
        let project = handler.project.lock().unwrap();
        assert_eq!(project.elements.len(), 3, "All 3 elements should be back after redo");
    }
}