use fontdb::{Database, Style};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::OnceLock;

static FALLBACK_DB: OnceLock<Database> = OnceLock::new();

fn fallback_db() -> &'static Database {
    FALLBACK_DB.get_or_init(|| {
        let mut db = Database::new();
        db.load_system_fonts();
        db
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub name: String,
    pub style: String,
    pub weight: u16,
}

fn style_to_string(s: Style) -> String {
    match s {
        Style::Normal => "Normal".into(),
        Style::Italic => "Italic".into(),
        Style::Oblique => "Oblique".into(),
    }
}

fn collect_fonts(db: &Database) -> Vec<FontInfo> {
    let mut seen = HashSet::new();
    let mut fonts = Vec::new();

    for face in db.faces() {
        let family = face.families.first().map(|(f, _)| f.clone()).unwrap_or_default();
        let style = style_to_string(face.style);
        let key = format!("{}-{}", family, style);

        if seen.insert(key) {
            fonts.push(FontInfo {
                name: family,
                style,
                weight: face.weight.0,
            });
        }
    }

    fonts.sort_by(|a, b| a.name.cmp(&b.name).then(a.weight.cmp(&b.weight)));
    fonts
}

pub fn list_system_fonts_with(db: &Database) -> Vec<FontInfo> {
    collect_fonts(db)
}

pub fn search_fonts_with(db: &Database, keyword: &str) -> Vec<FontInfo> {
    let lower = keyword.to_lowercase();
    collect_fonts(db)
        .into_iter()
        .filter(|f| f.name.to_lowercase().contains(&lower))
        .collect()
}

pub fn list_system_fonts() -> Vec<FontInfo> {
    collect_fonts(fallback_db())
}

pub fn search_fonts(keyword: &str) -> Vec<FontInfo> {
    let lower = keyword.to_lowercase();
    collect_fonts(fallback_db())
        .into_iter()
        .filter(|f| f.name.to_lowercase().contains(&lower))
        .collect()
}
