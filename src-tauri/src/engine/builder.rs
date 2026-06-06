use crate::colors;
use crate::engine::filter::filter_to_svg;
use crate::engine::utils::{escape_xml, escape_xml_attr};
use crate::error::AppError;
use crate::icons;
use crate::model::shapes::shape_to_svg;
use crate::model::symbol::apply_overrides;
use crate::model::{Element, GradientKind, GroupElement, IconProject, AnimationType};

/// Post-process SVG string for cleaner output.
fn optimize_svg(svg: &str) -> String {
    let mut result = svg.to_string();

    // Remove empty <defs></defs>
    result = result.replace("<defs></defs>", "");

    // Collapse consecutive whitespace into a single space
    let mut cleaned = String::with_capacity(result.len());
    let mut prev_space = false;
    for ch in result.chars() {
        if ch == ' ' || ch == '\n' || ch == '\r' || ch == '\t' {
            if !prev_space {
                cleaned.push(' ');
                prev_space = true;
            }
        } else {
            cleaned.push(ch);
            prev_space = false;
        }
    }

    // Remove spaces between structural tags (but not inside text content)
    let mut final_result = String::with_capacity(cleaned.len());
    let mut search_from = 0;
    while let Some(pos) = cleaned[search_from..].find("> <") {
        let abs_pos = search_from + pos;
        let before = &cleaned[..abs_pos + 1];
        if before.rfind("<text").is_none_or(|tp| {
            before.rfind("</text>").is_none_or(|cp| cp < tp)
        }) {
            final_result.push_str(&cleaned[search_from..abs_pos + 1]);
        } else {
            final_result.push_str(&cleaned[search_from..abs_pos + 2]);
        }
        search_from = abs_pos + 2;
    }
    final_result.push_str(&cleaned[search_from..]);
    final_result
}

pub fn build(project: &IconProject) -> Result<String, AppError> {
    let canvas = project.active_canvas();
    let elements = project.active_elements();
    let w = canvas.width;
    let h = canvas.height;
    let mut svg = String::with_capacity(4096);

    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#,
        w = w, h = h,
    ));

    let mut defs = String::new();
    let has_corner_radius = canvas.corner_radius > 0;

    if has_corner_radius {
        let rx = (w as f64) * (canvas.corner_radius as f64) / 100.0;
        let ry = (h as f64) * (canvas.corner_radius as f64) / 100.0;
        defs.push_str(&format!(
            r#"<clipPath id="app-icon-clip"><rect width="{w}" height="{h}" rx="{rx:.2}" ry="{ry:.2}"/></clipPath>"#,
            w = w, h = h, rx = rx, ry = ry,
        ));
    }

    let canvas_gradient_id = "canvas-bg-gradient";
    if let Some(ref g) = canvas.background_gradient {
        defs.push_str(&gradient_def_with_id(canvas_gradient_id, g));
    }

    for elem in elements {
        if !elem.common().visible { continue; }
        collect_defs(elem, &mut defs, &project.symbols);
    }

    collect_clip_mask_defs(elements, elements, &mut defs);

    if !defs.is_empty() {
        svg.push_str(&format!("<defs>{}</defs>", defs));
    }

    if has_corner_radius {
        svg.push_str(r#"<g clip-path="url(#app-icon-clip)">"#);
    }

    if canvas.background != "transparent" || canvas.background_gradient.is_some() {
        let fill = if canvas.background_gradient.is_some() {
            format!("url(#{})", canvas_gradient_id)
        } else {
            escape_xml_attr(&canvas.background)
        };
        svg.push_str(&format!(
            r#"<rect width="{w}" height="{h}" fill="{bg}"/>"#,
            w = w, h = h, bg = fill,
        ));
    }

    for elem in elements {
        if !elem.common().visible { continue; }
        svg.push_str(&render_element(elem, &project.symbols)?);
    }

    if has_corner_radius {
        svg.push_str("</g>");
    }
    svg.push_str("</svg>");

    Ok(optimize_svg(&svg))
}

fn collect_defs(elem: &Element, defs: &mut String, symbols: &std::collections::HashMap<String, crate::model::SymbolDef>) {
    let id = elem.id();
    match elem {
        Element::Symbol(inst) => {
            if let Some(def) = symbols.get(&inst.symbol_id) {
                let mut resolved = def.source_element.clone();
                apply_overrides(&mut resolved, &inst.overrides);
                collect_defs(&resolved, defs, symbols);
            }
            // Also collect defs for the instance's own common props
            for (i, s) in inst.common.shadows.iter().enumerate() {
                defs.push_str(&shadow_def_for(id, s, i));
            }
            if let Some(ref f) = inst.common.svg_filter {
                let filter_id = id.rsplit('-').next().and_then(|n| n.parse::<usize>().ok()).unwrap_or(0);
                defs.push_str(&filter_to_svg(f, filter_id));
            }
        }
        _ => {
            let gradient = match elem {
                Element::Shape(e) => e.gradient.as_ref(),
                Element::Text(e) => e.gradient.as_ref(),
                Element::Icon(e) => e.gradient.as_ref(),
                _ => None,
            };
            if let Some(g) = gradient {
                defs.push_str(&gradient_def_for(id, g));
            }

            for (i, s) in elem.common().shadows.iter().enumerate() {
                defs.push_str(&shadow_def_for(id, s, i));
            }

            if let Some(ref f) = elem.common().svg_filter {
                let filter_id = id.rsplit('-').next().and_then(|n| n.parse::<usize>().ok()).unwrap_or(0);
                defs.push_str(&filter_to_svg(f, filter_id));
            }

            if let Element::Group(g) = elem {
                for child in &g.children {
                    collect_defs(child, defs, symbols);
                }
            }
        }
    }
}

fn gradient_def_for(id: &str, gradient: &crate::model::Gradient) -> String {
    let def_id = format!("gradient-{}", id);
    gradient_def_with_id(&def_id, gradient)
}

fn gradient_def_with_id(id: &str, gradient: &crate::model::Gradient) -> String {
    let stops = gradient_stops(&gradient.colors, &gradient.stops);

    match gradient.gradient_type {
        GradientKind::Linear => format!(
            r#"<linearGradient id="{id}" gradientTransform="rotate({angle:.2}, 0.5, 0.5)">{stops}</linearGradient>"#,
            id = id, angle = gradient.angle, stops = stops,
        ),
        GradientKind::Radial => format!(
            r#"<radialGradient id="{id}">{stops}</radialGradient>"#,
            id = id, stops = stops,
        ),
    }
}

