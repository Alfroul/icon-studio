use super::shapes::ShapeType;
use super::{Element, Gradient, Shadow};
use super::filter::SvgFilter;

pub fn parse_shape_type(s: &str) -> Result<ShapeType, String> {
    match s {
        "circle" => Ok(ShapeType::Circle),
        "rect" => Ok(ShapeType::Rect),
        "rounded-rect" => Ok(ShapeType::RoundedRect),
        "hexagon" => Ok(ShapeType::Hexagon),
        "star" => Ok(ShapeType::Star),
        "shield" => Ok(ShapeType::Shield),
        "diamond" => Ok(ShapeType::Diamond),
        "triangle" => Ok(ShapeType::Triangle),
        "arrow-right" => Ok(ShapeType::ArrowRight),
        "cross" => Ok(ShapeType::Cross),
        "heart" => Ok(ShapeType::Heart),
        "pentagon" => Ok(ShapeType::Pentagon),
        "octagon" => Ok(ShapeType::Octagon),
        "wave" => Ok(ShapeType::Wave),
        "custom" => Ok(ShapeType::Custom { d: String::new() }),
        other => Err(format!("Unknown shape type: '{}'. Valid types: circle, rect, rounded-rect, hexagon, star, shield, diamond, triangle, arrow-right, cross, heart, pentagon, octagon, wave, custom", other)),
    }
}

pub fn element_id(elem: &Element) -> &str {
    elem.id()
}

pub fn find_element_mut<'a>(elements: &'a mut [Element], id: &str) -> Option<&'a mut Element> {
    elements.iter_mut().find(|e| element_id(e) == id)
}

pub fn find_element_index(elements: &[Element], id: &str) -> Option<usize> {
    elements.iter().position(|e| element_id(e) == id)
}

pub fn get_element_bounds(elem: &Element) -> (f64, f64, f64, f64) {
    let c = elem.common();
    (c.x, c.y, c.width, c.height)
}

pub fn set_element_position(elem: &mut Element, x: f64, y: f64) {
    let c = elem.common_mut();
    c.x = x;
    c.y = y;
}

pub fn scale_element_size(elem: &mut Element, factor: f64) {
    {
        let c = elem.common_mut();
        let ow = c.width;
        let oh = c.height;
        c.width *= factor;
        c.height *= factor;
        c.x -= (c.width - ow) / 2.0;
        c.y -= (c.height - oh) / 2.0;
    }
    match elem {
        Element::Shape(e) => { e.stroke_width *= factor; }
        Element::Text(e) => { e.font_size *= factor; e.stroke_width *= factor; }
        Element::Icon(e) => { e.stroke_width *= factor; }
        Element::Image(_) => {}
        Element::Path(e) => { e.stroke_width *= factor; }
        Element::Group(g) => {
            for child in &mut g.children {
                scale_element_size(child, factor);
            }
        }
        Element::Symbol(_) => {}
    }
}

/// Flip element horizontally or vertically by inverting scale and adjusting position.
pub fn flip_element(elem: &mut Element, direction: &str, canvas_width: f64, canvas_height: f64) {
    let (x, y, w, h) = get_element_bounds(elem);
    match direction {
        "horizontal" => {
            set_element_position(elem, canvas_width - x - w, y);
            reflect_rotation(elem, "horizontal");
        }
        "vertical" => {
            set_element_position(elem, x, canvas_height - y - h);
            reflect_rotation(elem, "vertical");
        }
        _ => {}
    }
    if let Element::Group(g) = elem {
        for child in &mut g.children {
            flip_element(child, direction, w, h);
        }
    }
}

fn reflect_rotation(elem: &mut Element, direction: &str) {
    let current = elem.common().rotation;
    let new_rotation = match direction {
        "horizontal" => (180.0 - current) % 360.0,
        "vertical" => -current,
        _ => return,
    };
    elem.common_mut().rotation = new_rotation;
}

pub fn set_element_gradient(elem: &mut Element, gradient: Gradient) {
    match elem {
        Element::Shape(e) => e.gradient = Some(gradient),
        Element::Text(e) => e.gradient = Some(gradient),
        Element::Icon(e) => e.gradient = Some(gradient),
        Element::Image(_) => {}
        Element::Path(_) => {}
        Element::Group(_) => {}
        Element::Symbol(_) => {}
    }
}

