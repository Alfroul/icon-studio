//! Geometric boolean operations (Union, Subtract, Intersect, Exclude) for shapes and paths.
//!
//! Uses the `geo` crate's `BooleanOps` trait on `MultiPolygon<f64>`. Bezier curves are
//! approximated as line-segment polygons before operations. Results are always polygons
//! (no bezier curves in output).

use crate::error::AppError;
use crate::model::helpers;
use crate::model::{CommonProps, Element, PathElement};
use crate::model::shapes::ShapeType;
use geo::{BooleanOps, Coord, LineString, MultiPolygon, Polygon};
use std::f64::consts::PI;

/// Boolean operation types.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BooleanOp {
    Union,
    Subtract,
    Intersect,
    Exclude,
}

/// Non-destructive recipe that records how a boolean result was produced.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BooleanSource {
    pub element_a: serde_json::Value,
    pub element_b: serde_json::Value,
    pub operation: BooleanOp,
}

// ---------------------------------------------------------------------------
// Shape → SVG path `d` conversion
// ---------------------------------------------------------------------------

/// Convert any `ShapeType` to an SVG path `d` attribute string.
/// Uses the same vertex/curve math as `model::shapes`.
pub fn shape_to_path_d(
    shape: &ShapeType,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    border_radius: f64,
) -> String {
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let r = w.min(h) / 2.0;

    match shape {
        ShapeType::Circle => {
            // 64-segment polygon approximation of a circle
            let n = 64usize;
            let pts: Vec<String> = (0..n)
                .map(|i| {
                    let angle = (i as f64) * (2.0 * PI / n as f64);
                    let px = cx + r * angle.cos();
                    let py = cy + r * angle.sin();
                    format!("{:.2},{:.2}", px, py)
                })
                .collect();
            format!("M {} Z", pts.join(" L "))
        }

        ShapeType::Rect => {
            if border_radius.abs() > f64::EPSILON {
                rounded_rect_path_d(x, y, w, h, border_radius, border_radius)
            } else {
                format!(
                    "M {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} Z",
                    x, y,
                    x + w, y,
                    x + w, y + h,
                    x, y + h,
                )
            }
        }

        ShapeType::RoundedRect => {
            let rx = if border_radius.abs() > f64::EPSILON {
                border_radius
            } else {
                w.min(h) * 0.15
            };
            rounded_rect_path_d(x, y, w, h, rx, rx)
        }

        ShapeType::Hexagon => {
            let pts: Vec<String> = (0..6)
                .map(|i| {
                    let angle = (i as f64) * (PI / 3.0) - PI / 6.0;
                    let px = cx + r * angle.cos();
                    let py = cy + r * angle.sin();
                    format!("{:.2},{:.2}", px, py)
                })
                .collect();
            format!("M {} Z", pts.join(" L "))
        }

        ShapeType::Star => {
            let inner_r = r * 0.38;
            let pts: Vec<String> = (0..10)
                .map(|i| {
                    let angle = (i as f64) * (PI / 5.0) - PI / 2.0;
                    let ri = if i % 2 == 0 { r } else { inner_r };
                    let px = cx + ri * angle.cos();
                    let py = cy + ri * angle.sin();
                    format!("{:.2},{:.2}", px, py)
                })
                .collect();
            format!("M {} Z", pts.join(" L "))
        }

        ShapeType::Shield => {
            let left = cx - w / 2.0;
            let right = cx + w / 2.0;
            let top = cy - h / 2.0;
            let bottom = cy + h / 2.0;
            let side_y = top + h * 0.65;
            let ctrl_y = top + h * 0.85;

            // Shield uses Q (quadratic bezier) — we'll output it directly and rely on
            // path_d_to_polygon to handle Q commands.
            format!(
                "M {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2} Q {:.2} {:.2} {:.2} {:.2} Q {:.2} {:.2} {:.2} {:.2} Z",
                left, top,
                right, top,
                right, side_y,
                right, ctrl_y, cx, bottom,
                left, ctrl_y, left, side_y,
            )
        }

        ShapeType::Diamond => {
            format!(
                "M {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} Z",
                cx, cy - h / 2.0,
                cx + w / 2.0, cy,
                cx, cy + h / 2.0,
                cx - w / 2.0, cy,
            )
        }

        ShapeType::Triangle => {
            let pts: Vec<String> = (0..3)
                .map(|i| {
                    let angle = (i as f64) * (2.0 * PI / 3.0) - PI / 2.0;
                    let px = cx + r * angle.cos();
                    let py = cy + r * angle.sin();
                    format!("{:.2},{:.2}", px, py)
                })
                .collect();
            format!("M {} Z", pts.join(" L "))
        }

        ShapeType::ArrowRight => {
            let left = cx - w / 2.0;
            let right = cx + w / 2.0;
            let top = cy - h / 2.0;
            let bottom = cy + h / 2.0;
            let mid_x = left + w * 0.6;
            format!(
                "M {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} Z",
                left, top,
                mid_x, top,
                right, cy,
                mid_x, bottom,
                left, bottom,
            )
        }

        ShapeType::Cross => {
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
            format!(
                "M {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} L {:.2},{:.2} Z",
                il, top,
                ir, top,
                ir, it,
                right, it,
                right, ib,
                ir, ib,
                ir, bottom,
                il, bottom,
                il, ib,
                left, ib,
                left, it,
                il, it,
            )
        }

        ShapeType::Heart => {
            let top = cy - h / 2.0;
            let bottom = cy + h / 2.0;
            let left = cx - w / 2.0;
            let right = cx + w / 2.0;
            format!(
                "M {:.2} {:.2} C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} Z",
                cx, bottom,
                left, cy,
                left, top,
                cx - w * 0.25, top,
                cx + w * 0.25, top,
                right, top,
                cx, bottom,
            )
        }

        ShapeType::Pentagon => {
            let pts: Vec<String> = (0..5)
                .map(|i| {
                    let angle = (i as f64) * (2.0 * PI / 5.0) - PI / 2.0;
                    let px = cx + r * angle.cos();
                    let py = cy + r * angle.sin();
                    format!("{:.2},{:.2}", px, py)
                })
                .collect();
            format!("M {} Z", pts.join(" L "))
        }

        ShapeType::Octagon => {
            let pts: Vec<String> = (0..8)
                .map(|i| {
                    let angle = (i as f64) * (2.0 * PI / 8.0) - PI / 8.0;
                    let px = cx + r * angle.cos();
                    let py = cy + r * angle.sin();
                    format!("{:.2},{:.2}", px, py)
                })
                .collect();
            format!("M {} Z", pts.join(" L "))
        }

        ShapeType::Wave => {
            let mid_y = y + h / 2.0;
            let amp = h / 2.0;
            let period = w / 2.0;
            let cp_offset = period / 2.0 * 0.5523;

            // Wave is normally open; close it by adding a line back to start
            format!(
                "M {:.2} {:.2} \
                 C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} \
                 C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} \
                 C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} \
                 C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} \
                 L {:.2} {:.2} Z",
                x, mid_y,
                x + period / 2.0 - cp_offset, mid_y - amp * 1.4,
                x + period / 2.0 + cp_offset, mid_y - amp * 1.4,
                x + period, mid_y,
                x + period + period / 2.0 - cp_offset, mid_y + amp * 1.4,
                x + period + period / 2.0 + cp_offset, mid_y + amp * 1.4,
                x + 2.0 * period, mid_y,
                x + 2.0 * period + period / 2.0 - cp_offset, mid_y - amp * 1.4,
                x + 2.0 * period + period / 2.0 + cp_offset, mid_y - amp * 1.4,
                x + 3.0 * period, mid_y,
                x + 3.0 * period + period / 2.0 - cp_offset, mid_y + amp * 1.4,
                x + 3.0 * period + period / 2.0 + cp_offset, mid_y + amp * 1.4,
                x + 4.0 * period, mid_y,
                x, mid_y,
            )
        }

        ShapeType::Custom { d } => d.clone(),
    }
}

