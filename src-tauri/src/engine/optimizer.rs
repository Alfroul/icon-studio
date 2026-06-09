use crate::error::AppError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CleanRule {
    RemoveNamespaces,
    RemoveMetadata,
    ReducePrecision,
    MergeSingleChildGroups,
    RemoveEmptyGroups,
    RemoveIdentityTransforms,
    RemoveFillNoneOnStroked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    pub cleaned_svg: String,
    pub rules_applied: Vec<String>,
    pub bytes_before: usize,
    pub bytes_after: usize,
}

impl CleanRule {
    pub fn name(&self) -> &str {
        match self {
            Self::RemoveNamespaces => "RemoveNamespaces",
            Self::RemoveMetadata => "RemoveMetadata",
            Self::ReducePrecision => "ReducePrecision",
            Self::MergeSingleChildGroups => "MergeSingleChildGroups",
            Self::RemoveEmptyGroups => "RemoveEmptyGroups",
            Self::RemoveIdentityTransforms => "RemoveIdentityTransforms",
            Self::RemoveFillNoneOnStroked => "RemoveFillNoneOnStroked",
        }
    }
}

pub fn default_rules() -> Vec<CleanRule> {
    vec![
        CleanRule::RemoveNamespaces,
        CleanRule::RemoveMetadata,
        CleanRule::ReducePrecision,
        CleanRule::MergeSingleChildGroups,
        CleanRule::RemoveEmptyGroups,
        CleanRule::RemoveIdentityTransforms,
    ]
}

fn apply_rule(svg: &str, rule: &CleanRule) -> Result<(String, bool), AppError> {
    let result = match rule {
        CleanRule::RemoveNamespaces => rule_remove_namespaces(svg),
        CleanRule::RemoveMetadata => rule_remove_metadata(svg),
        CleanRule::ReducePrecision => rule_reduce_precision(svg),
        CleanRule::MergeSingleChildGroups => rule_merge_single_child_groups(svg),
        CleanRule::RemoveEmptyGroups => rule_remove_empty_groups(svg),
        CleanRule::RemoveIdentityTransforms => rule_remove_identity_transforms(svg),
        CleanRule::RemoveFillNoneOnStroked => rule_remove_fill_none_on_stroked(svg),
    };
    let changed = result != svg;
    Ok((result, changed))
}

pub fn clean_svg(svg_str: &str, rules: &[CleanRule]) -> Result<CleanResult, AppError> {
    let bytes_before = svg_str.len();
    let mut result = svg_str.to_string();
    let mut applied = Vec::new();

    for rule in rules {
        let (new_svg, was_applied) = apply_rule(&result, rule)?;
        if was_applied {
            applied.push(rule.name().to_string());
            result = new_svg;
        }
    }

    let bytes_after = result.len();
    Ok(CleanResult {
        cleaned_svg: result,
        rules_applied: applied,
        bytes_before,
        bytes_after,
    })
}

// ---------------------------------------------------------------------------
// Rule implementations
// ---------------------------------------------------------------------------

static RE_NS_DECL: OnceLock<Regex> = OnceLock::new();
static RE_NS_ATTR: OnceLock<Regex> = OnceLock::new();

fn rule_remove_namespaces(svg: &str) -> String {
    let re_decl = RE_NS_DECL.get_or_init(|| {
        Regex::new(
            r#"\s+xmlns:(?:inkscape|sodipodi|dc|cc|rdf|serif|sketch|figma|illustrator|odf|office|x)[^\s=]*="[^"]*""#,
        ).unwrap()
    });
    let result = re_decl.replace_all(svg, "").to_string();
    let re_attr = RE_NS_ATTR.get_or_init(|| {
        Regex::new(
            r#"\s+(?:inkscape|sodipodi|dc|cc|rdf|serif|sketch|figma|illustrator):[a-zA-Z_-]+="[^"]*""#,
        ).unwrap()
    });
    re_attr.replace_all(&result, "").to_string()
}