pub fn set_element_shadow(elem: &mut Element, shadow: Shadow) {
    elem.common_mut().shadows = vec![shadow];
}

pub fn clear_element_gradient(elem: &mut Element) {
    match elem {
        Element::Shape(e) => e.gradient = None,
        Element::Text(e) => e.gradient = None,
        Element::Icon(e) => e.gradient = None,
        Element::Image(_) => {}
        Element::Path(_) => {}
        Element::Group(_) => {}
        Element::Symbol(_) => {}
    }
}

pub fn clear_element_shadow(elem: &mut Element) {
    elem.common_mut().shadows.clear();
}

// -- Getters for typed commands --

pub fn get_element_gradient(elem: &Element) -> Option<&Gradient> {
    match elem {
        Element::Shape(e) => e.gradient.as_ref(),
        Element::Text(e) => e.gradient.as_ref(),
        Element::Icon(e) => e.gradient.as_ref(),
        Element::Image(_) => None,
        Element::Path(_) => None,
        Element::Group(_) => None,
        Element::Symbol(_) => None,
    }
}

pub fn get_element_shadow(elem: &Element) -> Option<&Shadow> {
    elem.common().shadows.first()
}

// -- Option-accepting setters for typed commands --

pub fn set_element_gradient_option(elem: &mut Element, gradient: Option<Gradient>) {
    match elem {
        Element::Shape(e) => e.gradient = gradient,
        Element::Text(e) => e.gradient = gradient,
        Element::Icon(e) => e.gradient = gradient,
        Element::Image(_) => {}
        Element::Path(_) => {}
        Element::Group(_) => {}
        Element::Symbol(_) => {}
    }
}

pub fn set_element_shadow_option(elem: &mut Element, shadow: Option<Shadow>) {
    match shadow {
        Some(s) => elem.common_mut().shadows = vec![s],
        None => elem.common_mut().shadows.clear(),
    }
}

pub fn add_element_shadow(elem: &mut Element, shadow: Shadow) {
    elem.common_mut().shadows.push(shadow);
}

pub fn remove_element_shadow(elem: &mut Element, index: usize) -> bool {
    let shadows = &mut elem.common_mut().shadows;
    if index < shadows.len() {
        shadows.remove(index);
        true
    } else {
        false
    }
}

/// Find element by ID, searching recursively into Group children.
/// Returns mutable reference.
pub fn find_element_deep_mut<'a>(elements: &'a mut [Element], id: &str) -> Option<&'a mut Element> {
    for elem in elements.iter_mut() {
        if elem.id() == id {
            return Some(elem);
        }
        if let Element::Group(g) = elem {
            if let Some(found) = find_element_deep_mut(&mut g.children, id) {
                return Some(found);
            }
        }
    }
    None
}

/// Find element by ID with deep search, returning (top_level_index, mutable reference).
/// Useful for callers that need to know which top-level element contains the target.
pub fn find_element_deep_mut_with_index<'a>(
    elements: &'a mut [Element],
    id: &str,
) -> Option<(usize, &'a mut Element)> {
    for (i, elem) in elements.iter_mut().enumerate() {
        if elem.id() == id {
            return Some((i, elem));
        }
        if let Element::Group(g) = elem {
            if let Some(found) = find_element_deep_mut(&mut g.children, id) {
                return Some((i, found));
            }
        }
    }
    None
}

/// Find the parent container (as mutable slice) and index of an element by ID,
/// searching recursively into Group children. Returns (&mut Vec<Element>, index).
/// Useful for reorder operations that need to remove and reinsert within the same parent.
pub fn find_element_parent_mut<'a>(
    elements: &'a mut Vec<Element>,
    id: &str,
) -> Option<(&'a mut Vec<Element>, usize)> {
    if let Some(idx) = elements.iter().position(|e| e.id() == id) {
        return Some((elements, idx));
    }
    for elem in elements.iter_mut() {
        if let Element::Group(g) = elem {
            if let Some(result) = find_element_parent_mut(&mut g.children, id) {
                return Some(result);
            }
        }
    }
    None
}

