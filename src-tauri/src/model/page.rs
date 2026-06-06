use serde::{Deserialize, Serialize};

use super::{Canvas, Element, IconProject};

/// A page (independent canvas) within a multi-page project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    #[serde(default = "default_page_name")]
    pub name: String,
    #[serde(default)]
    pub canvas: Canvas,
    #[serde(default)]
    pub elements: Vec<Element>,
}

fn default_page_name() -> String {
    "Untitled".to_string()
}

impl Default for Page {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: default_page_name(),
            canvas: Canvas::default(),
            elements: Vec::new(),
        }
    }
}

impl Page {
    /// Create a new page with the given name and dimensions.
    pub fn new(name: &str, width: u32, height: u32) -> Self {
        Self {
            id: format!("page-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()),
            name: name.to_string(),
            canvas: Canvas {
                width,
                height,
                background: "#FFFFFF".to_string(),
                corner_radius: 0,
                background_gradient: None,
            },
            elements: Vec::new(),
        }
    }

    /// Create a page from the current project's canvas and elements.
    pub fn from_project(project: &IconProject) -> Self {
        Self {
            id: format!("page-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()),
            name: "Page 1".to_string(),
            canvas: project.canvas.clone(),
            elements: project.elements.clone(),
        }
    }

    /// Convert this page back into a full IconProject, borrowing base metadata.
    pub fn as_project(&self, base: &IconProject) -> IconProject {
        IconProject {
            schema_version: base.schema_version.clone(),
            canvas: self.canvas.clone(),
            elements: self.elements.clone(),
            exports: base.exports.clone(),
            templates: base.templates.clone(),
            next_element_id: base.next_element_id,
            version: base.version,
            pages: Vec::new(),
            symbols: base.symbols.clone(),
            active_page_index: 0,
            adaptive: None,
            brand_kits: Vec::new(),
            custom_style_presets: Vec::new(),
        }
    }
}