/// Build rounded-rect as SVG path `d` using arc commands.
fn rounded_rect_path_d(x: f64, y: f64, w: f64, h: f64, rx: f64, ry: f64) -> String {
    // Clamp radius so it doesn't exceed half the dimension
    let rx = rx.min(w / 2.0);
    let ry = ry.min(h / 2.0);
    format!(
        "M {:.2},{:.2} \
         L {:.2},{:.2} \
         A {:.2},{:.2} 0 0,1 {:.2},{:.2} \
         L {:.2},{:.2} \
         A {:.2},{:.2} 0 0,1 {:.2},{:.2} \
         L {:.2},{:.2} \
         A {:.2},{:.2} 0 0,1 {:.2},{:.2} \
         L {:.2},{:.2} \
         A {:.2},{:.2} 0 0,1 {:.2},{:.2} \
         Z",
        x + rx, y,
        x + w - rx, y,
        rx, ry, x + w, y + ry,
        x + w, y + h - ry,
        rx, ry, x + w - rx, y + h,
        x + rx, y + h,
        rx, ry, x, y + h - ry,
        x, y + ry,
        rx, ry, x + rx, y,
    )
}

// ---------------------------------------------------------------------------
// SVG path `d` → MultiPolygon conversion
// ---------------------------------------------------------------------------

/// Approximate a cubic bezier curve as a series of line-segment points.
fn cubic_bezier_points(
    p0: Coord<f64>,
    p1: Coord<f64>,
    p2: Coord<f64>,
    p3: Coord<f64>,
    segments: usize,
) -> Vec<Coord<f64>> {
    (1..=segments)
        .map(|i| {
            let t = i as f64 / segments as f64;
            let t2 = t * t;
            let t3 = t2 * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;
            Coord {
                x: mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
                y: mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
            }
        })
        .collect()
}

/// Approximate a quadratic bezier curve as a series of line-segment points.
fn quadratic_bezier_points(
    p0: Coord<f64>,
    p1: Coord<f64>,
    p2: Coord<f64>,
    segments: usize,
) -> Vec<Coord<f64>> {
    (1..=segments)
        .map(|i| {
            let t = i as f64 / segments as f64;
            let mt = 1.0 - t;
            Coord {
                x: mt * mt * p0.x + 2.0 * mt * t * p1.x + t * t * p2.x,
                y: mt * mt * p0.y + 2.0 * mt * t * p1.y + t * t * p2.y,
            }
        })
        .collect()
}

