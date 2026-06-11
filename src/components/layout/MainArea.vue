<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";
import SvgPreview from "@/components/canvas/SvgPreview.vue";
import ElementOverlay from "@/components/canvas/ElementOverlay.vue";
import PathEditorOverlay from "@/components/canvas/PathEditorOverlay.vue";
import PathToolbar from "@/components/canvas/PathToolbar.vue";
import QuickStartOverlay from "@/components/quickstart/QuickStartOverlay.vue";
import AppIcon from "@/components/common/AppIcon.vue";
import { serializePath } from "@/composables/usePathEditor";

const project = useProjectStore();
const ui = useUiStore();

const mainAreaEl = ref<HTMLElement | null>(null);
const quickstartRef = ref<InstanceType<typeof QuickStartOverlay> | null>(null);
const viewportW = ref(600);
const viewportH = ref(600);
const isPanning = ref(false);
const panStart = ref({ x: 0, y: 0, panX: 0, panY: 0 });
const spaceHeld = ref(false);
const fileDragOver = ref(false);

const PADDING = 80;

const fitScale = computed(() => {
  const availW = viewportW.value - PADDING;
  const availH = viewportH.value - PADDING;
  if (availW <= 0 || availH <= 0) return 1;
  const sx = availW / project.canvasWidth;
  const sy = availH / project.canvasHeight;
  return Math.min(sx, sy);
});

const effectiveScale = computed(() => fitScale.value * (ui.zoom / 100));

const displayZoom = computed(() => Math.round(effectiveScale.value * 100));

watch(displayZoom, (val) => { ui.displayZoom = val; }, { immediate: true });

function updateViewportSize() {
  if (mainAreaEl.value) {
    const rect = mainAreaEl.value.getBoundingClientRect();
    viewportW.value = rect.width;
    viewportH.value = rect.height;
  }
}

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  updateViewportSize();
  if (mainAreaEl.value) {
    resizeObserver = new ResizeObserver(() => updateViewportSize());
    resizeObserver.observe(mainAreaEl.value);
  }
});

onUnmounted(() => {
  resizeObserver?.disconnect();
});

watch(() => [project.canvasWidth, project.canvasHeight], () => {
  ui.setPan(0, 0);
});

function onKeyDown(e: KeyboardEvent) {
  if (e.code === "Space" && !e.repeat) {
    e.preventDefault();
    spaceHeld.value = true;
  }

  if (e.code === "Escape") {
    if (ui.pathEditing) {
      const newD = serializePath(ui.pathEditing.nodes);
      project.updateElement(ui.pathEditing.elementId, { d: newD });
      ui.exitPathEdit();
      return;
    }
    ui.selectElement(null);
    return;
  }
}

function onKeyUp(e: KeyboardEvent) {
  if (e.code === "Space") {
    spaceHeld.value = false;
  }
}

function onWheel(e: WheelEvent) {
  if (e.ctrlKey || e.metaKey) {
    e.preventDefault();
    const delta = e.deltaY > 0 ? -10 : 10;
    ui.setZoom(ui.zoom + delta);
  }
}

function onDragOver(e: DragEvent) {
  if (e.dataTransfer?.types.includes("application/json")) {
    e.preventDefault();
    e.dataTransfer.dropEffect = "copy";
    return;
  }
  if (e.dataTransfer?.types.includes("Files")) {
    e.preventDefault();
    e.dataTransfer.dropEffect = "copy";
    fileDragOver.value = true;
  }
}

async function onDrop(e: DragEvent) {
  e.preventDefault();
  fileDragOver.value = false;
  try {
    // Handle file drops first
    if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
      const file = e.dataTransfer.files[0];
      const validTypes = ["image/png", "image/jpeg", "image/webp", "image/svg+xml"];
      if (!validTypes.includes(file.type) && !file.name.endsWith(".svg")) {
        ui.showToast("Unsupported file type. Use PNG, JPG, SVG, or WebP.", "error");
        return;
      }
      if (quickstartRef.value) {
        await quickstartRef.value.handleImageFile(file);
      } else {
        await handleDroppedFile(file);
      }
      return;
    }

    const data = e.dataTransfer?.getData("application/json");
    if (!data) return;
    const parsed = JSON.parse(data);

    if (parsed && typeof parsed === 'object' && parsed.type === "library-asset" && typeof parsed.name === "string") {
      const canvasContainer = (e.currentTarget as HTMLElement).querySelector(".canvas-container") as HTMLElement;
      if (!canvasContainer) return;
      const rect = canvasContainer.getBoundingClientRect();
      const dropX = e.clientX - rect.left;
      const dropY = e.clientY - rect.top;

      const scaleX = project.canvasWidth / rect.width;
      const scaleY = project.canvasHeight / rect.height;
      const targetSize = Math.round(project.canvasWidth * 0.3);

      await invoke("add_library_asset", {
        assetName: parsed.name,
        targetX: Math.round(dropX * scaleX) - targetSize / 2,
        targetY: Math.round(dropY * scaleY) - targetSize / 2,
        targetSize,
      });
      await project.refreshElements();
      ui.showToast("Asset added", "success");
    }
  } catch (err) {
    ui.showToast(`Drop failed: ${err}`, "error");
  }
}