/// Find element by ID, searching recursively into Group children. Returns immutable reference.
pub fn find_element_deep<'a>(elements: &'a [Element], id: &str) -> Option<&'a Element> {
    for elem in elements.iter() {
        if elem.id() == id {
            return Some(elem);
        }
        if let Element::Group(g) = elem {
            if let Some(found) = find_element_deep(&g.children, id) {
                return Some(found);
            }
        }
    }
    None
}

/// Insert a cloned element right after the element with the given ID,
/// searching recursively into Group children.
pub fn insert_element_after(elements: &mut Vec<Element>, target_id: &str, new_elem: Element) {
    if let Some(idx) = elements.iter().position(|e| e.id() == target_id) {
        elements.insert(idx + 1, new_elem);
        return;
    }
    for elem in elements.iter_mut() {
        if let Element::Group(g) = elem {
            insert_element_after(&mut g.children, target_id, new_elem);
            return;
        }
    }
}

/// Calculate the bounding box of a group from its children.
/// Returns (x, y, width, height).
pub fn calc_group_bounds(children: &[Element]) -> (f64, f64, f64, f64) {
    if children.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    for child in children {
        let c = child.common();
        min_x = min_x.min(c.x);
        min_y = min_y.min(c.y);
        max_x = max_x.max(c.x + c.width);
        max_y = max_y.max(c.y + c.height);
    }
    (min_x, min_y, max_x - min_x, max_y - min_y)
}

/// Recompute `natural_width` / `natural_height` and re-normalise the `d`
/// attribute so the path starts at (0, 0).
///
/// Parses the `d` string for coordinate numbers, finds the bounding box,
/// shifts every coordinate by `(-min_x, -min_y)`, and rewrites `d`.
pub fn recompute_path_natural_dims(elem: &mut Element) {
    if let Element::Path(pe) = elem {
        let (min_x, min_y, max_x, max_y) = parse_path_bbox(&pe.d);
        let nw = max_x - min_x;
        let nh = max_y - min_y;
        if nw <= 0.0 || nh <= 0.0 {
            return;
        }
        if min_x.abs() < 0.01 && min_y.abs() < 0.01 {
            pe.natural_width = nw;
            pe.natural_height = nh;
            return;
        }
        let shifted = shift_path_coords(&pe.d, min_x, min_y);
        pe.d = shifted;
        pe.natural_width = nw;
        pe.natural_height = nh;
    }
}

