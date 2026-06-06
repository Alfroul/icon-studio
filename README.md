# IconStudio

> Tauri 2 + Vue 3 驱动的参数化图标/Logo 设计器 — GUI + MCP + CLI 三栖工具

## 是什么

IconStudio 是一款面向开发者的图标生产工具。通过侧边栏面板添加形状、调整颜色、排列元素，画布实时预览，一键导出 SVG / PNG / ICO / WebP / App 图标包。同时内嵌 MCP Server，支持通过对话驱动图标设计，也可通过 CLI 在 CI/CD 中批量导出。

## 核心能力

### 画布与元素

- **14 种形状** — circle、rect、rounded-rect、hexagon、star、shield、diamond、triangle、arrow、cross、heart、pentagon、octagon、wave
- **文字元素** — 系统字体枚举、字号/字重/字间距调节
- **Lucide 图标库** — ~1700 个内置图标，关键字搜索
- **图片元素** — 导入 PNG/JPG/SVG/WebP，自动 base64 编码
- **路径元素** — SVG path 数据输入，可视化贝塞尔编辑器（锚点拖拽、手柄调节、双击加节点、Delete 删节点）
- **图层组** — 编组、解组、组内添加/移除
- **模板系统** — 8 个预置模板（App 图标、字母 Logo、文字 Logo 等）+ 用户自定义模板

### 设计功能

- **布尔运算** — Union / Subtract / Intersect / Exclude，从简单形状组合复杂轮廓
- **蒙版与裁剪** — SVG clipPath 硬裁剪 + mask 柔蒙版
- **混合模式** — 13 种 SVG mix-blend-mode（multiply、screen、overlay 等）
- **渐变** — 线性/径向渐变，自定义色标和角度
- **阴影** — 多重阴影 + 内阴影（inset），独立控制颜色/模糊/偏移
- **SVG 滤镜** — 噪点、模糊、像素化、浮雕、色调分离、湍流，6 种效果
- **元素锁定 & 可见性** — 锁定防误操作，隐藏保留数据
- **符号/组件系统** — 定义主组件 → 创建实例 → Override 覆盖属性 → 修改主组件自动同步
- **多页面** — 项目内管理多个画布（亮色/暗色/多尺寸），独立元素和画布设置
- **设计分析** — 色彩分析、一致性检查、配色推荐

### 交互

- **拖拽重排** — 图层列表 HTML5 拖拽排序
- **键盘快捷键** — Delete 删除、Ctrl+D 复制、方向键微移、Ctrl+G 编组等
- **撤销/重做** — Command 模式，支持批量操作回退（Ctrl+Z / Ctrl+Y）
- **SVG 导入** — 拖拽 .svg 文件到窗口或通过文件对话框导入
- **文件拖拽** — 拖拽 .iconproject.json 或 .svg 直接打开

### 动画（Motion Mode）

- **多关键帧轨道** — 每个元素可挂载多条动画轨道
- **可视化时间轴** — 类 AE/Figma 的 Timeline 面板
- **实时回放** — 60fps DOM 直操式播放引擎
- **缓动曲线编辑** — Cubic-Bezier + Spring 弹簧物理
- **预设动画库** — pulse / spin / bounce / fade-in / slide / wiggle / breathe
- **多格式导出** — SMIL SVG、CSS @keyframes、GIF/WebP

### 导出

- **多格式** — SVG、多尺寸 PNG、ICO、WebP
- **App 图标打包** — Android（全尺寸 mipmap）+ iOS（全尺寸 AppIcon）
- **批量变异生成** — 一键生成色彩/尺寸/圆角/透明度变体
- **Favicon 包** — ico + 多尺寸 PNG 打包

### MCP 集成

内嵌 MCP Server，暴露 59 个工具，通过 `rmcp` crate 实现 stdio transport。默认加载 13 个核心工具，调用 `load_extended_tools` 扩展至全部。

| 分组 | 数量 | 能力 |
|---|---|---|
| core | 13 | 画布创建、状态查询、预览、基础元素、导出 |
| elements | 6 | 图片、元素查找/复制/重排、路径 |
| style | 7 | 投影、渐变、圆角、描边虚线 |
| export | 6 | ICO/WebP/Favicon、项目保存/打开、变异生成 |
| analysis | 5 | 色彩分析、一致性检查、配色推荐、字体/图标列表 |
| canvas | 7 | SVG 导入、画布缩放、翻转/对齐、随机图标 |
| group | 4 | 编组、解组、添加/移除组成员 |
| animation | 3 | 设置/清除/列出动画 |
| boolean | 4 | 布尔运算 |
| filter | 3 | 设置/清除/列出滤镜 |
| page | 5 | 页面增删改查、切换 |
| symbol | 6 | 符号创建/更新/脱离/覆盖 |

### CLI

```bash
# 单文件导出
iconstudio export --input app.iconproject.json --format png --sizes 16,32,64,128 --output ./export/

# 色彩分析与一致性检查
iconstudio analyze --input ./icons/

# 批量并行导出
iconstudio batch --input ./icons/ --output ./export/ --format png

# 批量变异生成
iconstudio variations --input app.json --config variations.json --output ./variants/
```

