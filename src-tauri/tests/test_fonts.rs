use icon_studio_lib::fonts::{list_system_fonts, search_fonts};

#[test]
fn list_system_fonts_returns_non_empty() {
    let fonts = list_system_fonts();
    assert!(!fonts.is_empty(), "System should have at least one font installed");
}

#[test]
fn font_info_has_valid_fields() {
    let fonts = list_system_fonts();
    for font in &fonts {
        assert!(!font.name.is_empty(), "Font name should not be empty");
        assert!(
            font.style == "Normal" || font.style == "Italic" || font.style == "Oblique",
            "Font style '{}' should be Normal, Italic, or Oblique",
            font.style
        );
        assert!(font.weight > 0, "Font weight should be > 0, got {}", font.weight);
    }
}

#[test]
fn search_fonts_filters_by_keyword() {
    let keyword = "Arial";
    let results = search_fonts(keyword);
    for font in &results {
        assert!(
            font.name.to_lowercase().contains(&keyword.to_lowercase()),
            "Font '{}' should contain keyword '{}'",
            font.name,
            keyword
        );
    }
}

#[test]
fn search_fonts_with_empty_keyword_returns_all() {
    let all = list_system_fonts();
    let searched = search_fonts("");
    assert_eq!(all.len(), searched.len(), "Empty keyword should return all fonts");
}

#[test]
fn search_fonts_with_non_matching_keyword_returns_empty() {
    let results = search_fonts("ZZZZZ_NOT_A_FONT_12345");
    assert!(
        results.is_empty(),
        "Non-matching keyword should return empty, got {} results",
        results.len()
    );
}
