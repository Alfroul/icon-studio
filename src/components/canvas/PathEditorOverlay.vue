<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { serializePath, usePathEditorThrottle } from "@/composables/usePathEditor";
import type { PathElement } from "@/types";

const ui = useUiStore();
const project = useProjectStore();
const { throttleUpdate, cancelUpdate } = usePathEditorThrottle();

const props = defineProps<{
  canvasWidth: number;
  canvasHeight: number;
}>();

const ANCHOR_SIZE = 6;
const HANDLE_RADIUS = 4;
const overlayRef = ref<SVGSVGElement | null>(null);

const nodes = computed(() => ui.pathEditing?.nodes ?? []);
const selectedIndex = computed(() => ui.pathEditing?.selectedIndex ?? null);

const pathD = computed(() => {
  if (nodes.value.length === 0) return "";
  return serializePath(nodes.value);
});

const editingElement = computed(() => {
  if (!ui.pathEditing) return null;
  return (project.elements.find((el) => el.id === ui.pathEditing!.elementId) ?? null) as PathElement | null;
});

const pathTransform = computed(() => {
  const el = editingElement.value;
  if (!el || el.type !== "path") return "";
  const sx = el.natural_width > 0 ? el.width / el.natural_width : 1;
  const sy = el.natural_height > 0 ? el.height / el.natural_height : 1;
  return `translate(${el.x}, ${el.y}) scale(${sx}, ${sy})`;
});

let isDragging = false;
let dragType: "anchor" | "handleIn" | "handleOut" = "anchor";
let dragIndex = -1;

function onAnchorMouseDown(e: MouseEvent, index: number) {
  e.stopPropagation();
  e.preventDefault();
  isDragging = true;
  dragType = "anchor";
  dragIndex = index;
  ui.setSelectedNode(index);
  ui.setDragging(true);
  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
}

function onHandleMouseDown(e: MouseEvent, index: number, type: "handleIn" | "handleOut") {
  e.stopPropagation();
  e.preventDefault();
  isDragging = true;
  dragType = type;
  dragIndex = index;
  ui.setSelectedNode(index);
  ui.setDragging(true);
  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
}

function toPathCoords(clientX: number, clientY: number): { x: number; y: number } | null {
  const container = overlayRef.value?.parentElement;
  if (!container) return null;
  const rect = container.getBoundingClientRect();
  const scale = props.canvasWidth / rect.width;
  let cx = (clientX - rect.left) * scale;
  let cy = (clientY - rect.top) * scale;

  const el = editingElement.value;
  if (el && el.type === "path") {
    const sx = el.natural_width > 0 ? el.width / el.natural_width : 1;
    const sy = el.natural_height > 0 ? el.height / el.natural_height : 1;
    cx = (cx - el.x) / sx;
    cy = (cy - el.y) / sy;
  }
  return { x: cx, y: cy };
}

function onMouseMove(e: MouseEvent) {
  if (!isDragging || dragIndex < 0) return;
  throttleUpdate(() => {
    const coords = toPathCoords(e.clientX, e.clientY);
    if (!coords) return;

    if (dragType === "anchor") {
      ui.updateNode(dragIndex, { anchor: { x: coords.x, y: coords.y } });
    } else if (dragType === "handleIn") {
      ui.updateNode(dragIndex, { handleIn: { x: coords.x, y: coords.y } });
    } else if (dragType === "handleOut") {
      ui.updateNode(dragIndex, { handleOut: { x: coords.x, y: coords.y } });
    }
  });
}

function onMouseUp() {
  if (isDragging) {
    isDragging = false;
    ui.setDragging(false);
    if (ui.pathEditing) {
      const newD = serializePath(nodes.value);
      project.updateElement(ui.pathEditing.elementId, { d: newD });
    }
  }
  window.removeEventListener("mousemove", onMouseMove);
  window.removeEventListener("mouseup", onMouseUp);
}

function onKeyDown(e: KeyboardEvent) {
  if (e.code === "Escape") {
    if (ui.pathEditing) {
      const newD = serializePath(nodes.value);
      project.updateElement(ui.pathEditing.elementId, { d: newD });
    }
    ui.exitPathEdit();
  }
  if ((e.code === "Delete" || e.code === "Backspace") && ui.pathEditing?.selectedIndex !== null) {
    e.preventDefault();
    ui.removeNode(ui.pathEditing.selectedIndex);
  }
}

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
});

onUnmounted(() => {
  if (isDragging && ui.pathEditing) {
    const newD = serializePath(nodes.value);
    project.updateElement(ui.pathEditing.elementId, { d: newD });
    isDragging = false;
  }
  window.removeEventListener("keydown", onKeyDown);
  cancelUpdate();
  window.removeEventListener("mousemove", onMouseMove);
  window.removeEventListener("mouseup", onMouseUp);
});
</script>

<template>
  <svg
    ref="overlayRef"
    class="path-editor-overlay"
    :viewBox="`0 0 ${canvasWidth} ${canvasHeight}`"
  >
    <path
      v-if="pathD"
      :d="pathD"
      :transform="pathTransform"
      fill="none"
      stroke="#3B82F6"
      stroke-width="1.5"
      opacity="0.5"
      class="path-preview"
    />

    <template v-for="(node, i) in nodes" :key="'h-' + i">
      <line
        v-if="node.handleIn"
        :x1="node.anchor.x"
        :y1="node.anchor.y"
        :x2="node.handleIn.x"
        :y2="node.handleIn.y"
        :transform="pathTransform"
        stroke="#999"
        stroke-width="1"
        stroke-dasharray="3,2"
        class="handle-line"
      />
      <line
        v-if="node.handleOut"
        :x1="node.anchor.x"
        :y1="node.anchor.y"
        :x2="node.handleOut.x"
        :y2="node.handleOut.y"
        :transform="pathTransform"
        stroke="#999"
        stroke-width="1"
        stroke-dasharray="3,2"
        class="handle-line"
      />
      <circle
        v-if="node.handleIn"
        :cx="node.handleIn.x"
        :cy="node.handleIn.y"
        :r="HANDLE_RADIUS"
        :transform="pathTransform"
        fill="white"
        stroke="#999"
        stroke-width="1"
        class="handle-circle"
        @mousedown.stop="onHandleMouseDown($event, i, 'handleIn')"
      />
      <circle
        v-if="node.handleOut"
        :cx="node.handleOut.x"
        :cy="node.handleOut.y"
        :r="HANDLE_RADIUS"
        :transform="pathTransform"
        fill="white"
        stroke="#999"
        stroke-width="1"
        class="handle-circle"
        @mousedown.stop="onHandleMouseDown($event, i, 'handleOut')"
      />
    </template>

    <rect
      v-for="(node, i) in nodes"
      :key="'a-' + i"
      :x="node.anchor.x - ANCHOR_SIZE / 2"
      :y="node.anchor.y - ANCHOR_SIZE / 2"
      :width="ANCHOR_SIZE"
      :height="ANCHOR_SIZE"
      :transform="pathTransform"
      :fill="selectedIndex === i ? '#3B82F6' : 'white'"
      :stroke="selectedIndex === i ? 'white' : '#3B82F6'"
      stroke-width="1.5"
      class="anchor-point"
      @mousedown.stop="onAnchorMouseDown($event, i)"
    />
  </svg>
</template>

<style scoped>
.path-editor-overlay {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: all;
  cursor: crosshair;
}

.path-preview {
  pointer-events: none;
}

.handle-line {
  pointer-events: none;
}

.handle-circle {
  pointer-events: all;
  cursor: pointer;
}

.anchor-point {
  pointer-events: all;
  cursor: move;
}
</style>
