use crate::engine::utils::escape_xml;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

struct TextMeasureCache {
    cache: HashMap<(String, String, u32, u16), f32>,
    insert_order: Vec<(String, String, u32, u16)>,
}

static CACHE: OnceLock<Mutex<TextMeasureCache>> = OnceLock::new();

fn get_cache() -> &'static Mutex<TextMeasureCache> {
    CACHE.get_or_init(|| {
        Mutex::new(TextMeasureCache {
            cache: HashMap::new(),
            insert_order: Vec::new(),
        })
    })
}

/// Measure text width precisely using usvg SVG parsing.
/// Falls back to estimation if measurement fails (e.g. font not found).
pub fn measure_text_width(
    text: &str,
    font_family: &str,
    font_size: f32,
    font_weight: u16,
) -> f32 {
    // Use f32 bits as cache key to get a hashable u32
    let cache_key = (
        text.to_string(),
        font_family.to_string(),
        font_size.to_bits(),
        font_weight,
    );

    {
        let mut cache = get_cache().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(&width) = cache.cache.get(&cache_key) {
            // Move key to end of insert_order for LRU behavior
            if let Some(pos) = cache.insert_order.iter().position(|k| k == &cache_key) {
                cache.insert_order.remove(pos);
                cache.insert_order.push(cache_key);
            }
            return width;
        }
        if cache.cache.len() > 5000 {
            let keep = 3000;
            let evict_count = cache.insert_order.len().saturating_sub(keep);
            let keys_to_evict: Vec<_> = cache.insert_order.drain(..evict_count).collect();
            for key in keys_to_evict {
                cache.cache.remove(&key);
            }
        }
    }

    let estimated_max_width = (text.chars().count() as f32 * font_size * 1.5 + 1000.0).min(50000.0);
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{:.0}" height="100">
        <text x="0" y="50" dominant-baseline="central" font-family="{}" font-size="{}" font-weight="{}">{}</text>
        </svg>"#,
        estimated_max_width,
        escape_xml(font_family),
        font_size,
        font_weight,
        escape_xml(text)
    );

    let opts = crate::engine::renderer::get_options();

    match usvg::Tree::from_str(&svg, opts) {
        Ok(tree) => {
            let width = extract_text_width(tree.root());
            let result = width.unwrap_or_else(|| estimate_text_width(text, font_size));

            {
                let mut cache = get_cache().lock().unwrap_or_else(|e| e.into_inner());
                cache.cache.insert(cache_key.clone(), result);
                cache.insert_order.push(cache_key);
            }
            result
        }
        Err(_) => estimate_text_width(text, font_size),
    }
}

fn extract_text_width(group: &usvg::Group) -> Option<f32> {
    for node in group.children() {
        match node {
            usvg::Node::Text(t) => {
                let bbox = t.abs_bounding_box();
                return Some(bbox.width());
            }
            usvg::Node::Group(g) => {
                if let Some(w) = extract_text_width(g) {
                    return Some(w);
                }
            }
            _ => {}
        }
    }
    None
}

/// Fallback estimation: chars * fontSize * 0.6
fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_fallback() {
        let w = estimate_text_width("Hello", 16.0);
        assert!(w > 0.0);
        assert!((w - 48.0).abs() < 0.01);
    }

    #[test]
    fn test_measure_returns_positive() {
        let w = measure_text_width("Test", "sans-serif", 24.0, 400);
        assert!(w > 0.0);
    }

    #[test]
    fn test_cache_hit() {
        let w1 = measure_text_width("CacheTest", "Arial", 20.0, 400);
        let w2 = measure_text_width("CacheTest", "Arial", 20.0, 400);
        assert!((w1 - w2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_empty_string() {
        let w = measure_text_width("", "sans-serif", 16.0, 400);
        assert!(w >= 0.0);
    }

    #[test]
    fn test_long_string() {
        let w = measure_text_width(
            "This is a very long string for testing",
            "sans-serif",
            14.0,
            400,
        );
        assert!(w > 0.0);
    }

    #[test]
    fn test_nonexistent_font_falls_back() {
        let w = measure_text_width("Hello", "NonExistentFont12345", 24.0, 400);
        assert!(w > 0.0);
    }

    #[test]
    fn test_different_font_sizes_produce_different_widths() {
        let w_small = measure_text_width("Test", "sans-serif", 12.0, 400);
        let w_large = measure_text_width("Test", "sans-serif", 48.0, 400);
        assert!(w_large > w_small);
    }
}