async function handleDroppedFile(file: File) {
  try {
    if (file.type === "image/svg+xml" || file.name.endsWith(".svg")) {
      const text = await file.text();
      await invoke("add_path_from_svg", { svgContent: text });
    } else {
      const dataUrl = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result as string);
        reader.onerror = () => reject(new Error("Failed to read file"));
        reader.readAsDataURL(file);
      });
      const img = new Image();
      await new Promise<void>((resolve, reject) => {
        img.onload = () => resolve();
        img.onerror = () => reject(new Error("Failed to load image"));
        img.src = dataUrl;
      });
      await invoke("add_image_from_data", {
        imageData: dataUrl,
        width: img.naturalWidth,
        height: img.naturalHeight,
      });
    }
    await project.refreshElements();
    ui.showToast("Image imported", "success");
  } catch (e) {
    ui.showToast(`Import failed: ${e}`, "error");
  }
}

function onDragLeave() {
  fileDragOver.value = false;
}

async function onPaste(e: ClipboardEvent) {
  const items = e.clipboardData?.items;
  if (!items) return;
  for (const item of items) {
    if (item.type.startsWith("image/")) {
      e.preventDefault();
      const file = item.getAsFile();
      if (!file) continue;
      await handleDroppedFile(file);
      return;
    }
  }
}

function onCanvasMouseDown(e: MouseEvent) {
  if (e.button === 1 || (e.button === 0 && spaceHeld.value)) {
    e.preventDefault();
    isPanning.value = true;
    panStart.value = {
      x: e.clientX,
      y: e.clientY,
      panX: ui.panX,
      panY: ui.panY,
    };
  }
}

function onMouseMove(e: MouseEvent) {
  if (!isPanning.value) return;
  const dx = e.clientX - panStart.value.x;
  const dy = e.clientY - panStart.value.y;
  ui.setPan(panStart.value.panX + dx, panStart.value.panY + dy);
}

function onMouseUp() {
  isPanning.value = false;
}

function onDblClick() {
  const el = project.selectedElement;
  if (el && el.type === "path") {
    ui.enterPathEdit(el.id, el.d);
    return;
  }
  ui.resetView();
}

function onDrawStart(e: MouseEvent) {
  if (!ui.isDrawing) return;
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const x = (e.clientX - rect.left) * (project.canvasWidth / rect.width);
  const y = (e.clientY - rect.top) * (project.canvasHeight / rect.height);
  ui.currentPath = [{ x, y }];
}

function onDrawMove(e: MouseEvent) {
  if (!ui.isDrawing || ui.currentPath.length === 0) return;
  if (!(e.buttons & 1)) return;
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const x = (e.clientX - rect.left) * (project.canvasWidth / rect.width);
  const y = (e.clientY - rect.top) * (project.canvasHeight / rect.height);
  ui.currentPath.push({ x, y });
}

const pathPreview = computed(() => {
  if (ui.currentPath.length < 2) return "";
  let d = `M${ui.currentPath[0].x.toFixed(1)},${ui.currentPath[0].y.toFixed(1)}`;
  for (let i = 1; i < ui.currentPath.length; i++) {
    d += ` L${ui.currentPath[i].x.toFixed(1)},${ui.currentPath[i].y.toFixed(1)}`;
  }
  return d;
});

const canvasStyle = computed(() => ({
  width: project.canvasWidth + "px",
  height: project.canvasHeight + "px",
  transform: `translate(${ui.panX}px, ${ui.panY}px) scale(${effectiveScale.value})`,
}));

