pub mod shapes;
pub mod history;
pub mod helpers;
pub mod group;
pub mod filter;
pub mod page;
pub mod symbol;
pub mod style_preset;
pub mod adaptive;
pub mod brand;
pub mod iconset;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub use group::GroupElement;
pub use filter::*;
pub use page::*;
pub use symbol::*;
pub use style_preset::*;
pub use adaptive::*;
pub use brand::*;
pub use iconset::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconProject {
    pub schema_version: String,
    pub canvas: Canvas,
    pub elements: Vec<Element>,
    pub exports: ExportConfig,
    #[serde(default)]
    pub templates: HashMap<String, IconProject>,
    #[serde(default)]
    pub next_element_id: u64,
    #[serde(default)]
    pub version: u64,
    #[serde(default)]
    pub pages: Vec<Page>,
    #[serde(default)]
    pub symbols: HashMap<String, SymbolDef>,
    #[serde(default)]
    pub active_page_index: usize,
    #[serde(default)]
    pub adaptive: Option<AdaptiveConfig>,
    #[serde(default)]
    pub brand_kits: Vec<BrandKit>,
    #[serde(default)]
    pub custom_style_presets: Vec<style_preset::CustomStylePreset>,
}

impl Default for IconProject {
    fn default() -> Self {
        Self {
            schema_version: "1.0".to_string(),
            canvas: Canvas::default(),
            elements: Vec::new(),
            exports: ExportConfig::default(),
            templates: HashMap::new(),
            next_element_id: 1,
            version: 0,
            pages: Vec::new(),
            symbols: HashMap::new(),
            active_page_index: 0,
            adaptive: None,
            brand_kits: Vec::new(),
            custom_style_presets: Vec::new(),
        }
    }
}

impl IconProject {
    pub fn alloc_element_id(&mut self, prefix: &str) -> String {
        let id = format!("{}-{}", prefix, self.next_element_id);
        self.next_element_id += 1;
        id
    }

    pub fn recalc_next_element_id(&mut self) {
        fn extract_max_id(elem: &Element) -> u64 {
            let own = elem.id().rsplit('-').next().and_then(|n| n.parse::<u64>().ok()).unwrap_or(0);
            let children_max = match elem {
                Element::Group(g) => g.children.iter().map(extract_max_id).max().unwrap_or(0),
                _ => 0,
            };
            own.max(children_max)
        }
        let max_id = self.active_elements().iter().map(extract_max_id).max().unwrap_or(0);
        self.next_element_id = max_id + 1;
    }

    pub fn bump_version(&mut self) {
        self.version += 1;
    }

    /// Create a copy of this project suitable for saving as a template.
    /// Clears nested templates to prevent recursive serialization.
    pub fn as_template(&self) -> IconProject {
        let mut tpl = self.clone();
        tpl.templates.clear();
        tpl
    }

    // ---- Multi-page compatibility layer ----

    /// Returns the active page index, clamped to valid range.
    pub fn active_page_index_clamped(&self) -> usize {
        if self.pages.is_empty() {
            0
        } else {
            self.active_page_index.min(self.pages.len() - 1)
        }
    }

    /// Get the active canvas (from pages if available, otherwise legacy field).
    pub fn active_canvas(&self) -> &Canvas {
        if !self.pages.is_empty() {
            let idx = self.active_page_index_clamped();
            &self.pages[idx].canvas
        } else {
            &self.canvas
        }
    }

    /// Get the active canvas mutably.
    pub fn active_canvas_mut(&mut self) -> &mut Canvas {
        if !self.pages.is_empty() {
            let idx = self.active_page_index_clamped();
            &mut self.pages[idx].canvas
        } else {
            &mut self.canvas
        }
    }

    /// Get the active elements slice.
    pub fn active_elements(&self) -> &[Element] {
        if !self.pages.is_empty() {
            let idx = self.active_page_index_clamped();
            &self.pages[idx].elements
        } else {
            &self.elements
        }
    }

    /// Get the active elements mutably.
    pub fn active_elements_mut(&mut self) -> &mut Vec<Element> {
        if !self.pages.is_empty() {
            let idx = self.active_page_index_clamped();
            &mut self.pages[idx].elements
        } else {
            &mut self.elements
        }
    }
}