fn gradient_stops(colors: &[String], positions: &[f64]) -> String {
    let n = colors.len();
    colors
        .iter()
        .enumerate()
        .map(|(i, color)| {
            let offset = if !positions.is_empty() && i < positions.len() {
                (positions[i] * 100.0).clamp(0.0, 100.0)
            } else if n > 1 {
                (i as f64) / ((n - 1) as f64) * 100.0
            } else {
                0.0
            };
            format!(r#"<stop offset="{:.4}%" stop-color="{}"/>"#, offset, escape_xml_attr(color))
        })
        .collect()
}

fn shadow_def_for(id: &str, shadow: &crate::model::Shadow, index: usize) -> String {
    let def_id = format!("shadow-{}-{}", id, index);
    let (color, opacity) = colors::split_hex_color_with_alpha(&shadow.color);

    if shadow.inset {
        if (opacity - 1.0).abs() > f64::EPSILON {
            format!(
                r#"<filter id="{id}" x="-50%" y="-50%" width="200%" height="200%"><feFlood flood-color="{color}" flood-opacity="{opacity:.4}" result="flood"/><feComposite in="flood" in2="SourceGraphic" operator="in" result="masked"/><feGaussianBlur in="masked" stdDeviation="{blur:.2}" result="blurred"/><feOffset dx="{dx:.2}" dy="{dy:.2}" in="blurred" result="offset"/><feComposite in="SourceGraphic" in2="offset" operator="over"/></filter>"#,
                id = def_id, color = escape_xml_attr(&color), opacity = opacity, blur = shadow.blur, dx = shadow.offset_x, dy = shadow.offset_y,
            )
        } else {
            format!(
                r#"<filter id="{id}" x="-50%" y="-50%" width="200%" height="200%"><feFlood flood-color="{color}" result="flood"/><feComposite in="flood" in2="SourceGraphic" operator="in" result="masked"/><feGaussianBlur in="masked" stdDeviation="{blur:.2}" result="blurred"/><feOffset dx="{dx:.2}" dy="{dy:.2}" in="blurred" result="offset"/><feComposite in="SourceGraphic" in2="offset" operator="over"/></filter>"#,
                id = def_id, color = escape_xml_attr(&color), blur = shadow.blur, dx = shadow.offset_x, dy = shadow.offset_y,
            )
        }
    } else {
        if (opacity - 1.0).abs() > f64::EPSILON {
            format!(
                r#"<filter id="{id}"><feDropShadow dx="{dx:.2}" dy="{dy:.2}" stdDeviation="{blur:.2}" flood-color="{color}" flood-opacity="{opacity:.4}"/></filter>"#,
                id = def_id, dx = shadow.offset_x, dy = shadow.offset_y, blur = shadow.blur, color = escape_xml_attr(&color), opacity = opacity,
            )
        } else {
            format!(
                r#"<filter id="{id}"><feDropShadow dx="{dx:.2}" dy="{dy:.2}" stdDeviation="{blur:.2}" flood-color="{color}"/></filter>"#,
                id = def_id, dx = shadow.offset_x, dy = shadow.offset_y, blur = shadow.blur, color = escape_xml_attr(&color),
            )
        }
    }
}

fn render_element(elem: &Element, symbols: &std::collections::HashMap<String, crate::model::SymbolDef>) -> Result<String, AppError> {
    let inner = match elem {
        Element::Shape(e) => Ok(render_shape(e)),
        Element::Text(e) => Ok(render_text(e)),
        Element::Icon(e) => render_icon(e),
        Element::Image(e) => Ok(render_image(e)),
        Element::Path(e) => Ok(render_path(e)),
        Element::Group(e) => render_group(e, symbols),
        Element::Symbol(inst) => {
            let id = &inst.common.id;
            match symbols.get(&inst.symbol_id) {
                Some(def) => {
                    let mut resolved = def.source_element.clone();
                    apply_overrides(&mut resolved, &inst.overrides);
                    let inner_svg = render_element(&resolved, symbols)?;
                    Ok(format!("<g id=\"instance-{}\">{}</g>", id, inner_svg))
                }
                None => Ok(format!("<!-- symbol {} not found -->", inst.symbol_id)),
            }
        }
    }?;

    let result = if let Some(ref anim) = elem.common().animation {
        wrap_with_animation(&inner, anim, elem.common())
    } else {
        inner
    };

    let result = if let Some(ref mode) = elem.common().blend_mode {
        format!("<g style=\"mix-blend-mode:{}\">{}</g>", escape_xml_attr(mode), result)
    } else {
        result
    };

    let result = if let Some(ref clip_id) = elem.common().clip_element_id {
        format!("<g clip-path=\"url(#clip-{})\">{}</g>", escape_xml_attr(clip_id), result)
    } else {
        result
    };

    let result = if let Some(ref mask_id) = elem.common().mask_element_id {
        format!("<g mask=\"url(#mask-{})\">{}</g>", escape_xml_attr(mask_id), result)
    } else {
        result
    };

    let result = if elem.common().svg_filter.is_some() {
        let elem_id = elem.id();
        let filter_id = elem_id.rsplit('-').next().and_then(|n| n.parse::<usize>().ok()).unwrap_or(0);
        format!("<g filter=\"url(#filter-{})\">{}</g>", filter_id, result)
    } else {
        result
    };

    Ok(result)
}

fn wrap_with_animation(svg: &str, anim: &crate::model::Animation, common: &crate::model::CommonProps) -> String {
    let repeat = if anim.repeat { "indefinite".to_string() } else { "1".to_string() };
    let dur = format!("{:.2}s", anim.duration);
    let begin = if anim.delay > 0.0 {
        format!(" begin=\"{:.2}s\"", anim.delay)
    } else {
        String::new()
    };
    let easing_attr = if anim.easing != "ease-in-out" && anim.easing != "linear" && !anim.easing.is_empty() {
        let values = match anim.easing.as_str() {
            "ease" => "0.25 0.1 0.25 1",
            "ease-in" => "0.42 0 1 1",
            "ease-out" => "0 0 0.58 1",
            _ => "0.42 0 0.58 1",
        };
        format!(" calcMode=\"spline\" keySplines=\"{}\" keyTimes=\"0;1\"", values)
    } else if anim.easing == "linear" {
        " calcMode=\"linear\"".to_string()
    } else {
        String::new()
    };

    let cx = common.x + common.width / 2.0;
    let cy = common.y + common.height / 2.0;

    let anim_tag = match anim.animation_type {
        AnimationType::Rotate => {
            format!(
                r#"<animateTransform attributeName="transform" type="rotate" from="0 {:.2} {:.2}" to="360 {:.2} {:.2}" dur="{}" repeatCount="{}"{}{}/>"#,
                cx, cy, cx, cy, dur, repeat, begin, easing_attr,
            )
        }
        AnimationType::Scale => {
            let min_scale = anim.params.get("min_scale")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.8);
            format!(
                r#"<animateTransform attributeName="transform" type="scale" from="1" to="{:.2}" dur="{}" repeatCount="{}"{}{}/>"#,
                min_scale, dur, repeat, begin, easing_attr,
            )
        }
        AnimationType::Fade => {
            let min_opacity = anim.params.get("min_opacity")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!(
                r#"<animate attributeName="opacity" values="{:.2};{:.2};{:.2}" dur="{}" repeatCount="{}"{}{}/>"#,
                common.opacity, min_opacity, common.opacity, dur, repeat, begin, easing_attr,
            )
        }
        AnimationType::Translate => {
            let dx = anim.params.get("dx")
                .and_then(|v| v.as_f64())
                .unwrap_or(10.0);
            let dy = anim.params.get("dy")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!(
                r#"<animateTransform attributeName="transform" type="translate" values="0 0;{:.2} {:.2};0 0" dur="{}" repeatCount="{}"{}{}/>"#,
                dx, dy, dur, repeat, begin, easing_attr,
            )
        }
        AnimationType::Path => {
            format!(
                r##"<animateMotion dur="{}" repeatCount="{}"{}{}><mpath href="#motion-path-{}"/></animateMotion>"##,
                dur, repeat, begin, easing_attr, common.id,
            )
        }
    };

    if matches!(anim.animation_type, AnimationType::Path) {
        format!(
            "<g><path id=\"motion-path-{}\" d=\"{}\" fill=\"none\" stroke=\"none\"/>{}</g>",
            common.id,
            escape_xml_attr(anim.params.get("path").and_then(|v| v.as_str()).unwrap_or("M0,0 L10,0 L10,10 L0,10 Z")),
            svg,
        )
    } else {
        let mut result = svg.to_string();
        if result.trim_end().ends_with("/>") {
            if let Some(close_pos) = result.rfind("/>") {
                let tag_start = result[..close_pos].rfind('<').unwrap_or(0) + 1;
                let name_end = result[tag_start..]
                    .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
                    .map(|i| tag_start + i)
                    .unwrap_or(close_pos);
                let tag_name = &result[tag_start..name_end];
                result.replace_range(close_pos..close_pos + 2, &format!(">{}  </{}>", &anim_tag, tag_name));
            }
        } else if let Some(pos) = result.rfind("</") {
            result.insert_str(pos, &anim_tag);
        }
        if matches!(anim.animation_type, AnimationType::Scale) {
            format!(
                "<g transform=\"translate({:.2},{:.2})\"><g transform=\"translate({:.2},{:.2})\">{}</g></g>",
                cx, cy, -cx, -cy, result,
            )
        } else {
            result
        }
    }
}

