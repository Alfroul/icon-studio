use crate::engine::utils::escape_xml_attr;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShapeType {
    Circle,
    Rect,
    RoundedRect,
    Hexagon,
    Star,
    Shield,
    Diamond,
    Triangle,
    ArrowRight,
    Cross,
    Heart,
    Pentagon,
    Octagon,
    Wave,
    #[serde(rename = "custom")]
    Custom { d: String },
}

/// Generate an SVG `<circle>` element.
fn circle_svg(cx: f64, cy: f64, r: f64) -> String {
    format!(r#"<circle cx="{cx:.2}" cy="{cy:.2}" r="{r:.2}"/>"#, cx = cx, cy = cy, r = r)
}

/// Generate an SVG `<rect>` element.
fn rect_svg(x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        r#"<rect x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}"/>"#,
        x = x, y = y, w = w, h = h
    )
}

/// Generate an SVG `<rect>` with rounded corners.
fn rounded_rect_svg(x: f64, y: f64, w: f64, h: f64, rx: f64, ry: f64) -> String {
    format!(
        r#"<rect x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" rx="{rx:.2}" ry="{ry:.2}"/>"#,
        x = x, y = y, w = w, h = h, rx = rx, ry = ry
    )
}

/// Generate an SVG `<polygon>` in the shape of a regular hexagon.
/// 6 vertices, starting at angle -30° (i * 60° - 30°).
fn hexagon_svg(cx: f64, cy: f64, r: f64) -> String {
    let points: Vec<String> = (0..6)
        .map(|i| {
            let angle = (i as f64) * (PI / 3.0) - PI / 6.0;
            let px = cx + r * angle.cos();
            let py = cy + r * angle.sin();
            format!("{:.2},{:.2}", px, py)
        })
        .collect();
    format!(r#"<polygon points="{}"/>"#, points.join(" "))
}

/// Generate an SVG `<polygon>` in the shape of a 5-pointed star.
/// 10 vertices alternating between outer radius and inner radius.
/// inner_r = outer_r × 0.38
fn star_svg(cx: f64, cy: f64, outer_r: f64) -> String {
    let inner_r = outer_r * 0.38;
    let points: Vec<String> = (0..10)
        .map(|i| {
            let angle = (i as f64) * (PI / 5.0) - PI / 2.0;
            let r = if i % 2 == 0 { outer_r } else { inner_r };
            let px = cx + r * angle.cos();
            let py = cy + r * angle.sin();
            format!("{:.2},{:.2}", px, py)
        })
        .collect();
    format!(r#"<polygon points="{}"/>"#, points.join(" "))
}

/// Generate an SVG `<path>` in the shape of a shield.
/// Flat top edge, slightly curved sides tapering inward, pointed bottom center.
fn shield_svg(cx: f64, cy: f64, w: f64, h: f64) -> String {
    let left = cx - w / 2.0;
    let right = cx + w / 2.0;
    let top = cy - h / 2.0;
    let bottom = cy + h / 2.0;
    // Side curve starts at 65% down from top
    let side_y = top + h * 0.65;
    // Control point Y for the quadratic curves to the bottom point
    let ctrl_y = top + h * 0.85;

    format!(
        r#"<path d="M {left:.2} {top:.2} \\
                    L {right:.2} {top:.2} \\
                    L {right:.2} {side_y:.2} \\
                    Q {right:.2} {ctrl_y:.2} {cx:.2} {bottom:.2} \\
                    Q {left:.2} {ctrl_y:.2} {left:.2} {side_y:.2} \\
                    Z"/>"#,
        left = left,
        right = right,
        top = top,
        side_y = side_y,
        ctrl_y = ctrl_y,
        cx = cx,
        bottom = bottom,
    )
}

/// Generate an SVG `<polygon>` in the shape of a diamond (rotated square).
/// 4 vertices at top, right, bottom, left.
fn diamond_svg(cx: f64, cy: f64, w: f64, h: f64) -> String {
    let points = format!(
        "{:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
        cx, cy - h / 2.0,          // top
        cx + w / 2.0, cy,          // right
        cx, cy + h / 2.0,          // bottom
        cx - w / 2.0, cy           // left
    );
    format!(r#"<polygon points="{}"/>"#, points)
}

/// Generate an SVG `<polygon>` in the shape of an equilateral triangle.
/// 3 vertices, starting at top (angle = i * 2PI/3 - PI/2).
fn triangle_svg(cx: f64, cy: f64, r: f64) -> String {
    let points: Vec<String> = (0..3)
        .map(|i| {
            let angle = (i as f64) * (2.0 * PI / 3.0) - PI / 2.0;
            let px = cx + r * angle.cos();
            let py = cy + r * angle.sin();
            format!("{:.2},{:.2}", px, py)
        })
        .collect();
    format!(r#"<polygon points="{}"/>"#, points.join(" "))
}

/// Generate an SVG `<polygon>` in the shape of a right-pointing arrow.
/// Rectangular base (60% width) + triangular tip (40% width).
fn arrow_right_svg(cx: f64, cy: f64, w: f64, h: f64) -> String {
    let left = cx - w / 2.0;
    let right = cx + w / 2.0;
    let top = cy - h / 2.0;
    let bottom = cy + h / 2.0;
    let mid_x = left + w * 0.6;
    let points = format!(
        "{:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
        left, top,
        mid_x, top,
        right, cy,
        mid_x, bottom,
        left, bottom,
    );
    format!(r#"<polygon points="{}"/>"#, points)
}

/// Generate an SVG `<polygon>` in the shape of a cross/plus.
/// 12 vertices forming 4 arms. arm_w = w*0.35, arm_h = h*0.35.
fn cross_svg(cx: f64, cy: f64, w: f64, h: f64) -> String {
    let arm_w = w * 0.35;
    let arm_h = h * 0.35;
    let left = cx - w / 2.0;
    let right = cx + w / 2.0;
    let top = cy - h / 2.0;
    let bottom = cy + h / 2.0;
    let il = cx - arm_w / 2.0;
    let ir = cx + arm_w / 2.0;
    let it = cy - arm_h / 2.0;
    let ib = cy + arm_h / 2.0;
    let points = format!(
        "{:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
        il, top,    // 1: top of left arm
        ir, top,    // 2: top of right arm
        ir, it,     // 3: inner top-right
        right, it,  // 4: right of top arm
        right, ib,  // 5: right of bottom arm
        ir, ib,     // 6: inner bottom-right
        ir, bottom, // 7: bottom-right
        il, bottom, // 8: bottom-left
        il, ib,     // 9: inner bottom-left
        left, ib,   // 10: left of bottom arm
        left, it,   // 11: left of top arm
        il, it,     // 12: inner top-left
    );
    format!(r#"<polygon points="{}"/>"#, points)
}

/// Generate an SVG `<path>` in the shape of a heart.
/// 2 cubic bezier curves forming a classic heart shape.
fn heart_svg(cx: f64, cy: f64, w: f64, h: f64) -> String {
    let top = cy - h / 2.0;
    let bottom = cy + h / 2.0;
    let left = cx - w / 2.0;
    let right = cx + w / 2.0;
    format!(
        r#"<path d="M {:.2} {:.2} \
                    C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} \
                    C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} \
                    Z"/>"#,
        cx, bottom,
        // Left curve: bottom center → top-left bump
        left, cy,
        left, top,
        cx - w * 0.25, top,
        // Right curve: top indentation → top-right bump → bottom center
        cx + w * 0.25, top,
        right, top,
        cx, bottom,
    )
}

/// Generate an SVG `<polygon>` in the shape of a regular pentagon.
/// 5 vertices, starting at top (angle = i * 2PI/5 - PI/2).
fn pentagon_svg(cx: f64, cy: f64, r: f64) -> String {
    let points: Vec<String> = (0..5)
        .map(|i| {
            let angle = (i as f64) * (2.0 * PI / 5.0) - PI / 2.0;
            let px = cx + r * angle.cos();
            let py = cy + r * angle.sin();
            format!("{:.2},{:.2}", px, py)
        })
        .collect();
    format!(r#"<polygon points="{}"/>"#, points.join(" "))
}

/// Generate an SVG `<polygon>` in the shape of a regular octagon.
/// 8 vertices, starting at angle -PI/8.
fn octagon_svg(cx: f64, cy: f64, r: f64) -> String {
    let points: Vec<String> = (0..8)
        .map(|i| {
            let angle = (i as f64) * (2.0 * PI / 8.0) - PI / 8.0;
            let px = cx + r * angle.cos();
            let py = cy + r * angle.sin();
            format!("{:.2},{:.2}", px, py)
        })
        .collect();
    format!(r#"<polygon points="{}"/>"#, points.join(" "))
}

/// Generate an SVG `<path>` in the shape of a sine wave.
/// 2 complete wave periods using cubic bezier curves. Open path (no Z close).
fn wave_svg(x: f64, y: f64, w: f64, h: f64) -> String {
    let mid_y = y + h / 2.0;
    let amp = h / 2.0;
    let half_period = w / 4.0;
    let cp_offset = half_period * 0.5523;
    format!(
        r#"<path d="M {:.2} {:.2} \
                    C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} \
                    C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} \
                    C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} \
                    C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} \
                    L {:.2} {:.2} \
                    L {:.2} {:.2} \
                    Z"/>"#,
        x, mid_y,
        x + half_period - cp_offset, mid_y - amp * 1.4,
        x + half_period + cp_offset, mid_y - amp * 1.4,
        x + half_period, mid_y,
        x + half_period * 2.0 - cp_offset, mid_y + amp * 1.4,
        x + half_period * 2.0 + cp_offset, mid_y + amp * 1.4,
        x + half_period * 2.0, mid_y,
        x + half_period * 3.0 - cp_offset, mid_y - amp * 1.4,
        x + half_period * 3.0 + cp_offset, mid_y - amp * 1.4,
        x + half_period * 3.0, mid_y,
        x + half_period * 4.0 - cp_offset, mid_y + amp * 1.4,
        x + half_period * 4.0 + cp_offset, mid_y + amp * 1.4,
        x + w, mid_y,
        x + w, y + h,
        x, y + h,
    )
}

/// Generate a complete SVG element for a shape, including fill, stroke, opacity,
/// and rotation transform.
#[allow(clippy::too_many_arguments)]
pub fn shape_to_svg(
    shape: &ShapeType,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    fill: &str,
    stroke: Option<&str>,
    stroke_width: f64,
    opacity: f64,
    rotation: f64,
    border_radius: f64,
    stroke_dasharray: Option<&str>,
) -> String {
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;

    let inner = match shape {
        ShapeType::Circle => {
            let r = w.min(h) / 2.0;
            circle_svg(cx, cy, r)
        }
        ShapeType::Rect => {
            if border_radius.abs() > f64::EPSILON {
                rounded_rect_svg(x, y, w, h, border_radius, border_radius)
            } else {
                rect_svg(x, y, w, h)
            }
        }
        ShapeType::RoundedRect => {
            let rx = if border_radius.abs() > f64::EPSILON { border_radius } else { w.min(h) * 0.15 };
            rounded_rect_svg(x, y, w, h, rx, rx)
        }
        ShapeType::Hexagon => {
            let r = w.min(h) / 2.0;
            hexagon_svg(cx, cy, r)
        }
        ShapeType::Star => {
            let r = w.min(h) / 2.0;
            star_svg(cx, cy, r)
        }
        ShapeType::Shield => shield_svg(cx, cy, w, h),
        ShapeType::Diamond => diamond_svg(cx, cy, w, h),
        ShapeType::Triangle => {
            let r = w.min(h) / 2.0;
            triangle_svg(cx, cy, r)
        }
        ShapeType::ArrowRight => arrow_right_svg(cx, cy, w, h),
        ShapeType::Cross => cross_svg(cx, cy, w, h),
        ShapeType::Heart => heart_svg(cx, cy, w, h),
        ShapeType::Pentagon => {
            let r = w.min(h) / 2.0;
            pentagon_svg(cx, cy, r)
        }
        ShapeType::Octagon => {
            let r = w.min(h) / 2.0;
            octagon_svg(cx, cy, r)
        }
        ShapeType::Wave => wave_svg(x, y, w, h),
        ShapeType::Custom { ref d } => {
            format!(r#"<path d="{}"/>"#, escape_xml_attr(d))
        }
    };

    let stroke_attr = match stroke {
        Some(s) => format!(r#" stroke="{}" stroke-width="{:.2}""#, escape_xml_attr(s), stroke_width),
        None => String::new(),
    };

    let dash_attr = match stroke_dasharray {
        Some(da) => format!(r#" stroke-dasharray="{}""#, escape_xml_attr(da)),
        None => String::new(),
    };

    let opacity_attr = if (opacity - 1.0).abs() > f64::EPSILON {
        format!(r#" opacity="{:.2}""#, opacity)
    } else {
        String::new()
    };

    let transform_attr = if rotation.abs() > f64::EPSILON {
        format!(r#" transform="rotate({:.2}, {:.2}, {:.2})""#, rotation, cx, cy)
    } else {
        String::new()
    };

    let inner = match inner.strip_suffix("/>") {
        Some(s) => s,
        None => {
            // Malformed shape SVG — return as-is without attributes rather than panicking
            return inner;
        }
    };
    format!(
        r#"{} fill="{}"{}{}{}{}/>"#,
        inner,
        escape_xml_attr(fill),
        stroke_attr,
        dash_attr,
        opacity_attr,
        transform_attr
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_svg_output() {
        let svg = shape_to_svg(&ShapeType::Circle, 50.0, 50.0, 100.0, 100.0, "#FF0000", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<circle"));
        assert!(svg.contains(r#"cx="100.00""#));
        assert!(svg.contains(r#"cy="100.00""#));
        assert!(svg.contains(r#"r="50.00""#));
        assert!(svg.contains(r##"fill="#FF0000""##));
    }

    #[test]
    fn test_rect_svg_output() {
        let svg = shape_to_svg(&ShapeType::Rect, 10.0, 20.0, 100.0, 200.0, "#00FF00", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<rect"));
        assert!(svg.contains(r#"x="10.00""#));
        assert!(svg.contains(r#"y="20.00""#));
        assert!(svg.contains(r#"width="100.00""#));
        assert!(svg.contains(r#"height="200.00""#));
        assert!(svg.contains(r##"fill="#00FF00""##));
    }

    #[test]
    fn test_rounded_rect_svg_output() {
        let svg = shape_to_svg(&ShapeType::RoundedRect, 0.0, 0.0, 100.0, 100.0, "#0000FF", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<rect"));
        assert!(svg.contains("rx="));
        assert!(svg.contains("ry="));
    }

    #[test]
    fn test_rect_with_border_radius() {
        let svg = shape_to_svg(&ShapeType::Rect, 0.0, 0.0, 100.0, 100.0, "#000", None, 0.0, 1.0, 0.0, 20.0, None);
        assert!(svg.contains("<rect"));
        assert!(svg.contains("rx="));
    }

    #[test]
    fn test_hexagon_svg_output() {
        let svg = shape_to_svg(&ShapeType::Hexagon, 50.0, 50.0, 100.0, 100.0, "#ABCDEF", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("points="));
        assert!(svg.contains(r##"fill="#ABCDEF""##));
    }

    #[test]
    fn test_star_svg_output() {
        let svg = shape_to_svg(&ShapeType::Star, 50.0, 50.0, 100.0, 100.0, "#123456", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("points="));
    }

    #[test]
    fn test_shield_svg_output() {
        let svg = shape_to_svg(&ShapeType::Shield, 50.0, 50.0, 100.0, 120.0, "#FF0", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<path"));
        assert!(svg.contains("d="));
    }

    #[test]
    fn test_diamond_svg_output() {
        let svg = shape_to_svg(&ShapeType::Diamond, 50.0, 50.0, 80.0, 100.0, "#F00", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("points="));
    }

    #[test]
    fn test_shape_with_stroke() {
        let svg = shape_to_svg(&ShapeType::Circle, 50.0, 50.0, 100.0, 100.0, "#F00", Some("#000"), 2.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains(r##"stroke="#000""##));
        assert!(svg.contains(r#"stroke-width="2.00""#));
    }

    #[test]
    fn test_shape_with_opacity() {
        let svg = shape_to_svg(&ShapeType::Circle, 0.0, 0.0, 100.0, 100.0, "#F00", None, 0.0, 0.5, 0.0, 0.0, None);
        assert!(svg.contains(r#"opacity="0.50""#));
    }

    #[test]
    fn test_shape_with_rotation() {
        let svg = shape_to_svg(&ShapeType::Rect, 0.0, 0.0, 100.0, 100.0, "#F00", None, 0.0, 1.0, 45.0, 0.0, None);
        assert!(svg.contains(r#"transform="rotate(45.00"#));
    }

    #[test]
    fn test_shape_with_dasharray() {
        let svg = shape_to_svg(&ShapeType::Rect, 0.0, 0.0, 100.0, 100.0, "#F00", None, 0.0, 1.0, 0.0, 0.0, Some("4 2"));
        assert!(svg.contains(r#"stroke-dasharray="4 2""#));
    }

    #[test]
    fn test_triangle_svg_output() {
        let svg = shape_to_svg(&ShapeType::Triangle, 50.0, 50.0, 100.0, 100.0, "#FF0000", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("points="));
        assert!(svg.contains(r##"fill="#FF0000""##));
    }

    #[test]
    fn test_arrow_right_svg_output() {
        let svg = shape_to_svg(&ShapeType::ArrowRight, 50.0, 50.0, 100.0, 100.0, "#00FF00", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("points="));
        assert!(svg.contains(r##"fill="#00FF00""##));
    }

    #[test]
    fn test_cross_svg_output() {
        let svg = shape_to_svg(&ShapeType::Cross, 50.0, 50.0, 100.0, 100.0, "#0000FF", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("points="));
        assert!(svg.contains(r##"fill="#0000FF""##));
    }

    #[test]
    fn test_heart_svg_output() {
        let svg = shape_to_svg(&ShapeType::Heart, 50.0, 50.0, 100.0, 100.0, "#FF69B4", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<path"));
        assert!(svg.contains("d="));
        assert!(svg.contains(r##"fill="#FF69B4""##));
    }

    #[test]
    fn test_pentagon_svg_output() {
        let svg = shape_to_svg(&ShapeType::Pentagon, 50.0, 50.0, 100.0, 100.0, "#ABCDEF", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("points="));
        assert!(svg.contains(r##"fill="#ABCDEF""##));
    }

    #[test]
    fn test_octagon_svg_output() {
        let svg = shape_to_svg(&ShapeType::Octagon, 50.0, 50.0, 100.0, 100.0, "#123456", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("points="));
        assert!(svg.contains(r##"fill="#123456""##));
    }

    #[test]
    fn test_wave_svg_output() {
        let svg = shape_to_svg(&ShapeType::Wave, 0.0, 0.0, 100.0, 50.0, "#F00", None, 0.0, 1.0, 0.0, 0.0, None);
        assert!(svg.contains("<path"));
        assert!(svg.contains("d="));
        assert!(svg.contains(r##"fill="#F00""##));
    }
}
