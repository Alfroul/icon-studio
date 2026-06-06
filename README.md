# IconStudio

[![CI](https://github.com/Alfroul/icon-studio/actions/workflows/ci.yml/badge.svg)](https://github.com/Alfroul/icon-studio/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)](https://github.com/Alfroul/icon-studio)
[![Tauri 2](https://img.shields.io/badge/Tauri-2.0-orange)](https://v2.tauri.app)
[![Vue 3](https://img.shields.io/badge/Vue-3.5-brightgreen)](https://vuejs.org)

> A parametric icon/logo design tool for developers — GUI + MCP Server + CLI in one binary.
> Built with Tauri 2 + Vue 3 + Rust.

**One tool to design, preview, and export icons for any platform** — from a simple circle to a full Android/iOS app icon pack, driven by a visual canvas or natural language via MCP.

<!-- Uncomment after adding screenshots:
## Screenshots

![IconStudio Canvas](docs/screenshots/canvas.png)
*Real-time SVG canvas with shape elements, gradient editor, and layer management*

![Export Panel](docs/screenshots/export.png)
*Multi-format export: SVG, PNG, ICO, WebP, and platform-specific icon packs*

![MCP Integration](docs/screenshots/mcp.png)
*Design icons through conversation using the built-in MCP Server*
-->

## Why IconStudio?

Existing icon tools are either too simple (online generators) or too complex (Figma/Sketch). IconStudio fills the gap:

- **Developer-first** — JSON project files, CLI batch export, MCP integration for AI-assisted design
- **Cross-platform desktop** — ~3MB binary (vs Electron's ~150MB), native performance via Tauri
- **Parametric design** — Every element is a data structure, not a pixel blob. Undo/redo, template reuse, batch variations
- **Full export pipeline** — SVG, PNG, ICO, WebP, Android mipmap, iOS AppIcon, favicon packs in one click

## Technical Highlights

| Challenge | Solution |
|-----------|----------|
| SVG rendering at arbitrary resolution | Rust-side `usvg` + `resvg` pipeline, no browser dependency |
| Boolean operations on shapes | `geo` crate polygon operations with Bezier→polygon approximation |
| Real-time preview performance | Version-based `RenderCache` — skips rebuild when nothing changed |
| Undo/redo for complex mutations | Command pattern with batched operations and automatic rollback |
| AI-native design workflow | Embedded MCP Server with 59 tools (stdio transport via `rmcp`) |
| CI/CD multi-platform builds | GitHub Actions matrix: Windows x64, macOS ARM64/x64, Linux x64 |

## Features

### Canvas & Elements
- **14 shape types** — circle, rect, hexagon, star, shield, diamond, triangle, arrow, cross, heart, pentagon, octagon, wave, rounded-rect
- **Text** — system font enumeration, size/weight/letter-spacing controls
- **Lucide icons** — ~1700 built-in icons with keyword search
- **Image import** — PNG/JPG/SVG/WebP with automatic base64 encoding
- **Path editor** — SVG path input with visual Bezier editor (anchor dragging, handle adjustment, double-click to add node)
- **Layer groups** — group/ungroup, add/remove members
- **Templates** — 8 presets (app icon, letter logo, text logo) + user-defined

### Design
- **Boolean operations** — Union / Subtract / Intersect / Exclude
- **Clipping & masking** — SVG `clipPath` hard clip + `mask` soft mask
- **Blend modes** — 13 SVG `mix-blend-mode` options
- **Gradients** — Linear/radial with custom color stops and angles
- **Shadows** — Multiple shadows + inset, independent color/blur/offset
- **SVG filters** — Noise, blur, pixelate, emboss, posterize, turbulence
- **Symbol system** — Master component → instances → override propagation
- **Multi-page** — Multiple canvases per project (light/dark/multi-size)
- **Design analysis** — Color analysis, consistency check, palette suggestions

### Interaction
- **Drag-and-drop reordering** — HTML5 drag on layer list
- **Keyboard shortcuts** — Delete, Ctrl+D duplicate, arrow keys nudge, Ctrl+G group
- **Undo/redo** — Command pattern with batch operation rollback (Ctrl+Z / Ctrl+Y)
- **SVG import** — Drag `.svg` file to window or file dialog

### Animation (Motion Mode)
- **Multi-keyframe tracks** — Multiple animation tracks per element
- **Visual timeline** — AE/Figma-style timeline panel
- **60fps playback** — DOM-direct manipulation engine
- **Easing curves** — Cubic-Bezier + Spring physics
- **Presets** — pulse / spin / bounce / fade-in / slide / wiggle / breathe
- **Export** — SMIL SVG, CSS `@keyframes`, GIF/WebP

### Export
- **Multi-format** — SVG, multi-size PNG, ICO, WebP
- **App icon packs** — Android (all mipmap sizes) + iOS (all AppIcon sizes)
- **Batch variations** — Color/size/border-radius/opacity variants in one click
- **Favicon pack** — ICO + multi-size PNG bundle

### MCP Integration
Embedded MCP Server exposing 59 tools via stdio transport (`rmcp` crate). Default: 13 core tools, `load_extended_tools` expands to full set.

| Group | Count | Capability |
|-------|-------|-----------|
| core | 13 | Canvas creation, status, preview, basic elements, export |
| elements | 6 | Image, element find/copy/reorder, path |
| style | 7 | Shadow, gradient, border-radius, stroke dash |
| export | 6 | ICO/WebP/favicon, project save/open, variation generation |
| analysis | 5 | Color analysis, consistency, palette, font/icon list |
| canvas | 7 | SVG import, zoom, flip/align, random icon |
| group | 4 | Group/ungroup, add/remove member |
| animation | 3 | Set/clear/list animations |
| boolean | 4 | Boolean operations |
| filter | 3 | Set/clear/list filters |
| page | 5 | Page CRUD, switch |
| symbol | 6 | Symbol create/update/detach/override |

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
WebSocket multi-client sync with Last-Writer-Wins conflict resolution. Frontend monitors changes via `useWebSocketSync` composable.

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

## Architecture

See [docs/architecture.md](docs/architecture.md) for the full technical breakdown including:
- Layered architecture diagram
- Command pattern design for undo/redo
- RenderCache version-based invalidation
- MCP ToolRouter composition
- ADRs for key technology choices

## Quick Start

```bash
# Prerequisites: Node.js 18+, pnpm, Rust 1.77+

pnpm install
pnpm tauri dev
```

### MCP Configuration

Headless mode (no window, MCP Server only):

```bash
cargo run --manifest-path src-tauri/Cargo.toml -- --mcp
```

In `opencode.json`:

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
│   ├── main.rs              # Entry: GUI / MCP / CLI three-mode routing
│   ├── lib.rs               # Tauri app config, command registration
│   ├── model/               # Data model
│   │   ├── mod.rs            # IconProject, CommonProps, Element enum
│   │   ├── filter.rs         # SvgFilter, FilterType
│   │   ├── page.rs           # Page multi-page
│   │   ├── symbol.rs         # SymbolDef, SymbolOverride, SymbolInstance
│   │   ├── group.rs          # GroupElement
│   │   ├── history.rs        # CommandHistory, Command trait, BatchCommand
│   │   ├── helpers.rs        # Element find/filter/path utilities
│   │   └── shapes.rs         # 14 shape SVG path algorithms
│   ├── engine/               # SVG engine
│   │   ├── builder.rs        # IconProject → SVG String (with cache)
│   │   ├── renderer.rs       # SVG → PNG (usvg + resvg)
│   │   ├── exporter.rs       # PNG → ICO/WebP, multi-size export
│   │   ├── filter.rs         # SVG filter generation (6 types)
│   │   ├── variation.rs      # Batch variation engine
│   │   ├── boolean.rs        # Boolean operations (geo polygon)
│   │   ├── importer.rs       # SVG file → Element list
│   │   ├── generator.rs      # Random icon generation
│   │   ├── analyzer.rs       # Color analysis, consistency check
│   │   ├── text_measure.rs   # usvg precise text measurement
│   │   └── utils.rs          # Path validation, XML escaping
│   ├── mcp/                  # MCP tool definitions (59 tools)
│   ├── commands/             # Tauri IPC commands (~40)
│   ├── cli/                  # CLI subcommands (export/analyze/batch/variations)
│   └── services/             # WebSocket sync, layout, project services
├── src/
│   ├── App.vue               # Root component (layout, theme, file drop)
│   ├── main.ts               # Vue app entry
│   ├── components/           # UI components by feature area
│   ├── composables/          # useKeyboard, useDragReorder, usePathEditor, etc.
│   ├── stores/               # Pinia stores (project, elements, canvas, ui, etc.)
│   └── types/                # TypeScript type definitions
└── README.md
```

## Testing

```bash
# Rust backend tests (240+ tests)
cd src-tauri && cargo test

# Frontend tests
pnpm test

# Type check
pnpm build

# Lint
pnpm lint

# Clippy
cd src-tauri && cargo clippy --lib

# Benchmarks
cd src-tauri && cargo bench
```

## Design System

**Refined Dark** theme with Amber (#FBBF24) accent, dark background layers (Obsidian #09090B → Carbon #111113 → Zinc gradients), glassmorphism panels, JetBrains Mono for numeric display. Light theme toggle supported.

## File Formats

- **`.iconproject.json`** — Project file (JSON) with full canvas state, elements, export config, pages, symbols. All new fields use `#[serde(default)]` for backward compatibility.
- **`.iconstudio-template.json`** — User-saved templates, same format as project files.

## Performance

Run benchmarks locally: `cd src-tauri && cargo bench`

| Benchmark | Description |
|-----------|-------------|
| `bench_build_small` | SVG build with 1 element |
| `bench_build_medium` | SVG build with 10 elements (gradients, shadows, text, icons) |
| `bench_render_512` | SVG → PNG at 512px |
| `bench_render_1024` | SVG → PNG at 1024px |
| `undo_redo/50_cycles` | 50 add → 50 undo → 50 redo cycles |
| `cache/cache_hit` | RenderCache with unchanged version (zero-cost path) |
| `cache/cache_miss` | RenderCache with version bump (full rebuild) |
| `text_measure/english` | Text width measurement (Latin) |
| `text_measure/chinese` | Text width measurement (CJK) |

## License

MIT