static RE_META_PAIR: OnceLock<Regex> = OnceLock::new();
static RE_META_SELF: OnceLock<Regex> = OnceLock::new();

fn rule_remove_metadata(svg: &str) -> String {
    let re_pair = RE_META_PAIR.get_or_init(|| {
        Regex::new(r"<(?:metadata|title|desc)(?:\s[^>]*)?>[\s\S]*?</(?:metadata|title|desc)>").unwrap()
    });
    let re_self = RE_META_SELF.get_or_init(|| {
        Regex::new(r"<(?:metadata|title|desc)(?:\s[^>]*)?/>").unwrap()
    });
    let result = re_pair.replace_all(svg, "").to_string();
    re_self.replace_all(&result, "").to_string()
}

static RE_DECIMAL: OnceLock<Regex> = OnceLock::new();

fn rule_reduce_precision(svg: &str) -> String {
    let re = RE_DECIMAL.get_or_init(|| Regex::new(r"(\d+\.\d{3,})").unwrap());
    re.replace_all(svg, |caps: &regex::Captures| {
        let s = &caps[1];
        s.parse::<f64>()
            .map(|f| format!("{:.2}", f))
            .unwrap_or_else(|_| s.to_string())
    })
    .to_string()
}

fn rule_merge_single_child_groups(svg: &str) -> String {
    let mut result = svg.to_string();
    let mut changed = true;
    while changed {
        let (new, c) = merge_one_single_child_group(&result);
        result = new;
        changed = c;
    }
    result
}

static RE_EMPTY_GROUP: OnceLock<Regex> = OnceLock::new();

fn rule_remove_empty_groups(svg: &str) -> String {
    let re = RE_EMPTY_GROUP.get_or_init(|| Regex::new(r"<g(?:\s[^>]*)?>\s*</g>").unwrap());
    let mut result = svg.to_string();
    let mut changed = true;
    while changed {
        let new = re.replace_all(&result, "").to_string();
        changed = new != result;
        result = new;
    }
    result
}

static RE_IDENTITY_TRANSLATE_2: OnceLock<Regex> = OnceLock::new();
static RE_IDENTITY_TRANSLATE_1: OnceLock<Regex> = OnceLock::new();
static RE_IDENTITY_SCALE: OnceLock<Regex> = OnceLock::new();
static RE_IDENTITY_MATRIX: OnceLock<Regex> = OnceLock::new();

fn rule_remove_identity_transforms(svg: &str) -> String {
    let regexes: [&OnceLock<Regex>; 4] = [
        &RE_IDENTITY_TRANSLATE_2,
        &RE_IDENTITY_TRANSLATE_1,
        &RE_IDENTITY_SCALE,
        &RE_IDENTITY_MATRIX,
    ];
    let patterns: &[&str] = &[
        r#"\s*transform="translate\s*\(\s*0\s*,\s*0\s*\)""#,
        r#"\s*transform="translate\s*\(\s*0\s*\)""#,
        r#"\s*transform="scale\s*\(\s*1\s*\)""#,
        r#"\s*transform="matrix\s*\(\s*1\s*,\s*0\s*,\s*0\s*,\s*1\s*,\s*0\s*,\s*0\s*\)""#,
    ];
    let mut result = svg.to_string();
    for (lock, pat) in regexes.iter().zip(patterns.iter()) {
        let re = lock.get_or_init(|| Regex::new(pat).unwrap());
        result = re.replace_all(&result, "").to_string();
    }
    result
}

static RE_TAG_ATTRS: OnceLock<Regex> = OnceLock::new();
static RE_FILL_NONE: OnceLock<Regex> = OnceLock::new();

