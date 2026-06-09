use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CodeFormat {
    ReactTs,
    VueTs,
    SwiftUI,
    Flutter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeExportOptions {
    pub component_name: String,
    pub format: CodeFormat,
    pub size: u32,
    pub parametrize_fill: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeExportResult {
    pub code: String,
    pub format: CodeFormat,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetCodeExportResult {
    pub files: Vec<CodeExportResult>,
}

struct SvgInfo {
    inner: String,
    view_box: String,
}

fn extract_svg_info(svg_content: &str) -> SvgInfo {
    let mut view_box = "0 0 24 24".to_string();
    let mut inner = String::new();

    if let Some(svg_start) = svg_content.find("<svg") {
        if let Some(tag_end) = svg_content[svg_start..].find('>') {
            let tag = &svg_content[svg_start..svg_start + tag_end];
            if let Some(vb) = extract_attr(tag, "viewBox") {
                view_box = vb;
            }
            let content_start = svg_start + tag_end + 1;
            if let Some(close_pos) = svg_content.rfind("</svg>") {
                inner = svg_content[content_start..close_pos].to_string();
            } else {
                inner = svg_content[content_start..].to_string();
            }
        }
    }

    SvgInfo {
        inner,
        view_box,
    }
}

fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let patterns = [
        format!("{}=\"", name),
        format!("{}='", name),
    ];
    for pat in patterns {
        if let Some(start) = tag.find(&pat) {
            let val_start = start + pat.len();
            let quote = &pat[pat.len() - 1..pat.len()];
            if let Some(end) = tag[val_start..].find(quote) {
                return Some(tag[val_start..val_start + end].to_string());
            }
        }
    }
    None
}

pub fn parametrize_svg(svg_content: &str, options: &CodeExportOptions) -> String {
    let mut result = svg_content.to_string();

    if options.parametrize_fill {
        result = regex_replace_fill(&result);
    }

    result
}

static RE_FILL_HEX: OnceLock<regex::Regex> = OnceLock::new();

fn regex_replace_fill(svg: &str) -> String {
    let mut result = svg.to_string();
    let re = RE_FILL_HEX.get_or_init(|| {
        regex::Regex::new(r#"fill="(#[0-9a-fA-F]{3,8})""#).unwrap()
    });
    let fills: Vec<_> = re
        .find_iter(svg)
        .filter(|m| m.as_str() != r#"fill="none""#)
        .map(|m| (m.start(), m.end()))
        .collect();
    for (start, end) in fills.into_iter().rev() {
        result.replace_range(start..end, r#"fill="currentColor""#);
    }
    result
}

pub fn to_pascal_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for ch in name.chars() {
        if !ch.is_alphanumeric() {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    if result.is_empty() {
        result = "Icon".to_string();
    }
    result
}

pub fn to_kebab_case(name: &str) -> String {
    let mut result = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('-');
        }
        result.push(ch.to_ascii_lowercase());
    }
    let result: String = result
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    result
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn generate_react_ts(name: &str, svg_inner: &str, view_box: &str) -> String {
    format!(
        r#"import React from 'react';

interface {name}Props {{
  size?: number;
  color?: string;
  className?: string;
  style?: React.CSSProperties;
}}

export const {name}: React.FC<{name}Props> = ({{
  size = 24,
  color = 'currentColor',
  className,
  style,
}}) => (
  <svg
    width={{size}}
    height={{size}}
    viewBox="{view_box}"
    fill={{color}}
    className={{className}}
    style={{style}}
  >
    {svg_inner}
  </svg>
);

export default {name};
"#,
        name = name,
        svg_inner = svg_inner.trim(),
        view_box = view_box,
    )
}

pub fn generate_vue_ts(_name: &str, svg_inner: &str, view_box: &str) -> String {
    format!(
        r#"<script setup lang="ts">
interface Props {{
  size?: number;
  color?: string;
  class?: string;
}}
const props = withDefaults(defineProps<Props>(), {{
  size: 24,
  color: 'currentColor',
}});
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="{view_box}"
    :fill="color"
    :class="$props.class"
  >
    {svg_inner}
  </svg>
</template>
"#,
        svg_inner = svg_inner.trim(),
        view_box = view_box,
    )
}

pub fn generate_swiftui(name: &str, svg_content: &str) -> String {
    format!(
        r#"import SwiftUI

struct {name}: View {{
    var size: CGFloat = 24
    var body: some View {{
        // SwiftUI does not natively embed SVG.
        // Use an SVG asset catalog or wrap the SVG in a WKWebView / UIImage.
        // Below is the raw SVG for reference:
        /*
        {svg_content}
        */
        Image(systemName: "square.dashed")
            .frame(width: size, height: size)
    }}
}}
"#,
        name = name,
        svg_content = svg_content.trim(),
    )
}

pub fn generate_flutter(name: &str, svg_content: &str) -> String {
    let class_name = format!("{}Icon", name);
    let safe_svg = svg_content.replace("'''", "\\'\\'\\'");
    format!(
        r#"import 'package:flutter_svg/flutter_svg.dart';

class {class_name} extends StatelessWidget {{
  final double size;
  final Color color;
  const {class_name}({{super.key, this.size = 24, this.color = Colors.black}});

  @override
  Widget build(BuildContext context) {{
    return SvgPicture.string(
      \'\'\'{svg_content}\'\'\',
      width: size,
      height: size,
      colorFilter: ColorFilter.mode(color, BlendMode.srcIn),
    );
  }}
}}
"#,
        class_name = class_name,
        svg_content = safe_svg.trim(),
    )
}
pub fn export_code(svg_content: &str, options: &CodeExportOptions) -> Result<CodeExportResult, AppError> {
    let parametrized = parametrize_svg(svg_content, options);
    let info = extract_svg_info(&parametrized);
    let pascal = to_pascal_case(&options.component_name);

    let code = match options.format {
        CodeFormat::ReactTs => generate_react_ts(&pascal, &info.inner, &info.view_box),
        CodeFormat::VueTs => generate_vue_ts(&pascal, &info.inner, &info.view_box),
        CodeFormat::SwiftUI => generate_swiftui(&pascal, &parametrized),
        CodeFormat::Flutter => generate_flutter(&pascal, &parametrized),
    };

    let ext = match options.format {
        CodeFormat::ReactTs => ".tsx",
        CodeFormat::VueTs => ".vue",
        CodeFormat::SwiftUI => ".swift",
        CodeFormat::Flutter => ".dart",
    };
    let filename = format!("{}{}", to_kebab_case(&options.component_name), ext);

    Ok(CodeExportResult {
        code,
        format: options.format.clone(),
        filename,
    })
}

pub fn export_set_code(
    set_id: &str,
    options: &CodeExportOptions,
) -> Result<SetCodeExportResult, AppError> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let set_dir = home.join(".iconstudio").join("sets").join(set_id);
    if !set_dir.exists() {
        return Err(AppError::NotFoundError(format!("Icon set '{}' not found", set_id)));
    }

    let index_path = set_dir.join("icon-set.json");
    let index_json = std::fs::read_to_string(&index_path)
        .map_err(AppError::IoError)?;
    let set: crate::model::IconSet = serde_json::from_str(&index_json)
        .map_err(AppError::SerdeError)?;

    let mut files = Vec::new();
    for entry in &set.entries {
        let project_path = Path::new(&entry.project_path);
        if !project_path.exists() {
            continue;
        }
        let project_json = std::fs::read_to_string(project_path)
            .map_err(AppError::IoError)?;
        let project: crate::model::IconProject = serde_json::from_str(&project_json)
            .map_err(AppError::SerdeError)?;

        let svg = crate::engine::builder::build(&project)
            .map_err(|e| AppError::BuildError(e.to_string()))?;

        let entry_options = CodeExportOptions {
            component_name: entry.name.clone(),
            ..options.clone()
        };

        let result = export_code(&svg, &entry_options)?;
        files.push(result);
    }

    Ok(SetCodeExportResult { files })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_svg() -> &'static str {
        concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"512\" height=\"512\" viewBox=\"0 0 512 512\">\n",
            "  <rect fill=\"#FF5500\" x=\"0\" y=\"0\" width=\"512\" height=\"512\" rx=\"64\"/>\n",
            "  <path fill=\"#FFFFFF\" d=\"M256 128l128 256H128z\"/>\n",
            "</svg>"
        )
    }

    #[test]
    fn test_parametrize_svg_replaces_fill() {
        let opts = CodeExportOptions {
            component_name: "TestIcon".into(),
            format: CodeFormat::ReactTs,
            size: 24,
            parametrize_fill: true,
        };
        let result = parametrize_svg(sample_svg(), &opts);
        assert!(result.contains(r#"fill="currentColor""#), "hex fills should be replaced with currentColor");
        assert!(!result.contains("#FF5500"), "original hex fill should be gone");
    }

    #[test]
    fn test_parametrize_svg_extracts_inner() {
        let info = extract_svg_info(sample_svg());
        assert!(info.inner.contains("<rect"), "inner should contain <rect>");
        assert!(info.inner.contains("<path"), "inner should contain <path>");
        assert!(!info.inner.contains("<svg"), "inner should not contain <svg> tag");
        assert_eq!(info.view_box, "0 0 512 512");
    }

    #[test]
    fn test_generate_react_ts() {
        let code = generate_react_ts("HomeIcon", "<path d=\"M10 20\"/>", "0 0 24 24");
        assert!(code.contains("interface HomeIconProps"));
        assert!(code.contains("export const HomeIcon"));
        assert!(code.contains("size = 24"));
        assert!(code.contains("color = 'currentColor'"));
        assert!(code.contains("viewBox=\"0 0 24 24\""));
        assert!(code.contains("<path d=\"M10 20\"/>"));
        assert!(code.contains("export default HomeIcon"));
    }

    #[test]
    fn test_generate_vue_ts() {
        let code = generate_vue_ts("HomeIcon", "<path d=\"M10 20\"/>", "0 0 24 24");
        assert!(code.contains("<script setup lang=\"ts\">"));
        assert!(code.contains("interface Props"));
        assert!(code.contains("defineProps<Props>"));
        assert!(code.contains("viewBox=\"0 0 24 24\""));
        assert!(code.contains("<path d=\"M10 20\"/>"));
    }

    #[test]
    fn test_generate_flutter() {
        let code = generate_flutter("Home", "<svg>...</svg>");
        assert!(code.contains("class HomeIcon extends StatelessWidget"));
        assert!(code.contains("SvgPicture.string"));
        assert!(code.contains("ColorFilter.mode"));
        assert!(code.contains("this.size = 24"));
    }

    #[test]
    fn test_export_code_filename() {
        let opts = CodeExportOptions {
            component_name: "MyAwesomeIcon".into(),
            format: CodeFormat::ReactTs,
            size: 24,
            parametrize_fill: false,
        };
        let result = export_code(sample_svg(), &opts).unwrap();
        assert_eq!(result.filename, "my-awesome-icon.tsx");

        let opts = CodeExportOptions {
            component_name: "MyAwesomeIcon".into(),
            format: CodeFormat::VueTs,
            size: 24,
            parametrize_fill: false,
        };
        let result = export_code(sample_svg(), &opts).unwrap();
        assert_eq!(result.filename, "my-awesome-icon.vue");

        let opts = CodeExportOptions {
            component_name: "MyAwesomeIcon".into(),
            format: CodeFormat::Flutter,
            size: 24,
            parametrize_fill: false,
        };
        let result = export_code(sample_svg(), &opts).unwrap();
        assert_eq!(result.filename, "my-awesome-icon.dart");

        let opts = CodeExportOptions {
            component_name: "MyAwesomeIcon".into(),
            format: CodeFormat::SwiftUI,
            size: 24,
            parametrize_fill: false,
        };
        let result = export_code(sample_svg(), &opts).unwrap();
        assert_eq!(result.filename, "my-awesome-icon.swift");
    }

    #[test]
    fn test_export_code_with_real_svg() {
        let svg = sample_svg();
        for fmt in [CodeFormat::ReactTs, CodeFormat::VueTs, CodeFormat::SwiftUI, CodeFormat::Flutter] {
            let opts = CodeExportOptions {
                component_name: "TestIcon".into(),
                format: fmt,
                size: 24,
                parametrize_fill: true,
            };
            let result = export_code(svg, &opts).unwrap();
            assert!(!result.code.is_empty(), "code should not be empty for {:?}", opts.format);
            assert!(!result.filename.is_empty(), "filename should not be empty");
        }
    }

    #[test]
    fn test_component_name_pascal_case() {
        assert_eq!(to_pascal_case("home-icon"), "HomeIcon");
        assert_eq!(to_pascal_case("home_icon"), "HomeIcon");
        assert_eq!(to_pascal_case("home icon"), "HomeIcon");
        assert_eq!(to_pascal_case("HomeIcon"), "HomeIcon");
        assert_eq!(to_pascal_case("home"), "Home");
        assert_eq!(to_pascal_case(""), "Icon");
    }
}