const cursorStyle = computed(() => {
  if (spaceHeld.value || isPanning.value) return "grab";
  return undefined;
});
</script>

<template>
  <div
    ref="mainAreaEl"
    class="main-area"
    :class="{ 'file-drag-over': fileDragOver }"
    :style="{ cursor: cursorStyle }"
    @wheel="onWheel"
    @mousedown="onCanvasMouseDown"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
    @mouseleave="onMouseUp"
    @dblclick="onDblClick"
    @dragover="onDragOver"
    @drop="onDrop"
    @dragleave="onDragLeave"
    @paste="onPaste"
    @keydown="onKeyDown"
    @keyup="onKeyUp"
    tabindex="0"
  >
    <div class="canvas-container" :style="canvasStyle">
      <div class="checkerboard">
        <SvgPreview :svg-content="project.svgPreview" />
      </div>
      <svg
        v-if="ui.isDrawing"
        class="draw-overlay"
        :viewBox="`0 0 ${project.canvasWidth} ${project.canvasHeight}`"
        @mousedown="onDrawStart"
        @mousemove="onDrawMove"
      >
        <path
          v-if="pathPreview"
          :d="pathPreview"
          fill="none"
          stroke="#333333"
          stroke-width="3"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      <PathEditorOverlay
        v-if="ui.pathEditing && !ui.isDrawing"
        :canvas-width="project.canvasWidth"
        :canvas-height="project.canvasHeight"
      />
      <ElementOverlay
        v-if="!ui.isDrawing && !ui.pathEditing"
        :elements="project.elements"
        :selected-id="ui.selectedElementId"
        :canvas-width="project.canvasWidth"
        :canvas-height="project.canvasHeight"
        :effective-scale="effectiveScale"
        :pan-x="ui.panX"
        :pan-y="ui.panY"
        @select="(id: string | null) => ui.selectElement(id)"
        @move="(id: string, x: number, y: number) => { project.updateElement(id, { x, y }); }"
        @resize="(id: string, x: number, y: number, width: number, height: number) => { project.updateElement(id, { x, y, width, height }); }"
      />
    </div>

    <PathToolbar />

    <div class="zoom-badge">{{ displayZoom }}%</div>

    <QuickStartOverlay
      v-if="project.elements.length === 0 && !project.svgPreview"
      ref="quickstartRef"
    />
    <div v-else-if="!project.svgPreview && project.elements.length > 0" class="loading">Loading preview...</div>

    <div v-if="fileDragOver" class="drop-indicator">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
      <span>Drop image to import</span>
    </div>
  </div>
</template>

<style scoped>
.main-area {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: radial-gradient(ellipse at center, #111113 0%, #09090B 70%);
  overflow: hidden;
  position: relative;
  outline: none;
}

.canvas-container {
  position: relative;
  width: 512px;
  height: 512px;
  border-radius: 8px;
  box-shadow: var(--shadow-lg);
  transform-origin: center center;
  flex-shrink: 0;
}

.draw-overlay {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  cursor: crosshair;
}

.checkerboard {
  width: 100%;
  height: 100%;
  background: repeating-conic-gradient(var(--bg-tertiary) 0% 25%, transparent 0% 50%) 50% / 12px 12px;
  border-radius: 8px;
  overflow: hidden;
}

.zoom-badge {
  position: absolute;
  bottom: 12px;
  right: 12px;
  background: var(--bg-glass);
  backdrop-filter: blur(8px);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 3px 8px;
  font-size: 11px;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
  pointer-events: none;
  user-select: none;
}

.loading {
  position: absolute;
  color: var(--text-muted);
  font-size: 12px;
  pointer-events: none;
}

.empty-state {
  position: absolute;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--text-muted);
  font-size: 13px;
  pointer-events: auto;
}

.empty-icon {
  width: 64px;
  height: 64px;
  opacity: 0.15;
  margin-bottom: 4px;
}

.btn-link {
  background: none;
  border: none;
  color: var(--accent);
  cursor: pointer;
  font-size: 12px;
  padding: 4px 8px;
  transition: color var(--transition-fast);
}
.btn-link:hover {
  color: var(--accent-hover);
  text-decoration: underline;
}

.main-area.file-drag-over {
  outline: 2px dashed var(--accent);
  outline-offset: -4px;
}

.drop-indicator {
  position: absolute;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: var(--accent);
  font-size: 13px;
  pointer-events: none;
  opacity: 0.8;
}
</style>
