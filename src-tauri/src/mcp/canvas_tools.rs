use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{internal_err, invalid_params, state_err, IconStudioHandler};
use crate::engine::generator;
use crate::model::helpers::*;
use crate::model::history::{AddElementCommand, CanvasCommand, SnapshotCommand};
use crate::model::Element;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct IconOpenParams {
    #[schemars(description = "SVG file path to import")]
    pub path: String,
}

fn default_anchor() -> String {
    "center".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ResizeCanvasParams {
    #[schemars(description = "New width in pixels")]
    pub width: u32,
    #[schemars(description = "New height in pixels")]
    pub height: u32,
    #[schemars(description = "Content anchor: center, top-left, top-right, bottom-left, bottom-right. Default: center")]
    #[serde(default = "default_anchor")]
    pub anchor: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FlipElementParams {
    #[schemars(description = "Element ID")]
    pub element_id: String,
    #[schemars(description = "Flip direction: horizontal or vertical")]
    pub direction: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AlignElementsParams {
    #[schemars(description = "Alignment: left, center-h, right, top, center-v, bottom, distribute-h, distribute-v")]
    pub alignment: String,
    #[schemars(description = "Element IDs to align (empty = all elements)")]
    #[serde(default)]
    pub element_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ScaleElementParams {
    #[schemars(description = "Element ID")]
    pub element_id: String,
    #[schemars(description = "Scale factor (1.0 = original, 2.0 = double, 0.5 = half)")]
    pub factor: f64,
}

fn default_style() -> Option<String> {
    None
}

fn default_512_u32() -> u32 {
    512
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GenerateRandomParams {
    #[schemars(description = "Style: minimal, geometric, lettermark, icon, badge. Empty = random")]
    #[serde(default = "default_style")]
    pub style: Option<String>,
    #[schemars(description = "Base color (hex, empty = random)")]
    pub base_color: Option<String>,
    #[schemars(description = "Text content (for lettermark/badge styles)")]
    pub text: Option<String>,
    #[schemars(description = "Icon name (for icon style, e.g. 'heart')")]
    pub icon_name: Option<String>,
    #[schemars(description = "Canvas size in pixels")]
    #[serde(default = "default_512_u32")]
    pub size: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ApplyPaletteParams {
    #[schemars(description = "Colors (hex) to apply sequentially to elements")]
    pub colors: Vec<String>,
    #[schemars(description = "Also apply first color as canvas background")]
    #[serde(default)]
    pub apply_to_canvas: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AddSvgElementsParams {
    #[schemars(description = "SVG file path")]
    pub path: String,
    #[schemars(description = "Target X position (canvas coords, default: centered)")]
    pub target_x: Option<f64>,
    #[schemars(description = "Target Y position (canvas coords, default: centered)")]
    pub target_y: Option<f64>,
    #[schemars(description = "Target size in pixels (default: 60% of canvas)")]
    pub target_size: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListLibraryParams {
    #[schemars(description = "Filter by category: filled-icons, shapes, brand-logos")]
    pub category: Option<String>,
    #[schemars(description = "Search by keyword (matches name, tags, label)")]
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AddLibraryAssetParams {
    #[schemars(description = "Asset name, e.g. filled-home, deco-circle-solid, brand-github")]
    pub asset_name: String,
    #[schemars(description = "Target X position (canvas coords, default: centered)")]
    pub target_x: Option<f64>,
    #[schemars(description = "Target Y position (canvas coords, default: centered)")]
    pub target_y: Option<f64>,
    #[schemars(description = "Target size in pixels (default: 60% of canvas)")]
    pub target_size: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetClipParams {
    #[schemars(description = "Element ID to clip")]
    pub element_id: String,
    #[schemars(description = "Element ID to use as clipping shape")]
    pub clip_element_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ClearClipParams {
    #[schemars(description = "Element ID to clear clipping")]
    pub element_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetMaskParams {
    #[schemars(description = "Element ID to mask")]
    pub element_id: String,
    #[schemars(description = "Element ID to use as mask")]
    pub mask_element_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ClearMaskParams {
    #[schemars(description = "Element ID to clear mask")]
    pub element_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ConvertToPathParams {
    #[schemars(description = "Shape element ID to convert to path")]
    pub element_id: String,
}

struct SvgBounds {
    min_x: f64,
    min_y: f64,
    size: f64,
}

fn calculate_svg_bounds(elements: &[Element]) -> SvgBounds {
    if elements.is_empty() {
        return SvgBounds { min_x: 0.0, min_y: 0.0, size: 24.0 };
    }
    let min_x = elements.iter().map(|e| e.common().x).fold(f64::MAX, f64::min);
    let min_y = elements.iter().map(|e| e.common().y).fold(f64::MAX, f64::min);
    let max_x = elements.iter().map(|e| e.common().x + e.common().width).fold(f64::MIN, f64::max);
    let max_y = elements.iter().map(|e| e.common().y + e.common().height).fold(f64::MIN, f64::max);
    SvgBounds {
        min_x,
        min_y,
        size: (max_x - min_x).max(max_y - min_y).max(1.0),
    }
}

#[tool_router(router = canvas_router, vis = "pub")]
impl IconStudioHandler {
    #[tool(name = "icon_open", description = "Import SVG file as new project (replaces current)")]
    async fn icon_open(
        &self,
        Parameters(params): Parameters<IconOpenParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let imported = crate::services::project::load_svg_project(&params.path)
            .map_err(|e| internal_err(format!("Failed to open SVG '{}': {}", params.path, e)))?;

        let (w, h, count) = {
            let mut project = self.project.lock().map_err(state_err)?;
            *project = imported;
            project.recalc_next_element_id();
            project.bump_version();
            let w = project.active_canvas().width;
            let h = project.active_canvas().height;
            let count = project.active_elements().len();
            (w, h, count)
        };

        {
            let mut cache = self.cache.lock().map_err(state_err)?;
            cache.invalidate_cache();
        }
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.clear();
        }
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Opened SVG: {}x{}, imported {} elements",
            w, h, count
        ))]))
    }

    #[tool(name = "resize_canvas", description = "Resize canvas with anchor control for content positioning")]
    async fn resize_canvas(
        &self,
        Parameters(params): Parameters<ResizeCanvasParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if params.width == 0 || params.height == 0 || params.width > 8192 || params.height > 8192 {
            return Err(invalid_params("Invalid canvas dimensions: width and height must be 1-8192"));
        }

        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;
        let old_canvas = project.active_canvas().clone();
        let old_w = project.active_canvas().width as f64;
        let old_h = project.active_canvas().height as f64;
        let new_w = params.width as f64;
        let new_h = params.height as f64;

        let (offset_x, offset_y) = match params.anchor.as_str() {
            "center" => ((new_w - old_w) / 2.0, (new_h - old_h) / 2.0),
            "top-left" => (0.0, 0.0),
            "top-right" => (new_w - old_w, 0.0),
            "bottom-left" => (0.0, new_h - old_h),
            "bottom-right" => (new_w - old_w, new_h - old_h),
            other => return Err(invalid_params(format!("Invalid anchor: {}. Valid: center, top-left, top-right, bottom-left, bottom-right", other))),
        };

        for elem in project.active_elements_mut().iter_mut() {
            let (x, y, ..) = get_element_bounds(elem);
            set_element_position(elem, x + offset_x, y + offset_y);
        }

        project.active_canvas_mut().width = params.width;
        project.active_canvas_mut().height = params.height;
        project.bump_version();

        let after = project.active_elements().to_vec();
        let new_canvas = project.active_canvas().clone();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.begin_batch("resize_canvas").map_err(internal_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
            history.record(Box::new(CanvasCommand::new(old_canvas, new_canvas)));
            history.commit_batch().map_err(internal_err)?;
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Canvas resized to {}x{} (anchor: {})", params.width, params.height, params.anchor
        ))]))
    }

    #[tool(name = "flip_element", description = "Flip element horizontally or vertically")]
    async fn flip_element(
        &self,
        Parameters(params): Parameters<FlipElementParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;
        let cw = project.active_canvas().width as f64;
        let ch = project.active_canvas().height as f64;

        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;

        match params.direction.as_str() {
            "horizontal" | "vertical" => {
                crate::model::helpers::flip_element(elem, &params.direction, cw, ch);
            }
            other => {
                drop(project);
                return Err(invalid_params(format!("Invalid direction: {}. Valid: horizontal, vertical", other)));
            }
        }

        project.bump_version();
        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Flipped element '{}' {}",
            params.element_id,
            params.direction
        ))]))
    }

    #[tool(name = "align_elements", description = "Align elements: left/center-h/right, top/center-v/bottom, distribute-h/v")]
    async fn align_elements(
        &self,
        Parameters(params): Parameters<AlignElementsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        if project.active_elements().is_empty() {
            drop(project);
            return Ok(CallToolResult::success(vec![Content::text("No elements to align")]));
        }

        let indices: Vec<usize> = if params.element_ids.is_empty() {
            (0..project.active_elements().len()).collect()
        } else {
            params.element_ids.iter().filter_map(|id| {
                find_element_index(project.active_elements(), id)
            }).collect()
        };

        if indices.is_empty() {
            drop(project);
            return Err(invalid_params("Specified elements not found"));
        }

        let cw = project.active_canvas().width as f64;
        let ch = project.active_canvas().height as f64;

        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        match params.alignment.as_str() {
            "left" => {
                let min_x = indices.iter().map(|&i| get_element_bounds(&project.active_elements()[i]).0).fold(f64::MAX, f64::min);
                for &i in &indices {
                    let (_, y, _, _) = get_element_bounds(&project.active_elements()[i]);
                    set_element_position(&mut project.active_elements_mut()[i], min_x, y);
                }
            }
            "right" => {
                let max_right = indices.iter().map(|&i| {
                    let (x, _, w, _) = get_element_bounds(&project.active_elements()[i]);
                    x + w
                }).fold(f64::MIN, f64::max);
                for &i in &indices {
                    let (_, y, w, _) = get_element_bounds(&project.active_elements()[i]);
                    set_element_position(&mut project.active_elements_mut()[i], max_right - w, y);
                }
            }
            "center-h" => {
                for &i in &indices {
                    let (_, y, w, _) = get_element_bounds(&project.active_elements()[i]);
                    set_element_position(&mut project.active_elements_mut()[i], (cw - w) / 2.0, y);
                }
            }
            "top" => {
                let min_y = indices.iter().map(|&i| get_element_bounds(&project.active_elements()[i]).1).fold(f64::MAX, f64::min);
                for &i in &indices {
                    let (x, _, _, _) = get_element_bounds(&project.active_elements()[i]);
                    set_element_position(&mut project.active_elements_mut()[i], x, min_y);
                }
            }
            "bottom" => {
                let max_bottom = indices.iter().map(|&i| {
                    let (_, y, _, h) = get_element_bounds(&project.active_elements()[i]);
                    y + h
                }).fold(f64::MIN, f64::max);
                for &i in &indices {
                    let (x, _, _, h) = get_element_bounds(&project.active_elements()[i]);
                    set_element_position(&mut project.active_elements_mut()[i], x, max_bottom - h);
                }
            }
            "center-v" => {
                for &i in &indices {
                    let (x, _, _, h) = get_element_bounds(&project.active_elements()[i]);
                    set_element_position(&mut project.active_elements_mut()[i], x, (ch - h) / 2.0);
                }
            }
            "distribute-h" => {
                if indices.len() < 2 {
                    drop(project);
                    return Ok(CallToolResult::success(vec![Content::text("At least 2 elements required for distribution")]));
                }
                let mut sorted = indices.clone();
                sorted.sort_by(|&a, &b| {
                    let ax = get_element_bounds(&project.active_elements()[a]).0;
                    let bx = get_element_bounds(&project.active_elements()[b]).0;
                    ax.partial_cmp(&bx).unwrap_or(std::cmp::Ordering::Equal)
                });

                let total_width: f64 = sorted.iter()
                    .map(|&i| get_element_bounds(&project.active_elements()[i]).2)
                    .sum();
                let first_x = get_element_bounds(&project.active_elements()[sorted[0]]).0;
                let last = sorted.last().expect("sorted is non-empty because indices.len() >= 2 was checked above");
                let (last_x, _, last_w, _) = get_element_bounds(&project.active_elements()[*last]);
                let span = (last_x + last_w) - first_x;
                let gap = if sorted.len() > 1 {
                    ((span - total_width) / (sorted.len() - 1) as f64).max(0.0)
                } else {
                    0.0
                };

                let mut current_x = first_x;
                for &i in &sorted {
                    let (_, oy, ew, _) = get_element_bounds(&project.active_elements()[i]);
                    set_element_position(&mut project.active_elements_mut()[i], current_x, oy);
                    current_x += ew + gap;
                }
            }
            "distribute-v" => {
                if indices.len() < 2 {
                    drop(project);
                    return Ok(CallToolResult::success(vec![Content::text("At least 2 elements required for distribution")]));
                }
                let mut sorted = indices.clone();
                sorted.sort_by(|&a, &b| {
                    let ay = get_element_bounds(&project.active_elements()[a]).1;
                    let by = get_element_bounds(&project.active_elements()[b]).1;
                    ay.partial_cmp(&by).unwrap_or(std::cmp::Ordering::Equal)
                });

                let total_height: f64 = sorted.iter()
                    .map(|&i| get_element_bounds(&project.active_elements()[i]).3)
                    .sum();
                let first_y = get_element_bounds(&project.active_elements()[sorted[0]]).1;
                let last = sorted.last().expect("sorted is non-empty because indices.len() >= 2 was checked above");
                let (_, last_y, _, last_h) = get_element_bounds(&project.active_elements()[*last]);
                let span = (last_y + last_h) - first_y;
                let gap = if sorted.len() > 1 {
                    ((span - total_height) / (sorted.len() - 1) as f64).max(0.0)
                } else {
                    0.0
                };

                let mut current_y = first_y;
                for &i in &sorted {
                    let (ox, _, _, eh) = get_element_bounds(&project.active_elements()[i]);
                    set_element_position(&mut project.active_elements_mut()[i], ox, current_y);
                    current_y += eh + gap;
                }
            }
            other => {
                drop(project);
                return Err(invalid_params(format!(
                    "Invalid alignment: {}. Valid: left, center-h, right, top, center-v, bottom, distribute-h, distribute-v", other
                )));
            }
        }

        let count = indices.len();
        project.bump_version();
        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Aligned {} elements ({})", count, params.alignment
        ))]))
    }

    #[tool(name = "scale_element", description = "Scale element by factor (>1 enlarge, <1 shrink)")]
    async fn scale_element(
        &self,
        Parameters(params): Parameters<ScaleElementParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if params.factor.is_nan() || params.factor <= 0.0 {
            return Err(invalid_params("Scale factor must be positive"));
        }

        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;
        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("Element not found: {}", params.element_id)))?;

        scale_element_size(elem, params.factor);
        project.bump_version();
        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Scaled element '{}' (x{:.2})", params.element_id, params.factor
        ))]))
    }

    #[tool(name = "generate_random_icon", description = "Generate a random icon. Styles: minimal, geometric, lettermark, icon, badge")]
    async fn generate_random_icon(
        &self,
        Parameters(params): Parameters<GenerateRandomParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let size = if params.size == 0 { 512 } else { params.size };
        let config = generator::GeneratorConfig {
            style: params.style,
            base_color: params.base_color,
            text: params.text,
            icon_name: params.icon_name,
            size,
        };

        let new_project = generator::generate_random(&config);

        let mut project = self.project.lock().map_err(state_err)?;
        let style_name = match new_project.elements.first() {
            Some(Element::Shape(s)) => format!("{:?}", s.shape_type),
            Some(Element::Text(t)) => format!("text '{}'", t.content),
            Some(Element::Icon(i)) => format!("icon '{}'", i.name),
            _ => "unknown".to_string(),
        };
        let element_count = new_project.elements.len();
        let bg = new_project.canvas.background.clone();
        *project = new_project;
        project.recalc_next_element_id();
        project.bump_version();
        drop(project);

        {
            let mut cache = self.cache.lock().map_err(state_err)?;
            cache.invalidate_cache();
        }
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.clear();
        }
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Generated random icon: {}x{}, background {}, {} elements (first: {})",
            size, size, bg, element_count, style_name
        ))]))
    }

    #[tool(name = "apply_palette", description = "Apply color palette to elements (cyclic) and optionally canvas background")]
    async fn apply_palette(
        &self,
        Parameters(params): Parameters<ApplyPaletteParams>,
    ) -> Result<CallToolResult, ErrorData> {
        if params.colors.is_empty() {
            return Err(invalid_params("Color list cannot be empty"));
        }

        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;
        let old_canvas = project.active_canvas().clone();
        let color_count = params.colors.len();

        if params.apply_to_canvas {
            project.active_canvas_mut().background = params.colors[0].clone();
        }

        for (i, elem) in project.active_elements_mut().iter_mut().enumerate() {
            let color = &params.colors[i % color_count];
            match elem {
                Element::Shape(e) => e.fill = color.clone(),
                Element::Text(e) => e.fill = color.clone(),
                Element::Icon(e) => e.fill = color.clone(),
                Element::Image(_) => {}
                Element::Path(e) => e.fill = color.clone(),
                Element::Group(_) => {}
                Element::Symbol(_) => {}
            }
        }

        let count = project.active_elements().len();
        project.bump_version();
        let after = project.active_elements().to_vec();
        let new_canvas = project.active_canvas().clone();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            if params.apply_to_canvas {
                history.begin_batch("apply_palette").map_err(internal_err)?;
                history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
                history.record(Box::new(CanvasCommand::new(old_canvas, new_canvas)));
                history.commit_batch().map_err(internal_err)?;
            } else {
                history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
            }
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Applied {} colors to {} elements{}",
            color_count,
            count,
            if params.apply_to_canvas { " and canvas background" } else { "" }
        ))]))
    }

    #[tool(name = "add_svg_elements", description = "Import SVG elements into current project (additive, does not replace)")]
    async fn add_svg_elements(
        &self,
        Parameters(params): Parameters<AddSvgElementsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let svg_str = std::fs::read_to_string(&params.path)
            .map_err(|e| internal_err(format!("Failed to read file '{}': {}", params.path, e)))?;

        let mut elements = crate::engine::importer::import_svg_as_elements(&svg_str)
            .map_err(|e| internal_err(format!("SVG parse error: {}", e)))?;

        if elements.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text("No importable elements found in SVG")]));
        }

        let mut project = self.project.lock().map_err(state_err)?;
        let canvas_size = project.active_canvas().width.min(project.active_canvas().height) as f64;

        let bounds = calculate_svg_bounds(&elements);
        let target_sz = params.target_size.unwrap_or(canvas_size * 0.6);
        let scale = target_sz / bounds.size;
        let tx = params.target_x.unwrap_or((canvas_size - target_sz) / 2.0);
        let ty = params.target_y.unwrap_or((canvas_size - target_sz) / 2.0);

        let mut ids = Vec::new();
        for elem in &mut elements {
            let prefix = match elem {
                Element::Shape(_) => "shape",
                Element::Text(_) => "text",
                Element::Icon(_) => "icon",
                Element::Image(_) => "image",
                Element::Path(_) => "path",
                Element::Group(_) => "group",
                Element::Symbol(_) => "symbol",
            };
            let new_id = project.alloc_element_id(prefix);
            elem.common_mut().id = new_id.clone();
            ids.push(new_id);

            let c = elem.common_mut();
            c.x = (c.x - bounds.min_x) * scale + tx;
            c.y = (c.y - bounds.min_y) * scale + ty;
            c.width *= scale;
            c.height *= scale;
        }

        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.begin_batch("add_svg_elements").map_err(internal_err)?;
            for e in elements {
                history.push_and_execute(
                    Box::new(AddElementCommand::new(e)),
                    &mut project,
                ).map_err(internal_err)?;
            }
            history.commit_batch().map_err(internal_err)?;
        }

        project.bump_version();
        drop(project);

        {
            let mut cache = self.cache.lock().map_err(state_err)?;
            cache.invalidate_cache();
        }
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Imported {} elements, IDs: {:?}",
            ids.len(), ids
        ))]))
    }

    #[tool(name = "list_library", description = "List library assets by category and keyword")]
    async fn list_library(
        &self,
        Parameters(params): Parameters<ListLibraryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let assets = crate::library::list_assets(
            params.category.as_deref(),
            params.keyword.as_deref(),
        );

        if assets.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No matching assets found. Categories: filled-icons, shapes, brand-logos"
            )]));
        }

        let categories = crate::library::list_categories();
        let lines: Vec<String> = assets.iter().map(|a| {
            format!("  [{}] {} — {} (tags: {})", a.category, a.name, a.label, a.tags.join(", "))
        }).collect();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "找到 {} 个素材（分类：{}）\n{}",
            assets.len(),
            categories.join(", "),
            lines.join("\n")
        ))]))
    }

    #[tool(name = "add_library_asset", description = "从素材库添加素材到画布。先 list_library 查看可用素材，再用此命令添加")]
    async fn add_library_asset(
        &self,
        Parameters(params): Parameters<AddLibraryAssetParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let svg_str = crate::library::get_asset_svg(&params.asset_name)
            .ok_or_else(|| invalid_params(format!(
                "素材 '{}' 不存在。使用 list_library 查看可用素材", params.asset_name
            )))?;

        let mut elements = crate::engine::importer::import_svg_as_elements(svg_str)
            .map_err(|e| internal_err(format!("解析素材 SVG 失败：{}", e)))?;

        if elements.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "素材解析后没有可添加的元素"
            )]));
        }

        let mut project = self.project.lock().map_err(state_err)?;
        let canvas_size = project.active_canvas().width.min(project.active_canvas().height) as f64;
        let bounds = calculate_svg_bounds(&elements);
        let target_sz = params.target_size.unwrap_or(canvas_size * 0.6);
        let scale = target_sz / bounds.size;
        let tx = params.target_x.unwrap_or((canvas_size - target_sz) / 2.0);
        let ty = params.target_y.unwrap_or((canvas_size - target_sz) / 2.0);

        let mut ids = Vec::new();
        for elem in &mut elements {
            let prefix = match elem {
                Element::Shape(_) => "shape",
                Element::Text(_) => "text",
                Element::Icon(_) => "icon",
                Element::Image(_) => "image",
                Element::Path(_) => "path",
                Element::Group(_) => "group",
                Element::Symbol(_) => "symbol",
            };
            let new_id = project.alloc_element_id(prefix);
            elem.common_mut().id = new_id.clone();
            ids.push(new_id);

            let c = elem.common_mut();
            c.x = (c.x - bounds.min_x) * scale + tx;
            c.y = (c.y - bounds.min_y) * scale + ty;
            c.width *= scale;
            c.height *= scale;
        }

        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.begin_batch("add_library_asset").map_err(internal_err)?;
            for e in elements {
                history.push_and_execute(
                    Box::new(AddElementCommand::new(e)),
                    &mut project,
                ).map_err(internal_err)?;
            }
            history.commit_batch().map_err(internal_err)?;
        }

        project.bump_version();
        drop(project);

        {
            let mut cache = self.cache.lock().map_err(state_err)?;
            cache.invalidate_cache();
        }
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "已添加素材 '{}'，{} 个元素，ID：{:?}",
            params.asset_name, ids.len(), ids
        ))]))
    }

    #[tool(name = "set_clip", description = "设置裁剪路径。用 ref_element_id 指定的元素作为裁剪形状来裁剪 element_id 指定的元素")]
    async fn set_clip(
        &self,
        Parameters(params): Parameters<SetClipParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        crate::services::elements::validate_clip_mask_reference(
            project.active_elements(), &params.element_id, &params.clip_element_id,
        ).map_err(|e| invalid_params(e.to_string()))?;

        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;
        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("元素未找到：{}", params.element_id)))?;
        elem.common_mut().clip_element_id = Some(params.clip_element_id.clone());

        project.bump_version();
        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "已设置裁剪：元素 '{}' 被元素 '{}' 裁剪",
            params.element_id, params.clip_element_id
        ))]))
    }

    #[tool(name = "clear_clip", description = "清除元素的裁剪路径")]
    async fn clear_clip(
        &self,
        Parameters(params): Parameters<ClearClipParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("元素未找到：{}", params.element_id)))?;
        elem.common_mut().clip_element_id = None;

        project.bump_version();
        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "已清除元素 '{}' 的裁剪", params.element_id
        ))]))
    }

    #[tool(name = "set_mask", description = "设置蒙版。用 ref_element_id 指定的元素作为蒙版来遮罩 element_id 指定的元素")]
    async fn set_mask(
        &self,
        Parameters(params): Parameters<SetMaskParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;

        crate::services::elements::validate_clip_mask_reference(
            project.active_elements(), &params.element_id, &params.mask_element_id,
        ).map_err(|e| invalid_params(e.to_string()))?;

        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;
        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("元素未找到：{}", params.element_id)))?;
        elem.common_mut().mask_element_id = Some(params.mask_element_id.clone());

        project.bump_version();
        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "已设置蒙版：元素 '{}' 被元素 '{}' 蒙版",
            params.element_id, params.mask_element_id
        ))]))
    }

    #[tool(name = "clear_mask", description = "清除元素的蒙版")]
    async fn clear_mask(
        &self,
        Parameters(params): Parameters<ClearMaskParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut project = self.project.lock().map_err(state_err)?;
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        let elem = find_element_deep_mut(project.active_elements_mut(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("元素未找到：{}", params.element_id)))?;
        elem.common_mut().mask_element_id = None;

        project.bump_version();
        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        self.emit_change();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "已清除元素 '{}' 的蒙版", params.element_id
        ))]))
    }

    #[tool(name = "convert_to_path", description = "将形状元素转换为可编辑的路径元素，保留原始样式")]
    async fn convert_to_path(
        &self,
        Parameters(params): Parameters<ConvertToPathParams>,
    ) -> Result<CallToolResult, ErrorData> {
        use crate::engine::boolean::shape_to_path_d;

        let mut project = self.project.lock().map_err(state_err)?;

        let idx = find_element_index(project.active_elements(), &params.element_id)
            .ok_or_else(|| invalid_params(format!("元素未找到：{}", params.element_id)))?;

        let elem = &project.active_elements()[idx];
        
        let (shape_type, x, y, w, h, border_radius) = match elem {
            Element::Shape(s) => (&s.shape_type, s.common.x, s.common.y, s.common.width, s.common.height, s.border_radius),
            _ => return Err(invalid_params("元素不是形状类型，无法转换为路径")),
        };
        
        let d = shape_to_path_d(shape_type, x, y, w, h, border_radius);
        
        let shape = match elem { Element::Shape(s) => s, _ => unreachable!() };
        
        let mut path_element = crate::model::PathElement {
            common: crate::model::CommonProps {
                id: params.element_id.clone(),
                x: 0.0,
                y: 0.0,
                width: w,
                height: h,
                opacity: shape.common.opacity,
                rotation: shape.common.rotation,
                shadows: shape.common.shadows.clone(),
                animation: shape.common.animation.clone(),
                blend_mode: shape.common.blend_mode.clone(),
                clip_element_id: shape.common.clip_element_id.clone(),
                mask_element_id: shape.common.mask_element_id.clone(),
                locked: shape.common.locked,
                visible: shape.common.visible,
                svg_filter: shape.common.svg_filter.clone(),
            },
            d,
            fill: shape.fill.clone(),
            stroke: shape.stroke.clone().unwrap_or_else(|| "none".to_string()),
            stroke_width: shape.stroke_width,
            stroke_dasharray: shape.stroke_dasharray.clone(),
            natural_width: 0.0,
            natural_height: 0.0,
            boolean_source: None,
        };
        
        // Recompute natural dimensions
        {
            let mut wrapper = Element::Path(path_element.clone());
            crate::model::helpers::recompute_path_natural_dims(&mut wrapper);
            if let Element::Path(pe) = wrapper {
                path_element = pe;
            }
        }
        
        let before = project.active_elements().to_vec();
        let before_next_id = project.next_element_id;

        project.active_elements_mut()[idx] = Element::Path(path_element);
        project.bump_version();

        let after = project.active_elements().to_vec();
        {
            let mut history = self.history.lock().map_err(state_err)?;
            history.record(Box::new(SnapshotCommand::new(before, before_next_id, after, project.next_element_id)));
        }
        drop(project);
        
        {
            let mut cache = self.cache.lock().map_err(state_err)?;
            cache.invalidate_cache();
        }
        self.emit_change();
        
        Ok(CallToolResult::success(vec![Content::text(format!(
            "已将形状 '{}' 转换为路径元素", params.element_id
        ))]))
    }
}
