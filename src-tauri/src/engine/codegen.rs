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
    Xaml,
    VectorDrawable,
    SvgSymbol,
    SvgMinified,
    Cpp,
    Svelte,
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

pub fn gen_xaml(svg_content: &str, _name: &str, size: u32, parametrize: bool) -> String {
    let info = extract_svg_info(svg_content);
    let parts = parse_viewbox(&info.view_box);

    let mut paths = String::new();
    extract_paths(&info.inner, |d, fill| {
        let fill_attr = if parametrize {
            r#"Fill="{Binding Color, FallbackValue=Black}""#.to_string()
        } else if let Some(f) = fill {
            format!("Fill=\"{}\"", f)
        } else {
            String::new()
        };
        paths.push_str(&format!(
            "    <Path Data=\"{}\" {} />\n",
            d, fill_attr
        ));
    });

    format!(
        r#"<Viewbox Width="{size}" Height="{size}">
  <Canvas Width="{w}" Height="{h}">
{paths}  </Canvas>
</Viewbox>
"#,
        size = size,
        w = parts.2,
        h = parts.3,
        paths = paths.trim_end(),
    )
}

pub fn gen_vector_drawable(svg_content: &str, name: &str, _size: u32) -> String {
    let info = extract_svg_info(svg_content);
    let parts = parse_viewbox(&info.view_box);

    let mut paths = String::new();
    extract_paths(&info.inner, |d, fill| {
        let fill_attr = match fill {
            Some(f) if f != "none" => format!("android:fillColor=\"{}\"", f),
            _ => String::new(),
        };
        paths.push_str(&format!(
            "  <path\n    android:pathData=\"{}\"\n    {} />\n",
            d, fill_attr
        ));
    });

    format!(
        r#"<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:name="{name}"
    android:viewportWidth="{vw}"
    android:viewportHeight="{vh}"
    android:width="{vw}dp"
    android:height="{vh}dp">
{paths}</vector>
"#,
        name = name,
        vw = parts.2,
        vh = parts.3,
        paths = paths.trim_end(),
    )
}

pub fn gen_svg_symbol(svg_content: &str, name: &str) -> String {
    let info = extract_svg_info(svg_content);

    format!(
        r##"<!-- Usage: <svg><use href="#{name}"/></svg> -->
<svg style="display:none" xmlns="http://www.w3.org/2000/svg">
  <symbol id="{name}" viewBox="{view_box}">
    {inner}
  </symbol>
</svg>
"##,
        name = name,
        view_box = info.view_box,
        inner = info.inner.trim(),
    )
}

static RE_NUM: OnceLock<regex::Regex> = OnceLock::new();

