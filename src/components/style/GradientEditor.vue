<script setup lang="ts">
import { ref, watch, onUnmounted } from "vue";
import type { Gradient } from "@/types";
import ToggleSwitch from "@/components/common/ToggleSwitch.vue";

const props = defineProps<{
  elementId: string;
  gradient: Gradient | null;
}>();

const emit = defineEmits<{
  update: [gradient: Gradient | null];
}>();

const enabled = ref(!!props.gradient);
const gradientType = ref<"linear" | "radial">(props.gradient?.type ?? "linear");
const colors = ref<string[]>(props.gradient?.colors?.slice() ?? ["#4488FF", "#FF6644"]);
const angle = ref(props.gradient?.angle ?? 0);

let emitTimer: ReturnType<typeof setTimeout> | null = null;
let skipWatch = false;

function debouncedEmitGradient() {
  if (emitTimer) clearTimeout(emitTimer);
  emitTimer = setTimeout(emitGradient, 150);
}

watch(
  () => props.gradient,
  (g) => {
    if (skipWatch) { skipWatch = false; return; }
    enabled.value = !!g;
    if (g) {
      gradientType.value = g.type;
      colors.value = g.colors.slice();
      angle.value = g.angle;
    } else {
      gradientType.value = "linear";
      colors.value = ["#4488FF", "#FF6644"];
      angle.value = 0;
    }
  }
);

function emitGradient() {
  if (!enabled.value) {
    emit("update", null);
    return;
  }
  emit("update", {
    type: gradientType.value,
    colors: colors.value.slice(),
    angle: angle.value,
  });
}

function toggleEnabled() {
  enabled.value = !enabled.value;
  skipWatch = true;
  emitGradient();
}

function addStop() {
  if (colors.value.length >= 5) return;
  // Interpolate a color between last two
  colors.value.push("#888888");
  emitGradient();
}

function removeStop(index: number) {
  if (colors.value.length <= 2) return;
  colors.value.splice(index, 1);
  emitGradient();
}

function updateColor(index: number, value: string) {
  colors.value[index] = value;
  emitGradient();
}

function updateType(type: "linear" | "radial") {
  gradientType.value = type;
  emitGradient();
}

function updateAngle(val: number) {
  angle.value = val;
  debouncedEmitGradient();
}

onUnmounted(() => {
  if (emitTimer) clearTimeout(emitTimer);
});
</script>

<template>
  <div class="section">
    <div class="section-header">
      <span class="section-title">渐变</span>
      <ToggleSwitch :model-value="enabled" @update:model-value="toggleEnabled" />
    </div>

    <div v-if="enabled" class="section-body">
      <!-- Type selector -->
      <div class="type-row">
        <label class="radio" :class="{ active: gradientType === 'linear' }">
          <input
            type="radio"
            name="gradient-type"
            value="linear"
            :checked="gradientType === 'linear'"
            @change="updateType('linear')"
          />
          线性
        </label>
        <label class="radio" :class="{ active: gradientType === 'radial' }">
          <input
            type="radio"
            name="gradient-type"
            value="radial"
            :checked="gradientType === 'radial'"
            @change="updateType('radial')"
          />
          径向
        </label>
      </div>

      <!-- Preview bar -->
      <div
        class="gradient-preview"
        :style="{
          background: enabled
            ? gradientType === 'radial'
              ? `radial-gradient(circle, ${colors.join(', ')})`
              : `linear-gradient(90deg, ${colors.join(', ')})`
            : 'transparent',
        }"
      />

      <!-- Color stops -->
      <div class="stops-list">
        <div v-for="(color, i) in colors" :key="i" class="stop-row">
          <input
            type="color"
            class="color-input"
            :value="color"
            @input="updateColor(i, ($event.target as HTMLInputElement).value)"
          />
          <span class="stop-hex">{{ color.toUpperCase() }}</span>
          <button
            v-if="colors.length > 2"
            class="stop-remove"
            @click="removeStop(i)"
          >
            ×
          </button>
        </div>
      </div>

      <!-- Add stop -->
      <button
        v-if="colors.length < 5"
        class="btn-add"
        @click="addStop"
      >
        + 添加色标
      </button>

      <!-- Angle (linear only) -->
      <div v-if="gradientType === 'linear'" class="angle-row">
        <label class="field-label">角度</label>
        <input
          type="range"
          class="angle-slider"
          min="0"
          max="360"
          :value="angle"
          @input="updateAngle(Number(($event.target as HTMLInputElement).value))"
        />
        <span class="angle-value">{{ angle }}°</span>
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

/* Type selector */
.type-row {
  display: flex;
  gap: 6px;
}

.radio {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 4px 0;
  font-size: 12px;
  color: var(--text-muted);
  background: var(--bg-elevated);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.radio input {
  display: none;
}

.radio.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-muted);
}

/* Gradient preview */
.gradient-preview {
  height: 14px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-color);
}

/* Stops */
.stops-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stop-row {
  display: flex;
  align-items: center;
  gap: 6px;
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

.stop-hex {
  font-size: 11px;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
  min-width: 60px;
}

.stop-remove {
  margin-left: auto;
  width: 20px;
  height: 20px;
  border: none;
  background: none;
  color: var(--text-muted);
  font-size: 14px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
}

.stop-remove:hover {
  color: var(--danger);
  background: var(--danger-muted);
}

.btn-add {
  padding: 4px 0;
  border: 1px dashed var(--border-color);
  background: none;
  color: var(--text-muted);
  font-size: 11px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.btn-add:hover {
  border-color: var(--accent);
  color: var(--accent);
}

/* Angle */
.angle-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.field-label {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 28px;
}

.angle-slider {
  flex: 1;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--border-color);
  border-radius: 2px;
  outline: none;
}

.angle-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  background: var(--accent);
  border-radius: 50%;
  cursor: pointer;
}

.angle-value {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 30px;
  text-align: right;
  font-family: "JetBrains Mono", monospace;
}
</style>