/// Parse an SVG path `d` attribute and convert to a `MultiPolygon<f64>`.
///
/// Handles M/m, L/l, H/h, V/v, C/c, Q/q, A/a, Z/z commands.
/// Curves are approximated with 16 line segments each.
/// Arcs are approximated with 16 line segments.
pub fn path_d_to_polygon(d: &str) -> Result<MultiPolygon<f64>, AppError> {
    let tokens = helpers::tokenize_d(d);
    if tokens.is_empty() {
        return Err(AppError::BuildError("Empty path data".to_string()));
    }

    let mut polygons: Vec<Polygon<f64>> = Vec::new();
    let mut current_points: Vec<Coord<f64>> = Vec::new();
    let mut subpath_start = Coord { x: 0.0, y: 0.0 };
    let mut cursor = Coord { x: 0.0, y: 0.0 };
    let bezier_segments = 16;

    let mut i = 0;
    while i < tokens.len() {
        let cmd = &tokens[i];
        match cmd.as_str() {
            "M" => {
                // Close any open subpath first
                if current_points.len() >= 3 {
                    let ring = LineString::from(current_points.clone());
                    polygons.push(Polygon::new(ring, Vec::new()));
                }
                current_points.clear();

                i += 1;
                if i + 1 >= tokens.len() {
                    break;
                }
                let x: f64 = tokens[i].parse().unwrap_or(0.0);
                let y: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                let pt = Coord { x, y };
                cursor = pt;
                subpath_start = pt;
                current_points.push(pt);
                i += 2;

                // Implicit L commands after M
                while i + 1 < tokens.len() && !is_command(&tokens[i]) {
                    let lx: f64 = tokens[i].parse().unwrap_or(0.0);
                    let ly: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                    let pt = Coord { x: lx, y: ly };
                    cursor = pt;
                    current_points.push(pt);
                    i += 2;
                }
            }
            "m" => {
                if current_points.len() >= 3 {
                    let ring = LineString::from(current_points.clone());
                    polygons.push(Polygon::new(ring, Vec::new()));
                }
                current_points.clear();

                i += 1;
                if i + 1 >= tokens.len() {
                    break;
                }
                let dx: f64 = tokens[i].parse().unwrap_or(0.0);
                let dy: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                let pt = Coord {
                    x: cursor.x + dx,
                    y: cursor.y + dy,
                };
                cursor = pt;
                subpath_start = pt;
                current_points.push(pt);
                i += 2;

                // Implicit l commands after m
                while i + 1 < tokens.len() && !is_command(&tokens[i]) {
                    let dx: f64 = tokens[i].parse().unwrap_or(0.0);
                    let dy: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                    let pt = Coord {
                        x: cursor.x + dx,
                        y: cursor.y + dy,
                    };
                    cursor = pt;
                    current_points.push(pt);
                    i += 2;
                }
            }
            "L" => {
                i += 1;
                while i + 1 < tokens.len() && !is_command(&tokens[i]) {
                    let x: f64 = tokens[i].parse().unwrap_or(0.0);
                    let y: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                    let pt = Coord { x, y };
                    cursor = pt;
                    current_points.push(pt);
                    i += 2;
                }
            }
            "l" => {
                i += 1;
                while i + 1 < tokens.len() && !is_command(&tokens[i]) {
                    let dx: f64 = tokens[i].parse().unwrap_or(0.0);
                    let dy: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                    let pt = Coord {
                        x: cursor.x + dx,
                        y: cursor.y + dy,
                    };
                    cursor = pt;
                    current_points.push(pt);
                    i += 2;
                }
            }
            "H" => {
                i += 1;
                while i < tokens.len() && !is_command(&tokens[i]) {
                    let x: f64 = tokens[i].parse().unwrap_or(0.0);
                    cursor.x = x;
                    current_points.push(cursor);
                    i += 1;
                }
            }
            "h" => {
                i += 1;
                while i < tokens.len() && !is_command(&tokens[i]) {
                    let dx: f64 = tokens[i].parse().unwrap_or(0.0);
                    cursor.x += dx;
                    current_points.push(cursor);
                    i += 1;
                }
            }
            "V" => {
                i += 1;
                while i < tokens.len() && !is_command(&tokens[i]) {
                    let y: f64 = tokens[i].parse().unwrap_or(0.0);
                    cursor.y = y;
                    current_points.push(cursor);
                    i += 1;
                }
            }
            "v" => {
                i += 1;
                while i < tokens.len() && !is_command(&tokens[i]) {
                    let dy: f64 = tokens[i].parse().unwrap_or(0.0);
                    cursor.y += dy;
                    current_points.push(cursor);
                    i += 1;
                }
            }
            "C" => {
                i += 1;
                while i + 5 < tokens.len() && !is_command(&tokens[i]) {
                    let x1: f64 = tokens[i].parse().unwrap_or(0.0);
                    let y1: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                    let x2: f64 = tokens[i + 2].parse().unwrap_or(0.0);
                    let y2: f64 = tokens[i + 3].parse().unwrap_or(0.0);
                    let x3: f64 = tokens[i + 4].parse().unwrap_or(0.0);
                    let y3: f64 = tokens[i + 5].parse().unwrap_or(0.0);

                    let pts = cubic_bezier_points(
                        cursor,
                        Coord { x: x1, y: y1 },
                        Coord { x: x2, y: y2 },
                        Coord { x: x3, y: y3 },
                        bezier_segments,
                    );
                    current_points.extend_from_slice(&pts);
                    cursor = Coord { x: x3, y: y3 };
                    i += 6;
                }
            }
            "c" => {
                i += 1;
                while i + 5 < tokens.len() && !is_command(&tokens[i]) {
                    let dx1: f64 = tokens[i].parse().unwrap_or(0.0);
                    let dy1: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                    let dx2: f64 = tokens[i + 2].parse().unwrap_or(0.0);
                    let dy2: f64 = tokens[i + 3].parse().unwrap_or(0.0);
                    let dx3: f64 = tokens[i + 4].parse().unwrap_or(0.0);
                    let dy3: f64 = tokens[i + 5].parse().unwrap_or(0.0);

                    let p1 = Coord {
                        x: cursor.x + dx1,
                        y: cursor.y + dy1,
                    };
                    let p2 = Coord {
                        x: cursor.x + dx2,
                        y: cursor.y + dy2,
                    };
                    let p3 = Coord {
                        x: cursor.x + dx3,
                        y: cursor.y + dy3,
                    };

                    let pts = cubic_bezier_points(cursor, p1, p2, p3, bezier_segments);
                    current_points.extend_from_slice(&pts);
                    cursor = p3;
                    i += 6;
                }
            }
            "Q" => {
                i += 1;
                while i + 3 < tokens.len() && !is_command(&tokens[i]) {
                    let x1: f64 = tokens[i].parse().unwrap_or(0.0);
                    let y1: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                    let x2: f64 = tokens[i + 2].parse().unwrap_or(0.0);
                    let y2: f64 = tokens[i + 3].parse().unwrap_or(0.0);

                    let pts = quadratic_bezier_points(
                        cursor,
                        Coord { x: x1, y: y1 },
                        Coord { x: x2, y: y2 },
                        bezier_segments,
                    );
                    current_points.extend_from_slice(&pts);
                    cursor = Coord { x: x2, y: y2 };
                    i += 4;
                }
            }
            "q" => {
                i += 1;
                while i + 3 < tokens.len() && !is_command(&tokens[i]) {
                    let dx1: f64 = tokens[i].parse().unwrap_or(0.0);
                    let dy1: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                    let dx2: f64 = tokens[i + 2].parse().unwrap_or(0.0);
                    let dy2: f64 = tokens[i + 3].parse().unwrap_or(0.0);

                    let p1 = Coord {
                        x: cursor.x + dx1,
                        y: cursor.y + dy1,
                    };
                    let p2 = Coord {
                        x: cursor.x + dx2,
                        y: cursor.y + dy2,
                    };

                    let pts = quadratic_bezier_points(cursor, p1, p2, bezier_segments);
                    current_points.extend_from_slice(&pts);
                    cursor = p2;
                    i += 4;
                }
            }
            "A" | "a" => {
                let relative = cmd == "a";
                i += 1;
                while i + 6 < tokens.len() && !is_command(&tokens[i]) {
                    let rx: f64 = tokens[i].parse().unwrap_or(0.0);
                    let ry: f64 = tokens[i + 1].parse().unwrap_or(0.0);
                    let x_rot: f64 = tokens[i + 2].parse().unwrap_or(0.0);
                    let large_arc: f64 = tokens[i + 3].parse().unwrap_or(0.0);
                    let sweep: f64 = tokens[i + 4].parse().unwrap_or(0.0);
                    let x: f64 = tokens[i + 5].parse().unwrap_or(0.0);
                    let y: f64 = tokens[i + 6].parse().unwrap_or(0.0);

                    let end = if relative {
                        Coord { x: cursor.x + x, y: cursor.y + y }
                    } else {
                        Coord { x, y }
                    };

                    if rx.abs() < f64::EPSILON || ry.abs() < f64::EPSILON {
                        current_points.push(end);
                    } else {
                        let n_segments = 16usize;
                        let rot_rad = x_rot.to_radians();
                        let cos_rot = rot_rad.cos();
                        let sin_rot = rot_rad.sin();
                        let mut d_theta;

                        let dx = (cursor.x - end.x) / 2.0;
                        let dy = (cursor.y - end.y) / 2.0;
                        let x1p = cos_rot * dx + sin_rot * dy;
                        let y1p = -sin_rot * dx + cos_rot * dy;

                        let lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
                        let sq = if lambda > 1.0 { (lambda).sqrt() } else { 1.0 };
                        let rx_adj = rx * sq;
                        let ry_adj = ry * sq;

                        let num = rx_adj * rx_adj * (ry_adj * ry_adj)
                            - rx_adj * rx_adj * y1p * y1p
                            - ry_adj * ry_adj * x1p * x1p;
                        let den = rx_adj * rx_adj * y1p * y1p + ry_adj * ry_adj * x1p * x1p;
                        let sc = if den.abs() < f64::EPSILON { 0.0 } else { (num / den).abs().sqrt() };
                        let sc = if large_arc > 0.5 && sweep <= 0.5
                            || large_arc < 0.5 && sweep > 0.5 { sc } else { -sc };

                        let cxp = sc * rx_adj * y1p / ry_adj;
                        let cyp = -sc * ry_adj * x1p / rx_adj;

                        let cx = cos_rot * cxp - sin_rot * cyp + (cursor.x + end.x) / 2.0;
                        let cy = sin_rot * cxp + cos_rot * cyp + (cursor.y + end.y) / 2.0;

                        fn angle(ux: f64, uy: f64, vx: f64, vy: f64) -> f64 {
                            let dot = ux * vx + uy * vy;
                            let len = (ux * ux + uy * uy).sqrt() * (vx * vx + vy * vy).sqrt();
                            let ang = if len < f64::EPSILON { 0.0 } else { (dot / len).clamp(-1.0, 1.0).acos() };
                            if ux * vy - uy * vx < 0.0 { -ang } else { ang }
                        }

                        let theta1 = angle((x1p - cxp) / rx_adj, (y1p - cyp) / ry_adj, 1.0, 0.0);
                        d_theta = angle(
                            (x1p - cxp) / rx_adj, (y1p - cyp) / ry_adj,
                            (-x1p - cxp) / rx_adj, (-y1p - cyp) / ry_adj,
                        );

                        if sweep > 0.5 && d_theta < 0.0 {
                            d_theta += 2.0 * std::f64::consts::PI;
                        } else if sweep <= 0.5 && d_theta > 0.0 {
                            d_theta -= 2.0 * std::f64::consts::PI;
                        }

                        for seg_i in 1..=n_segments {
                            let t = seg_i as f64 / n_segments as f64;
                            let a = theta1 + d_theta * t;
                            let px = cos_rot * rx_adj * a.cos() - sin_rot * ry_adj * a.sin() + cx;
                            let py = sin_rot * rx_adj * a.cos() + cos_rot * ry_adj * a.sin() + cy;
                            current_points.push(Coord { x: px, y: py });
                        }
                    }

                    cursor = end;
                    i += 7;
                }
            }
            "Z" | "z" => {
                // Close path — ensure the ring is closed back to subpath start
                if !current_points.is_empty() {
                    let last = *current_points.last().unwrap_or(&Coord { x: 0.0, y: 0.0 });
                    if (last.x - subpath_start.x).abs() > f64::EPSILON
                        || (last.y - subpath_start.y).abs() > f64::EPSILON
                    {
                        current_points.push(subpath_start);
                    }
                }
                if current_points.len() >= 3 {
                    let ring = LineString::from(current_points.clone());
                    polygons.push(Polygon::new(ring, Vec::new()));
                }
                current_points.clear();
                cursor = subpath_start;
                i += 1;
            }
            _ => {
                // Unknown command — skip
                i += 1;
            }
        }
    }

    // Close any remaining open subpath
    if current_points.len() >= 3 {
        let ring = LineString::from(current_points);
        polygons.push(Polygon::new(ring, Vec::new()));
    }

    if polygons.is_empty() {
        return Err(AppError::BuildError(
            "Path data did not produce any polygons".to_string(),
        ));
    }

    Ok(MultiPolygon(polygons))
}