pub fn gen_svg_minified(svg_content: &str) -> String {
    let mut result = svg_content.to_string();

    // Remove XML comments
    let re_comment = regex::Regex::new(r"<!--[\s\S]*?-->").unwrap();
    result = re_comment.replace_all(&result, "").to_string();

    // Reduce number precision to 2 decimal places
    let re_num = RE_NUM.get_or_init(|| {
        regex::Regex::new(r"(\d+\.\d{3,})").unwrap()
    });
    result = re_num.replace_all(&result, |caps: &regex::Captures| {
        let num: f64 = caps[1].parse().unwrap_or(0.0);
        format!("{:.2}", num)
    }).to_string();

    // Remove default fill="black" (black is the SVG default)
    let re_fill_black = regex::Regex::new(r#"\s*fill="black""#).unwrap();
    result = re_fill_black.replace_all(&result, "").to_string();

    // Collapse multiple blank lines into one
    let re_blank = regex::Regex::new(r"\n\s*\n\s*\n").unwrap();
    result = re_blank.replace_all(&result, "\n\n").to_string();

    // Trim leading/trailing whitespace per line
    result = result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    result
}

pub fn gen_cpp(svg_content: &str, name: &str, _size: u32) -> String {
    let info = extract_svg_info(svg_content);
    let parts = parse_viewbox(&info.view_box);
    let upper = name.to_uppercase().replace('-', "_");

    let mut path_defs = String::new();
    let mut idx = 0;
    extract_paths(&info.inner, |d, _fill| {
        path_defs.push_str(&format!(
            "constexpr const char* k{}Path{} = \"{}\";\n",
            name, idx, d
        ));
        idx += 1;
    });

    format!(
        r#"#pragma once

namespace icons {{
constexpr int k{upper}ViewBoxX = {vx};
constexpr int k{upper}ViewBoxY = {vy};
constexpr int k{upper}ViewBoxW = {vw};
constexpr int k{upper}ViewBoxH = {vh};
{path_defs}}} // namespace icons
"#,
        upper = upper,
        vx = parts.0,
        vy = parts.1,
        vw = parts.2,
        vh = parts.3,
        path_defs = path_defs,
    )
}

pub fn gen_svelte(svg_content: &str, _name: &str, _size: u32, parametrize: bool) -> String {
    let info = extract_svg_info(svg_content);
    let inner = if parametrize {
        let re = regex::Regex::new(r#"fill="[^"]*""#).unwrap();
        re.replace_all(&info.inner, r#"fill={color}"#).to_string()
    } else {
        info.inner.clone()
    };

    let props_block = if parametrize {
        r#"interface Props {
  size?: number;
  color?: string;
}

let { size = 24, color = 'currentColor' }: Props = $props();"#
    } else {
        r#"interface Props {
  size?: number;
}

let { size = 24 }: Props = $props();"#
    };

    format!(
        r#"<script lang="ts">
{props_block}
</script>

<svg
  width={{size}}
  height={{size}}
  viewBox="{view_box}"
>
  {inner}
</svg>
"#,
        props_block = props_block,
        view_box = info.view_box,
        inner = inner.trim(),
    )
}

/// Parse viewBox "x y w h" into (x, y, w, h) with defaults.
fn parse_viewbox(view_box: &str) -> (u32, u32, u32, u32) {
    let parts: Vec<&str> = view_box.split_whitespace().collect();
    let parse = |i: usize| -> u32 {
        parts.get(i).and_then(|s| s.parse().ok()).unwrap_or(0)
    };
    let x = parse(0);
    let y = parse(1);
    let w = parse(2);
    let h = parse(3);
    (x, y, if w > 0 { w } else { 24 }, if h > 0 { h } else { 24 })
}

/// Extract <path d="..." fill="..."/> entries from SVG inner content.
fn extract_paths<F: FnMut(String, Option<String>)>(inner: &str, mut handler: F) {
    let re = regex::Regex::new(r#"<path\b[^>]*>"#).unwrap();
    for cap in re.captures_iter(inner) {
        let tag = &cap[0];
        if let Some(d) = extract_attr(tag, "d") {
            let fill = extract_attr(tag, "fill");
            handler(d, fill);
        }
    }
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
        CodeFormat::Xaml => gen_xaml(&parametrized, &pascal, options.size, options.parametrize_fill),
        CodeFormat::VectorDrawable => gen_vector_drawable(&parametrized, &pascal, options.size),
        CodeFormat::SvgSymbol => gen_svg_symbol(&parametrized, &pascal),
        CodeFormat::SvgMinified => gen_svg_minified(&parametrized),
        CodeFormat::Cpp => gen_cpp(&parametrized, &pascal, options.size),
        CodeFormat::Svelte => gen_svelte(&parametrized, &pascal, options.size, options.parametrize_fill),
    };

    let ext = match options.format {
        CodeFormat::ReactTs => ".tsx",
        CodeFormat::VueTs => ".vue",
        CodeFormat::SwiftUI => ".swift",
        CodeFormat::Flutter => ".dart",
        CodeFormat::Xaml => ".xaml",
        CodeFormat::VectorDrawable => ".xml",
        CodeFormat::SvgSymbol => ".svg",
        CodeFormat::SvgMinified => ".min.svg",
        CodeFormat::Cpp => ".hpp",
        CodeFormat::Svelte => ".svelte",
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

    // ---- Stage 9: Extended format tests ----

    #[test]
    fn test_gen_xaml() {
        let code = gen_xaml(sample_svg(), "HomeIcon", 24, false);
        assert!(code.contains("<Viewbox"), "should contain Viewbox");
        assert!(code.contains("<Canvas"), "should contain Canvas");
        assert!(code.contains("<Path"), "should contain Path");
        assert!(code.contains("Width=\"24\""), "should use requested size");
        assert!(code.contains("Data=\""), "should have path data");
    }

    #[test]
    fn test_gen_xaml_parametrize() {
        let code = gen_xaml(sample_svg(), "HomeIcon", 24, true);
        assert!(code.contains("Binding Color"), "parametrized fill should use binding");
    }

    #[test]
    fn test_gen_vector_drawable() {
        let code = gen_vector_drawable(sample_svg(), "home_icon", 24);
        assert!(code.contains("android:pathData"), "should contain pathData");
        assert!(code.contains("android:viewportWidth"), "should contain viewportWidth");
        assert!(code.contains("android:viewportHeight"), "should contain viewportHeight");
        assert!(code.contains("android:fillColor"), "should contain fillColor");
        assert!(code.contains("android:name=\"home_icon\""), "should contain name");
    }

    #[test]
    fn test_gen_svg_symbol() {
        let code = gen_svg_symbol(sample_svg(), "home-icon");
        assert!(code.contains("<symbol id=\"home-icon\""), "should contain symbol with id");
        assert!(code.contains("viewBox=\"0 0 512 512\""), "should contain viewBox");
        assert!(code.contains("display:none"), "should hide the sprite");
        assert!(code.contains("Usage:"), "should contain usage comment");
    }

    #[test]
    fn test_gen_svg_minified() {
        let input = "<!-- comment -->\n<svg xmlns=\"http://www.w3.org/2000/svg\">\n  <path d=\"M10.12345 20.67890\" fill=\"black\"/>\n\n\n\n</svg>";
        let output = gen_svg_minified(input);
        assert!(output.len() < input.len(), "minified should be shorter");
        assert!(!output.contains("<!--"), "should remove comments");
        assert!(!output.contains("fill=\"black\""), "should remove default black fill");
    }

    #[test]
    fn test_gen_svg_minified_reduces_precision() {
        let input = "<svg><path d=\"M10.12345 20.67890\"/></svg>";
        let output = gen_svg_minified(input);
        assert!(output.contains("10.12"), "should reduce to 2 decimals");
        assert!(output.contains("20.68"), "should round to 2 decimals");
    }

    #[test]
    fn test_gen_cpp() {
        let code = gen_cpp(sample_svg(), "HomeIcon", 24);
        assert!(code.contains("constexpr const char*"), "should contain constexpr");
        assert!(code.contains("namespace icons"), "should contain namespace");
        assert!(code.contains("#pragma once"), "should contain include guard");
        assert!(code.contains("kHOMEICONViewBoxW"), "should contain viewBox width constant");
    }

    #[test]
    fn test_gen_svelte() {
        let code = gen_svelte(sample_svg(), "HomeIcon", 24, false);
        assert!(code.contains("$props()"), "should use Svelte 5 runes");
        assert!(code.contains("<script lang=\"ts\">"), "should use TypeScript");
        assert!(code.contains("<svg"), "should contain svg element");
        assert!(code.contains("interface Props"), "should define Props interface");
    }

    #[test]
    fn test_gen_svelte_parametrize() {
        let code = gen_svelte(sample_svg(), "HomeIcon", 24, true);
        assert!(code.contains("color = 'currentColor'"), "should default color prop");
        assert!(code.contains("fill={color}"), "should use color variable");
    }

    #[test]
    fn test_all_formats_round_trip() {
        let svg = sample_svg();
        let formats = [
            CodeFormat::ReactTs,
            CodeFormat::VueTs,
            CodeFormat::SwiftUI,
            CodeFormat::Flutter,
            CodeFormat::Xaml,
            CodeFormat::VectorDrawable,
            CodeFormat::SvgSymbol,
            CodeFormat::SvgMinified,
            CodeFormat::Cpp,
            CodeFormat::Svelte,
        ];
        for fmt in formats {
            let opts = CodeExportOptions {
                component_name: "TestIcon".into(),
                format: fmt.clone(),
                size: 24,
                parametrize_fill: true,
            };
            let result = export_code(svg, &opts).unwrap();
            assert!(!result.code.is_empty(), "code should not be empty for {:?}", fmt);
            assert!(!result.filename.is_empty(), "filename should not be empty for {:?}", fmt);
        }
    }

    #[test]
    fn test_export_code_filename_extended() {
        let pairs = [
            (CodeFormat::Xaml, "my-awesome-icon.xaml"),
            (CodeFormat::VectorDrawable, "my-awesome-icon.xml"),
            (CodeFormat::SvgSymbol, "my-awesome-icon.svg"),
            (CodeFormat::SvgMinified, "my-awesome-icon.min.svg"),
            (CodeFormat::Cpp, "my-awesome-icon.hpp"),
            (CodeFormat::Svelte, "my-awesome-icon.svelte"),
        ];
        for (fmt, expected) in pairs {
            let opts = CodeExportOptions {
                component_name: "MyAwesomeIcon".into(),
                format: fmt,
                size: 24,
                parametrize_fill: false,
            };
            let result = export_code(sample_svg(), &opts).unwrap();
            assert_eq!(result.filename, expected);
        }
    }
}