fn default_512() -> u32 { 512 }
fn default_height() -> u32 { 512 }
fn default_bg() -> String { "#FFFFFF".to_string() }
fn default_0() -> u32 { 0 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    #[serde(default = "default_512")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_bg")]
    pub background: String,
    #[serde(default = "default_0")]
    pub corner_radius: u32,
    #[serde(default)]
    pub background_gradient: Option<Gradient>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            background: "#FFFFFF".to_string(),
            corner_radius: 0,
            background_gradient: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Element {
    Shape(ShapeElement),
    Text(TextElement),
    Icon(IconElement),
    Image(ImageElement),
    Path(PathElement),
    Group(GroupElement),
    Symbol(SymbolInstanceElement),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GradientKind {
    #[serde(rename = "linear")]
    Linear,
    #[serde(rename = "radial")]
    Radial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gradient {
    #[serde(rename = "type")]
    pub gradient_type: GradientKind,
    pub colors: Vec<String>,
    pub angle: f64,
    #[serde(default)]
    pub stops: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shadow {
    pub color: String,
    pub blur: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    #[serde(default)]
    pub inset: bool,
}

/// Animation types supported by SVG SMIL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimationType {
    Rotate,
    Scale,
    Fade,
    Translate,
    Path,
}

/// Element-level SVG animation definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    pub animation_type: AnimationType,
    #[serde(default = "default_duration")]
    pub duration: f64,
    #[serde(default)]
    pub delay: f64,
    #[serde(default = "default_true")]
    pub repeat: bool,
    #[serde(default = "default_easing")]
    pub easing: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

fn default_duration() -> f64 {
    2.0
}
fn default_easing() -> String {
    "ease-in-out".to_string()
}
fn default_true() -> bool {
    true
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            animation_type: AnimationType::Rotate,
            duration: default_duration(),
            delay: 0.0,
            repeat: true,
            easing: default_easing(),
            params: serde_json::Value::Null,
        }
    }
}

impl Default for Shadow {
    fn default() -> Self {
        Self {
            color: "#00000040".to_string(),
            blur: 8.0,
            offset_x: 0.0,
            offset_y: 4.0,
            inset: false,
        }
    }
}

/// Fields shared by all element types.
/// Uses `#[serde(flatten)]` in each variant to maintain JSON backward compatibility.
#[derive(Debug, Clone, Serialize)]
pub struct CommonProps {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub opacity: f64,
    pub rotation: f64,
    #[serde(default)]
    pub shadows: Vec<Shadow>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub blend_mode: Option<String>,
    #[serde(default)]
    pub clip_element_id: Option<String>,
    #[serde(default)]
    pub mask_element_id: Option<String>,
    #[serde(default)]
    pub locked: bool,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default)]
    pub svg_filter: Option<filter::SvgFilter>,
}

impl<'de> Deserialize<'de> for CommonProps {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CommonPropsHelper {
            pub id: String,
            pub x: f64,
            pub y: f64,
            pub width: f64,
            pub height: f64,
            pub opacity: f64,
            pub rotation: f64,
            #[serde(default)]
            pub shadow: Option<Shadow>,
            #[serde(default)]
            pub shadows: Vec<Shadow>,
            #[serde(default)]
            pub animation: Option<Animation>,
            #[serde(default)]
            pub blend_mode: Option<String>,
            #[serde(default)]
            pub clip_element_id: Option<String>,
            #[serde(default)]
            pub mask_element_id: Option<String>,
            #[serde(default)]
            pub locked: bool,
            #[serde(default = "default_true")]
            pub visible: bool,
            #[serde(default)]
            pub svg_filter: Option<filter::SvgFilter>,
        }

        let h = CommonPropsHelper::deserialize(deserializer)?;
        let shadows = if !h.shadows.is_empty() {
            h.shadows
        } else if let Some(s) = h.shadow {
            vec![s]
        } else {
            vec![]
        };

        Ok(CommonProps {
            id: h.id,
            x: h.x,
            y: h.y,
            width: h.width,
            height: h.height,
            opacity: h.opacity,
            rotation: h.rotation,
            shadows,
            animation: h.animation,
            blend_mode: h.blend_mode,
            clip_element_id: h.clip_element_id,
            mask_element_id: h.mask_element_id,
            locked: h.locked,
            visible: h.visible,
            svg_filter: h.svg_filter,
        })
    }
}

impl Element {
    /// Access the common fields shared by all element types.
    pub fn common(&self) -> &CommonProps {
        match self {
            Element::Shape(e) => &e.common,
            Element::Text(e) => &e.common,
            Element::Icon(e) => &e.common,
            Element::Image(e) => &e.common,
            Element::Path(e) => &e.common,
            Element::Group(e) => &e.common,
            Element::Symbol(e) => &e.common,
        }
    }

