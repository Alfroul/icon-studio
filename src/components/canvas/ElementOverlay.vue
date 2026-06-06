<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import type { Element } from "@/types";
import { useElementInteraction } from "@/composables/useElementInteraction";

const props = defineProps<{
  elements: Element[];
  selectedId: string | null;
  canvasWidth: number;
  canvasHeight: number;
  effectiveScale: number;
  panX: number;
  panY: number;
}>();

const emit = defineEmits<{
  select: [id: string | null];
  move: [id: string, x: number, y: number];
  resize: [id: string, x: number, y: number, width: number, height: number];
}>();

const svgRef = ref<HTMLElement | null>(null);

const selectedElement = computed(() => {
  if (!props.selectedId) return null;
  const el = props.elements.find((el) => el.id === props.selectedId) ?? null;
  return el;
});

const isLocked = computed(() => {
  return selectedElement.value?.locked === true;
});

const HANDLE_SIZE = 8;
const HANDLE_CURSORS = ["nw-resize", "ne-resize", "sw-resize", "se-resize"] as const;

const handles = computed(() => {
  const el = selectedElement.value;
  if (!el) return [];
  const h = HANDLE_SIZE;
  return [
    { x: el.x - h / 2, y: el.y - h / 2 },
    { x: el.x + el.width - h / 2, y: el.y - h / 2 },
    { x: el.x - h / 2, y: el.y + el.height - h / 2 },
    { x: el.x + el.width - h / 2, y: el.y + el.height - h / 2 },
  ];
});

const { isDragging, isResizing, handleOverlayMouseDown, handleOverlayMouseMove, handleOverlayMouseUp, handleResizeMouseDown } =
  useElementInteraction({
    elements: () => props.elements,
    canvasWidth: () => props.canvasWidth,
    canvasHeight: () => props.canvasHeight,
    effectiveScale: () => props.effectiveScale,
    panX: () => props.panX,
    panY: () => props.panY,
    onSelect: (id) => emit("select", id),
    onMove: (id, x, y) => { if (!isLocked.value) emit("move", id, x, y); },
    onResize: (id, x, y, width, height) => { if (!isLocked.value) emit("resize", id, x, y, width, height); },
    containerRef: svgRef,
  });
</script>

<template>
  <svg
    ref="svgRef"
    class="element-overlay"
    :viewBox="`0 0 ${canvasWidth} ${canvasHeight}`"
    :style="{ cursor: isDragging ? 'grabbing' : isResizing ? 'nwse-resize' : 'default' }"
    @mousedown="handleOverlayMouseDown"
    @mousemove="handleOverlayMouseMove"
    @mouseup="handleOverlayMouseUp"
    @mouseleave="handleOverlayMouseUp"
  >
    <template v-if="selectedElement">
      <rect
        :x="selectedElement.x"
        :y="selectedElement.y"
        :width="selectedElement.width"
        :height="selectedElement.height"
        fill="none"
        :stroke="isLocked ? '#94A3B8' : '#3B82F6'"
        stroke-width="1.5"
        :stroke-dasharray="isLocked ? '3,3' : '5,3'"
        class="selection-border"
      />
      <template v-if="!isLocked">
        <rect
          v-for="(handle, i) in handles"
          :key="i"
          :x="handle.x"
          :y="handle.y"
          :width="HANDLE_SIZE"
          :height="HANDLE_SIZE"
          fill="#3B82F6"
          stroke="white"
          stroke-width="1"
          :style="{ cursor: HANDLE_CURSORS[i], pointerEvents: 'all' }"
          @mousedown.stop="handleResizeMouseDown($event, i, selectedElement!.id)"
        />
      </template>
    </template>
  </svg>
</template>

<style scoped>
.element-overlay {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: all;
}

.selection-border {
  pointer-events: none;
}
</style>
