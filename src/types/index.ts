// ---- Phase 3 types ----

export type IssueSeverity = "info" | "warning" | "error";

export type FillStyle = "outline" | "filled" | "duotone" | "none";

export type WeightPreset = "thin" | "light" | "regular" | "medium" | "bold" | "fill";

export type IconStyleKind = "strokeBased" | "fillBased" | "mixed";

export interface SpriteSheetIcon {
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface SpriteSheetResult {
  image_path: string;
  icons: SpriteSheetIcon[];
  total_width: number;
  total_height: number;
}

export interface AllPlatformsResult {
  ios_paths: string[];
  android_paths: string[];
  pwa_paths: string[];
  favicon_paths: string[];
}

export interface FaviconPackageResult {
  paths: string[];
  html_snippet: string;
}

// ---- Phase 2 types ----

export type OverlayKind =
  | "add"
  | "remove"
  | "check"
  | "info"
  | "warning"
  | "error"
  | "star"
  | "lock"
  | "new"
  | "custom";

export type OverlayPosition =
  | "topLeft"
  | "topRight"
  | "bottomLeft"
  | "bottomRight";

export interface Overlay {
  kind: OverlayKind;
  position: OverlayPosition;
  color?: string | null;
  size_ratio?: number | null;
  offset_x?: number | null;
  offset_y?: number | null;
  custom_path?: string | null;
}

export type ThemeRule =
  | "invertColors"
  | { replaceColor: { from: string; to: string } }
  | { adjustOpacity: { factor: number } }
  | "grayscale"
  | { desaturate: { factor: number } }
  | { customFill: { color: string } };

export interface ThemeVariant {
  name: string;
  base_page_index?: number;
  rules: ThemeRule[];
}

export type PresetShape =
  | "squircle"
  | "circle"
  | "roundedRect"
  | "square"
  | "hexagon"
  | "shield";

export interface ThemePreset {
  id: string;
  name: string;
  corner_radius?: number;
  padding_ratio?: number;
  background?: string | null;
  shadow?: Shadow | null;
  shape: PresetShape;
  preview_svg?: string | null;
}

export type AiProvider = "openAi" | "recraft" | "custom" | "ollama";

export type AiTask =
  | "textToIcon"
  | "sketchToIcon"
  | "styleTransfer"
  | "varyIcon"
  | "removeBackground";

export type IconStyle =
  | "flat"
  | "outline"
  | "duotone"
  | "gradient"
  | "threeD"
  | "minimal"
  | "cartoon"
  | "pixelArt"
  | "lineArt"
  | "neon";

// ---- Phase 1 types ----

export interface Canvas {
  width: number;
  height: number;
  background: string;
  corner_radius: number;
  background_gradient?: Gradient | null;
}

/** Shape of canvas data returned from Tauri backend commands */
export interface CanvasResult {
  width: number;
  height: number;
  background: string;
  corner_radius: number;
  background_gradient?: Gradient | null;
}

export interface ExportConfig {
  formats: string[];
  sizes: number[];
}

export type ShapeType =
  | "circle"
  | "rect"
  | "rounded-rect"
  | "hexagon"
  | "star"
  | "shield"
  | "diamond"
  | "triangle"
  | "arrow-right"
  | "cross"
  | "heart"
  | "pentagon"
  | "octagon"
  | "wave"
  | "custom";

export interface Gradient {
  type: "linear" | "radial";
  colors: string[];
  angle: number;
  stops?: number[];
}

export interface Shadow {
  color: string;
  blur: number;
  offset_x: number;
  offset_y: number;
  inset: boolean;
}

export type FilterType = "noise" | "blur" | "pixelate" | "emboss" | "posterize" | "turbulence";

export interface SvgFilter {
  filter_type: FilterType;
  params: Record<string, number>;
}

export type AnimationType = "rotate" | "scale" | "fade" | "translate" | "path";

export interface Animation {
  animation_type: AnimationType;
  duration: number;
  delay: number;
  repeat: boolean;
  easing: string;
  params?: Record<string, unknown> | null;
}

export interface ShapeElement {
  id: string;
  type: "shape";
  shape_type: ShapeType;
  x: number;
  y: number;
  width: number;
  height: number;
  fill: string;
  stroke: string | null;
  stroke_width: number;
  opacity: number;
  rotation: number;
  border_radius: number;
  stroke_dasharray?: string | null;
  gradient: Gradient | null;
  shadows: Shadow[];
  animation: Animation | null;
  blend_mode?: string | null;
  clip_element_id?: string | null;
  mask_element_id?: string | null;
  boolean_source?: BooleanSource | null;
  locked?: boolean;
  visible?: boolean;
  svg_filter?: SvgFilter | null;
  overlay?: Overlay | null;
}

export interface PathNode {
  command: "M" | "L" | "C" | "Q" | "Z";
  anchor: { x: number; y: number };
  handleIn?: { x: number; y: number };
  handleOut?: { x: number; y: number };
}

export interface TextElement {
  id: string;
  type: "text";
  x: number;
  y: number;
  width: number;
  height: number;
  fill: string;
  stroke: string | null;
  stroke_width: number;
  opacity: number;
  rotation: number;
  content: string;
  font_family: string;
  font_size: number;
  font_weight: string;
  letter_spacing: number;
  gradient: Gradient | null;
  shadows: Shadow[];
  animation: Animation | null;
  blend_mode?: string | null;
  clip_element_id?: string | null;
  mask_element_id?: string | null;
  locked?: boolean;
  visible?: boolean;
  svg_filter?: SvgFilter | null;
  overlay?: Overlay | null;
}

export interface IconElement {
  id: string;
  type: "icon";
  x: number;
  y: number;
  width: number;
  height: number;
  fill: string;
  stroke: string | null;
  stroke_width: number;
  opacity: number;
  rotation: number;
  name: string;
  gradient: Gradient | null;
  shadows: Shadow[];
  animation: Animation | null;
  blend_mode?: string | null;
  clip_element_id?: string | null;
  mask_element_id?: string | null;
  locked?: boolean;
  visible?: boolean;
  svg_filter?: SvgFilter | null;
  overlay?: Overlay | null;
}

export interface ImageElement {
  id: string;
  type: "image";
  x: number;
  y: number;
  width: number;
  height: number;
  fill: string;
  stroke: string | null;
  stroke_width: number;
  opacity: number;
  rotation: number;
  data: string;
  gradient: Gradient | null;
  shadows: Shadow[];
  animation: Animation | null;
  blend_mode?: string | null;
  clip_element_id?: string | null;
  mask_element_id?: string | null;
  locked?: boolean;
  visible?: boolean;
  svg_filter?: SvgFilter | null;
  overlay?: Overlay | null;
}

export interface PathElement {
  id: string;
  type: "path";
  x: number;
  y: number;
  width: number;
  height: number;
  fill: string;
  stroke: string;
  stroke_width: number;
  stroke_dasharray?: string | null;
  opacity: number;
  rotation: number;
  d: string;
  natural_width: number;
  natural_height: number;
  gradient: Gradient | null;
  shadows: Shadow[];
  animation: Animation | null;
  blend_mode?: string | null;
  clip_element_id?: string | null;
  mask_element_id?: string | null;
  boolean_source?: BooleanSource | null;
  locked?: boolean;
  visible?: boolean;
  svg_filter?: SvgFilter | null;
  overlay?: Overlay | null;
}

export interface GroupElement {
  id: string;
  type: "group";
  x: number;
  y: number;
  width: number;
  height: number;
  opacity: number;
  rotation: number;
  children: Element[];
  expanded: boolean;
  shadows: Shadow[];
  animation: Animation | null;
  blend_mode?: string | null;
  clip_element_id?: string | null;
  mask_element_id?: string | null;
  locked?: boolean;
  visible?: boolean;
  svg_filter?: SvgFilter | null;
  overlay?: Overlay | null;
}

export interface SymbolOverride {
  property: string;
  value: unknown;
}

export interface SymbolElement {
  id: string;
  type: "symbol";
  x: number;
  y: number;
  width: number;
  height: number;
  opacity: number;
  rotation: number;
  symbol_id: string;
  overrides: SymbolOverride[];
  shadows: Shadow[];
  animation: Animation | null;
  blend_mode?: string | null;
  locked?: boolean;
  visible?: boolean;
  svg_filter?: SvgFilter | null;
  overlay?: Overlay | null;
}

export interface SymbolDef {
  id: string;
  name: string;
  instance_count: number;
  source_type: string;
}

export interface BooleanSource {
  element_a: Record<string, unknown>;
  element_b: Record<string, unknown>;
  operation: "union" | "subtract" | "intersect" | "exclude";
}

export type Element = ShapeElement | TextElement | IconElement | ImageElement | PathElement | GroupElement | SymbolElement;

export interface Page {
  id: string;
  name: string;
  canvas: Canvas;
  elements: Element[];
}

export interface PageInfo {
  id: string;
  name: string;
  width: number;
  height: number;
  element_count: number;
  active: boolean;
}

export interface IconProject {
  schema_version: string;
  canvas: Canvas;
  elements: Element[];
  exports: ExportConfig;
  templates: Record<string, IconProject>;
  pages?: Page[];
  active_page_index?: number;
  theme_variants?: ThemeVariant[];
}

export interface ColorInfo {
  hex: string;
  usage_count: number;
  element_ids: string[];
}

export interface ColorAnalysis {
  all_colors: ColorInfo[];
  primary: ColorInfo | null;
  secondary: ColorInfo[];
  accent: ColorInfo[];
}

export interface ConsistencyIssue {
  property: string;
  expected: string;
  actual: string;
  element_id: string;
  severity?: IssueSeverity;
}

export interface ConsistencyReport {
  border_radius_consistent: boolean;
  stroke_width_consistent: boolean;
  font_size_consistent: boolean;
  opacity_consistent: boolean;
  stroke_weight_consistent?: boolean;
  fill_style_consistent?: boolean;
  proportions_consistent?: boolean;
  visual_center_drift?: number | null;
  issues: ConsistencyIssue[];
}

export interface FontInfo {
  name: string;
  family: string;
  style: string;
  weight: number;
}

export interface FindResult {
  matching_ids: string[];
  count: number;
}
