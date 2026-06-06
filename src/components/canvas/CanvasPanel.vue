<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from "vue";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";
import type { Gradient } from "@/types";

const store = useProjectStore();
const ui = useUiStore();

const presets = [
  { label: "16 × 16", w: 16, h: 16 },
  { label: "32 × 32", w: 32, h: 32 },
  { label: "64 × 64", w: 64, h: 64 },
  { label: "128 × 128", w: 128, h: 128 },
  { label: "256 × 256", w: 256, h: 256 },
  { label: "512 × 512", w: 512, h: 512 },
  { label: "1024 × 1024", w: 1024, h: 1024 },
];

const selectedPreset = ref("512 × 512");
const width = ref(store.canvasWidth);
const height = ref(store.canvasHeight);
const background = ref(store.canvasBackground);
const cornerRadius = ref(store.canvasCornerRadius);

// Gradient state
const gradientEnabled = ref(false);
const bgGradient = ref<Gradient>({
  type: "linear",
  colors: ["#FF5733", "#33C1FF"],
  angle: 0,
  stops: [0, 1],
});

watch(() => store.canvasWidth, (v) => width.value = v);
watch(() => store.canvasHeight, (v) => height.value = v);
watch(() => store.canvasBackground, (v) => background.value = v);
watch(() => store.canvasCornerRadius, (v) => cornerRadius.value = v);

watch(() => store.canvasBackgroundGradient, (v) => {
  if (v) {
    gradientEnabled.value = true;
    bgGradient.value = { ...v, stops: v.stops ?? (v.colors.length > 1 ? v.colors.map((_, i, arr) => i / (arr.length - 1)) : [0]) };
  } else {
    gradientEnabled.value = false;
  }
});

watch([() => store.canvasWidth, () => store.canvasHeight], () => {
  const match = presets.find(p => p.w === store.canvasWidth && p.h === store.canvasHeight);
  selectedPreset.value = match ? match.label : "Custom";
});

const isCustom = computed(() => selectedPreset.value === "Custom");
const isLinear = computed(() => bgGradient.value.type === "linear");
const canRemoveStop = computed(() => bgGradient.value.colors.length > 2);

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

function onPresetChange() {
  if (isCustom.value) return;
  const preset = presets.find((p) => p.label === selectedPreset.value);
  if (preset) {
    width.value = preset.w;
    height.value = preset.h;
    applyCanvas();
  }
}

function onCanvasInput() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(applyCanvas, 200);
}

function toggleGradient() {
  gradientEnabled.value = !gradientEnabled.value;
  applyCanvas();
}

function onGradientTypeChange() {
  applyCanvas();
}

function onGradientAngleChange() {
  onCanvasInput();
}

function updateStopColor(index: number, color: string) {
  bgGradient.value.colors[index] = color;
  applyCanvas();
}

function addStop() {
  if (bgGradient.value.colors.length >= 5) return;
  const n = bgGradient.value.colors.length;
  const last = bgGradient.value.colors[n - 1];
  bgGradient.value.colors.push(last);
  const stops = bgGradient.value.stops;
  stops.push(1);
  redistributeStops();
  applyCanvas();
}

function removeStop(index: number) {
  if (bgGradient.value.colors.length <= 2) return;
  bgGradient.value.colors.splice(index, 1);
  bgGradient.value.stops.splice(index, 1);
  redistributeStops();
  applyCanvas();
}

function redistributeStops() {
  const n = bgGradient.value.colors.length;
  bgGradient.value.stops = bgGradient.value.colors.map((_, i) => i / (n - 1));
}

async function applyCanvas() {
  const w = Math.min(8192, Math.max(1, Number(width.value) || 512));
  const h = Math.min(8192, Math.max(1, Number(height.value) || 512));
  width.value = w;
  height.value = h;
  const gradientPayload: Gradient | null = gradientEnabled.value
    ? {
        type: bgGradient.value.type,
        colors: [...bgGradient.value.colors],
        angle: bgGradient.value.angle,
        stops: [...bgGradient.value.stops],
      }
    : null;
  try {
    await store.updateCanvas({
      width: w,
      height: h,
      background: background.value,
      corner_radius: cornerRadius.value,
      background_gradient: gradientPayload,
    });
  } catch {
    width.value = store.canvasWidth;
    height.value = store.canvasHeight;
    background.value = store.canvasBackground;
    cornerRadius.value = store.canvasCornerRadius;
    const stored = store.canvasBackgroundGradient;
    if (stored) {
      gradientEnabled.value = true;
      bgGradient.value = { ...stored, stops: stored.stops ?? (stored.colors.length > 1 ? stored.colors.map((_, i, arr) => i / (arr.length - 1)) : [0]) };
    } else {
      gradientEnabled.value = false;
    }
  }
}

onUnmounted(() => {
  if (debounceTimer) clearTimeout(debounceTimer);
});
</script>