fn rule_remove_fill_none_on_stroked(svg: &str) -> String {
    let re = RE_TAG_ATTRS.get_or_init(|| {
        Regex::new(r"<([a-zA-Z][a-zA-Z0-9]*)((?:\s+[^>]*?)?)>").unwrap()
    });
    let re_fn = RE_FILL_NONE.get_or_init(|| {
        Regex::new(r#"\s*fill="none""#).unwrap()
    });
    re.replace_all(svg, |caps: &regex::Captures| {
        let tag = &caps[1];
        let attrs = &caps[2];

        if !attrs.contains("fill=\"none\"") {
            return caps[0].to_string();
        }

        let stroke = extract_attr(attrs, "stroke");
        match stroke {
            Some(s) if s != "none" => {
                let new_attrs = re_fn.replace(attrs, "").to_string();
                format!("<{}{}>", tag, new_attrs)
            }
            _ => caps[0].to_string(),
        }
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Helpers for group merging
// ---------------------------------------------------------------------------

struct ChildElement {
    tag_name: String,
    attrs: String,
    content: String,
    self_closing: bool,
}

fn merge_one_single_child_group(svg: &str) -> (String, bool) {
    let mut i = 0;
    while i < svg.len() {
        if !svg[i..].starts_with("<g") {
            i += 1;
            continue;
        }
        let rest = &svg[i + 2..];
        let next_ch = rest.chars().next();
        if next_ch != Some('>') && next_ch != Some(' ') && next_ch != Some('\n') {
            i += 1;
            continue;
        }

        let tag_end_offset = match rest.find('>') {
            Some(p) => p,
            None => {
                i += 1;
                continue;
            }
        };
        let tag_end = i + 2 + tag_end_offset;

        let tag_content = &svg[i + 2..tag_end];
        if tag_content.ends_with('/') {
            i = tag_end + 1;
            continue;
        }

        let group_attrs = if tag_content.starts_with('>') {
            ""
        } else {
            tag_content.trim()
        };

        let content_start = tag_end + 1;
        let close_pos = match find_matching_close_tag(svg, content_start, "g") {
            Some(p) => p,
            None => {
                i = tag_end + 1;
                continue;
            }
        };

        let content = svg[content_start..close_pos].trim();

        let child = match parse_single_child(content) {
            Some(c) => c,
            None => {
                i = tag_end + 1;
                continue;
            }
        };

        let group_transform = extract_attr(group_attrs, "transform");
        let merged_attrs = if let Some(gt) = group_transform {
            let ct = extract_attr(&child.attrs, "transform");
            match ct {
                Some(cv) => replace_or_append_attr(&child.attrs, "transform", &format!("{} {}", gt, cv)),
                None => replace_or_append_attr(&child.attrs, "transform", gt),
            }
        } else {
            child.attrs.clone()
        };

        let replacement = if child.self_closing {
            format!(
                "<{}{}{}/>",
                child.tag_name,
                if merged_attrs.is_empty() { "" } else { " " },
                merged_attrs
            )
        } else {
            format!(
                "<{}{}{}>{}</{}>",
                child.tag_name,
                if merged_attrs.is_empty() { "" } else { " " },
                merged_attrs,
                child.content,
                child.tag_name
            )
        };

        let close_end = close_pos + 4; // "</g>".len()
        let new_svg = format!("{}{}{}", &svg[..i], replacement, &svg[close_end..]);
        return (new_svg, true);
    }
    (svg.to_string(), false)
}

fn parse_single_child(content: &str) -> Option<ChildElement> {
    let content = content.trim();
    if !content.starts_with('<') {
        return None;
    }

    let tag_end = content.find('>')?;
    let opening = &content[..tag_end + 1];

    let self_closing = opening.ends_with("/>");
    let inner = if self_closing {
        &opening[1..opening.len() - 2]
    } else {
        &opening[1..opening.len() - 1]
    };

    let (tag_name, attrs) = split_tag_name_attrs(inner);
    if tag_name.is_empty() {
        return None;
    }

    if self_closing {
        let rest = content[tag_end + 1..].trim();
        if !rest.is_empty() {
            return None;
        }
        Some(ChildElement {
            tag_name,
            attrs,
            content: String::new(),
            self_closing: true,
        })
    } else {
        let close_pos = find_matching_close_tag(content, tag_end + 1, &tag_name)?;
        let close_tag_len = tag_name.len() + 3;
        let rest = content[close_pos + close_tag_len..].trim();
        if !rest.is_empty() {
            return None;
        }
        let child_content = content[tag_end + 1..close_pos].to_string();
        Some(ChildElement {
            tag_name,
            attrs,
            content: child_content,
            self_closing: false,
        })
    }
}

fn find_matching_close_tag(svg: &str, from: usize, tag: &str) -> Option<usize> {
    let open_tag = format!("<{}", tag);
    let close_tag = format!("</{}>", tag);
    let mut depth = 1;
    let mut i = from;

    while i < svg.len() {
        if svg[i..].starts_with('<') {
            if svg[i..].starts_with(&close_tag) {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += close_tag.len();
                continue;
            }
            if svg[i..].starts_with(&open_tag) {
                let after = &svg[i + open_tag.len()..];
                let next_ch = after.chars().next();
                if next_ch == Some('>') || next_ch == Some(' ') || next_ch == Some('\n') {
                    if let Some(end_pos) = after.find('>') {
                        if end_pos > 0 && after.as_bytes()[end_pos - 1] == b'/' {
                            // self-closing <tag .../>, skip
                            i += open_tag.len() + end_pos + 1;
                            continue;
                        }
                        depth += 1;
                        i += open_tag.len() + end_pos + 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    None
}

fn split_tag_name_attrs(inner: &str) -> (String, String) {
    let inner = inner.trim();
    if let Some(pos) = inner.find(|c: char| c.is_whitespace()) {
        (inner[..pos].to_string(), inner[pos..].trim().to_string())
    } else {
        (inner.to_string(), String::new())
    }
}

fn extract_attr<'a>(attrs: &'a str, name: &str) -> Option<&'a str> {
    let pattern = format!("{}=\"", name);
    let start = attrs.find(&pattern)?;
    let val_start = start + pattern.len();
    let val_end = attrs[val_start..].find('"')?;
    Some(&attrs[val_start..val_start + val_end])
}

fn replace_or_append_attr(attrs: &str, name: &str, value: &str) -> String {
    let pattern = format!("{}=\"", name);
    if let Some(start) = attrs.find(&pattern) {
        let val_start = start + pattern.len();
        let val_end = attrs[val_start..].find('"').unwrap();
        format!(
            "{}{}=\"{}\"{}",
            &attrs[..start],
            name,
            value,
            &attrs[val_start + val_end + 1..]
        )
    } else if attrs.is_empty() {
        format!("{}=\"{}\"", name, value)
    } else {
        format!("{} {}=\"{}\"", attrs, name, value)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_namespaces() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd">
  <g inkscape:label="Layer 1">
    <rect x="0" y="0" width="100" height="100"/>
  </g>
</svg>"#;
        let result =
            clean_svg(svg, &[CleanRule::RemoveNamespaces]).unwrap();
        assert!(!result.cleaned_svg.contains("xmlns:inkscape"));
        assert!(!result.cleaned_svg.contains("xmlns:sodipodi"));
        assert!(!result.cleaned_svg.contains("inkscape:label"));
        assert!(result.cleaned_svg.contains("rect"));
        assert!(result.rules_applied.contains(&"RemoveNamespaces".to_string()));
    }

    #[test]
    fn test_remove_metadata() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
  <metadata id="meta1">Some metadata</metadata>
  <title>My Icon</title>
  <desc>A description</desc>
  <rect x="0" y="0" width="100" height="100"/>
</svg>"#;
        let result =
            clean_svg(svg, &[CleanRule::RemoveMetadata]).unwrap();
        assert!(!result.cleaned_svg.contains("<metadata"));
        assert!(!result.cleaned_svg.contains("<title>"));
        assert!(!result.cleaned_svg.contains("<desc>"));
        assert!(result.cleaned_svg.contains("<rect"));
    }

    #[test]
    fn test_reduce_precision() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
  <path d="M 3.14159265 2.71828183 L 1.41421356 0"/>
</svg>"#;
        let result =
            clean_svg(svg, &[CleanRule::ReducePrecision]).unwrap();
        assert!(result.cleaned_svg.contains("3.14"));
        assert!(result.cleaned_svg.contains("2.72"));
        assert!(result.cleaned_svg.contains("1.41"));
        assert!(!result.cleaned_svg.contains("3.141"));
    }

    #[test]
    fn test_merge_single_child_groups() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><g transform="translate(10,10)"><rect x="0" y="0" width="100" height="100"/></g></svg>"#;
        let result =
            clean_svg(svg, &[CleanRule::MergeSingleChildGroups]).unwrap();
        assert!(!result.cleaned_svg.contains("<g"));
        assert!(result.cleaned_svg.contains("translate(10,10)"));
        assert!(result.cleaned_svg.contains("<rect"));
    }

    #[test]
    fn test_remove_empty_groups() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
  <g></g>
  <g id="empty"></g>
  <rect x="0" y="0" width="100" height="100"/>
</svg>"#;
        let result =
            clean_svg(svg, &[CleanRule::RemoveEmptyGroups]).unwrap();
        assert!(!result.cleaned_svg.contains("<g></g>"));
        assert!(!result.cleaned_svg.contains("<g id=\"empty\"></g>"));
        assert!(result.cleaned_svg.contains("<rect"));
    }

    #[test]
    fn test_remove_identity_transforms() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
  <g transform="translate(0,0)"><rect x="0" y="0" width="100" height="100"/></g>
  <g transform="translate(0)"><circle cx="50" cy="50" r="25"/></g>
  <g transform="scale(1)"><path d="M0 0L10 10"/></g>
</svg>"#;
        let result = clean_svg(svg, &[CleanRule::RemoveIdentityTransforms]).unwrap();
        assert!(!result.cleaned_svg.contains("translate(0,0)"));
        assert!(!result.cleaned_svg.contains("translate(0)"));
        assert!(!result.cleaned_svg.contains("scale(1)"));
    }

    #[test]
    fn test_default_rules_applied() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape">
  <metadata>data</metadata>
  <g transform="translate(0,0)">
    <g><path d="M 3.14159265 2.71828183" fill="none" stroke="black"/></g>
  </g>
</svg>"#;
        let result = clean_svg(svg, &default_rules()).unwrap();
        assert!(!result.cleaned_svg.contains("inkscape"));
        assert!(!result.cleaned_svg.contains("metadata"));
        assert!(!result.cleaned_svg.contains("translate(0,0)"));
        assert!(result.cleaned_svg.contains("path"));
    }

    #[test]
    fn test_clean_result_metrics() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd">
  <metadata>Very long metadata content that should be removed</metadata>
  <title>Title to remove</title>
  <desc>Description to remove</desc>
  <g transform="translate(0,0)">
    <g><path d="M 3.14159265358979 2.71828182845904" fill="none" stroke="black"/></g>
  </g>
</svg>"#;
        let result = clean_svg(svg, &default_rules()).unwrap();
        assert!(
            result.bytes_before > result.bytes_after,
            "Expected bytes_before ({}) > bytes_after ({})",
            result.bytes_before,
            result.bytes_after
        );
        assert!(!result.rules_applied.is_empty());
    }

    #[test]
    fn test_clean_valid_svg_preserved() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
  <rect x="10" y="10" width="80" height="80" fill="red"/>
</svg>"#;
        let result = clean_svg(svg, &default_rules()).unwrap();
        // usvg should be able to parse the cleaned SVG
        let opts = usvg::Options::default();
        let parsed = usvg::Tree::from_str(&result.cleaned_svg, &opts);
        assert!(parsed.is_ok(), "Cleaned SVG should be valid: {:?}", parsed.err());
    }
}
