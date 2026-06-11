# IconStudio

[![CI](https://github.com/Alfroul/icon-studio/actions/workflows/ci.yml/badge.svg)](https://github.com/Alfroul/icon-studio/actions/workflows/ci.yml)
![Tests](https://img.shields.io/badge/tests-360%2B-green)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)](https://github.com/Alfroul/icon-studio)
[![Tauri 2](https://img.shields.io/badge/Tauri-2.0-orange)](https://v2.tauri.app)
[![Vue 3](https://img.shields.io/badge/Vue-3.5-brightgreen)](https://vuejs.org)

> Parametric icon design tool for developers — GUI + MCP Server + CLI in one binary.

Design, preview, and export icons for any platform. Work visually on a canvas, drive everything from natural language via MCP, or automate from the CLI — all from the same local-first application.

## Why IconStudio

| | IconStudio | Recraft / IconifyAI | Figma + Plugins | Axialis |
|---|---|---|---|---|
| **Offline** | Yes (Rust native) | No (web only) | Partial | Yes |
| **Pricing** | Free (MIT) | Subscription / credits | Subscription | Free + paid packs |
| **Code export** | 10 formats | SVG/PNG only | Via plugins | SVG/XAML/PDF |
| **AI integration** | 4 providers + MCP bridge | Proprietary only | Via plugins | None |
| **SVG clean** | Built-in (8 rules) | No | Manual / SVGO | No |
| **Automation** | CLI + MCP (113 tools) | API only | No | No |
| **Icon fonts** | TTF + WOFF + demo | No | No | Web fonts only |
| **All-platform export** | iOS + Android + PWA + Favicon | No | Via plugins | Partial |

The only tool that lets an AI agent (Claude, Copilot, Cursor) design and export icons through conversation via MCP — while also providing a full visual editor for manual refinement.

## Features

### Canvas & Elements

- **14 shape types** — circle, rect, hexagon, star, shield, diamond, triangle, arrow, cross, heart, pentagon, octagon, wave, rounded-rect
- **Text** — system font enumeration, size/weight/letter-spacing controls
- **Lucide icons** — ~1700 built-in icons with keyword search
- **Image import** — PNG/JPG/SVG/WebP with automatic base64 encoding
- **Path editor** — SVG path input with visual Bezier editor
- **Layer groups** — group/ungroup, add/remove members
- **Templates** — 8 presets (app icon, letter logo, text logo) + user-defined
- **Multi-page** — Multiple canvases per project (light/dark/multi-size)

### Drag & Drop Workflow

- **Drag import** — Drop PNG/JPG/SVG/WebP files onto canvas; auto-centered with aspect ratio preserved
- **Clipboard paste** — Paste images from clipboard directly
- **QuickStart guide** — Empty canvas shows 4 entry points: drag image, template, AI generate, or blank canvas
- **Quick export** — One-click dropdown: iOS Pack / Android Pack / All Platforms / SVG / PNG / Icon Font

### Design

- **Boolean operations** — Union / Subtract / Intersect / Exclude
- **Clipping & masking** — SVG `clipPath` + `mask`
- **Blend modes** — 13 SVG `mix-blend-mode` options
- **Gradients** — Linear/radial with custom color stops and angles
- **Shadows** — Multiple shadows + inset, independent color/blur/offset
- **SVG filters** — Noise, blur, pixelate, emboss, posterize, turbulence
- **Symbol system** — Master component → instances → override propagation
- **Design analysis** — Color analysis, consistency check, palette suggestions
- **Brand kits** — Named color palettes with semantic roles (primary/secondary/accent), apply project-wide

### Overlay / Badge System

Attach status badges to any element:

- **10 preset types** — Add / Remove / Check / Info / Warning / Error / Star / Lock / New / Custom
- **4 positions** — TopLeft / TopRight / BottomLeft / BottomRight
- **Customizable** — Background color, size ratio (0.2–0.6), custom SVG path
- **Batch operations** — Multi-select and apply/remove in bulk
- **Export control** — Include or exclude overlays in exported files

### Theme Presets

20 built-in presets for one-click icon styling:

iOS · Android · macOS · Windows 11 · Material · Flat · Glassmorphism · Neon · Pixel Art · 3D Clay · Minimal · Duotone · Gradient · Outline · Round · Hexagon · Shield · Magazine · Retro · Custom

Each defines shape, corner radius, padding, background, shadow, and includes a preview thumbnail. Save your own as custom presets.

### Theme Variants

Generate appearance variants from transformation rules — not full copies:

- **4 built-in presets** — Dark Mode (color inversion), Hover (saturation boost), Active (darken), Disabled (grayscale + low opacity)
- **6 rule types** — InvertColors / ReplaceColor / AdjustOpacity / Grayscale / Desaturate / CustomFill
- **Rule composition** — Chain multiple rules per variant
- **Batch export** — All variants at once, named `{icon}.{variant}.{ext}`
- **Derived design** — Variants stored as rules; always reflect the latest source icon

### Variable Weight System

Generate thin → bold → fill variants from a single stroke icon (Phosphor-style):

- **6 weight presets** — Thin (0.5×) / Light (0.75×) / Regular (1.0×) / Medium (1.25×) / Bold (1.5×) / Fill (stroke-to-fill conversion)
- **Auto-detection** — Classifies icons as StrokeBased / FillBased / Mixed
- **Recursive** — Handles grouped elements with nested children
- **Clamped** — Stroke widths clamped to 0.1–20.0 range for safety

### Animation

- **Multi-keyframe tracks** — Multiple animation tracks per element
- **Visual timeline** — AE/Figma-style timeline panel
- **60fps playback** — DOM-direct manipulation engine
- **Easing curves** — Cubic-Bezier + Spring physics
- **Presets** — pulse / spin / bounce / fade-in / slide / wiggle / breathe
- **Export** — Lottie JSON, SMIL SVG, CSS `@keyframes`, GIF, WebP

### Device Preview

Real-time preview in simulated device frames:

- **iPhone 15** — 4×6 icon grid, status bar, Dock, notch
- **Android (Pixel 8)** — 4×5 grid, adaptive icon mask preview
- **MacBook** — macOS desktop with Dock
- **Browser** — Tab favicon (16×16) + page title
- **8 wallpapers** — iOS default, light, Material You, dark, nature, sunset, white, black

### Export

- **Multi-format** — SVG, multi-size PNG, ICO, WebP
- **App icon packs** — Android (all mipmap sizes + adaptive layers) + iOS (all AppIcon sizes)
- **Favicon pack** — ICO + multi-size PNGs + SVG + apple-touch-icon + site.webmanifest + browserconfig.xml + mstile icons + HTML `<link>` snippet
- **PWA icons** — manifest.json with 192×192 and 512×512 PNGs
- **All-platforms export** — One click generates iOS AppIcon.appiconset + Android mipmap + PWA manifest + Favicon bundle into `ios/` `android/` `pwa/` `favicon/` directories
- **Sprite sheet** — Multiple icons in a single row/grid image with `icons.json` positioning data for CSS sprites and toolbars
- **Pixel grid snapping** — Align SVG coordinates to pixel grid (default 0.5px) for crisp rendering at 16px/24px/32px sizes
- **Batch variations** — Color/size/border-radius/opacity variants in one click

### SVG Clean & Optimize

8 optimization rules targeting messy exported SVGs. Removes editor namespaces (Inkscape/Sodipodi/Figma), metadata, reduces coordinate precision, merges single-child groups, removes empty groups and identity transforms, strips redundant `fill="none"`, and snaps coordinates to pixel grid. Round-trip safe — cleaned output verified parseable by usvg.

### Code Export

Generate framework-ready components from any icon — **10 formats**:

| Category | Format | Output |
|----------|--------|--------|
| Web | React TS | Functional component with `size`/`color`/`className`/`style` props |
| Web | Vue TS | `<script setup>` with typed props |
| Web | Svelte | Svelte 5 component with `$props()` rune syntax |
| Web | SVG Symbol | `<symbol>` + `<use>` pattern with usage comment |
| Web | SVG Minified | Stripped comments, reduced precision, compact output |
| Mobile | SwiftUI | View struct with SVG reference |
| Mobile | Flutter | `StatelessWidget` with `flutter_svg` integration |
| Mobile | Android | VectorDrawable XML with `android:pathData` |
| Desktop | XAML | `<Viewbox><Canvas><Path>` with `RenderTransform` mapping |
| Desktop | C++ | Header-only `constexpr const char*` path constants |

Fill colors can be parameterized as `currentColor` for theming.

### Consistency Validator

Scan icon sets for design inconsistencies and auto-fix:

- **7 check dimensions** — Border radius, stroke width, font size, opacity, fill style, proportions, visual center drift
- **3 severity levels** — Info (<15% deviation) / Warning (15–30%) / Error (>30%)
- **Fill style detection** — Classifies elements as Outline / Filled / Duotone / None
- **Auto-fix** — Sets deviated properties to their mode (most common) values

### AI Icon Generation

Three-tier AI strategy — use what fits your workflow:

1. **MCP Bridge (zero-cost)** — External AI (Claude/GPT) operates the canvas directly through 113 MCP tools. No API key needed.
2. **Cloud API** — OpenAI (DALL-E / GPT-4o), Recraft V3/V4, custom endpoints
3. **Local Model** — Ollama (`localhost:11434`), fully offline, privacy-first

AI capabilities: text-to-icon, sketch-to-icon, style transfer, icon variation, background removal, color suggestion, brand analysis. 10 icon styles: Flat / Outline / Duotone / Gradient / 3D / Minimal / Cartoon / Pixel Art / Line Art / Neon.

### Design Tokens

Extract design tokens — colors (brand semantic + element), border radii, shadows, stroke widths, icon sizes. Output in 4 formats:

- **CSS Variables** (`:root`)
- **JSON** (DTCG `$value`/`$type`)
- **SCSS variables**
- **Tailwind Config**

### Icon Pack Library

- **Import from directory** — Recursively scan folders for SVG files
- **Auto-categorize** — Subdirectory names become categories
- **Validation** — Each SVG verified by usvg, invalid files skipped
- **Search** — Filter by name, category, or auto-generated tags
- **Lazy loading** — SVG content loaded on demand, index in memory
- **Pagination** — 100 icons per page, handles 5000+ icon packs

### Icon Font Generation

- **TTF + WOFF output** — TrueType font with WOFF compressed variant
- **SVG path → glyph** — Cubic Bezier → Quadratic Bezier conversion for TrueType compatibility
- **CSS + HTML demo** — Auto-generated `@font-face` CSS and interactive demo page
- **Private Use Area** — Unicode allocation starting at U+E000

### Icon Set Management

- **Create icon sets** from project canvases
- **Batch manage** — tag, search, consistency check across entries
- **Multi-format export** — SVG, PNG, code, tokens, font from a single set

### Adaptive Icons (Android)

- **Background + foreground layers** — Separate layers for adaptive icon design
- **All density buckets** — mdpi through xxxhdpi
- **Preview** — Visual preview with rounded mask

### MCP Integration

Embedded MCP Server exposing **113 tools** across 21 domains via stdio transport (`rmcp` crate).

| Group | Tools | Capability |
|-------|-------|-----------|
| core | 13 | Canvas creation, status, preview, basic elements, export |
| elements | 6 | Image, element find/copy/reorder, path |
| style | 10 | Shadow, gradient, border-radius, stroke dash, dash presets |
| export | 10 | ICO/WebP/favicon, project save/open, variation, code, tokens |
| analysis | 5 | Color analysis, consistency, palette, font/icon list |
| canvas | 16 | SVG import, zoom, flip/align, random icon, SVG elements |
| group | 4 | Group/ungroup, add/remove member |
| animation | 3 | Set/clear/list animations |
| boolean | 1 | Boolean operations |
| filter | 2 | Set/clear/list filters |
| page | 6 | Page CRUD, switch, reorder |
| symbol | 6 | Symbol create/update/detach/override |
| pack | 6 | Icon pack import/browse/search/remove |
| iconset | 7 | Icon set CRUD, consistency check, tag, search |
| brand | 6 | Brand kit CRUD, apply brand colors |
| adaptive | 5 | Adaptive icon layers, export Android |
| lottie | 3 | Lottie JSON export |
| style preset | 4 | Style preset save/load/list/delete |
| overlay | 3 | Add/remove/batch overlay badges |
| variant | 5 | Create/export theme variants, weight variants |
| ai | 8 | Generate, style transfer, remove BG, vectorize, suggest colors, complete icon, analyze brand |

Default mode exposes 13 core tools. `load_extended_tools` expands to the full set.

### CLI

```bash
# Single file export
iconstudio export --input app.iconproject.json --format png --sizes 16,32,64,128 --output ./export/

# Color analysis & consistency check
iconstudio analyze --input ./icons/

# Batch parallel export
iconstudio batch --input ./icons/ --output ./export/ --format png

# Batch variation generation
iconstudio variations --input app.json --config variations.json --output ./variants/
```

### Real-time Collaboration

WebSocket multi-client sync with Last-Writer-Wins conflict resolution.

## Architecture

```
Frontend (Vue 3 + Pinia)
  │  invoke / events
Tauri Bridge (120+ commands)
  │
Rust Engine
  ├── Model — IconProject / Elements / Canvas / History
  │          Overlay / ThemeVariant / ThemeRule / ThemePreset
  │          AiProvider / AiTask / IconStyle
  ├── Engine — Builder → Renderer → Exporter
  │          Optimizer → Codegen (10 formats) → Tokens
  │          Fontgen → PackImporter
  │          Lottie → Adaptive → Brand
  │          AI → Variants → ThemePresets → Weight
  │          Analyzer → Consistency → Auto-fix
  ├── MCP Server — 113 tools via rmcp
  ├── CLI — export / analyze / batch
  └── WebSocket Sync — Multi-client
```

Three execution modes from one binary:

```
icon-studio.exe              → GUI mode
icon-studio.exe --mcp        → MCP Server (stdio)
icon-studio.exe export ...   → CLI mode
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | Tauri 2 |
| Frontend | Vue 3 + TypeScript + Pinia |
| Build | Vite |
| SVG rendering | resvg / usvg (Rust) |
| MCP Server | rmcp crate (stdio transport) |
| Fonts | fontdb crate |
| Icon library | Lucide Icons SVG paths |
| Boolean ops | geo crate |
| HTTP client | reqwest (rustls-tls) |
| CLI | clap crate |
| Parallelism | rayon crate |

## Quick Start

```bash
# Prerequisites: Node.js 20+, pnpm 10, Rust 1.77+

git clone https://github.com/Alfroul/icon-studio.git && cd icon-studio
pnpm install
pnpm tauri dev
```

### MCP Configuration

```bash
# Headless mode (MCP Server only, no window)
cargo run --manifest-path src-tauri/Cargo.toml -- --mcp
```

In your MCP client config:

```json
{
  "mcpServers": {
    "iconstudio": {
      "command": "path/to/icon-studio.exe",
      "args": ["--mcp"]
    }
  }
}
```

## Project Structure

```
IconStudio/
├── src-tauri/src/
│   ├── main.rs              # Entry: GUI / MCP / CLI routing
│   ├── lib.rs               # Tauri app config, command registrations
│   ├── model/               # Data model (IconProject, Elements, Canvas, History)
│   ├── engine/              # SVG engine
│   │   ├── builder.rs       # IconProject → SVG (version-cached)
│   │   ├── renderer.rs      # SVG → PNG (usvg + resvg)
│   │   ├── exporter.rs      # PNG → ICO/WebP, favicon, PWA, sprite sheet, pixel snap
│   │   ├── optimizer.rs     # SVG clean & optimize (8 rules incl. pixel grid snap)
│   │   ├── codegen.rs       # Code export (10 formats)
│   │   ├── tokens.rs        # Design token extraction
│   │   ├── fontgen.rs       # Icon font generation (TTF/WOFF)
│   │   ├── pack_importer.rs # Icon pack import + index
│   │   ├── lottie.rs        # Lottie JSON animation export
│   │   ├── adaptive.rs      # Android adaptive icon layers
│   │   ├── brand.rs         # Brand kit management
│   │   ├── iconset.rs       # Icon set management
│   │   ├── ai.rs            # AI icon generation (cloud + local)
│   │   ├── variants.rs      # Theme variant engine (6 rule types)
│   │   ├── theme_presets.rs # 20 built-in theme presets
│   │   ├── weight.rs        # Variable weight system (6 presets)
│   │   └── analyzer.rs      # Consistency check + auto-fix
│   ├── mcp/                 # MCP tools across 21 domains
│   ├── commands/            # Tauri IPC commands
│   ├── cli/                 # CLI subcommands
│   └── services/            # WebSocket sync, layout, project, AI config
├── src/
│   ├── components/          # UI components by feature area
│   │   ├── ai/              # AiPanel — AI interaction
│   │   ├── preview/         # DevicePreview — device frame previews
│   │   ├── variants/        # VariantsPanel — theme variant management
│   │   ├── analysis/        # AnalysisPanel — consistency check
│   │   └── quickstart/      # QuickStartOverlay — empty canvas guide
│   ├── composables/         # useKeyboard, useDragReorder, usePathEditor
│   ├── stores/              # Pinia stores (project, elements, canvas, ui, ai, variants, export)
│   └── types/               # TypeScript type definitions
└── README.md
```

## Testing

```bash
# Rust backend tests
cd src-tauri && cargo test

# Frontend tests
pnpm test

# Type check
pnpm build

# Lint
pnpm lint && cd src-tauri && cargo clippy --lib
```

## File Formats

- **`.iconproject.json`** — Project file with full canvas state, elements, export config, pages, symbols, overlays, theme variants. All new fields use `#[serde(default)]` for backward compatibility.
- **`.iconstudio-template.json`** — User-saved templates, same format as project files.

## License

MIT
