# IconStudio

[![CI](https://github.com/Alfroul/icon-studio/actions/workflows/ci.yml/badge.svg)](https://github.com/Alfroul/icon-studio/actions/workflows/ci.yml)
![Tests](https://img.shields.io/badge/tests-360%2B-green)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)](https://github.com/Alfroul/icon-studio)
[![Tauri 2](https://img.shields.io/badge/Tauri-2.0-orange)](https://v2.tauri.app)
[![Vue 3](https://img.shields.io/badge/Vue-3.5-brightgreen)](https://vuejs.org)

> A parametric icon/logo design tool for developers — GUI + MCP Server + CLI in one binary.

Design, preview, and export icons for any platform — from a simple circle to a full Android/iOS app icon pack, driven by a visual canvas or natural language via MCP.

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

### Design

- **Boolean operations** — Union / Subtract / Intersect / Exclude
- **Clipping & masking** — SVG `clipPath` + `mask`
- **Blend modes** — 13 SVG `mix-blend-mode` options
- **Gradients** — Linear/radial with custom color stops and angles
- **Shadows** — Multiple shadows + inset, independent color/blur/offset
- **SVG filters** — Noise, blur, pixelate, emboss, posterize, turbulence
- **Symbol system** — Master component → instances → override propagation
- **Design analysis** — Color analysis, consistency check, palette suggestions
- **Brand kits** — Named color palettes with semantic roles (primary/secondary/accent), apply brand colors project-wide

### Animation

- **Multi-keyframe tracks** — Multiple animation tracks per element
- **Visual timeline** — AE/Figma-style timeline panel
- **60fps playback** — DOM-direct manipulation engine
- **Easing curves** — Cubic-Bezier + Spring physics
- **Presets** — pulse / spin / bounce / fade-in / slide / wiggle / breathe
- **Export** — Lottie JSON, SMIL SVG, CSS `@keyframes`, GIF, WebP

### Export

- **Multi-format** — SVG, multi-size PNG, ICO, WebP
- **App icon packs** — Android (all mipmap sizes + adaptive layers) + iOS (all AppIcon sizes)
- **Batch variations** — Color/size/border-radius/opacity variants in one click
- **Favicon pack** — ICO + multi-size PNG bundle

### SVG Clean & Optimize

7 optimization rules: remove editor namespaces (Inkscape/Sodipodi/Figma), metadata/title/desc, reduce coordinate precision, merge single-child groups, remove empty groups and identity transforms, remove redundant `fill="none"`. Round-trip safe — cleaned SVG verified parseable by usvg.

### Code Export

Generate framework components from any icon:

| Framework | Output |
|-----------|--------|
| React TS | Functional component with `size`/`color`/`className`/`style` props |
| Vue TS | `<script setup>` with typed props |
| SwiftUI | View struct with SVG reference |
| Flutter | `StatelessWidget` with `flutter_svg` integration |

Fill colors can be parameterized as `currentColor` for theming.

### Design Tokens

Extract design tokens from a project — colors (brand semantic + element), border radii, shadows, stroke widths, icon sizes. Output in 4 formats:

- **CSS Variables** (`:root`)
- **JSON** (DTCG `$value`/`$type`)
- **SCSS variables**
- **Tailwind Config**

### Icon Pack Library

- **Import from directory** — Recursively scan folders for SVG files
- **Auto-categorize** — Subdirectory names become categories
- **Validation** — Each SVG verified by usvg, invalid files skipped
- **Search** — Filter icons by name, category, or auto-generated tags
- **Lazy loading** — SVG content loaded on demand, index kept in memory
- **Pagination** — 100 icons per page, handles 5000+ icon packs smoothly
- **One-click add to canvas** — Import pack icons directly into the design

### Icon Font Generation

Export icon sets as fonts with full tooling:

- **TTF + WOFF output** — TrueType font with WOFF compressed variant
- **SVG path → glyph** — Cubic Bezier → Quadratic Bezier conversion for TrueType compatibility
- **CSS + HTML demo** — Auto-generated `@font-face` CSS and interactive demo page
- **Private Use Area** — Unicode allocation starting at U+E000

### Icon Set Management

- **Create icon sets** from project canvases
- **Batch manage** — tag, search, consistency check across entries
- **Multi-format export** — SVG, PNG, code, tokens, font from a single set

### Adaptive Icons (Android)

- **Background + foreground layers** — Design adaptive icons with separate background and foreground
- **All density buckets** — mdpi through xxxhdpi
- **Preview** — Visual preview of adaptive icon with rounded mask

### MCP Integration

Embedded MCP Server exposing **113 tools** across 18 domains via stdio transport (`rmcp` crate).

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

Default mode exposes 13 core tools. `load_extended_tools` expands to the full 113.

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
Tauri Bridge (120 commands)
  │
Rust Engine
  ├── Model — IconProject / Elements / Canvas / History
  ├── Engine — Builder → Renderer → Exporter
  │          Optimizer → Codegen → Tokens
  │          Fontgen → PackImporter
  │          Lottie → Adaptive → Brand
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
│   ├── lib.rs               # Tauri app config, 120 command registrations
│   ├── model/               # Data model (IconProject, Elements, Canvas, History)
│   ├── engine/              # SVG engine
│   │   ├── builder.rs       # IconProject → SVG (version-cached)
│   │   ├── renderer.rs      # SVG → PNG (usvg + resvg)
│   │   ├── exporter.rs      # PNG → ICO/WebP, multi-size export
│   │   ├── optimizer.rs     # SVG clean & optimize (7 rules)
│   │   ├── codegen.rs       # Code export (React/Vue/SwiftUI/Flutter)
│   │   ├── tokens.rs        # Design token extraction
│   │   ├── fontgen.rs       # Icon font generation (TTF/WOFF)
│   │   ├── pack_importer.rs # Icon pack import + index
│   │   ├── lottie.rs        # Lottie JSON animation export
│   │   ├── adaptive.rs      # Android adaptive icon layers
│   │   ├── brand.rs         # Brand kit management
│   │   ├── iconset.rs       # Icon set management
│   │   └── ...
│   ├── mcp/                 # 113 MCP tools across 18 domains
│   ├── commands/            # 120 Tauri IPC commands
│   ├── cli/                 # CLI subcommands
│   └── services/            # WebSocket sync, layout, project services
├── src/
│   ├── components/          # UI components by feature area
│   ├── composables/         # useKeyboard, useDragReorder, usePathEditor
│   ├── stores/              # Pinia stores (project, elements, canvas, ui, etc.)
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

- **`.iconproject.json`** — Project file with full canvas state, elements, export config, pages, symbols. All new fields use `#[serde(default)]` for backward compatibility.
- **`.iconstudio-template.json`** — User-saved templates, same format as project files.

## License

MIT
