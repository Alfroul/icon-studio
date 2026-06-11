use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::*;

fn common_defaults(id: &str) -> CommonProps {
    CommonProps {
        id: id.to_string(),
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
        opacity: 1.0,
        rotation: 0.0,
        shadows: vec![],
                animation: None,
        blend_mode: None,
    clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None, overlay: None,
        }
}

pub fn make_shape(id: &str, shape_type: ShapeType) -> ShapeElement {
    ShapeElement {
        common: common_defaults(id),
        shape_type,
        fill: "#FF0000".to_string(),
        stroke: None,
        stroke_width: 0.0,
        border_radius: 0.0,
        stroke_dasharray: None,
        gradient: None,
    }
}

pub fn make_shape_at(id: &str, shape_type: ShapeType, x: f64, y: f64) -> ShapeElement {
    let mut s = make_shape(id, shape_type);
    s.common.x = x;
    s.common.y = y;
    s
}

pub fn make_shape_sized(
    id: &str,
    shape_type: ShapeType,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) -> ShapeElement {
    let mut s = make_shape(id, shape_type);
    s.common.x = x;
    s.common.y = y;
    s.common.width = w;
    s.common.height = h;
    s
}

pub fn make_shape_detailed(
    id: &str,
    shape_type: ShapeType,
    fill: &str,
    stroke_width: f64,
    opacity: f64,
    width: f64,
) -> ShapeElement {
    let mut s = make_shape(id, shape_type);
    s.fill = fill.to_string();
    s.stroke = if stroke_width > 0.0 {
        Some("#000000".to_string())
    } else {
        None
    };
    s.stroke_width = stroke_width;
    s.common.opacity = opacity;
    s.common.width = width;
    s.common.height = width;
    s
}

pub fn shape_el(shape: ShapeElement) -> Element {
    Element::Shape(shape)
}

pub fn make_text(id: &str, content: &str) -> TextElement {
    TextElement {
        common: CommonProps {
            id: id.to_string(),
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 50.0,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None, overlay: None,
        },
        content: content.to_string(),
        fill: "#000000".to_string(),
        font_family: "sans-serif".to_string(),
        font_size: 24.0,
        font_weight: "normal".to_string(),
        letter_spacing: 0.0,
        stroke: None,
        stroke_width: 0.0,
        gradient: None,
    }
}

pub fn make_text_detailed(
    id: &str,
    content: &str,
    fill: &str,
    font_size: f64,
    opacity: f64,
    width: f64,
) -> TextElement {
    let mut t = make_text(id, content);
    t.fill = fill.to_string();
    t.font_size = font_size;
    t.common.opacity = opacity;
    t.common.width = width;
    t.common.height = font_size;
    t
}

pub fn text_el(text: TextElement) -> Element {
    Element::Text(text)
}

pub fn make_icon(id: &str, name: &str) -> IconElement {
    IconElement {
        common: CommonProps {
            id: id.to_string(),
            x: 100.0,
            y: 100.0,
            width: 48.0,
            height: 48.0,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None, overlay: None,
        },
        name: name.to_string(),
        fill: "#000000".to_string(),
        stroke: None,
        stroke_width: 0.0,
        gradient: None,
    }
}

pub fn icon_el(icon: IconElement) -> Element {
    Element::Icon(icon)
}

pub fn make_image(id: &str, data: &str) -> ImageElement {
    ImageElement {
        common: CommonProps {
            id: id.to_string(),
            x: 50.0,
            y: 50.0,
            width: 100.0,
            height: 100.0,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None, overlay: None,
        },
        data: data.to_string(),
    }
}

pub fn image_el(image: ImageElement) -> Element {
    Element::Image(image)
}

pub fn make_path(id: &str, d: &str) -> PathElement {
    PathElement {
        common: CommonProps {
            id: id.to_string(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None, overlay: None,
        },
        d: d.to_string(),
        fill: "#000000".to_string(),
        stroke: "none".to_string(),
        stroke_width: 0.0,
        stroke_dasharray: None,
        natural_width: 100.0,
        natural_height: 100.0,
        boolean_source: None,
    }
}

pub fn path_el(path: PathElement) -> Element {
    Element::Path(path)
}

pub fn make_default_project() -> IconProject {
    IconProject::default()
}

pub fn make_project_with_elements(elements: Vec<Element>) -> IconProject {
    let mut p = IconProject::default();
    p.elements = elements;
    p
}