/// Check if a token is an SVG path command letter.
fn is_command(s: &str) -> bool {
    matches!(
        s,
        "M" | "m"
            | "L" | "l"
            | "H" | "h"
            | "V" | "v"
            | "C" | "c"
            | "S" | "s"
            | "Q" | "q"
            | "T" | "t"
            | "A" | "a"
            | "Z" | "z"
    )
}

// ---------------------------------------------------------------------------
// Polygon → SVG path `d` conversion
// ---------------------------------------------------------------------------

/// Convert a single `Polygon` to an SVG path `d` string.
pub fn polygon_to_path_d(poly: &Polygon<f64>) -> String {
    let mut parts = Vec::new();

    // Exterior ring
    let exterior = poly.exterior();
    let coords = exterior.0.as_slice();
    if coords.len() >= 2 {
        let mut d = format!("M {:.2},{:.2}", coords[0].x, coords[0].y);
        for c in &coords[1..] {
            d.push_str(&format!(" L {:.2},{:.2}", c.x, c.y));
        }
        d.push_str(" Z");
        parts.push(d);
    }

    // Interior rings (holes)
    for interior in poly.interiors() {
        let coords = interior.0.as_slice();
        if coords.len() >= 2 {
            let mut d = format!("M {:.2},{:.2}", coords[0].x, coords[0].y);
            for c in &coords[1..] {
                d.push_str(&format!(" L {:.2},{:.2}", c.x, c.y));
            }
            d.push_str(" Z");
            parts.push(d);
        }
    }

    parts.join(" ")
}