### 实时协作

WebSocket 多客户端同步，Last-Writer-Wins 冲突解决。前端通过 `useWebSocketSync` composable 监听变更。

## 技术栈

| 层 | 方案 |
|---|---|
| 桌面框架 | Tauri 2 |
| 前端 | Vue 3 + TypeScript + Pinia |
| 构建 | Vite |
| SVG 渲染 | resvg / usvg (Rust) |
| MCP Server | rmcp crate (Rust, stdio transport) |
| 字体 | fontdb crate |
| 图标库 | Lucide Icons SVG paths |
| 布尔运算 | geo crate |
| 命令行 | clap crate |
| 并行 | rayon crate |

## 快速开始

```bash
# 前置要求：Node.js 18+, pnpm, Rust 1.77+

pnpm install
pnpm tauri dev
```

## MCP 配置

Headless 模式启动（无窗口，仅 MCP Server）：

```bash
cargo run --manifest-path src-tauri/Cargo.toml -- --mcp
```

在 `opencode.json` 中配置：

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

## 项目结构

```
IconStudio/
├── src-tauri/src/
│   ├── main.rs              # 入口：GUI / MCP / CLI 三模式路由
│   ├── lib.rs               # Tauri 应用配置、命令注册
│   ├── model/               # 数据模型
│   │   ├── mod.rs            # IconProject, CommonProps, Element 枚举
│   │   ├── filter.rs         # SvgFilter, FilterType
│   │   ├── page.rs           # Page 多页面
│   │   ├── symbol.rs         # SymbolDef, SymbolOverride, SymbolInstance
│   │   ├── group.rs          # GroupElement
│   │   ├── history.rs        # CommandHistory, Command trait, BatchCommand
│   │   ├── helpers.rs        # 元素查找/过滤/路径工具
│   │   └── shapes.rs         # 14 种形状 SVG 路径算法
│   ├── engine/               # SVG 引擎
│   │   ├── builder.rs        # IconProject → SVG String（含缓存）
│   │   ├── renderer.rs       # SVG → PNG (usvg + resvg)
│   │   ├── exporter.rs       # PNG → ICO/WebP, 多尺寸导出
│   │   ├── filter.rs         # SVG 滤镜生成（6 种）
│   │   ├── variation.rs      # 批量变异引擎
│   │   ├── boolean.rs        # 布尔运算（geo 多边形）
│   │   ├── importer.rs       # SVG 文件 → Element 列表
│   │   ├── generator.rs      # 随机图标生成
│   │   ├── analyzer.rs       # 色彩分析、一致性检查
│   │   ├── text_measure.rs   # usvg 精确文字测量
│   │   └── utils.rs          # 路径校验、XML 转义
│   ├── mcp/                  # MCP 工具定义（59 个）
│   ├── commands/             # Tauri IPC 命令（~40 个）
│   ├── cli/                  # CLI 子命令（export/analyze/batch/variations）
│   └── services/             # WebSocket 同步、布局、项目服务
├── src/
│   ├── App.vue               # 根组件（布局、主题、文件拖拽）
│   ├── main.ts               # Vue 应用入口
│   ├── components/
│   │   ├── layout/           # ActivityBar, Sidebar, MainArea, StatusBar
│   │   ├── canvas/           # SvgPreview, ElementOverlay, PathEditorOverlay
│   │   ├── elements/         # ElementsPanel, AddShapeBar, IconBrowser
│   │   ├── properties/       # PropertiesPanel, FontPicker│   │   ├── style/            # StylePanel, FilterPanel, ShadowEditor, GradientEditor
│   │   ├── pages/            # PagesPanel
│   │   ├── symbols/          # SymbolsPanel
│   │   ├── analysis/         # AnalysisPanel
│   │   ├── library/          # LibraryPanel
│   │   ├── templates/        # TemplatesPanel
│   │   ├── export/           # ExportPanel
│   │   ├── settings/         # SettingsPanel
│   │   └── common/           # AppIcon, Toast, ToggleSwitch
│   ├── composables/          # useKeyboard, useDragReorder, usePathEditor, useProjectSync, useWebSocketSync
│   ├── stores/               # Pinia stores（project, elements, canvas, ui, pages, settings, export）
│   └── types/                # TypeScript 类型定义
└── README.md
```

## 测试

```bash
# Rust 后端测试
cd src-tauri && cargo test

# 前端测试
pnpm test

# 类型检查
pnpm build

# Lint
pnpm lint

# Clippy
cd src-tauri && cargo clippy --lib
```

## 设计系统

应用采用 **Refined Dark** 主题，以 Amber (#FBBF24) 为强调色，深色背景层次系统（Obsidian #09090B → Carbon #111113 → Zinc 渐变），毛玻璃面板，JetBrains Mono 等宽字体用于数值显示。支持 Light 主题切换。

## 文件格式

- **`.iconproject.json`** — 项目文件，JSON 格式，包含完整画布状态、元素列表、导出配置、页面和符号数据。所有新字段通过 `#[serde(default)]` 保证向后兼容。
- **`.iconstudio-template.json`** — 用户保存的模板文件，格式与项目文件相同。

## License

MIT