fn render_shape(e: &crate::model::ShapeElement) -> String {
    let id = &e.common.id;
    let fill = resolve_fill_for(id, &e.fill, e.gradient.as_ref());
    let filter = shadow_filter_for(id, &e.common.shadows);
    let mut svg = shape_to_svg(
        &e.shape_type, e.common.x, e.common.y, e.common.width, e.common.height, &fill,
        e.stroke.as_deref(), e.stroke_width, e.common.opacity, e.common.rotation,
        e.border_radius, e.stroke_dasharray.as_deref(),
    );
    if let Some(f) = filter {
        if let Some(pos) = svg.rfind("/>") {
            svg.insert_str(pos, &f);
        }
    }
    svg
}

fn render_text(e: &crate::model::TextElement) -> String {
    let id = &e.common.id;
    let fill = resolve_fill_for(id, &e.fill, e.gradient.as_ref());
    let filter = shadow_filter_for(id, &e.common.shadows);

    let mut attrs = format!(
        r#" x="{:.2}" y="{:.2}" font-family="{}" font-size="{:.2}" font-weight="{}" fill="{}" dominant-baseline="central" text-anchor="middle""#,
        e.common.x + e.common.width / 2.0, e.common.y + e.common.height / 2.0,
        escape_xml_attr(&e.font_family), e.font_size, escape_xml_attr(&e.font_weight), escape_xml_attr(&fill),
    );
    if e.letter_spacing.abs() > f64::EPSILON {
        attrs.push_str(&format!(r#" letter-spacing="{:.2}""#, e.letter_spacing));
    }
    if let Some(s) = &e.stroke {
        attrs.push_str(&format!(r#" stroke="{}" stroke-width="{:.2}""#, escape_xml_attr(s), e.stroke_width));
    }
    if (e.common.opacity - 1.0).abs() > f64::EPSILON {
        attrs.push_str(&format!(r#" opacity="{:.2}""#, e.common.opacity));
    }
    if e.common.rotation.abs() > f64::EPSILON {
        let cx = e.common.x + e.common.width / 2.0;
        let cy = e.common.y + e.common.height / 2.0;
        attrs.push_str(&format!(r#" transform="rotate({:.2}, {:.2}, {:.2})""#, e.common.rotation, cx, cy));
    }
    if let Some(f) = filter {
        attrs.push_str(&f);
    }

    format!("<text{}>{}</text>", attrs, escape_xml(&e.content))
}

fn render_icon(e: &crate::model::IconElement) -> Result<String, AppError> {
    let icon_svg = icons::get_icon_path(&e.name).ok_or_else(|| {
        AppError::BuildError(format!("Icon '{}' not found in Lucide library", e.name))
    })?;

    let id = &e.common.id;
    let fill = resolve_fill_for(id, &e.fill, e.gradient.as_ref());
    let filter = shadow_filter_for(id, &e.common.shadows);

    let scale_x = e.common.width / 24.0;
    let scale_y = e.common.height / 24.0;

    let mut g_attrs = format!(
        r#" transform="translate({:.2}, {:.2}) scale({:.4}, {:.4})""#,
        e.common.x, e.common.y, scale_x, scale_y,
    );
    if (e.common.opacity - 1.0).abs() > f64::EPSILON {
        g_attrs.push_str(&format!(r#" opacity="{:.2}""#, e.common.opacity));
    }
    if let Some(f) = filter {
        g_attrs.push_str(&f);
    }

    let mut inner_attrs = format!(r#" fill="{}""#, escape_xml_attr(&fill));
    if let Some(s) = &e.stroke {
        let scaled_stroke = (e.stroke_width / ((scale_x + scale_y) / 2.0).abs().max(0.001)).min(e.stroke_width * 10.0);
        inner_attrs.push_str(&format!(r#" stroke="{}" stroke-width="{:.2}""#, escape_xml_attr(s), scaled_stroke));
    }

    let rotated = if e.common.rotation.abs() > f64::EPSILON {
        format!(
            r#"<g transform="rotate({:.2}, 12, 12)"{}>{}</g>"#,
            e.common.rotation, inner_attrs, icon_svg
        )
    } else {
        format!("<g{}>{}</g>", inner_attrs, icon_svg)
    };

    Ok(format!("<g{}>{}</g>", g_attrs, rotated))
}

fn render_image(e: &crate::model::ImageElement) -> String {
    let id = &e.common.id;
    let filter = shadow_filter_for(id, &e.common.shadows);

    let mut attrs = format!(
        r#" href="{}" x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}""#,
        e.data, e.common.x, e.common.y, e.common.width, e.common.height,
    );
    if (e.common.opacity - 1.0).abs() > f64::EPSILON {
        attrs.push_str(&format!(r#" opacity="{:.2}""#, e.common.opacity));
    }
    if e.common.rotation.abs() > f64::EPSILON {
        let cx = e.common.x + e.common.width / 2.0;
        let cy = e.common.y + e.common.height / 2.0;
        attrs.push_str(&format!(r#" transform="rotate({:.2}, {:.2}, {:.2})""#, e.common.rotation, cx, cy));
    }
    if let Some(f) = filter {
        attrs.push_str(&f);
    }

    format!("<image{}/>", attrs)
}

fn render_path(e: &crate::model::PathElement) -> String {
    let id = &e.common.id;
    let filter = shadow_filter_for(id, &e.common.shadows);

    let has_natural = e.natural_width > 0.0 && e.natural_height > 0.0;
    let sx = if has_natural && e.natural_width > 0.0 {
        e.common.width / e.natural_width
    } else {
        1.0
    };
    let sy = if has_natural && e.natural_height > 0.0 {
        e.common.height / e.natural_height
    } else {
        1.0
    };

    let mut attrs = format!(
        r#" d="{}" fill="{}" stroke="{}" stroke-width="{:.2}""#,
        escape_xml(&e.d), escape_xml_attr(&e.fill), escape_xml_attr(&e.stroke), e.stroke_width,
    );
    if let Some(ref da) = e.stroke_dasharray {
        attrs.push_str(&format!(r#" stroke-dasharray="{}""#, escape_xml_attr(da)));
    }
    if (e.common.opacity - 1.0).abs() > f64::EPSILON {
        attrs.push_str(&format!(r#" opacity="{:.2}""#, e.common.opacity));
    }
    if let Some(f) = filter {
        attrs.push_str(&f);
    }

    let path_svg = format!("<path{}/>", attrs);

    if !has_natural {
        return path_svg;
    }

    let mut transforms = Vec::new();
    transforms.push(format!("translate({:.2},{:.2})", e.common.x, e.common.y));

    if (sx - 1.0).abs() > f64::EPSILON || (sy - 1.0).abs() > f64::EPSILON {
        transforms.push(format!("scale({:.6},{:.6})", sx, sy));
    }

    if e.common.rotation.abs() > f64::EPSILON {
        let cx = e.natural_width / 2.0;
        let cy = e.natural_height / 2.0;
        transforms.push(format!("rotate({:.2},{:.2},{:.2})", e.common.rotation, cx, cy));
    }

    format!("<g transform=\"{}\">{}</g>", transforms.join(" "), path_svg)
}

fn render_group(e: &GroupElement, symbols: &std::collections::HashMap<String, crate::model::SymbolDef>) -> Result<String, AppError> {
    let mut transform_parts = Vec::new();

    if e.common.x.abs() > f64::EPSILON || e.common.y.abs() > f64::EPSILON {
        transform_parts.push(format!("translate({:.2}, {:.2})", e.common.x, e.common.y));
    }
    if e.common.rotation.abs() > f64::EPSILON {
        let cx = e.common.x + e.common.width / 2.0;
        let cy = e.common.y + e.common.height / 2.0;
        transform_parts.push(format!("rotate({:.2}, {:.2}, {:.2})", e.common.rotation, cx, cy));
    }

    let mut g_attrs = String::new();
    if !transform_parts.is_empty() {
        g_attrs.push_str(&format!(r#" transform="{}""#, transform_parts.join(" ")));
    }
    if (e.common.opacity - 1.0).abs() > f64::EPSILON {
        g_attrs.push_str(&format!(r#" opacity="{:.2}""#, e.common.opacity));
    }
    if !e.common.shadows.is_empty() {
        let refs: Vec<String> = e.common.shadows.iter().enumerate()
            .map(|(i, _)| format!("url(#shadow-{}-{})", e.common.id, i))
            .collect();
        g_attrs.push_str(&format!(r#" filter="{}""#, refs.join(" ")));
    }

    let mut children_svg = String::new();
    for child in &e.children {
        children_svg.push_str(&render_element(child, symbols)?);
    }

    Ok(format!("<g{}>{}</g>", g_attrs, children_svg))
}

fn resolve_fill_for(id: &str, fill: &str, gradient: Option<&crate::model::Gradient>) -> String {
    match gradient {
        Some(_) => format!("url(#gradient-{})", id),
        None => fill.to_string(),
    }
}

fn shadow_filter_for(id: &str, shadows: &[crate::model::Shadow]) -> Option<String> {
    if shadows.is_empty() {
        return None;
    }
    let refs: Vec<String> = shadows
        .iter()
        .enumerate()
        .map(|(i, _)| format!("url(#shadow-{}-{})", id, i))
        .collect();
    Some(format!(r#" filter="{}""#, refs.join(" ")))
}

fn find_element_deep<'a>(elements: &'a [Element], id: &str) -> Option<&'a Element> {
    for elem in elements {
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

fn collect_clip_mask_defs(all_elements: &[Element], elements: &[Element], defs: &mut String) {
    for elem in elements {
        let common = elem.common();
        if let Some(ref clip_id) = common.clip_element_id {
            if let Some(clip_elem) = find_element_deep(all_elements, clip_id) {
                defs.push_str(&clip_def_for(elem.id(), clip_elem));
            }
        }
        if let Some(ref mask_id) = common.mask_element_id {
            if let Some(mask_elem) = find_element_deep(all_elements, mask_id) {
                defs.push_str(&mask_def_for(elem.id(), mask_elem));
            }
        }
        if let Element::Group(g) = elem {
            collect_clip_mask_defs(all_elements, &g.children, defs);
        }
    }
}

fn clip_def_for(id: &str, clip_element: &Element) -> String {
    let inner = render_element_stripped(clip_element);
    format!(
        r#"<clipPath id="clip-{id}">{inner}</clipPath>"#,
        id = id, inner = inner,
    )
}

fn mask_def_for(id: &str, mask_element: &Element) -> String {
    let inner = render_element_stripped(mask_element);
    format!(
        r#"<mask id="mask-{id}">{inner}</mask>"#,
        id = id, inner = inner,
    )
}

/// Render element for use inside clipPath/mask defs — strips own clip/mask references to prevent cycles.
fn render_element_stripped(elem: &Element) -> String {
    let mut clone = elem.clone();
    clone.common_mut().clip_element_id = None;
    clone.common_mut().mask_element_id = None;
    match render_element(&clone, &std::collections::HashMap::new()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("render_element_stripped error: {:?}", e);
            String::new()
        }
    }
}

#[derive(Default)]
pub struct RenderCache {
    cached_svg: Option<String>,
    cached_version: u64,
}

impl RenderCache {
    pub fn build(&mut self, project: &IconProject) -> Result<String, AppError> {
        if let Some(ref svg) = self.cached_svg {
            if project.version == self.cached_version {
                return Ok(svg.clone());
            }
        }
        let svg = build(project)?;
        self.cached_svg = Some(svg.clone());
        self.cached_version = project.version;
        Ok(svg)
    }

    pub fn invalidate_cache(&mut self) {
        self.cached_svg = None;
        self.cached_version = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn make_project_with_element(elem: Element) -> IconProject {
        let mut p = IconProject::default();
        p.elements.push(elem);
        p.bump_version();
        p
    }

    fn make_shape() -> Element {
        Element::Shape(ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
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
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        })
    }

    fn make_text() -> Element {
        Element::Text(TextElement {
            common: CommonProps {
                id: "text-1".to_string(),
                x: 100.0,
                y: 200.0,
                width: 200.0,
                height: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            content: "Hello".to_string(),
            fill: "#000000".to_string(),
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
    fn test_build_empty_project() {
        let project = IconProject::default();
        let svg = build(&project).unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains(r#"width="512""#));
        assert!(svg.contains(r#"height="512""#));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains(r##"fill="#FFFFFF""##));
    }

    #[test]
    fn test_build_with_shape_element() {
        let project = make_project_with_element(make_shape());
        let svg = build(&project).unwrap();
        assert!(svg.contains("<circle"));
        assert!(svg.contains(r##"fill="#FF0000""##));
    }

    #[test]
    fn test_build_with_text_element() {
        let project = make_project_with_element(make_text());
        let svg = build(&project).unwrap();
        assert!(svg.contains("<text"));
        assert!(svg.contains(">Hello</text>"));
        assert!(svg.contains(r#"font-family="sans-serif""#));
    }

    #[test]
    fn test_build_with_icon_element() {
        let project = make_project_with_element(Element::Icon(IconElement {
            common: CommonProps {
                id: "icon-1".to_string(),
                x: 100.0, y: 100.0, width: 200.0, height: 200.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            name: "heart".to_string(),
            fill: "#FF0000".to_string(),
            stroke: None,
            stroke_width: 0.0,
            gradient: None,
        }));
        let svg = build(&project).unwrap();
        assert!(svg.contains("<g"));
        assert!(svg.contains(r##"fill="#FF0000""##));
    }

    #[test]
    fn test_build_with_image_element() {
        let project = make_project_with_element(Element::Image(ImageElement {
            common: CommonProps {
                id: "image-1".to_string(),
                x: 0.0, y: 0.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            data: "data:image/png;base64,iVBOR".to_string(),
        }));
        let svg = build(&project).unwrap();
        assert!(svg.contains("<image"));
        assert!(svg.contains("data:image/png;base64"));
    }

    #[test]
    fn test_build_with_linear_gradient() {
        let mut elem = make_shape();
        if let Element::Shape(ref mut s) = elem {
            s.gradient = Some(Gradient {
                gradient_type: GradientKind::Linear,
                colors: vec!["#FF0000".to_string(), "#0000FF".to_string()],
                angle: 90.0,
                stops: vec![],
            });
        }
        let project = make_project_with_element(elem);
        let svg = build(&project).unwrap();
        assert!(svg.contains("<defs>"));
        assert!(svg.contains("<linearGradient"));
        assert!(svg.contains(r#"gradient-shape-1"#));
        assert!(svg.contains(r#"fill="url(#gradient-shape-1)""#));
    }

    #[test]
    fn test_build_with_radial_gradient() {
        let mut elem = make_shape();
        if let Element::Shape(ref mut s) = elem {
            s.gradient = Some(Gradient {
                gradient_type: GradientKind::Radial,
                colors: vec!["#FF0000".to_string(), "#00FF00".to_string()],
                angle: 0.0,
                stops: vec![],
            });
        }
        let project = make_project_with_element(elem);
        let svg = build(&project).unwrap();
        assert!(svg.contains("<radialGradient"));
    }

    #[test]
    fn test_build_with_shadow() {
        let mut elem = make_shape();
        elem.common_mut().shadows = vec![Shadow::default()];
        let project = make_project_with_element(elem);
        let svg = build(&project).unwrap();
        assert!(svg.contains("<defs>"));
        assert!(svg.contains("<filter"));
        assert!(svg.contains("shadow-shape-1"));
        assert!(svg.contains("feDropShadow"));
    }

    #[test]
    fn test_build_with_corner_radius_clip_path() {
        let mut project = IconProject::default();
        project.canvas.corner_radius = 20;
        project.elements.push(make_shape());
        let svg = build(&project).unwrap();
        assert!(svg.contains("<clipPath"));
        assert!(svg.contains(r#"id="app-icon-clip""#));
        assert!(svg.contains(r#"clip-path="url(#app-icon-clip)""#));
    }

    #[test]
    fn test_build_with_canvas_background_gradient() {
        let mut project = IconProject::default();
        project.canvas.background_gradient = Some(Gradient {
            gradient_type: GradientKind::Linear,
            colors: vec!["#FF0000".to_string(), "#0000FF".to_string()],
            angle: 180.0,
            stops: vec![],
        });
        let svg = build(&project).unwrap();
        assert!(svg.contains("canvas-bg-gradient"));
        assert!(svg.contains(r#"fill="url(#canvas-bg-gradient)""#));
    }

    #[test]
    fn test_render_cache_returns_cached_when_version_unchanged() {
        let project = IconProject::default();
        let mut cache = RenderCache::default();
        let svg1 = cache.build(&project).unwrap();
        let svg2 = cache.build(&project).unwrap();
        assert_eq!(svg1, svg2);
    }

    #[test]
    fn test_render_cache_rebuilds_on_version_change() {
        let mut project = IconProject::default();
        let mut cache = RenderCache::default();
        let svg1 = cache.build(&project).unwrap();
        project.bump_version();
        let svg2 = cache.build(&project).unwrap();
        assert_eq!(svg1, svg2);
    }

    #[test]
    fn test_render_cache_invalidate() {
        let project = IconProject::default();
        let mut cache = RenderCache::default();
        cache.build(&project).unwrap();
        cache.invalidate_cache();
        assert!(cache.cached_svg.is_none());
        assert_eq!(cache.cached_version, 0);
    }

    #[test]
    fn test_build_with_group_element() {
        let child1 = ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
                x: 10.0, y: 10.0, width: 50.0, height: 50.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        };
        let child2 = ShapeElement {
            common: CommonProps {
                id: "shape-2".to_string(),
                x: 60.0, y: 60.0, width: 50.0, height: 50.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            shape_type: shapes::ShapeType::Rect,
            fill: "#00FF00".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        };
        let group = GroupElement {
            common: CommonProps {
                id: "group-1".to_string(),
                x: 20.0, y: 20.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            children: vec![Element::Shape(child1), Element::Shape(child2)],
            expanded: false,
        };
        let project = make_project_with_element(Element::Group(group));
        let svg = build(&project).unwrap();
        assert!(svg.contains(r#"<g transform="translate(20.00, 20.00)">"#));
        assert!(svg.contains("<circle"));
        assert!(svg.contains("<rect"));
        assert!(svg.contains(r##"fill="#FF0000""##));
        assert!(svg.contains(r##"fill="#00FF00""##));
    }

    #[test]
    fn test_build_with_nested_group() {
        let inner_child = ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
                x: 0.0, y: 0.0, width: 30.0, height: 30.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        };
        let inner_group = GroupElement {
            common: CommonProps {
                id: "group-2".to_string(),
                x: 10.0, y: 10.0, width: 30.0, height: 30.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            children: vec![Element::Shape(inner_child)],
            expanded: false,
        };
        let outer_group = GroupElement {
            common: CommonProps {
                id: "group-1".to_string(),
                x: 50.0, y: 50.0, width: 60.0, height: 60.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            children: vec![Element::Group(inner_group)],
            expanded: false,
        };
        let project = make_project_with_element(Element::Group(outer_group));
        let svg = build(&project).unwrap();
        assert!(svg.contains(r#"<g transform="translate(50.00, 50.00)">"#));
        assert!(svg.contains(r#"<g transform="translate(10.00, 10.00)">"#));
        assert!(svg.contains("<circle"));
    }

    #[test]
    fn test_build_with_group_shadow() {
        let child = ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
                x: 0.0, y: 0.0, width: 50.0, height: 50.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        };
        let group = GroupElement {
            common: CommonProps {
                id: "group-1".to_string(),
                x: 100.0, y: 100.0, width: 50.0, height: 50.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![Shadow::default()],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        },
            children: vec![Element::Shape(child)],
            expanded: false,
        };
        let project = make_project_with_element(Element::Group(group));
        let svg = build(&project).unwrap();
        assert!(svg.contains("shadow-group-1"));
        assert!(svg.contains(r#"filter="url(#shadow-group-1)""#));
        assert!(svg.contains("feDropShadow"));
    }

    #[test]
    fn test_blend_mode_rendering() {
        let mut elem = make_shape();
        elem.common_mut().blend_mode = Some("multiply".to_string());
        let project = make_project_with_element(elem);
        let svg = build(&project).unwrap();
        assert!(svg.contains(r#"style="mix-blend-mode:multiply""#));
    }

    #[test]
    fn test_inset_shadow_rendering() {
        let mut elem = make_shape();
        elem.common_mut().shadows = vec![Shadow {
            color: "#00000040".to_string(),
            blur: 8.0,
            offset_x: 0.0,
            offset_y: 4.0,
            inset: true,
        }];
        let project = make_project_with_element(elem);
        let svg = build(&project).unwrap();
        assert!(svg.contains("shadow-shape-1-0"));
        assert!(svg.contains("feComposite"));
        assert!(svg.contains("feFlood"));
        assert!(svg.contains("feGaussianBlur"));
    }

    #[test]
    fn test_multi_shadow_rendering() {
        let mut elem = make_shape();
        elem.common_mut().shadows = vec![
            Shadow {
                color: "#00000040".to_string(),
                blur: 8.0,
                offset_x: 0.0,
                offset_y: 4.0,
                inset: false,
            },
            Shadow {
                color: "#FF000080".to_string(),
                blur: 12.0,
                offset_x: 2.0,
                offset_y: 6.0,
                inset: false,
            },
        ];
        let project = make_project_with_element(elem);
        let svg = build(&project).unwrap();
        assert!(svg.contains("shadow-shape-1-0"));
        assert!(svg.contains("shadow-shape-1-1"));
        assert!(svg.contains(r#"id="shadow-shape-1-0""#));
        assert!(svg.contains(r#"id="shadow-shape-1-1""#));
        assert!(svg.contains(r#"filter="url(#shadow-shape-1-0) url(#shadow-shape-1-1)""#));
    }

    #[test]
    fn test_clip_path_rendering() {
        let clip_shape = ShapeElement {
            common: CommonProps {
                id: "shape-2".to_string(),
                x: 50.0, y: 50.0, width: 200.0, height: 200.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
                mask_element_id: None, locked: false, visible: true, svg_filter: None,
            },
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        };
        let mut clipped = make_shape();
        clipped.common_mut().clip_element_id = Some("shape-2".to_string());

        let mut p = IconProject::default();
        p.elements.push(clipped);
        p.elements.push(Element::Shape(clip_shape));
        p.bump_version();

        let svg = build(&p).unwrap();
        assert!(svg.contains(r#"id="clip-shape-1""#), "should contain clipPath def");
        assert!(svg.contains(r#"<clipPath"#), "should contain clipPath element");
        assert!(svg.contains(r#"clip-path="url(#clip-shape-1)""#), "should reference clipPath");
    }

    #[test]
    fn test_mask_rendering() {
        let mask_shape = ShapeElement {
            common: CommonProps {
                id: "shape-2".to_string(),
                x: 50.0, y: 50.0, width: 200.0, height: 200.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
                mask_element_id: None, locked: false, visible: true, svg_filter: None,
            },
            shape_type: shapes::ShapeType::Rect,
            fill: "#FFFFFF".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        };
        let mut masked = make_shape();
        masked.common_mut().mask_element_id = Some("shape-2".to_string());

        let mut p = IconProject::default();
        p.elements.push(masked);
        p.elements.push(Element::Shape(mask_shape));
        p.bump_version();

        let svg = build(&p).unwrap();
        assert!(svg.contains(r#"id="mask-shape-1""#), "should contain mask def");
        assert!(svg.contains(r#"<mask"#), "should contain mask element");
        assert!(svg.contains(r#"mask="url(#mask-shape-1)""#), "should reference mask");
    }

    #[test]
    fn test_clip_without_reference() {
        let project = make_project_with_element(make_shape());
        let svg = build(&project).unwrap();
        assert!(!svg.contains("clip-"), "no clip defs should be generated for elements without clip");
        assert!(!svg.contains("mask-"), "no mask defs should be generated for elements without mask");
    }

    #[test]
    fn test_boolean_result_renders_in_svg() {
        use crate::engine::boolean::{self, BooleanOp};
        let shape_a = crate::model::ShapeElement {
            common: crate::model::CommonProps {
                id: "shape-1".to_string(),
                x: 0.0, y: 0.0, width: 200.0, height: 200.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None, locked: false, visible: true, svg_filter: None,
            },
            shape_type: crate::model::shapes::ShapeType::Rect,
            fill: "#FF0000".to_string(), stroke: None, stroke_width: 0.0,
            border_radius: 0.0, stroke_dasharray: None, gradient: None,
        };
        let shape_b = crate::model::ShapeElement {
            common: crate::model::CommonProps {
                id: "shape-2".to_string(),
                x: 100.0, y: 0.0, width: 200.0, height: 200.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None, locked: false, visible: true, svg_filter: None,
            },
            shape_type: crate::model::shapes::ShapeType::Rect,
            fill: "#00FF00".to_string(), stroke: None, stroke_width: 0.0,
            border_radius: 0.0, stroke_dasharray: None, gradient: None,
        };

        let (path_elem, _) = boolean::perform_boolean(
            &Element::Shape(shape_a),
            &Element::Shape(shape_b),
            BooleanOp::Union,
        ).unwrap();

        let project = IconProject {
            elements: vec![Element::Path(path_elem)],
            ..IconProject::default()
        };

        let svg = build(&project).unwrap();
        assert!(svg.contains("<path"), "boolean result should render as <path> in SVG");
        assert!(svg.contains("d=\"M"), "path should have d attribute");
        assert!(svg.contains("#FF0000"), "fill should inherit from element A");
    }

    #[test]
    fn test_invisible_element_not_in_svg() {
        let mut shape = make_shape();
        shape.common_mut().visible = false;
        let project = make_project_with_element(shape);
        let svg = build(&project).unwrap();
        assert!(!svg.contains("shape-1"), "invisible element should not appear in SVG");
        assert!(!svg.contains("#FF0000"), "invisible element fill should not appear in SVG");
    }

    #[test]
    fn test_locked_element_still_in_svg() {
        let mut shape = make_shape();
        shape.common_mut().locked = true;
        let project = make_project_with_element(shape);
        let svg = build(&project).unwrap();
        assert!(svg.contains("shape-1"), "locked element should still appear in SVG");
        assert!(svg.contains("#FF0000"), "locked element fill should appear in SVG");
    }

    #[test]
    fn test_mixed_visibility() {
        let shape_visible = ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
                x: 0.0, y: 0.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None,
                locked: false, visible: true, svg_filter: None,
            },
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(), stroke: None, stroke_width: 0.0,
            border_radius: 0.0, stroke_dasharray: None, gradient: None,
        };
        let shape_hidden = ShapeElement {
            common: CommonProps {
                id: "shape-2".to_string(),
                x: 0.0, y: 0.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None,
                locked: false, visible: false, svg_filter: None,
            },
            shape_type: shapes::ShapeType::Circle,
            fill: "#00FF00".to_string(), stroke: None, stroke_width: 0.0,
            border_radius: 0.0, stroke_dasharray: None, gradient: None,
        };
        let mut project = IconProject::default();
        project.elements.push(Element::Shape(shape_visible));
        project.elements.push(Element::Shape(shape_hidden));
        project.bump_version();
        let svg = build(&project).unwrap();
        assert!(svg.contains("#FF0000"), "visible element should be in SVG");
        assert!(!svg.contains("#00FF00"), "hidden element should not be in SVG");
    }

    #[test]
    fn test_build_with_pages_renders_active_page() {
        let mut project = IconProject::default();

        // Create two pages with different elements
        let mut page1 = Page::new("Page 1", 256, 256);
        page1.elements.push(Element::Shape(ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
                x: 0.0, y: 0.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None,
                locked: false, visible: true, svg_filter: None,
            },
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(), stroke: None, stroke_width: 0.0,
            border_radius: 0.0, stroke_dasharray: None, gradient: None,
        }));

        let mut page2 = Page::new("Page 2", 128, 128);
        page2.elements.push(Element::Shape(ShapeElement {
            common: CommonProps {
                id: "shape-2".to_string(),
                x: 0.0, y: 0.0, width: 50.0, height: 50.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None,
                locked: false, visible: true, svg_filter: None,
            },
            shape_type: shapes::ShapeType::Rect,
            fill: "#00FF00".to_string(), stroke: None, stroke_width: 0.0,
            border_radius: 0.0, stroke_dasharray: None, gradient: None,
        }));

        project.pages.push(page1);
        project.pages.push(page2);

        // Render page 0
        project.active_page_index = 0;
        let svg = build(&project).unwrap();
        assert!(svg.contains(r#"width="256""#), "should use page 1 dimensions");
        assert!(svg.contains("#FF0000"), "should contain page 1 element");
        assert!(!svg.contains("#00FF00"), "should NOT contain page 2 element");

        // Render page 1
        project.active_page_index = 1;
        let svg = build(&project).unwrap();
        assert!(svg.contains(r#"width="128""#), "should use page 2 dimensions");
        assert!(svg.contains("#00FF00"), "should contain page 2 element");
        assert!(!svg.contains("#FF0000"), "should NOT contain page 1 element");
    }

    #[test]
    fn test_build_with_symbol_instance() {
        use crate::model::symbol::{SymbolDef, SymbolInstanceElement, SymbolOverride};

        let source = ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
                x: 50.0, y: 50.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None,
                locked: false, visible: true, svg_filter: None,
            },
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(), stroke: None, stroke_width: 0.0,
            border_radius: 0.0, stroke_dasharray: None, gradient: None,
        };

        let def = SymbolDef {
            id: "symbol-1".to_string(),
            name: "Red Circle".to_string(),
            source_element: Element::Shape(source),
            overridable_props: vec![],
        };

        let instance = SymbolInstanceElement {
            common: CommonProps {
                id: "inst-1".to_string(),
                x: 100.0, y: 100.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None,
                locked: false, visible: true, svg_filter: None,
            },
            symbol_id: "symbol-1".to_string(),
            overrides: vec![SymbolOverride {
                property: "fill".to_string(),
                value: serde_json::json!("#00FF00"),
            }],
        };

        let mut project = IconProject::default();
        project.symbols.insert("symbol-1".to_string(), def);
        project.elements.push(Element::Symbol(instance));
        project.bump_version();

        let svg = build(&project).unwrap();
        assert!(svg.contains("instance-inst-1"), "should wrap in instance group");
        assert!(svg.contains("#00FF00"), "should use overridden fill");
        assert!(svg.contains("<circle"), "should render the source shape");
    }

    #[test]
    fn test_build_with_symbol_no_def() {
        use crate::model::symbol::SymbolInstanceElement;

        let instance = SymbolInstanceElement {
            common: CommonProps {
                id: "inst-1".to_string(),
                x: 100.0, y: 100.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None,
                locked: false, visible: true, svg_filter: None,
            },
            symbol_id: "nonexistent".to_string(),
            overrides: vec![],
        };

        let mut project = IconProject::default();
        project.elements.push(Element::Symbol(instance));
        project.bump_version();

        let svg = build(&project).unwrap();
        assert!(svg.contains("not found"), "should render comment when symbol def missing");
    }

    #[test]
    fn test_build_with_symbol_multiple_instances() {
        use crate::model::symbol::{SymbolDef, SymbolInstanceElement, SymbolOverride};

        let source = ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
                x: 0.0, y: 0.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None,
                locked: false, visible: true, svg_filter: None,
            },
            shape_type: shapes::ShapeType::Rect,
            fill: "#FF0000".to_string(), stroke: None, stroke_width: 0.0,
            border_radius: 0.0, stroke_dasharray: None, gradient: None,
        };

        let def = SymbolDef {
            id: "symbol-1".to_string(),
            name: "Rect".to_string(),
            source_element: Element::Shape(source),
            overridable_props: vec![],
        };

        let inst1 = SymbolInstanceElement {
            common: CommonProps {
                id: "inst-1".to_string(),
                x: 0.0, y: 0.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None,
                locked: false, visible: true, svg_filter: None,
            },
            symbol_id: "symbol-1".to_string(),
            overrides: vec![],
        };

        let inst2 = SymbolInstanceElement {
            common: CommonProps {
                id: "inst-2".to_string(),
                x: 200.0, y: 200.0, width: 100.0, height: 100.0,
                opacity: 0.5, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None,
                locked: false, visible: true, svg_filter: None,
            },
            symbol_id: "symbol-1".to_string(),
            overrides: vec![SymbolOverride {
                property: "fill".to_string(),
                value: serde_json::json!("#0000FF"),
            }],
        };

        let mut project = IconProject::default();
        project.symbols.insert("symbol-1".to_string(), def);
        project.elements.push(Element::Symbol(inst1));
        project.elements.push(Element::Symbol(inst2));
        project.bump_version();

        let svg = build(&project).unwrap();
        assert!(svg.contains("instance-inst-1"), "first instance");
        assert!(svg.contains("instance-inst-2"), "second instance");
        assert!(svg.contains("#FF0000"), "first instance uses source fill");
        assert!(svg.contains("#0000FF"), "second instance uses override fill");
        assert!(svg.contains("opacity=\"0.50\""), "second instance has custom opacity");
    }
}
