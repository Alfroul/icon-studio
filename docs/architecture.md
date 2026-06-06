# IconStudio Architecture

## Overview

IconStudio is a parametric icon/logo design tool built as a native desktop application. It uses a **three-tier architecture** with a Rust backend engine, a Tauri IPC bridge, and a Vue 3 frontend — all compiled into a single ~3MB binary that also doubles as a CLI tool and MCP server.

```
┌─────────────────────────────────────────────────────────┐
│  Frontend (Vue 3 + TypeScript + Pinia)                  │
│  Components · Composables · Stores                      │
└──────────────────────────┬──────────────────────────────┘
                           │ Tauri IPC (invoke / events)
┌──────────────────────────┴──────────────────────────────┐
│  Tauri Bridge Layer                                      │
│  60+ commands · Managed State (Arc<Mutex<>>) · Events   │
└──────────────────────────┬──────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────┐
│  Rust Engine                                             │
│  Data Model · Command History · SVG Builder              │
│  Renderer · Boolean Ops · MCP Handler · CLI              │
└─────────────────────────────────────────────────────────┘
```

## Three-Mode Entry Point

The single binary supports three execution modes, routed in `main.rs`:

```
icon-studio.exe              → GUI mode (Tauri window)
icon-studio.exe --mcp        → MCP Server (stdio transport)
icon-studio.exe export ...   → CLI mode (batch export/analyze)
```

This is achieved by checking CLI args before initializing Tauri. The MCP and CLI modes share the same Rust engine but skip the WebView entirely — zero GUI overhead.

## Key Design Patterns

### 1. Command Pattern (Undo/Redo)

**File:** `src-tauri/src/model/history.rs`

Every mutation to the project goes through the Command pattern:

```rust
trait Command {
    fn execute(&mut self, project: &mut IconProject) -> Result<()>;
    fn undo(&mut self, project: &mut IconProject) -> Result<()>;
}
```

Key properties:
- **Batched operations** — `UndoBatch` groups multiple commands into an atomic unit. If any sub-command fails, the entire batch rolls back automatically.
- **Snapshot optimization** — `SnapshotCommand` stores a full project snapshot every N commands (configurable, default 30). This avoids deep recursion in long undo chains.
- **History cap** — Maximum 200 commands to bound memory usage.

Concrete commands: `AddElementCommand`, `RemoveElementCommand`, `SetPropsCommand`, `SetGradientCommand`, `SetShadowCommand`, `ReorderCommand`, `CanvasCommand`, etc.

### 2. RenderCache (Version-Based Invalidation)

**File:** `src-tauri/src/engine/builder.rs`

```rust
struct RenderCache {
    cached_svg: Option<String>,
    cached_version: u64,
}
```

The SVG builder is the most expensive operation. The cache strategy is:
1. Every mutation bumps `project.version`
2. `RenderCache::build()` compares its `cached_version` against `project.version`
3. On match → return cached SVG string (zero-cost)
4. On mismatch → rebuild SVG, cache result, update version

This makes the preview loop (triggered on every keystroke) effectively free when nothing actually changed.

### 3. MCP ToolRouter Composition

**File:** `src-tauri/src/mcp/mod.rs`

```rust
struct IconStudioHandler {
    core_router: ToolRouter<Self>,    // 6 core tools
    full_router: ToolRouter<Self>,     // 59 total tools
    full_mode: AtomicBool,
}
```

Tools are organized by domain (core, elements, style, export, analysis, canvas, group, animation, boolean, filter, page, symbol). The handler starts in core mode and expands to full mode when the client calls `load_extended_tools`.

Each tool is a plain async function annotated with `#[tool]` macros from the `rmcp` crate, providing automatic schema generation for LLM consumption.

### 4. Element Enum (Unified Data Model)

**File:** `src-tauri/src/model/mod.rs`

```rust
enum Element {
    Shape(ShapeElement),
    Text(TextElement),
    Icon(IconElement),
    Image(ImageElement),
    Path(PathElement),
    Group(GroupElement),
}
```

All element types share `CommonProps` (position, size, opacity, rotation, shadows, filters, blend mode, clipping, masking, locking, visibility) and extend with type-specific fields. This makes the SVG builder a single `match` statement that handles all cases uniformly.

## Architecture Decision Records

### ADR-1: Tauri over Electron

**Decision:** Use Tauri 2 as the desktop framework.

**Context:** We needed a cross-platform desktop app with native performance. Options were Electron, Tauri, and Qt.

**Rationale:**
- **Bundle size:** Tauri produces ~3MB binaries vs Electron's ~150MB. For a developer tool, download size matters.
- **Memory:** Tauri uses the system WebView, adding ~10MB RAM vs Electron's bundled Chromium at ~100MB+.
- **Backend:** Rust gives us zero-cost abstractions for the SVG engine, boolean operations, and rendering — no GC pauses during real-time preview.
- **Tradeoff:** Platform-specific build complexity (WebView2 on Windows, WebKit on macOS/Linux) and a smaller ecosystem vs Electron. Acceptable for a focused tool.

### ADR-2: Rust-Side SVG Rendering

**Decision:** Use `usvg` + `resvg` for SVG parsing and PNG rendering, not the browser.

