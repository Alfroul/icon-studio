<script setup lang="ts">
import { ref, watch, onUnmounted } from "vue";
import type { Shadow } from "@/types";
import ToggleSwitch from "@/components/common/ToggleSwitch.vue";

const props = defineProps<{
  elementId: string;
  shadow: Shadow | null;
}>();

const emit = defineEmits<{
  update: [shadow: Shadow | null];
}>();

function parseHexColor(hex: string): { r: number; g: number; b: number } {
  const h = hex.replace("#", "");
  return {
    r: parseInt(h.substring(0, 2), 16),
    g: parseInt(h.substring(2, 4), 16),
    b: parseInt(h.substring(4, 6), 16),
  };
}

function colorFromShadow(shadow: Shadow | null): string {
  if (!shadow) return "#000000";
  const hex = shadow.color;
  // Color may be in #RRGGBBAA format, extract RGB part
  if (hex.startsWith("#") && hex.length >= 7) {
    return hex.substring(0, 7);
  }
  return "#000000";
}

function opacityFromShadow(shadow: Shadow | null): number {
  if (!shadow) return 40;
  const hex = shadow.color;
  if (hex.startsWith("#") && hex.length === 9) {
    return Math.round((parseInt(hex.substring(7, 9), 16) / 255) * 100);
  }
  return 40;
}

const enabled = ref(!!props.shadow);
const colorHex = ref(colorFromShadow(props.shadow));
const opacity = ref(opacityFromShadow(props.shadow));
const blur = ref(props.shadow?.blur ?? 8);
const offsetX = ref(props.shadow?.offset_x ?? 0);
const offsetY = ref(props.shadow?.offset_y ?? 4);
const inset = ref(props.shadow?.inset ?? false);

let emitTimer: ReturnType<typeof setTimeout> | null = null;
let skipWatch = false;

onUnmounted(() => {
  if (emitTimer) { clearTimeout(emitTimer); emitTimer = null; }
});

function debouncedEmitShadow() {
  if (emitTimer) clearTimeout(emitTimer);
  emitTimer = setTimeout(emitShadow, 150);
}

watch(
  () => props.shadow,
  (s) => {
    if (skipWatch) { skipWatch = false; return; }
    enabled.value = !!s;
    if (s) {
      colorHex.value = colorFromShadow(s);
      opacity.value = opacityFromShadow(s);
      blur.value = s.blur;
      offsetX.value = s.offset_x;
      offsetY.value = s.offset_y;
      inset.value = s.inset ?? false;
    } else {
      colorHex.value = "#000000";
      opacity.value = 40;
      blur.value = 8;
      offsetX.value = 0;
      offsetY.value = 4;
    }
  }
);

function buildColorWithAlpha(): string {
  const { r, g, b } = parseHexColor(colorHex.value);
  const a = Math.round((opacity.value / 100) * 255);
  const toHex = (n: number) => n.toString(16).padStart(2, "0");
  return `#${toHex(r)}${toHex(g)}${toHex(b)}${toHex(a)}`;
}

function emitShadow() {
  if (!enabled.value) {
    emit("update", null);
    return;
  }
  emit("update", {
    color: buildColorWithAlpha(),
    blur: blur.value,
    offset_x: offsetX.value,
    offset_y: offsetY.value,
    inset: inset.value,
  });
}

function toggleEnabled() {
  enabled.value = !enabled.value;
  skipWatch = true;
  emitShadow();
}
</script>

<template>
  <div class="section">
    <div class="section-header">
      <span class="section-title">投影</span>
      <ToggleSwitch :model-value="enabled" @update:model-value="toggleEnabled" />
    </div>

    <div v-if="enabled" class="section-body">
      <!-- Color + Opacity row -->
      <div class="prop-row">
        <label class="field-label">颜色</label>
        <input
          type="color"
          class="color-input"
          :value="colorHex"
          @input="colorHex = ($event.target as HTMLInputElement).value; debouncedEmitShadow()"
        />
        <input
          type="range"
          class="opacity-slider"
          min="0"
          max="100"
          :value="opacity"
          @input="opacity = Number(($event.target as HTMLInputElement).value); debouncedEmitShadow()"
        />
        <span class="field-value">{{ opacity }}%</span>
      </div>

      <!-- Blur -->
      <div class="prop-row">
        <label class="field-label">模糊</label>
        <input
          type="number"
          class="num-input"
          min="0"
          max="100"
          :value="blur"
          @input="blur = Number(($event.target as HTMLInputElement).value); debouncedEmitShadow()"
        />
      </div>

      <!-- X offset -->
      <div class="prop-row">
        <label class="field-label">X 偏移</label>
        <input
          type="number"
          class="num-input"
          min="-100"
          max="100"
          :value="offsetX"
          @input="offsetX = Number(($event.target as HTMLInputElement).value); debouncedEmitShadow()"
        />
      </div>

      <!-- Y offset -->
      <div class="prop-row">
        <label class="field-label">Y 偏移</label>
        <input
          type="number"
          class="num-input"
          min="-100"
          max="100"
          :value="offsetY"
          @input="offsetY = Number(($event.target as HTMLInputElement).value); debouncedEmitShadow()"
        />
      </div>

      <!-- Inset -->
      <div class="prop-row">
        <label class="field-label">内阴影</label>
        <ToggleSwitch :model-value="inset" @update:model-value="inset = $event; debouncedEmitShadow()" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.section {
  border-bottom: 1px solid var(--border-color);
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
}

.section-body {
  padding: 0 16px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* Property row */
.prop-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.field-label {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 36px;
}

.field-value {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 30px;
  text-align: right;
  font-family: "JetBrains Mono", monospace;
}

.color-input {
  width: 28px;
  height: 22px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: none;
  cursor: pointer;
  padding: 1px;
}

.color-input::-webkit-color-swatch-wrapper {
  padding: 0;
}

.color-input::-webkit-color-swatch {
  border: none;
  border-radius: 2px;
}

.opacity-slider {
  flex: 1;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--border-color);
  border-radius: 2px;
  outline: none;
}

.opacity-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  background: var(--accent);
  border-radius: 50%;
  cursor: pointer;
}

.num-input {
  flex: 1;
  height: 24px;
  padding: 0 8px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
  transition: border-color var(--transition-fast) ease, box-shadow var(--transition-fast) ease;
}

.num-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-muted);
}

.num-input::-webkit-inner-spin-button {
  opacity: 0.3;
}
</style>