/// Walk a `d` attribute string and return (min_x, min_y, max_x, max_y).
/// Only considers numbers that appear after move/line/curve commands.
fn parse_path_bbox(d: &str) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    let tokens = tokenize_d(d);
    let mut i = 0;
    let mut cursor_x = 0.0_f64;
    let mut cursor_y = 0.0_f64;
    while i < tokens.len() {
        let t = &tokens[i];
        match t.as_str() {
            "M" | "m" | "L" | "l" | "T" | "t"
                if i + 2 <= tokens.len() => {
                    if let (Some(x), Some(y)) = (parse_f64(&tokens[i + 1]), parse_f64(&tokens[i + 2])) {
                        let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                        let (ax, ay) = if is_rel {
                            (cursor_x + x, cursor_y + y)
                        } else {
                            (x, y)
                        };
                        min_x = min_x.min(ax);
                        min_y = min_y.min(ay);
                        max_x = max_x.max(ax);
                        max_y = max_y.max(ay);
                        cursor_x = ax;
                        cursor_y = ay;
                    }
                    i += 3;
                    continue;
                }
            "C" | "c" => {
                for j in 0..3 {
                    if i + 2 + j * 2 < tokens.len() {
                        if let (Some(x), Some(y)) = (parse_f64(&tokens[i + 1 + j * 2]), parse_f64(&tokens[i + 2 + j * 2])) {
                            let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                            let (ax, ay) = if is_rel {
                                (cursor_x + x, cursor_y + y)
                            } else {
                                (x, y)
                            };
                            min_x = min_x.min(ax);
                            min_y = min_y.min(ay);
                            max_x = max_x.max(ax);
                            max_y = max_y.max(ay);
                        }
                    }
                }
                if let (Some(x), Some(y)) = (parse_f64(tokens.get(i + 5).unwrap_or(&"".to_string())), parse_f64(tokens.get(i + 6).unwrap_or(&"".to_string()))) {
                    let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                    if is_rel { cursor_x += x; cursor_y += y; } else { cursor_x = x; cursor_y = y; }
                }
                i += 7;
                continue;
            }
            "S" | "s" | "Q" | "q" => {
                for j in 0..2 {
                    let idx = i + 1 + j * 2;
                    if idx + 1 < tokens.len() {
                        if let (Some(x), Some(y)) = (parse_f64(&tokens[idx]), parse_f64(&tokens[idx + 1])) {
                            let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                            let (ax, ay) = if is_rel {
                                (cursor_x + x, cursor_y + y)
                            } else {
                                (x, y)
                            };
                            min_x = min_x.min(ax);
                            min_y = min_y.min(ay);
                            max_x = max_x.max(ax);
                            max_y = max_y.max(ay);
                        }
                    }
                }
                if let (Some(x), Some(y)) = (parse_f64(tokens.get(i + 3).unwrap_or(&"".to_string())), parse_f64(tokens.get(i + 4).unwrap_or(&"".to_string()))) {
                    let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                    if is_rel { cursor_x += x; cursor_y += y; } else { cursor_x = x; cursor_y = y; }
                }
                i += 5;
                continue;
            }
            "A" | "a" => {
                if i + 7 <= tokens.len() {
                    if let (Some(x), Some(y)) = (parse_f64(&tokens[i + 5]), parse_f64(&tokens[i + 6])) {
                        let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                        let (ax, ay) = if is_rel {
                            (cursor_x + x, cursor_y + y)
                        } else {
                            (x, y)
                        };
                        min_x = min_x.min(ax);
                        min_y = min_y.min(ay);
                        max_x = max_x.max(ax);
                        max_y = max_y.max(ay);
                        cursor_x = ax;
                        cursor_y = ay;
                    }
                }
                i += 7;
                continue;
            }
            "H" | "h" => {
                if i + 1 < tokens.len() {
                    if let Some(x) = parse_f64(&tokens[i + 1]) {
                        let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                        let ax = if is_rel { cursor_x + x } else { x };
                        min_x = min_x.min(ax);
                        max_x = max_x.max(ax);
                        cursor_x = ax;
                    }
                }
                i += 2;
                // Handle implicit repeated coordinates
                while i < tokens.len() {
                    let tok = &tokens[i];
                    if tok.chars().count() == 1 && tok.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
                        break;
                    }
                    if let Some(val) = parse_f64(tok) {
                        let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                        let ax = if is_rel { cursor_x + val } else { val };
                        min_x = min_x.min(ax);
                        max_x = max_x.max(ax);
                        cursor_x = ax;
                    }
                    i += 1;
                }
                continue;
            }
            "V" | "v" => {
                if i + 1 < tokens.len() {
                    if let Some(y) = parse_f64(&tokens[i + 1]) {
                        let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                        let ay = if is_rel { cursor_y + y } else { y };
                        min_y = min_y.min(ay);
                        max_y = max_y.max(ay);
                        cursor_y = ay;
                    }
                }
                i += 2;
                // Handle implicit repeated coordinates
                while i < tokens.len() {
                    let tok = &tokens[i];
                    if tok.chars().count() == 1 && tok.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
                        break;
                    }
                    if let Some(val) = parse_f64(tok) {
                        let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                        let ay = if is_rel { cursor_y + val } else { val };
                        min_y = min_y.min(ay);
                        max_y = max_y.max(ay);
                        cursor_y = ay;
                    }
                    i += 1;
                }
                continue;
            }
            "Z" | "z" => { i += 1; continue; }
            _ => {}
        }
        i += 1;
    }

    if min_x == f64::MAX {
        (0.0, 0.0, 0.0, 0.0)
    } else {
        (min_x, min_y, max_x, max_y)
    }
}