    /// Mutably access the common fields shared by all element types.
    pub fn common_mut(&mut self) -> &mut CommonProps {
        match self {
            Element::Shape(e) => &mut e.common,
            Element::Text(e) => &mut e.common,
            Element::Icon(e) => &mut e.common,
            Element::Image(e) => &mut e.common,
            Element::Path(e) => &mut e.common,
            Element::Group(e) => &mut e.common,
            Element::Symbol(e) => &mut e.common,
        }
    }

    /// Shortcut to get the element ID.
    pub fn id(&self) -> &str {
        &self.common().id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeElement {
    #[serde(flatten)]
    pub common: CommonProps,
    pub shape_type: shapes::ShapeType,
    pub fill: String,
    pub stroke: Option<String>,
    pub stroke_width: f64,
    #[serde(default)]
    pub border_radius: f64,
    #[serde(default)]
    pub stroke_dasharray: Option<String>,
    #[serde(default)]
    pub gradient: Option<Gradient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextElement {
    #[serde(flatten)]
    pub common: CommonProps,
    pub content: String,
    pub fill: String,
    pub font_family: String,
    pub font_size: f64,
    pub font_weight: String,
    #[serde(default)]
    pub letter_spacing: f64,
    pub stroke: Option<String>,
    pub stroke_width: f64,
    #[serde(default)]
    pub gradient: Option<Gradient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconElement {
    #[serde(flatten)]
    pub common: CommonProps,
    pub name: String,
    pub fill: String,
    pub stroke: Option<String>,
    pub stroke_width: f64,
    #[serde(default)]
    pub gradient: Option<Gradient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageElement {
    #[serde(flatten)]
    pub common: CommonProps,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathElement {
    #[serde(flatten)]
    pub common: CommonProps,
    pub d: String,
    pub fill: String,
    pub stroke: String,
    pub stroke_width: f64,
    #[serde(default)]
    pub stroke_dasharray: Option<String>,
    /// Width of the path data's bounding box (used as denominator for scale).
    /// When 0 the path is treated as legacy (pre-normalisation) data.
    #[serde(default)]
    pub natural_width: f64,
    /// Height of the path data's bounding box (used as denominator for scale).
    #[serde(default)]
    pub natural_height: f64,
    /// Non-destructive recipe recording how this path was produced by a boolean operation.
    #[serde(default)]
    pub boolean_source: Option<crate::engine::boolean::BooleanSource>,
}

fn default_formats() -> Vec<String> { vec!["svg".to_string(), "png".to_string()] }
fn default_sizes() -> Vec<u32> { vec![16, 32, 64, 128, 256, 512] }

impl CommonProps {
    /// Create a new CommonProps with default values for optional fields.
    /// Useful in tests and construction sites.
    #[allow(dead_code)]
    pub fn new(id: String, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    #[serde(default = "default_formats")]
    pub formats: Vec<String>,
    #[serde(default = "default_sizes")]
    pub sizes: Vec<u32>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            formats: vec!["svg".to_string(), "png".to_string()],
            sizes: vec![16, 32, 64, 128, 256, 512],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_icon_project() {
        let p = IconProject::default();
        assert_eq!(p.schema_version, "1.0");
        assert_eq!(p.canvas.width, 512);
        assert_eq!(p.canvas.height, 512);
        assert_eq!(p.canvas.background, "#FFFFFF");
        assert_eq!(p.canvas.corner_radius, 0);
        assert!(p.elements.is_empty());
        assert!(p.templates.is_empty());
        assert_eq!(p.next_element_id, 1);
        assert_eq!(p.version, 0);
    }

    #[test]
    fn test_alloc_element_id() {
        let mut p = IconProject::default();
        assert_eq!(p.alloc_element_id("shape"), "shape-1");
        assert_eq!(p.alloc_element_id("text"), "text-2");
        assert_eq!(p.alloc_element_id("icon"), "icon-3");
        assert_eq!(p.next_element_id, 4);
    }

    #[test]
    fn test_recalc_next_element_id() {
        let mut p = IconProject::default();
        p.elements.push(Element::Shape(ShapeElement {
            common: CommonProps {
                id: "shape-5".to_string(),
                x: 0.0, y: 0.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
        blend_mode: None,
        clip_element_id: None,
        mask_element_id: None,
        locked: false, visible: true, svg_filter: None,
        },
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        }));
        p.elements.push(Element::Shape(ShapeElement {
            common: CommonProps {
                id: "shape-10".to_string(),
                x: 0.0, y: 0.0, width: 100.0, height: 100.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
            blend_mode: None,
            clip_element_id: None,
            mask_element_id: None,
            locked: false, visible: true, svg_filter: None,
            },
            shape_type: shapes::ShapeType::Rect,
            fill: "#00FF00".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        }));
        p.recalc_next_element_id();
        assert_eq!(p.next_element_id, 11);

        p.elements.clear();
        p.recalc_next_element_id();
        assert_eq!(p.next_element_id, 1);
    }

    #[test]
    fn test_bump_version() {
        let mut p = IconProject::default();
        assert_eq!(p.version, 0);
        p.bump_version();
        assert_eq!(p.version, 1);
        p.bump_version();
        assert_eq!(p.version, 2);
    }

    #[test]
    fn test_as_template() {
        let mut p = IconProject::default();
        p.templates.insert("sub".to_string(), IconProject::default());
        assert_eq!(p.templates.len(), 1);

        let tpl = p.as_template();
        assert!(tpl.templates.is_empty());
        assert_eq!(p.templates.len(), 1);
    }

    #[test]
    fn test_group_element_serde() {
        let child = ShapeElement {
            common: CommonProps {
                id: "shape-1".to_string(),
                x: 10.0, y: 20.0, width: 50.0, height: 60.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None,
        locked: false, visible: true, svg_filter: None,
        },
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        };
        let group = GroupElement {
            common: CommonProps {
                id: "group-1".to_string(),
                x: 100.0, y: 200.0, width: 300.0, height: 400.0,
                opacity: 0.8, rotation: 45.0, shadows: vec![], animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None,
        locked: false, visible: true, svg_filter: None,
        },
            children: vec![Element::Shape(child)],
            expanded: true,
        };
        let json = serde_json::to_string(&Element::Group(group.clone())).unwrap();
        assert!(json.contains(r#""type":"group""#));
        assert!(json.contains(r#""children""#));

        let parsed: Element = serde_json::from_str(&json).unwrap();
        if let Element::Group(g) = parsed {
            assert_eq!(g.common.id, "group-1");
            assert_eq!(g.children.len(), 1);
            assert!(g.expanded);
        } else {
            panic!("Expected Group variant");
        }
    }

    #[test]
    fn test_group_in_project() {
        let mut p = IconProject::default();
        let child = ShapeElement {
            common: CommonProps {
                id: p.alloc_element_id("shape"),
                x: 10.0, y: 10.0, width: 50.0, height: 50.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![],
                animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None,
        locked: false, visible: true, svg_filter: None,
        },
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        };
        let group = GroupElement {
            common: CommonProps {
                id: p.alloc_element_id("group"),
                x: 0.0, y: 0.0, width: 50.0, height: 50.0,
                opacity: 1.0, rotation: 0.0, shadows: vec![], animation: None,
            blend_mode: None,
            clip_element_id: None,
        mask_element_id: None,
        locked: false, visible: true, svg_filter: None,
        },
            children: vec![Element::Shape(child)],
            expanded: false,
        };
        p.elements.push(Element::Group(group));
        p.recalc_next_element_id();
        assert_eq!(p.next_element_id, 3);
    }

    #[test]
    fn test_legacy_shadow_deserialization() {
        let json = r##"{ "id": "shape-1", "x": 10.0, "y": 20.0, "width": 100.0, "height": 100.0, "opacity": 1.0, "rotation": 0.0, "shadow": {"color": "#00000040", "blur": 8.0, "offset_x": 0.0, "offset_y": 4.0} }"##;
        let props: CommonProps = serde_json::from_str(json).unwrap();
        assert_eq!(props.id, "shape-1");
        assert_eq!(props.shadows.len(), 1);
        assert_eq!(props.shadows[0].color, "#00000040");
        assert_eq!(props.shadows[0].blur, 8.0);
        assert!(!props.shadows[0].inset);
    }

    // ---- Multi-page tests ----

    #[test]
    fn test_page_new() {
        let page = Page::new("Test Page", 256, 256);
        assert_eq!(page.name, "Test Page");
        assert_eq!(page.canvas.width, 256);
        assert_eq!(page.canvas.height, 256);
        assert_eq!(page.canvas.background, "#FFFFFF");
        assert!(page.elements.is_empty());
        assert!(!page.id.is_empty());
    }

    #[test]
    fn test_page_from_project() {
        let mut project = IconProject::default();
        project.canvas.width = 1024;
        project.canvas.height = 768;
        project.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-1".to_string(), 0.0, 0.0, 100.0, 100.0),
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        }));

        let page = Page::from_project(&project);
        assert_eq!(page.canvas.width, 1024);
        assert_eq!(page.canvas.height, 768);
        assert_eq!(page.elements.len(), 1);
    }

    #[test]
    fn test_page_as_project() {
        let base = IconProject::default();
        let page = Page::new("Test", 256, 256);
        let restored = page.as_project(&base);
        assert_eq!(restored.canvas.width, 256);
        assert!(restored.elements.is_empty());
        assert!(restored.pages.is_empty());
    }

    #[test]
    fn test_page_serde() {
        let page = Page::new("Test", 512, 512);
        let json = serde_json::to_string(&page).unwrap();
        let parsed: Page = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Test");
        assert_eq!(parsed.canvas.width, 512);
    }

    #[test]
    fn test_active_canvas_legacy() {
        let mut project = IconProject::default();
        project.canvas.width = 1024;
        assert!(project.pages.is_empty());
        assert_eq!(project.active_canvas().width, 1024);
    }

    #[test]
    fn test_active_canvas_with_pages() {
        let mut project = IconProject::default();
        let page = Page::new("Page 1", 256, 256);
        project.pages.push(page);
        project.active_page_index = 0;
        assert_eq!(project.active_canvas().width, 256);
    }

    #[test]
    fn test_active_elements_legacy() {
        let mut project = IconProject::default();
        project.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-1".to_string(), 0.0, 0.0, 100.0, 100.0),
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        }));
        assert!(project.pages.is_empty());
        assert_eq!(project.active_elements().len(), 1);
    }

    #[test]
    fn test_active_elements_with_pages() {
        let mut project = IconProject::default();
        let mut page = Page::new("Page 1", 512, 512);
        page.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-1".to_string(), 0.0, 0.0, 100.0, 100.0),
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        }));
        project.pages.push(page);
        project.active_page_index = 0;
        assert_eq!(project.active_elements().len(), 1);
        // Legacy elements should NOT be returned
        project.elements.push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-99".to_string(), 0.0, 0.0, 100.0, 100.0),
            shape_type: shapes::ShapeType::Rect,
            fill: "#00FF00".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        }));
        assert_eq!(project.active_elements().len(), 1); // Still page's elements
    }

    #[test]
    fn test_active_page_index_clamped() {
        let mut project = IconProject::default();
        // No pages: should return 0
        assert_eq!(project.active_page_index_clamped(), 0);

        let page1 = Page::new("Page 1", 512, 512);
        let page2 = Page::new("Page 2", 512, 512);
        project.pages.push(page1);
        project.pages.push(page2);

        project.active_page_index = 0;
        assert_eq!(project.active_page_index_clamped(), 0);

        project.active_page_index = 1;
        assert_eq!(project.active_page_index_clamped(), 1);

        // Out of bounds: should clamp to last index
        project.active_page_index = 99;
        assert_eq!(project.active_page_index_clamped(), 1);
    }

    #[test]
    fn test_backward_compat_json() {
        // Old-format JSON without pages field should deserialize correctly
        let json = r##"{"schema_version":"1.0","canvas":{"width":512,"height":512,"background":"#FFFFFF","corner_radius":0},"elements":[],"exports":{"formats":["svg"],"sizes":[16,32,64,128,256,512]},"templates":{}}"##;
        let project: IconProject = serde_json::from_str(json).unwrap();
        assert!(project.pages.is_empty());
        assert_eq!(project.active_page_index, 0);
        // active_canvas should fall back to legacy canvas
        assert_eq!(project.active_canvas().width, 512);
    }

    #[test]
    fn test_active_elements_mut_with_pages() {
        let mut project = IconProject::default();
        let page = Page::new("Page 1", 512, 512);
        project.pages.push(page);
        project.active_page_index = 0;

        project.active_elements_mut().push(Element::Shape(ShapeElement {
            common: CommonProps::new("shape-1".to_string(), 0.0, 0.0, 100.0, 100.0),
            shape_type: shapes::ShapeType::Circle,
            fill: "#FF0000".to_string(),
            stroke: None, stroke_width: 0.0, border_radius: 0.0,
            stroke_dasharray: None, gradient: None,
        }));

        assert_eq!(project.pages[0].elements.len(), 1);
        assert!(project.elements.is_empty()); // Legacy should be unchanged
    }
}