/// Convert a `MultiPolygon` to a single SVG path `d` string.
pub fn multipolygon_to_path_d(mp: &MultiPolygon<f64>) -> String {
    mp.0.iter().map(polygon_to_path_d).collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// Element → MultiPolygon conversion
// ---------------------------------------------------------------------------

/// Convert any `Element` to a `MultiPolygon`.
///
/// Only Shape and Path elements are supported. For Path elements the local `d`
/// coordinates are shifted by the element's `(x, y)` position into canvas space.
pub fn element_to_polygon(element: &Element) -> Result<MultiPolygon<f64>, AppError> {
    match element {
        Element::Shape(shape) => {
            let d = shape_to_path_d(
                &shape.shape_type,
                shape.common.x,
                shape.common.y,
                shape.common.width,
                shape.common.height,
                shape.border_radius,
            );
            path_d_to_polygon(&d)
        }
        Element::Path(path_elem) => {
            // Path d is in local coordinates; shift to canvas coordinates
            let shifted = helpers::shift_path_coords(
                &path_elem.d,
                -path_elem.common.x,
                -path_elem.common.y,
            );
            // Apply scale factor for scaled paths
            let natural_w = if path_elem.natural_width.abs() > f64::EPSILON {
                path_elem.natural_width
            } else {
                path_elem.common.width
            };
            let natural_h = if path_elem.natural_height.abs() > f64::EPSILON {
                path_elem.natural_height
            } else {
                path_elem.common.height
            };
            let scale_x = path_elem.common.width / natural_w;
            let scale_y = path_elem.common.height / natural_h;
            let scaled = if (scale_x - 1.0).abs() > f64::EPSILON
                || (scale_y - 1.0).abs() > f64::EPSILON
            {
                helpers::scale_path_coords(&shifted, scale_x, scale_y)
            } else {
                shifted
            };
            path_d_to_polygon(&scaled)
        }
        _ => Err(AppError::ValidationError(
            "Boolean operations only support Shape and Path elements".to_string(),
        )),
    }
}

// ---------------------------------------------------------------------------
// Boolean operation execution
// ---------------------------------------------------------------------------

/// Execute a boolean operation on two `MultiPolygon`s.
pub fn boolean_operation(
    poly_a: &MultiPolygon<f64>,
    poly_b: &MultiPolygon<f64>,
    op: BooleanOp,
) -> Result<MultiPolygon<f64>, AppError> {
    let result = match op {
        BooleanOp::Union => poly_a.union(poly_b),
        BooleanOp::Subtract => poly_a.difference(poly_b),
        BooleanOp::Intersect => poly_a.intersection(poly_b),
        BooleanOp::Exclude => poly_a.xor(poly_b),
    };

    if result.0.is_empty() {
        return Err(AppError::BuildError(
            "Boolean operation produced empty result".to_string(),
        ));
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Result → PathElement conversion
// ---------------------------------------------------------------------------

/// Convert a `MultiPolygon` result into a `PathElement`.
///
/// The path data is shifted to be relative to `(0, 0)` and the bounding box
/// position is stored in `common.x` / `common.y`.
pub fn operation_result_to_path(mp: &MultiPolygon<f64>, fill: &str) -> PathElement {
    let d = multipolygon_to_path_d(mp);

    // Calculate bounding box from all coordinates
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for poly in &mp.0 {
        for coord in poly.exterior().0.as_slice() {
            min_x = min_x.min(coord.x);
            min_y = min_y.min(coord.y);
            max_x = max_x.max(coord.x);
            max_y = max_y.max(coord.y);
        }
    }

    let width = max_x - min_x;
    let height = max_y - min_y;

    // Shift path to origin (relative to 0,0)
    let d_shifted = helpers::shift_path_coords(&d, min_x, min_y);

    PathElement {
        common: CommonProps {
            id: String::new(), // caller will assign
            x: min_x,
            y: min_y,
            width,
            height,
            opacity: 1.0,
            rotation: 0.0,
            shadows: Vec::new(),
            animation: None,
            blend_mode: None,
            clip_element_id: None,
            mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
        },
        d: d_shifted,
        fill: fill.to_string(),
        stroke: "none".to_string(),
        stroke_width: 0.0,
        stroke_dasharray: None,
        natural_width: width,
        natural_height: height,
        boolean_source: None, // caller sets this
    }
}

// ---------------------------------------------------------------------------
// High-level orchestration
// ---------------------------------------------------------------------------

/// Perform a full boolean operation: convert elements → polygons → execute → result.
///
/// Returns the resulting `PathElement` and the `BooleanSource` recipe.
pub fn perform_boolean(
    a: &Element,
    b: &Element,
    op: BooleanOp,
) -> Result<(PathElement, BooleanSource), AppError> {
    let poly_a = element_to_polygon(a)?;
    let poly_b = element_to_polygon(b)?;
    let result = boolean_operation(&poly_a, &poly_b, op.clone())?;

    // Inherit fill from element a
    let fill = match a {
        Element::Shape(s) => s.fill.clone(),
        Element::Path(p) => p.fill.clone(),
        _ => "#000000".to_string(),
    };

    let mut path_element = operation_result_to_path(&result, &fill);

    let boolean_source = BooleanSource {
        element_a: serde_json::to_value(a).map_err(|e| {
            AppError::BuildError(format!("Failed to serialize element A: {}", e))
        })?,
        element_b: serde_json::to_value(b).map_err(|e| {
            AppError::BuildError(format!("Failed to serialize element B: {}", e))
        })?,
        operation: op,
    };

    path_element.boolean_source = Some(boolean_source.clone());

    Ok((path_element, boolean_source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ShapeElement;

    #[test]
    fn test_shape_to_path_d_circle() {
        let d = shape_to_path_d(&ShapeType::Circle, 0.0, 0.0, 100.0, 100.0, 0.0);
        assert!(d.starts_with("M "));
        assert!(d.ends_with(" Z"));
        // Should contain "L" for line segments
        assert!(d.contains(" L "));
    }

    #[test]
    fn test_shape_to_path_d_rect() {
        let d = shape_to_path_d(&ShapeType::Rect, 10.0, 20.0, 100.0, 200.0, 0.0);
        assert!(d.starts_with("M "));
        assert!(d.ends_with(" Z"));
    }

    #[test]
    fn test_shape_to_path_d_hexagon() {
        let d = shape_to_path_d(&ShapeType::Hexagon, 0.0, 0.0, 100.0, 100.0, 0.0);
        assert!(d.starts_with("M "));
        assert!(d.ends_with(" Z"));
    }

    #[test]
    fn test_shape_to_path_d_star() {
        let d = shape_to_path_d(&ShapeType::Star, 0.0, 0.0, 100.0, 100.0, 0.0);
        assert!(d.starts_with("M "));
        assert!(d.ends_with(" Z"));
        assert!(d.contains(" L "));
    }

    #[test]
    fn test_shape_to_path_d_custom() {
        let d = shape_to_path_d(
            &ShapeType::Custom {
                d: "M 0 0 L 10 10 Z".to_string(),
            },
            0.0,
            0.0,
            100.0,
            100.0,
            0.0,
        );
        assert_eq!(d, "M 0 0 L 10 10 Z");
    }

    #[test]
    fn test_path_d_to_polygon_rect() {
        let d = "M 0,0 L 100,0 L 100,100 L 0,100 Z";
        let mp = path_d_to_polygon(d).unwrap();
        assert_eq!(mp.0.len(), 1);
        let exterior = &mp.0[0].exterior().0;
        assert_eq!(exterior.len(), 5); // 4 corners + closing point
    }

    #[test]
    fn test_polygon_to_path_d_roundtrip() {
        let d_in = "M 0.00,0.00 L 100.00,0.00 L 100.00,100.00 L 0.00,100.00 Z";
        let mp = path_d_to_polygon(d_in).unwrap();
        let d_out = multipolygon_to_path_d(&mp);
        assert!(d_out.starts_with("M "));
        assert!(d_out.contains("Z"));
    }

    #[test]
    fn test_boolean_union_two_rects() {
        // Two overlapping rectangles
        let d_a = "M 0,0 L 100,0 L 100,100 L 0,100 Z";
        let d_b = "M 50,0 L 150,0 L 150,100 L 50,100 Z";
        let poly_a = path_d_to_polygon(d_a).unwrap();
        let poly_b = path_d_to_polygon(d_b).unwrap();
        let result = boolean_operation(&poly_a, &poly_b, BooleanOp::Union).unwrap();
        // Union of two overlapping rects should produce one polygon
        assert_eq!(result.0.len(), 1);
    }

    #[test]
    fn test_boolean_subtract() {
        let d_a = "M 0,0 L 100,0 L 100,100 L 0,100 Z";
        let d_b = "M 50,50 L 150,50 L 150,150 L 50,150 Z";
        let poly_a = path_d_to_polygon(d_a).unwrap();
        let poly_b = path_d_to_polygon(d_b).unwrap();
        let result = boolean_operation(&poly_a, &poly_b, BooleanOp::Subtract).unwrap();
        assert_eq!(result.0.len(), 1);
    }

    #[test]
    fn test_boolean_intersect() {
        let d_a = "M 0,0 L 100,0 L 100,100 L 0,100 Z";
        let d_b = "M 50,0 L 150,0 L 150,100 L 50,100 Z";
        let poly_a = path_d_to_polygon(d_a).unwrap();
        let poly_b = path_d_to_polygon(d_b).unwrap();
        let result = boolean_operation(&poly_a, &poly_b, BooleanOp::Intersect).unwrap();
        assert_eq!(result.0.len(), 1);
    }

    #[test]
    fn test_boolean_exclude() {
        let d_a = "M 0,0 L 100,0 L 100,100 L 0,100 Z";
        let d_b = "M 50,0 L 150,0 L 150,100 L 50,100 Z";
        let poly_a = path_d_to_polygon(d_a).unwrap();
        let poly_b = path_d_to_polygon(d_b).unwrap();
        let result = boolean_operation(&poly_a, &poly_b, BooleanOp::Exclude).unwrap();
        assert_eq!(result.0.len(), 2); // Two non-overlapping regions
    }

    #[test]
    fn test_boolean_empty_result() {
        let d_a = "M 0,0 L 10,0 L 10,10 L 0,10 Z";
        let d_b = "M 100,100 L 200,100 L 200,200 L 100,200 Z";
        let poly_a = path_d_to_polygon(d_a).unwrap();
        let poly_b = path_d_to_polygon(d_b).unwrap();
        let result = boolean_operation(&poly_a, &poly_b, BooleanOp::Intersect);
        assert!(result.is_err());
    }

    #[test]
    fn test_element_to_polygon_shape() {
        let shape = ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 100.0,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
                mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
            },
            shape_type: ShapeType::Rect,
            fill: "#FF0000".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        };
        let elem = Element::Shape(shape);
        let mp = element_to_polygon(&elem).unwrap();
        assert_eq!(mp.0.len(), 1);
    }

    #[test]
    fn test_perform_boolean_full() {
        let shape_a = ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
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
                mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
            },
            shape_type: ShapeType::Rect,
            fill: "#FF0000".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        };
        let shape_b = ShapeElement {
            common: CommonProps {
                id: "shape-2".to_string(),
                x: 50.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
                mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
            },
            shape_type: ShapeType::Rect,
            fill: "#00FF00".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        };

        let (path_elem, source) =
            perform_boolean(&Element::Shape(shape_a), &Element::Shape(shape_b), BooleanOp::Union)
                .unwrap();

        assert!(path_elem.boolean_source.is_some());
        assert_eq!(source.operation, BooleanOp::Union);
        assert!(path_elem.fill == "#FF0000");
    }

    #[test]
    fn test_union_two_circles() {
        let shape_a = ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
                x: 50.0, y: 50.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
            },
            shape_type: ShapeType::Circle,
            fill: "#FF0000".to_string(), stroke: None, stroke_width: 0.0,
            border_radius: 0.0, stroke_dasharray: None, gradient: None,
        };
        let shape_b = ShapeElement {
            common: CommonProps {
                id: "shape-2".to_string(),
                x: 100.0, y: 50.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
            },
            shape_type: ShapeType::Circle,
            fill: "#00FF00".to_string(), stroke: None, stroke_width: 0.0,
            border_radius: 0.0, stroke_dasharray: None, gradient: None,
        };
        let (path_elem, source) =
            perform_boolean(&Element::Shape(shape_a), &Element::Shape(shape_b), BooleanOp::Union).unwrap();
        assert!(path_elem.boolean_source.is_some());
        assert_eq!(source.operation, BooleanOp::Union);
        // Union of two overlapping circles should produce one polygon
        assert!(!path_elem.d.is_empty());
    }

    #[test]
    fn test_subtract_circle_from_rect() {
        let rect_shape = ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
                x: 0.0, y: 0.0, width: 200.0, height: 200.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
            },
            shape_type: ShapeType::Rect,
            fill: "#0000FF".to_string(), stroke: None, stroke_width: 0.0,
            border_radius: 0.0, stroke_dasharray: None, gradient: None,
        };
        let circle_shape = ShapeElement {
            common: CommonProps {
                id: "shape-2".to_string(),
                x: 50.0, y: 50.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
            },
            shape_type: ShapeType::Circle,
            fill: "#FF0000".to_string(), stroke: None, stroke_width: 0.0,
            border_radius: 0.0, stroke_dasharray: None, gradient: None,
        };
        let (path_elem, _) =
            perform_boolean(&Element::Shape(rect_shape), &Element::Shape(circle_shape), BooleanOp::Subtract).unwrap();
        // Rect with a circular hole — should have non-empty path
        assert!(!path_elem.d.is_empty());
        // Fill should inherit from element A (rect)
        assert_eq!(path_elem.fill, "#0000FF");
    }

    #[test]
    fn test_element_to_polygon_path() {
        let path_elem = PathElement {
            common: CommonProps {
                id: "path-1".to_string(),
                x: 10.0, y: 20.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                blend_mode: None, clip_element_id: None, mask_element_id: None, locked: false, visible: true, svg_filter: None,
            overlay: None,
            },
            d: "M 0,0 L 100,0 L 100,100 L 0,100 Z".to_string(),
            fill: "#FF0000".to_string(), stroke: "none".to_string(), stroke_width: 0.0,
            stroke_dasharray: None, natural_width: 100.0, natural_height: 100.0,
            boolean_source: None,
        };
        let mp = element_to_polygon(&Element::Path(path_elem)).unwrap();
        assert_eq!(mp.0.len(), 1);
        // Path element at x=10,y=20 should shift coords by those amounts
        let exterior = &mp.0[0].exterior().0;
        assert_eq!(exterior.len(), 5); // 4 corners + closing
        // Verify coordinate shift: first point should be at (10, 20)
        assert!((exterior[0].x - 10.0).abs() < 0.01);
        assert!((exterior[0].y - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_shape_to_path_circle_produces_valid_d() {
        let d = shape_to_path_d(&ShapeType::Circle, 10.0, 20.0, 100.0, 100.0, 0.0);
        assert!(!d.is_empty());
        assert!(d.starts_with("M "));
        assert!(d.ends_with(" Z"));
    }

    #[test]
    fn test_shape_to_path_star_produces_valid_d() {
        let d = shape_to_path_d(&ShapeType::Star, 0.0, 0.0, 200.0, 200.0, 0.0);
        assert!(!d.is_empty());
        assert!(d.starts_with("M "));
        assert!(d.ends_with(" Z"));
    }

    #[test]
    fn test_unsupported_element_type() {
        let text_elem = crate::model::TextElement {
            common: CommonProps {
                id: "text-1".to_string(),
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
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
            fill: "#000".to_string(),
            font_family: "Arial".to_string(),
            font_size: 24.0,
            font_weight: "normal".to_string(),
            letter_spacing: 0.0,
            stroke: None,
            stroke_width: 0.0,
            gradient: None,
        };
        let result = element_to_polygon(&Element::Text(text_elem));
        assert!(result.is_err());
    }
}