/// Shift all coordinate numbers in a `d` string by `(-ox, -oy)`.
pub fn shift_path_coords(d: &str, ox: f64, oy: f64) -> String {
    let tokens = tokenize_d(d);
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];
        match t.as_str() {
            "M" | "m" | "L" | "l" | "T" | "t" => {
                out.push(t.clone());
                if i + 2 < tokens.len() {
                    if let (Some(x), Some(y)) = (parse_f64(&tokens[i + 1]), parse_f64(&tokens[i + 2])) {
                        let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                        if !is_rel {
                            out.push(format!("{:.2}", x - ox));
                            out.push(format!("{:.2}", y - oy));
                        } else {
                            out.push(format!("{:.2}", x));
                            out.push(format!("{:.2}", y));
                        }
                        i += 3;
                        continue;
                    }
                }
            }
            "C" | "c" => {
                out.push(t.clone());
                let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                for j in 0..3 {
                    let idx1 = i + 1 + j * 2;
                    let idx2 = i + 2 + j * 2;
                    if idx2 < tokens.len() {
                        if let (Some(x), Some(y)) = (parse_f64(&tokens[idx1]), parse_f64(&tokens[idx2])) {
                            if !is_rel {
                                out.push(format!("{:.2}", x - ox));
                                out.push(format!("{:.2}", y - oy));
                            } else {
                                out.push(format!("{:.2}", x));
                                out.push(format!("{:.2}", y));
                            }
                        }
                    }
                }
                i += 7;
                continue;
            }
            "S" | "s" | "Q" | "q" => {
                out.push(t.clone());
                let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                for j in 0..2 {
                    let idx1 = i + 1 + j * 2;
                    let idx2 = i + 2 + j * 2;
                    if idx2 < tokens.len() {
                        if let (Some(x), Some(y)) = (parse_f64(&tokens[idx1]), parse_f64(&tokens[idx2])) {
                            if !is_rel {
                                out.push(format!("{:.2}", x - ox));
                                out.push(format!("{:.2}", y - oy));
                            } else {
                                out.push(format!("{:.2}", x));
                                out.push(format!("{:.2}", y));
                            }
                        }
                    }
                }
                i += 5;
                continue;
            }
            "A" | "a" => {
                out.push(t.clone());
                for j in 1..=4 {
                    if i + j < tokens.len() {
                        out.push(tokens[i + j].clone());
                    }
                }
                if i + 6 < tokens.len() {
                    if let (Some(x), Some(y)) = (parse_f64(&tokens[i + 5]), parse_f64(&tokens[i + 6])) {
                        let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                        if !is_rel {
                            out.push(format!("{:.2}", x - ox));
                            out.push(format!("{:.2}", y - oy));
                        } else {
                            out.push(format!("{:.2}", x));
                            out.push(format!("{:.2}", y));
                        }
                    }
                }
                i += 7;
                continue;
            }
            "H" | "h" => {
                out.push(t.clone());
                if i + 1 < tokens.len() {
                    if let Some(x) = parse_f64(&tokens[i + 1]) {
                        let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                        if !is_rel {
                            out.push(format!("{:.2}", x - ox));
                        } else {
                            out.push(format!("{:.2}", x));
                        }
                    }
                }
                i += 2;
                continue;
            }
            "V" | "v" => {
                out.push(t.clone());
                if i + 1 < tokens.len() {
                    if let Some(y) = parse_f64(&tokens[i + 1]) {
                        let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                        if !is_rel {
                            out.push(format!("{:.2}", y - oy));
                        } else {
                            out.push(format!("{:.2}", y));
                        }
                    }
                }
                i += 2;
                continue;
            }
            "Z" | "z" => {
                out.push(t.clone());
                i += 1;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    out.join(" ")
}

/// Scale all coordinate numbers in a `d` string by `(sx, sy)`.
pub fn scale_path_coords(d: &str, sx: f64, sy: f64) -> String {
    let tokens = tokenize_d(d);
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];
        match t.as_str() {
            "M" | "m" | "L" | "l" | "T" | "t" => {
                out.push(t.clone());
                if i + 2 < tokens.len() {
                    if let (Some(x), Some(y)) = (parse_f64(&tokens[i + 1]), parse_f64(&tokens[i + 2])) {
                        let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                        if !is_rel {
                            out.push(format!("{:.2}", x * sx));
                            out.push(format!("{:.2}", y * sy));
                        } else {
                            out.push(format!("{:.2}", x));
                            out.push(format!("{:.2}", y));
                        }
                        i += 3;
                        continue;
                    }
                }
            }
            "C" | "c" => {
                out.push(t.clone());
                let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                for j in 0..3 {
                    let idx1 = i + 1 + j * 2;
                    let idx2 = i + 2 + j * 2;
                    if idx2 < tokens.len() {
                        if let (Some(x), Some(y)) = (parse_f64(&tokens[idx1]), parse_f64(&tokens[idx2])) {
                            if !is_rel {
                                out.push(format!("{:.2}", x * sx));
                                out.push(format!("{:.2}", y * sy));
                            } else {
                                out.push(format!("{:.2}", x));
                                out.push(format!("{:.2}", y));
                            }
                        }
                    }
                }
                i += 7;
                continue;
            }
            "S" | "s" | "Q" | "q" => {
                out.push(t.clone());
                let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                for j in 0..2 {
                    let idx1 = i + 1 + j * 2;
                    let idx2 = i + 2 + j * 2;
                    if idx2 < tokens.len() {
                        if let (Some(x), Some(y)) = (parse_f64(&tokens[idx1]), parse_f64(&tokens[idx2])) {
                            if !is_rel {
                                out.push(format!("{:.2}", x * sx));
                                out.push(format!("{:.2}", y * sy));
                            } else {
                                out.push(format!("{:.2}", x));
                                out.push(format!("{:.2}", y));
                            }
                        }
                    }
                }
                i += 5;
                continue;
            }
            "A" | "a" => {
                out.push(t.clone());
                let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                if i + 7 <= tokens.len() {
                    if let (Some(rx), Some(ry)) = (parse_f64(&tokens[i + 1]), parse_f64(&tokens[i + 2])) {
                        out.push(format!("{:.2}", rx * sx));
                        out.push(format!("{:.2}", ry * sy));
                    } else {
                        out.push(tokens[i + 1].clone());
                        out.push(tokens[i + 2].clone());
                    }
                    out.push(tokens[i + 3].clone());
                    out.push(tokens[i + 4].clone());
                    out.push(tokens[i + 5].clone());
                    if let (Some(x), Some(y)) = (parse_f64(&tokens[i + 6]), parse_f64(&tokens[i + 7])) {
                        if !is_rel {
                            out.push(format!("{:.2}", x * sx));
                            out.push(format!("{:.2}", y * sy));
                        } else {
                            out.push(format!("{:.2}", x));
                            out.push(format!("{:.2}", y));
                        }
                    } else {
                        out.push(tokens[i + 6].clone());
                        out.push(tokens[i + 7].clone());
                    }
                }
                i += 8;
                continue;
            }
            "H" | "h" => {
                out.push(t.clone());
                if i + 1 < tokens.len() {
                    if let Some(x) = parse_f64(&tokens[i + 1]) {
                        let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                        if !is_rel {
                            out.push(format!("{:.2}", x * sx));
                        } else {
                            out.push(format!("{:.2}", x));
                        }
                    }
                }
                i += 2;
                continue;
            }
            "V" | "v" => {
                out.push(t.clone());
                if i + 1 < tokens.len() {
                    if let Some(y) = parse_f64(&tokens[i + 1]) {
                        let is_rel = t.chars().next().map(|c| c.is_lowercase()).unwrap_or(false);
                        if !is_rel {
                            out.push(format!("{:.2}", y * sy));
                        } else {
                            out.push(format!("{:.2}", y));
                        }
                    }
                }
                i += 2;
                continue;
            }
            "Z" | "z" => {
                out.push(t.clone());
                i += 1;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    out.join(" ")
}

pub fn tokenize_d(d: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in d.chars() {
        match ch {
            'M' | 'm' | 'L' | 'l' | 'C' | 'c' | 'Q' | 'q' | 'S' | 's'
            | 'T' | 't' | 'A' | 'a' | 'H' | 'h' | 'V' | 'v' | 'Z' | 'z' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push(ch.to_string());
            }
            ' ' | ',' | '\t' | '\n' | '\r' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            '-' => {
                if !current.is_empty() && !current.ends_with('e') && !current.ends_with('E') {
                    tokens.push(current.clone());
                    current.clear();
                }
                current.push(ch);
            }
            '+' => {
                if !current.is_empty() && !current.ends_with('e') && !current.ends_with('E') {
                    tokens.push(current.clone());
                    current.clear();
                }
                current.push(ch);
            }
            '.' if current.contains('.') => {
                tokens.push(current.clone());
                current.clear();
                current.push(ch);
            }
            _ => {
                current.push(ch);
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn parse_f64(s: &str) -> Option<f64> {
    s.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CommonProps, GroupElement, ShapeElement};

    fn make_shape_element(id: &str) -> Element {
        Element::Shape(ShapeElement {
            common: CommonProps {
                id: id.to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 200.0,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            shape_type: ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        })
    }

    #[test]
    fn test_parse_shape_type_valid() {
        assert!(matches!(parse_shape_type("circle"), Ok(ShapeType::Circle)));
        assert!(matches!(parse_shape_type("rect"), Ok(ShapeType::Rect)));
        assert!(matches!(parse_shape_type("rounded-rect"), Ok(ShapeType::RoundedRect)));
        assert!(matches!(parse_shape_type("hexagon"), Ok(ShapeType::Hexagon)));
        assert!(matches!(parse_shape_type("star"), Ok(ShapeType::Star)));
        assert!(matches!(parse_shape_type("shield"), Ok(ShapeType::Shield)));
        assert!(matches!(parse_shape_type("diamond"), Ok(ShapeType::Diamond)));
    }

    #[test]
    fn test_parse_shape_type_invalid() {
        assert!(parse_shape_type("foobar").is_err());
        assert!(parse_shape_type("").is_err());
    }

    #[test]
    fn test_find_element_mut_found() {
        let mut elems = vec![make_shape_element("shape-1"), make_shape_element("shape-2")];
        let found = find_element_mut(&mut elems, "shape-2");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), "shape-2");
    }

    #[test]
    fn test_find_element_mut_not_found() {
        let mut elems = vec![make_shape_element("shape-1")];
        assert!(find_element_mut(&mut elems, "shape-99").is_none());
    }

    #[test]
    fn test_find_element_index_found() {
        let elems = vec![make_shape_element("shape-1"), make_shape_element("shape-2")];
        assert_eq!(find_element_index(&elems, "shape-1"), Some(0));
        assert_eq!(find_element_index(&elems, "shape-2"), Some(1));
    }

    #[test]
    fn test_find_element_index_not_found() {
        let elems = vec![make_shape_element("shape-1")];
        assert_eq!(find_element_index(&elems, "shape-99"), None);
    }

    #[test]
    fn test_get_element_bounds() {
        let elem = make_shape_element("shape-1");
        let (x, y, w, h) = get_element_bounds(&elem);
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
        assert_eq!(w, 100.0);
        assert_eq!(h, 200.0);
    }

    #[test]
    fn test_set_element_position() {
        let mut elem = make_shape_element("shape-1");
        set_element_position(&mut elem, 50.0, 60.0);
        let (x, y, _, _) = get_element_bounds(&elem);
        assert_eq!(x, 50.0);
        assert_eq!(y, 60.0);
    }

    #[test]
    fn test_scale_element_size() {
        let mut elem = make_shape_element("shape-1");
        let (orig_x, orig_y, orig_w, orig_h) = get_element_bounds(&elem);
        scale_element_size(&mut elem, 2.0);
        let (new_x, new_y, new_w, new_h) = get_element_bounds(&elem);
        assert_eq!(new_w, orig_w * 2.0);
        assert_eq!(new_h, orig_h * 2.0);
        assert_eq!(new_x, orig_x - (new_w - orig_w) / 2.0);
        assert_eq!(new_y, orig_y - (new_h - orig_h) / 2.0);
    }

    #[test]
    fn test_flip_element_horizontal() {
        let mut elem = make_shape_element("shape-1");
        let canvas_w = 512.0;
        let canvas_h = 512.0;
        let (x, y, w, _h) = get_element_bounds(&elem);
        flip_element(&mut elem, "horizontal", canvas_w, canvas_h);
        let (nx, ny, _, _) = get_element_bounds(&elem);
        assert_eq!(nx, canvas_w - x - w);
        assert_eq!(ny, y);
    }

    #[test]
    fn test_flip_element_vertical() {
        let mut elem = make_shape_element("shape-1");
        let canvas_w = 512.0;
        let canvas_h = 512.0;
        let (x, y, _w, h) = get_element_bounds(&elem);
        flip_element(&mut elem, "vertical", canvas_w, canvas_h);
        let (nx, ny, _, _) = get_element_bounds(&elem);
        assert_eq!(nx, x);
        assert_eq!(ny, canvas_h - y - h);
    }

    #[test]
    fn test_set_and_get_element_gradient() {
        let mut elem = make_shape_element("shape-1");
        assert!(get_element_gradient(&elem).is_none());
        let grad = Gradient {
            gradient_type: crate::model::GradientKind::Linear,
            colors: vec!["#FF0000".to_string(), "#0000FF".to_string()],
            angle: 90.0,
            stops: vec![],
        };
        set_element_gradient(&mut elem, grad);
        assert!(get_element_gradient(&elem).is_some());
    }

    #[test]
    fn test_set_and_get_element_shadow() {
        let mut elem = make_shape_element("shape-1");
        assert!(get_element_shadow(&elem).is_none());
        let shadow = Shadow {
            color: "#00000040".to_string(),
            blur: 8.0,
            offset_x: 0.0,
            offset_y: 4.0,
            inset: false,
        };
        set_element_shadow(&mut elem, shadow);
        let s = get_element_shadow(&elem).unwrap();
        assert_eq!(s.blur, 8.0);
    }

    #[test]
    fn test_clear_element_gradient() {
        let mut elem = make_shape_element("shape-1");
        let grad = Gradient {
            gradient_type: crate::model::GradientKind::Linear,
            colors: vec!["#FF0000".to_string()],
            angle: 0.0,
            stops: vec![],
        };
        set_element_gradient(&mut elem, grad);
        assert!(get_element_gradient(&elem).is_some());
        clear_element_gradient(&mut elem);
        assert!(get_element_gradient(&elem).is_none());
    }

    #[test]
    fn test_clear_element_shadow() {
        let mut elem = make_shape_element("shape-1");
        set_element_shadow(&mut elem, Shadow::default());
        assert!(get_element_shadow(&elem).is_some());
        clear_element_shadow(&mut elem);
        assert!(get_element_shadow(&elem).is_none());
    }

    #[test]
    fn test_find_element_deep_mut_in_group() {
        let child1 = make_shape_element("shape-10");
        let child2 = make_shape_element("shape-20");
        let group = GroupElement {
            common: CommonProps {
                id: "group-1".to_string(),
                x: 0.0, y: 0.0, width: 200.0, height: 200.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            children: vec![child1, child2],
            expanded: false,
        };
        let mut elems = vec![Element::Group(group), make_shape_element("shape-99")];

        let found = find_element_deep_mut(&mut elems, "shape-20");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), "shape-20");

        let found_top = find_element_deep_mut(&mut elems, "shape-99");
        assert!(found_top.is_some());
        assert_eq!(found_top.unwrap().id(), "shape-99");

        let not_found = find_element_deep_mut(&mut elems, "nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_calc_group_bounds() {
        let c1 = make_shape_element_with_pos("a", 10.0, 20.0, 50.0, 60.0);
        let c2 = make_shape_element_with_pos("b", 100.0, 200.0, 80.0, 90.0);
        let (x, y, w, h) = calc_group_bounds(&[c1, c2]);
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
        assert_eq!(w, 170.0);
        assert_eq!(h, 270.0);
    }

    #[test]
    fn test_calc_group_bounds_empty() {
        let (x, y, w, h) = calc_group_bounds(&[]);
        assert_eq!((x, y, w, h), (0.0, 0.0, 0.0, 0.0));
    }

    fn make_shape_element_with_pos(id: &str, x: f64, y: f64, w: f64, h: f64) -> Element {
        Element::Shape(ShapeElement {
            common: CommonProps {
                id: id.to_string(),
                x, y, width: w, height: h,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            shape_type: ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        })
    }
}

pub fn set_element_filter(elem: &mut Element, filter: SvgFilter) {
    elem.common_mut().svg_filter = Some(filter);
}

pub fn clear_element_filter(elem: &mut Element) {
    elem.common_mut().svg_filter = None;
}

pub fn set_element_filter_option(elem: &mut Element, filter: Option<SvgFilter>) {
    elem.common_mut().svg_filter = filter;
}

pub fn get_element_filter(elem: &Element) -> Option<&SvgFilter> {
    elem.common().svg_filter.as_ref()
}