**Context:** The frontend displays SVG in a WebView, but export (PNG, ICO, WebP) needs pixel-perfect rendering without a browser.

**Rationale:**
- **Deterministic output:** `resvg` produces identical pixels regardless of platform or WebView version. Browser SVG rendering varies.
- **Headless CLI/MCP:** The CLI and MCP modes have no browser. All rendering must work without a GUI.
- **Performance:** `resvg` is a purpose-built SVG renderer — faster and lighter than headless Chromium.
- **Tradeoff:** Limited to SVG 1.1 features (no CSS animations, no `<foreignObject>`). Acceptable since the tool generates SVG, not consumes arbitrary SVG.

### ADR-3: geo Crate for Boolean Operations

**Decision:** Use the `geo` crate for boolean operations on shapes.

**Context:** Icon design requires combining simple shapes into complex outlines (union, subtract, intersect, exclude).

**Rationale:**
- **Maturity:** `geo` is a well-tested computational geometry library with proper polygon clipping algorithms.
- **Approach:** SVG shapes → polygon approximation (Bezier curves → 16-segment polygon) → `geo::BooleanOps` → polygon → SVG path.
- **Tradeoff:** Bezier curves are approximated, not exact. At 16 segments per curve the visual difference is imperceptible at icon sizes (16–1024px).

### ADR-4: Command Pattern for State Management

**Decision:** Implement undo/redo via the Command pattern on the Rust side.

**Context:** The frontend needs undo/redo that works across all operations — element mutations, canvas changes, style updates.

**Rationale:**
- **Atomicity:** Each command encapsulates both `execute()` and `undo()`, making it impossible to have inconsistent state.
- **Batching:** Multi-step operations (e.g., "paste and position") can be grouped into a single undo step.
- **Snapshot fallback:** For complex mutations where undo logic is error-prone, `SnapshotCommand` stores a full project copy.
- **Tradeoff:** More boilerplate than event-sourcing or CRDT approaches. Simpler to reason about for a single-user desktop tool.

### ADR-5: Pinia Stores on Frontend

**Decision:** Use Pinia stores with Tauri IPC as the single source of truth.

**Context:** The frontend manages UI state (selection, panels, theme) while the Rust backend owns the project data.

**Rationale:**
- **Clear ownership:** Rust owns the data model and SVG engine. Frontend owns the UI state and user interactions.
- **IPC boundary:** Every data mutation flows through `invoke()` → Rust command → `project.version++`. The frontend then fetches the updated preview.
- **No sync issues:** Single-directional data flow prevents stale state.

## Data Flow

### User Adds a Shape

```
User clicks "Add Circle"
  → Vue component calls store.addShape()
    → Store calls invoke('add_shape', { type: 'Circle', ... })
      → Rust AddElementCommand::execute()
        → project.elements.push(shape)
        → project.version++
      → Command stored in history
    → Store calls invoke('fetch_preview')
      → RenderCache::build() detects version change
      → builder::build() generates SVG string
    → Store updates svgPreview
  → Canvas component re-renders
```

### User Exports PNG

```
User clicks "Export PNG"
  → ExportPanel calls invoke('export_png', { sizes: [512], outputDir })
    → Rust engine fetches SVG from RenderCache
    → renderer::render(svg, 512) → usvg parsing → resvg rasterization
    → PNG bytes written to file
    → Returns file path to frontend
  → Toast shows success
```

### MCP Client Creates an Icon

```
LLM calls icon_new(512, 512)
  → MCP handler creates IconProject
LLM calls add_shape("circle", ...)
  → MCP handler creates AddElementCommand, executes on project
LLM calls export_png([256, 512])
  → Engine builds SVG, renders at each size, writes files
LLM receives file paths
```

## Thread Safety

The Rust side uses `Arc<Mutex<T>>` for shared state:
- `ProjectState` = `Arc<Mutex<IconProject>>`
- `HistoryState` = `Arc<Mutex<CommandHistory>>`
- `RenderCacheState` = `Arc<Mutex<RenderCache>>`

Tauri's `manage()` registers these as managed state, injected into every command handler. The Mutex is held only during the command execution (typically microseconds), not across the IPC boundary.

For WebSocket sync, `tokio` async tasks lock the project briefly to compute diffs, then release before sending over the network.

## Testing Strategy

| Layer | Tool | Coverage |
|-------|------|---------|
| Rust unit tests | `cargo test` | 240+ tests across model, engine, commands, MCP |
| Rust benchmarks | `cargo bench` (Criterion) | SVG build, render, cache hit/miss, undo/redo, text measure |
| Frontend unit tests | Vitest + happy-dom | Store actions, composable logic, component rendering |
| Frontend type check | `vue-tsc --noEmit` | Full TypeScript strict mode |
| Lint | ESLint + Clippy | Both frontend and backend |

## Build & Release

- **CI** (`.github/workflows/ci.yml`): Windows runner, runs `cargo test`, `pnpm build`, `cargo bench --quick`
- **Release** (`.github/workflows/release.yml`): 4-platform matrix (Windows x64, macOS ARM64, macOS x64, Linux x64), Tauri Action handles bundling and GitHub Release upload