<template>
  <div class="canvas-panel">
    <div class="field">
      <label>Size Preset</label>
      <select v-model="selectedPreset" @change="onPresetChange">
        <option v-for="p in presets" :key="p.label" :value="p.label">
          {{ p.label }}
        </option>
        <option value="Custom">Custom</option>
      </select>
    </div>

    <div class="field-row">
      <div class="field">
        <label>Width</label>
        <input
          type="number"
          v-model.number="width"
          :disabled="!isCustom"
          min="1"
          max="8192"
          @change="applyCanvas"
        />
      </div>
      <div class="field">
        <label>Height</label>
        <input
          type="number"
          v-model.number="height"
          :disabled="!isCustom"
          min="1"
          max="8192"
          @change="applyCanvas"
        />
      </div>
    </div>

    <div class="field">
      <label>Background</label>
      <div class="color-row">
        <input type="color" v-model="background" @change="applyCanvas" />
        <span class="color-value">{{ background }}</span>
      </div>
    </div>

    <div class="field">
      <label>Corner Radius</label>
      <div class="slider-row">
        <input
          type="range"
          v-model.number="cornerRadius"
          min="0"
          max="50"
          step="1"
          @input="onCanvasInput"
        />
        <span class="slider-value">{{ cornerRadius }}%</span>
      </div>
    </div>

    <div class="gradient-section">
      <div class="gradient-header">
        <label>Background Gradient</label>
        <button
          class="toggle-btn"
          :class="{ active: gradientEnabled }"
          @click="toggleGradient"
        >
          <span class="toggle-knob" />
        </button>
      </div>

      <template v-if="gradientEnabled">
        <div class="field" style="margin-top: 8px">
          <label>Type</label>
          <select v-model="bgGradient.type" @change="onGradientTypeChange">
            <option value="linear">Linear</option>
            <option value="radial">Radial</option>
          </select>
        </div>

        <div v-if="isLinear" class="field">
          <label>Angle</label>
          <div class="slider-row">
            <input
              type="range"
              v-model.number="bgGradient.angle"
              min="0"
              max="360"
              step="1"
              @input="onGradientAngleChange"
            />
            <span class="slider-value">{{ bgGradient.angle }}°</span>
          </div>
        </div>

        <div class="field">
          <label>Color Stops</label>
          <div class="stops-list">
            <div
              v-for="(color, i) in bgGradient.colors"
              :key="i"
              class="stop-row"
            >
              <input
                type="color"
                :value="color"
                @input="updateStopColor(i, ($event.target as HTMLInputElement).value)"
              />
              <span class="color-value">{{ color }}</span>
              <button
                v-if="canRemoveStop"
                class="stop-remove"
                @click="removeStop(i)"
                title="Remove stop"
              >×</button>
            </div>
          </div>
          <button
            v-if="bgGradient.colors.length < 5"
            class="add-stop-btn"
            @click="addStop"
          >+ Add Stop</button>
        </div>
      </template>
    </div>

    <div class="canvas-info">
      {{ width }} × {{ height }} px
    </div>

    <div class="canvas-actions">
      <div class="action-group-label">File</div>
      <button class="file-action-btn" @click="store.openProject()">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/></svg>
        Open Project
      </button>
      <button class="file-action-btn" @click="store.importSvg()">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
        Import SVG
      </button>
    </div>
  </div>
</template>

<style scoped>
.canvas-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field label {
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.field select,
.field input[type="number"] {
  height: 28px;
  padding: 0 10px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  color: var(--text-primary);
  border-radius: var(--radius-sm);
  font-size: 12px;
  outline: none;
  cursor: pointer;
  transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
}

.field select:focus,
.field input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-muted);
}

.field input[type="number"]:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.field-row {
  display: flex;
  gap: 8px;
}

.field-row .field {
  flex: 1;
}

.color-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.color-row input[type="color"] {
  width: 28px;
  height: 28px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: none;
  padding: 0;
  cursor: pointer;
}

.color-value {
  font-size: 12px;
  color: var(--text-secondary);
  font-family: "JetBrains Mono", monospace;
}

.slider-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.slider-row input[type="range"] {
  flex: 1;
  accent-color: var(--accent);
}

.slider-value {
  font-family: "JetBrains Mono", monospace;
  font-size: 11px;
  color: var(--text-muted);
  min-width: 32px;
  text-align: right;
}

.canvas-info {
  font-family: "JetBrains Mono", monospace;
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  padding-top: 4px;
  border-top: 1px solid var(--border-color);
}

.canvas-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-top: 8px;
}

.action-group-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-muted);
}

.file-action-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  height: 32px;
  padding: 0 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.file-action-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.file-action-btn:active {
  background: var(--accent);
  color: var(--bg-primary);
}

.gradient-section {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.gradient-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.gradient-header label {
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.toggle-btn {
  position: relative;
  width: 36px;
  height: 20px;
  border-radius: 10px;
  border: 1px solid var(--input-border);
  background: var(--input-bg);
  cursor: pointer;
  padding: 0;
  transition: background var(--transition-fast), border-color var(--transition-fast);
}

.toggle-btn.active {
  background: var(--accent);
  border-color: var(--accent);
}

.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--text-primary);
  transition: transform var(--transition-fast);
}

.toggle-btn.active .toggle-knob {
  transform: translateX(16px);
  background: var(--bg-primary);
}

.stops-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.stop-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.stop-row input[type="color"] {
  width: 28px;
  height: 28px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: none;
  padding: 0;
  cursor: pointer;
  flex-shrink: 0;
}

.stop-remove {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
  font-size: 14px;
  line-height: 1;
  padding: 0;
  flex-shrink: 0;
  transition: color var(--transition-fast), background var(--transition-fast);
}

.stop-remove:hover {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
}

.add-stop-btn {
  margin-top: 6px;
  height: 26px;
  padding: 0 10px;
  background: var(--bg-tertiary);
  border: 1px dashed var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.add-stop-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}
</style>
