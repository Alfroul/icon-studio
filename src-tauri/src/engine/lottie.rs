use crate::engine::builder;
use crate::engine::exporter::render_to_png;
use crate::model::{AnimationType, Element, IconProject};
use image::codecs::gif::GifEncoder;
use serde::Serialize;
use std::io::Cursor;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Lottie JSON data structures
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LottieAnimation {
    pub v: &'static str,
    pub fr: f64,
    pub ip: f64,
    pub op: f64,
    pub w: u32,
    pub h: u32,
    pub nm: &'static str,
    pub ddd: u32,
    pub assets: Vec<serde_json::Value>,
    pub layers: Vec<LottieLayer>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LottieLayer {
    pub ddd: u32,
    pub ind: usize,
    pub ty: u32,
    pub nm: String,
    pub sr: f64,
    pub ks: LottieTransform,
    pub ao: u32,
    pub ip: f64,
    pub op: f64,
    pub st: f64,
    pub bm: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shapes: Option<Vec<serde_json::Value>>,
}

#[derive(Serialize)]
pub struct LottieTransform {
    pub a: LottieValue,
    pub p: LottieValue,
    pub s: LottieValue,
    pub r: LottieValue,
    pub o: LottieValue,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum LottieValue {
    Fixed(serde_json::Value),
    Animated { a: u32, k: Vec<LottieKeyframe> },
}

#[derive(Serialize)]
pub struct LottieKeyframe {
    pub t: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub o: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Easing helpers
// ---------------------------------------------------------------------------

fn easing_bezier(easing: &str) -> (serde_json::Value, serde_json::Value) {
    let (ix, iy, ox, oy) = match easing {
        "linear" => ([0.0], [0.0], [1.0], [1.0]),
        "ease-in" => ([0.42], [0.0], [1.0], [1.0]),
        "ease-out" => ([0.0], [0.0], [0.58], [1.0]),
        _ => ([0.42], [0.0], [0.58], [1.0]), // ease-in-out default
    };
    let i = serde_json::json!({ "x": ix, "y": iy });
    let o = serde_json::json!({ "x": ox, "y": oy });
    (i, o)
}

// ---------------------------------------------------------------------------
// Build helpers
// ---------------------------------------------------------------------------

fn fixed_vec2(v: [f64; 2]) -> LottieValue {
    LottieValue::Fixed(serde_json::json!(v))
}

fn fixed_num(v: f64) -> LottieValue {
    LottieValue::Fixed(serde_json::json!(v))
}

fn static_transform(cx: f64, cy: f64, opacity: f64, rotation: f64) -> LottieTransform {
    LottieTransform {
        a: fixed_vec2([0.0, 0.0]),
        p: fixed_vec2([cx, cy]),
        s: fixed_vec2([100.0, 100.0]),
        r: fixed_num(rotation),
        o: fixed_num(opacity * 100.0),
    }
}

#[allow(dead_code)]
fn animated_value(
    start: serde_json::Value,
    end: serde_json::Value,
    end_frame: f64,
    easing: &str,
) -> LottieValue {
    let (i, o) = easing_bezier(easing);
    LottieValue::Animated {
        a: 1,
        k: vec![
            LottieKeyframe {
                t: 0.0,
                s: Some(start),
                e: Some(end.clone()),
                i: Some(i),
                o: Some(o),
            },
            LottieKeyframe {
                t: end_frame,
                s: Some(end),
                e: None,
                i: None,
                o: None,
            },
        ],
    }
}

fn animated_rotation(end_frame: f64, easing: &str) -> LottieValue {
    let (i, o) = easing_bezier(easing);
    LottieValue::Animated {
        a: 1,
        k: vec![
            LottieKeyframe {
                t: 0.0,
                s: Some(serde_json::json!(0)),
                e: Some(serde_json::json!(360)),
                i: Some(i),
                o: Some(o),
            },
            LottieKeyframe {
                t: end_frame,
                s: Some(serde_json::json!(360)),
                e: None,
                i: None,
                o: None,
            },
        ],
    }
}

fn animated_scale(scale_to: f64, end_frame: f64, easing: &str) -> LottieValue {
    let (i, o) = easing_bezier(easing);
    let to_pct = scale_to * 100.0;
    LottieValue::Animated {
        a: 1,
        k: vec![
            LottieKeyframe {
                t: 0.0,
                s: Some(serde_json::json!([100.0, 100.0])),
                e: Some(serde_json::json!([to_pct, to_pct])),
                i: Some(i),
                o: Some(o),
            },
            LottieKeyframe {
                t: end_frame,
                s: Some(serde_json::json!([to_pct, to_pct])),
                e: None,
                i: None,
                o: None,
            },
        ],
    }
}

fn animated_opacity(min_opacity: f64, end_frame: f64, easing: &str) -> LottieValue {
    let (i, o) = easing_bezier(easing);
    LottieValue::Animated {
        a: 1,
        k: vec![
            LottieKeyframe {
                t: 0.0,
                s: Some(serde_json::json!(100)),
                e: Some(serde_json::json!(min_opacity * 100.0)),
                i: Some(i),
                o: Some(o),
            },
            LottieKeyframe {
                t: end_frame,
                s: Some(serde_json::json!(min_opacity * 100.0)),
                e: None,
                i: None,
                o: None,
            },
        ],
    }
}

fn animated_position(
    cx: f64,
    cy: f64,
    dx: f64,
    dy: f64,
    end_frame: f64,
    easing: &str,
) -> LottieValue {
    let (i, o) = easing_bezier(easing);
    LottieValue::Animated {
        a: 1,
        k: vec![
            LottieKeyframe {
                t: 0.0,
                s: Some(serde_json::json!([cx, cy])),
                e: Some(serde_json::json!([cx + dx, cy + dy])),
                i: Some(i),
                o: Some(o),
            },
            LottieKeyframe {
                t: end_frame,
                s: Some(serde_json::json!([cx + dx, cy + dy])),
                e: None,
                i: None,
                o: None,
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Element shape description for Lottie
// ---------------------------------------------------------------------------

fn element_shapes(elem: &Element) -> Vec<serde_json::Value> {
    match elem {
        Element::Shape(s) => {
            let fill = serde_json::json!({
                "ty": "fl",
                "c": { "a": 0, "k": hex_to_lottie_color(&s.fill) },
                "o": { "a": 0, "k": 100 },
                "r": 1
            });
            let shape = shape_item_for(&s.shape_type, s.common.width, s.common.height, s.border_radius);
            vec![shape, fill]
        }
        Element::Text(t) => {
            let fill = serde_json::json!({
                "ty": "fl",
                "c": { "a": 0, "k": hex_to_lottie_color(&t.fill) },
                "o": { "a": 0, "k": 100 },
                "r": 1
            });
            let text = serde_json::json!({
                "ty": "el",
                "p": { "a": 0, "k": [t.common.width / 2.0, t.common.height / 2.0] },
                "s": { "a": 0, "k": [t.common.width, t.common.height] },
                "nm": t.content
            });
            vec![text, fill]
        }
        _ => {
            let common = elem.common();
            let rect = serde_json::json!({
                "ty": "rc",
                "d": 1,
                "s": { "a": 0, "k": [common.width, common.height] },
                "p": { "a": 0, "k": [common.width / 2.0, common.height / 2.0] },
                "r": { "a": 0, "k": 0 },
                "nm": elem.id()
            });
            let fill = serde_json::json!({
                "ty": "fl",
                "c": { "a": 0, "k": [1, 1, 1, 1] },
                "o": { "a": 0, "k": 100 },
                "r": 1
            });
            vec![rect, fill]
        }
    }
}

fn shape_item_for(shape_type: &crate::model::shapes::ShapeType, w: f64, h: f64, border_radius: f64) -> serde_json::Value {
    match shape_type {
        crate::model::shapes::ShapeType::Circle => serde_json::json!({
            "ty": "el",
            "p": { "a": 0, "k": [w / 2.0, h / 2.0] },
            "s": { "a": 0, "k": [w, h] },
            "nm": "circle"
        }),
        crate::model::shapes::ShapeType::RoundedRect => serde_json::json!({
            "ty": "rc",
            "d": 1,
            "s": { "a": 0, "k": [w, h] },
            "p": { "a": 0, "k": [w / 2.0, h / 2.0] },
            "r": { "a": 0, "k": border_radius },
            "nm": "rounded-rect"
        }),
        _ => serde_json::json!({
            "ty": "rc",
            "d": 1,
            "s": { "a": 0, "k": [w, h] },
            "p": { "a": 0, "k": [w / 2.0, h / 2.0] },
            "r": { "a": 0, "k": border_radius },
            "nm": "shape"
        }),
    }
}

fn hex_to_lottie_color(hex: &str) -> serde_json::Value {
    let hex = hex.trim_start_matches('#');
    let (r, g, b) = if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(0);
        ((r * 17) as f64 / 255.0, (g * 17) as f64 / 255.0, (b * 17) as f64 / 255.0)
    } else {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
        (r, g, b)
    };
    let a = if hex.len() > 7 {
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f64 / 255.0
    } else {
        1.0
    };
    serde_json::json!([r, g, b, a])
}

// ---------------------------------------------------------------------------
// Main export entry
// ---------------------------------------------------------------------------

pub fn export_lottie(project: &IconProject, fps: Option<f64>) -> Result<String, String> {
    let canvas = project.active_canvas();
    let elements = project.active_elements();
    let fps = fps.unwrap_or(30.0);

    let mut layers = Vec::new();
    for (idx, elem) in elements.iter().enumerate() {
        if !elem.common().visible {
            continue;
        }
        let common = elem.common();
        let cx = common.x + common.width / 2.0;
        let cy = common.y + common.height / 2.0;

        let shapes = Some(element_shapes(elem));

        let ks = if let Some(ref anim) = common.animation {
            let end_frame = (anim.duration * fps).round();

            match anim.animation_type {
                AnimationType::Rotate => {
                    let mut st = static_transform(cx, cy, common.opacity, common.rotation);
                    st.r = animated_rotation(end_frame, &anim.easing);
                    st
                }
                AnimationType::Scale => {
                    let scale_to = anim
                        .params
                        .get("min_scale")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.8);
                    let mut st = static_transform(cx, cy, common.opacity, common.rotation);
                    st.s = animated_scale(scale_to, end_frame, &anim.easing);
                    st
                }
                AnimationType::Fade => {
                    let min_opacity = anim
                        .params
                        .get("min_opacity")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let mut st = static_transform(cx, cy, common.opacity, common.rotation);
                    st.o = animated_opacity(min_opacity, end_frame, &anim.easing);
                    st
                }
                AnimationType::Translate => {
                    let dx = anim.params.get("dx").and_then(|v| v.as_f64()).unwrap_or(10.0);
                    let dy = anim.params.get("dy").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let mut st = static_transform(cx, cy, common.opacity, common.rotation);
                    st.p = animated_position(cx, cy, dx, dy, end_frame, &anim.easing);
                    st
                }
                AnimationType::Path => {
                    static_transform(cx, cy, common.opacity, common.rotation)
                }
            }
        } else {
            static_transform(cx, cy, common.opacity, common.rotation)
        };

        layers.push(LottieLayer {
            ddd: 0,
            ind: idx,
            ty: 4,
            nm: common.id.clone(),
            sr: 1.0,
            ks,
            ao: 0,
            ip: 0.0,
            op: fps * 3.0,
            st: 0.0,
            bm: 0,
            shapes,
        });
    }

    let animation = LottieAnimation {
        v: "5.7.0",
        fr: fps,
        ip: 0.0,
        op: fps * 3.0,
        w: canvas.width,
        h: canvas.height,
        nm: "IconStudio Export",
        ddd: 0,
        assets: vec![],
        layers,
    };

    serde_json::to_string_pretty(&animation).map_err(|e| format!("Lottie serialization failed: {e}"))
}

// ---------------------------------------------------------------------------
// GIF export
// ---------------------------------------------------------------------------

pub fn export_gif_frames(
    project: &IconProject,
    fps: u32,
) -> Result<Vec<Vec<u8>>, String> {
    let elements = project.active_elements();
    let has_animation = elements.iter().any(|e| {
        e.common().visible && e.common().animation.is_some()
    });

    if !has_animation {
        let svg = builder::build(project).map_err(|e| e.to_string())?;
        let canvas = project.active_canvas();
        let png = render_to_png(&svg, canvas.width.max(canvas.height)).map_err(|e| e.to_string())?;
        return Ok(vec![png]);
    }

    let max_duration = elements.iter().filter_map(|e| {
        e.common().animation.as_ref().map(|a| a.duration + a.delay)
    }).fold(0.0_f64, f64::max);

    let total_frames = ((max_duration * fps as f64).ceil() as u32).clamp(1, 300);
    let canvas = project.active_canvas();
    let render_size = canvas.width.max(canvas.height);

    let mut frames = Vec::with_capacity(total_frames as usize);

    for frame_idx in 0..total_frames {
        let time_s = frame_idx as f64 / fps as f64;

        let mut frame_project = project.clone();
        update_animation_state(&mut frame_project, time_s);

        let svg = builder::build(&frame_project).map_err(|e| e.to_string())?;
        let png = render_to_png(&svg, render_size).map_err(|e| e.to_string())?;
        frames.push(png);
    }

    Ok(frames)
}

fn update_animation_state(project: &mut IconProject, time_s: f64) {
    for elem in project.active_elements_mut() {
        let anim = match elem.common().animation.clone() {
            Some(a) => a,
            None => continue,
        };
        let local_t = (time_s - anim.delay).max(0.0);
        let progress = if anim.duration > 0.0 {
            let raw = local_t / anim.duration;
            if anim.repeat { raw % 1.0 } else { raw.min(1.0) }
        } else {
            0.0
        };

        let common = elem.common_mut();
        match anim.animation_type {
            AnimationType::Rotate => {
                common.rotation = progress * 360.0;
            }
            AnimationType::Scale => {
                let min_scale = anim.params.get("min_scale").and_then(|v| v.as_f64()).unwrap_or(0.8);
                let scale = 1.0 + (min_scale - 1.0) * progress;
                common.width = common.width.max(1.0) * scale;
                common.height = common.height.max(1.0) * scale;
            }
            AnimationType::Fade => {
                let min_opacity = anim.params.get("min_opacity").and_then(|v| v.as_f64()).unwrap_or(0.0);
                common.opacity = 1.0 + (min_opacity - 1.0) * progress;
            }
            AnimationType::Translate => {
                let dx = anim.params.get("dx").and_then(|v| v.as_f64()).unwrap_or(10.0);
                let dy = anim.params.get("dy").and_then(|v| v.as_f64()).unwrap_or(0.0);
                common.x += dx * progress;
                common.y += dy * progress;
            }
            AnimationType::Path => {}
        }
    }
}

pub fn export_gif(
    project: &IconProject,
    fps: u32,
    width: u32,
    height: u32,
    output_path: &Path,
) -> Result<PathBuf, String> {
    let png_frames = export_gif_frames(project, fps)?;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut buf = Cursor::new(Vec::new());
    {
        let mut encoder = GifEncoder::new(&mut buf);
        encoder
            .set_repeat(image::codecs::gif::Repeat::Infinite)
            .map_err(|e| format!("GIF repeat setting failed: {e}"))?;

        for png_bytes in &png_frames {
            let img = image::load_from_memory(png_bytes)
                .map_err(|e| format!("Failed to decode PNG frame: {e}"))?;
            let resized = img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
            encoder
                .encode(&resized.to_rgba8(), width, height, image::ExtendedColorType::Rgba8)
                .map_err(|e| format!("GIF frame encoding failed: {e}"))?;
        }
    }

    let gif_bytes = buf.into_inner();
    if gif_bytes.len() < 6 || &gif_bytes[0..3] != b"GIF" {
        return Err("GIF encoding produced invalid output".to_string());
    }

    std::fs::write(output_path, &gif_bytes).map_err(|e| e.to_string())?;
    Ok(output_path.to_path_buf())
}

pub fn preview_frame(
    project: &IconProject,
    time_ms: f64,
) -> Result<Vec<u8>, String> {
    let time_s = time_ms / 1000.0;
    let mut frame_project = project.clone();
    update_animation_state(&mut frame_project, time_s);

    let svg = builder::build(&frame_project).map_err(|e| e.to_string())?;
    let canvas = project.active_canvas();
    let size = canvas.width.max(canvas.height);
    render_to_png(&svg, size).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use crate::model::shapes::ShapeType;

    fn make_anim_shape(anim: Animation) -> IconProject {
        let mut p = IconProject::default();
        let shape = ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
                x: 50.0,
                y: 50.0,
                width: 100.0,
                height: 100.0,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: Some(anim),
                blend_mode: None,
                clip_element_id: None,
                mask_element_id: None,
                locked: false,
                visible: true,
                svg_filter: None,
            },
            shape_type: ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        };
        p.elements.push(Element::Shape(shape));
        p
    }

    fn make_static_shape() -> IconProject {
        let mut p = IconProject::default();
        let shape = ShapeElement {
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
                mask_element_id: None,
                locked: false,
                visible: true,
                svg_filter: None,
            },
            shape_type: ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        };
        p.elements.push(Element::Shape(shape));
        p
    }

    #[test]
    fn test_rotate_animation_lottie_json() {
        let anim = Animation {
            animation_type: AnimationType::Rotate,
            duration: 2.0,
            delay: 0.0,
            repeat: true,
            easing: "linear".to_string(),
            params: serde_json::Value::Null,
        };
        let project = make_anim_shape(anim);
        let json = export_lottie(&project, None).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();

        let layer = &val["layers"][0];
        let r = &layer["ks"]["r"];
        assert!(r["a"].as_u64() == Some(1), "rotation should be animated");
        let keyframes = r["k"].as_array().unwrap();
        assert!(keyframes[0]["s"].as_array().unwrap()[0].as_f64() == Some(0.0));
        assert!(keyframes[0]["e"].as_array().unwrap()[0].as_f64() == Some(360.0));
        assert!(keyframes[1]["s"].as_array().unwrap()[0].as_f64() == Some(360.0));
    }

    #[test]
    fn test_scale_animation_lottie_json() {
        let anim = Animation {
            animation_type: AnimationType::Scale,
            duration: 2.0,
            delay: 0.0,
            repeat: true,
            easing: "ease-in-out".to_string(),
            params: serde_json::json!({ "min_scale": 0.5 }),
        };
        let project = make_anim_shape(anim);
        let json = export_lottie(&project, None).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();

        let s = &val["layers"][0]["ks"]["s"];
        assert!(s["a"].as_u64() == Some(1), "scale should be animated");
        let keyframes = s["k"].as_array().unwrap();
        let start = keyframes[0]["s"].as_array().unwrap();
        assert_eq!(start[0].as_f64(), Some(100.0));
        let end = keyframes[0]["e"].as_array().unwrap();
        assert_eq!(end[0].as_f64(), Some(50.0));
    }

    #[test]
    fn test_fade_animation_lottie_json() {
        let anim = Animation {
            animation_type: AnimationType::Fade,
            duration: 1.5,
            delay: 0.0,
            repeat: true,
            easing: "ease-in-out".to_string(),
            params: serde_json::json!({ "min_opacity": 0.0 }),
        };
        let project = make_anim_shape(anim);
        let json = export_lottie(&project, None).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();

        let o = &val["layers"][0]["ks"]["o"];
        assert!(o["a"].as_u64() == Some(1), "opacity should be animated");
        let keyframes = o["k"].as_array().unwrap();
        assert_eq!(keyframes[0]["s"].as_array().unwrap()[0].as_f64(), Some(100.0));
        assert_eq!(keyframes[0]["e"].as_array().unwrap()[0].as_f64(), Some(0.0));
    }

    #[test]
    fn test_translate_animation_lottie_json() {
        let anim = Animation {
            animation_type: AnimationType::Translate,
            duration: 2.0,
            delay: 0.0,
            repeat: true,
            easing: "ease-out".to_string(),
            params: serde_json::json!({ "dx": 20.0, "dy": -10.0 }),
        };
        let project = make_anim_shape(anim);
        let json = export_lottie(&project, None).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();

        let p = &val["layers"][0]["ks"]["p"];
        assert!(p["a"].as_u64() == Some(1), "position should be animated");
        let keyframes = p["k"].as_array().unwrap();
        let start = keyframes[0]["s"].as_array().unwrap();
        assert_eq!(start[0].as_f64(), Some(100.0)); // cx = 50 + 100/2
        assert_eq!(start[1].as_f64(), Some(100.0)); // cy = 50 + 100/2
        let end = keyframes[0]["e"].as_array().unwrap();
        assert_eq!(end[0].as_f64(), Some(120.0)); // cx + dx
        assert_eq!(end[1].as_f64(), Some(90.0));  // cy + dy
    }

    #[test]
    fn test_easing_linear() {
        let (i, o) = easing_bezier("linear");
        assert_eq!(i["x"].as_array().unwrap()[0].as_f64(), Some(0.0));
        assert_eq!(i["y"].as_array().unwrap()[0].as_f64(), Some(0.0));
        assert_eq!(o["x"].as_array().unwrap()[0].as_f64(), Some(1.0));
        assert_eq!(o["y"].as_array().unwrap()[0].as_f64(), Some(1.0));
    }

    #[test]
    fn test_easing_ease_in() {
        let (i, o) = easing_bezier("ease-in");
        assert_eq!(i["x"].as_array().unwrap()[0].as_f64(), Some(0.42));
        assert_eq!(o["x"].as_array().unwrap()[0].as_f64(), Some(1.0));
    }

    #[test]
    fn test_easing_ease_out() {
        let (i, o) = easing_bezier("ease-out");
        assert_eq!(i["x"].as_array().unwrap()[0].as_f64(), Some(0.0));
        assert_eq!(o["x"].as_array().unwrap()[0].as_f64(), Some(0.58));
    }

    #[test]
    fn test_easing_ease_in_out() {
        let (i, o) = easing_bezier("ease-in-out");
        assert_eq!(i["x"].as_array().unwrap()[0].as_f64(), Some(0.42));
        assert_eq!(o["x"].as_array().unwrap()[0].as_f64(), Some(0.58));
    }

    #[test]
    fn test_static_element_fixed_values() {
        let project = make_static_shape();
        let json = export_lottie(&project, None).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();

        let layer = &val["layers"][0];
        let r = &layer["ks"]["r"];
        assert!(r.get("a").is_none() || r["a"].as_u64() == Some(0));
        assert_eq!(r["k"].as_f64(), Some(0.0));

        let o = &layer["ks"]["o"];
        assert_eq!(o["k"].as_f64(), Some(100.0));
    }

    #[test]
    fn test_multi_element_layers_count() {
        let mut project = IconProject::default();
        for i in 0..3 {
            let shape = ShapeElement {
                common: CommonProps {
                    id: format!("shape-{}", i + 1),
                    x: 0.0, y: 0.0, width: 50.0, height: 50.0,
                    opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
                    blend_mode: None, clip_element_id: None, mask_element_id: None,
                    locked: false, visible: true, svg_filter: None,
                },
                shape_type: ShapeType::Circle,
                fill: "#FF0000".to_string(), stroke: None, stroke_width: 0.0,
                border_radius: 0.0, stroke_dasharray: None, gradient: None,
            };
            project.elements.push(Element::Shape(shape));
        }
        let json = export_lottie(&project, None).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["layers"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_json_validity() {
        let project = make_anim_shape(Animation::default());
        let json = export_lottie(&project, None).unwrap();
        let result = serde_json::from_str::<serde_json::Value>(&json);
        assert!(result.is_ok(), "Output should be valid JSON");
    }

    #[test]
    fn test_gif_export_produces_file() {
        let project = make_static_shape();
        let dir = std::env::temp_dir().join("iconstudio_test_gif");
        let _ = std::fs::create_dir_all(&dir);
        let output_path = dir.join("test.gif");

        let result = export_gif(&project, 10, 64, 64, &output_path);
        if let Ok(path) = result {
            let bytes = std::fs::read(&path).unwrap();
            assert!(bytes.len() > 0, "GIF file should not be empty");
            assert_eq!(&bytes[0..3], b"GIF", "File should start with GIF header");
            let _ = std::fs::remove_file(&path);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
