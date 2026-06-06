<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "@/stores/project";
import type { AnimationType, Animation } from "@/types";

const project = useProjectStore();

const element = computed(() => project.selectedElement);

const animation = computed(() => {
  if (!element.value) return null;
  return element.value.animation ?? null;
});

const animationType = ref<AnimationType>("rotate");
const duration = ref(2);
const delay = ref(0);
const repeat = ref(true);
const easing = ref("ease-in-out");
const pathData = ref("M0,0 L10,0 L10,10 L0,10 Z");

watch(
  () => animation.value,
  (a) => {
    if (a) {
      animationType.value = a.animation_type;
      duration.value = a.duration;
      delay.value = a.delay;
      repeat.value = a.repeat;
      easing.value = a.easing;
      if (a.animation_type === "path" && a.params?.path) {
        pathData.value = a.params.path as string;
      }
    } else {
      animationType.value = "rotate";
      duration.value = 2;
      delay.value = 0;
      repeat.value = true;
      easing.value = "ease-in-out";
      pathData.value = "M0,0 L10,0 L10,10 L0,10 Z";
    }
  },
  { immediate: true }
);

const animationTypes: { value: AnimationType; label: string }[] = [
  { value: "rotate", label: "Rotate" },
  { value: "scale", label: "Scale" },
  { value: "fade", label: "Fade" },
  { value: "translate", label: "Translate" },
  { value: "path", label: "Path" },
];

const easingOptions = [
  { value: "ease-in-out", label: "Ease In Out" },
  { value: "linear", label: "Linear" },
  { value: "ease", label: "Ease" },
  { value: "ease-in", label: "Ease In" },
  { value: "ease-out", label: "Ease Out" },
];

async function applyAnimation() {
  if (!element.value) return;
  const params: Record<string, unknown> = {};
  if (animationType.value === "scale") params.min_scale = 0.8;
  if (animationType.value === "fade") params.min_opacity = 0.0;
  if (animationType.value === "translate") {
    params.dx = 10;
    params.dy = 0;
  }
  if (animationType.value === "path") {
    params.path = pathData.value;
  }
  try {
    await invoke("set_animation", {
      request: {
        elementId: element.value.id,
        animationType: animationType.value,
        duration: duration.value,
        delay: delay.value,
        repeat: repeat.value,
        easing: easing.value,
        params,
      },
    });
    await project.refreshElements();
  } catch (e) {
    console.error("Failed to set animation:", e);
  }
}

async function removeAnimation() {
  if (!element.value) return;
  try {
    await invoke("clear_animation", { elementId: element.value.id });
    await project.refreshElements();
  } catch (e) {
    console.error("Failed to clear animation:", e);
  }
}
</script>

<template>
  <div v-if="element" class="animation-section">
    <div class="prop-group-label">Animation</div>

    <div v-if="animation" class="animation-active">
      <div class="prop-row">
        <label class="prop-label">Type</label>
        <select class="prop-select" v-model="animationType" @change="applyAnimation">
          <option v-for="t in animationTypes" :key="t.value" :value="t.value">{{ t.label }}</option>
        </select>
      </div>
      <div class="prop-row">
        <label class="prop-label">Duration</label>
        <input
          type="range"
          class="prop-range"
          v-model.number="duration"
          min="0.1"
          max="10"
          step="0.1"
          @change="applyAnimation"
        />
        <span class="range-value">{{ duration.toFixed(1) }}s</span>
      </div>
      <div class="prop-row">
        <label class="prop-label">Delay</label>
        <input
          type="number"
          class="prop-input"
          v-model.number="delay"
          min="0"
          step="0.1"
          @change="applyAnimation"
        />
        <span class="range-value">sec</span>
      </div>
      <div class="prop-row">
        <label class="prop-label">Easing</label>
        <select class="prop-select" v-model="easing" @change="applyAnimation">
          <option v-for="e in easingOptions" :key="e.value" :value="e.value">{{ e.label }}</option>
        </select>
      </div>
      <div v-if="animationType === 'path'" class="prop-row">
        <label class="prop-label">Path Data</label>
        <input
          type="text"
          class="prop-input"
          v-model="pathData"
          placeholder="M0,0 L10,0 L10,10 L0,10 Z"
          @change="applyAnimation"
        />
      </div>
      <div class="prop-row">
        <label class="prop-label">Loop</label>
        <input
          type="checkbox"
          v-model="repeat"
          class="prop-checkbox"
          @change="applyAnimation"
        />
      </div>
      <button class="clear-btn" @click="removeAnimation">Remove Animation</button>
    </div>

    <div v-else class="animation-empty">
      <button class="add-btn" @click="applyAnimation">Add Animation</button>
    </div>
  </div>
</template>

<style scoped>
.animation-section {
  padding: 4px 12px 6px;
  border-bottom: 1px solid var(--bg-tertiary);
}

.prop-group-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--accent);
  margin-bottom: 4px;
}

.prop-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}

.prop-label {
  width: 52px;
  flex-shrink: 0;
  font-size: 11px;
  color: var(--text-muted);
}

.prop-input {
  flex: 1;
  height: 26px;
  padding: 0 8px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
}

.prop-input:focus {
  border-color: var(--accent);
}

.prop-input[type="number"] {
  -moz-appearance: textfield;
}

.prop-input[type="number"]::-webkit-inner-spin-button,
.prop-input[type="number"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
}

.prop-select {
  flex: 1;
  height: 26px;
  padding: 0 6px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 11px;
  outline: none;
}

.prop-select:focus {
  border-color: var(--accent);
}

.prop-range {
  flex: 1;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--bg-active);
  border-radius: 2px;
  outline: none;
}

.prop-range::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 14px;
  height: 14px;
  background: var(--accent);
  border-radius: 50%;
  cursor: pointer;
}

.prop-checkbox {
  accent-color: var(--accent);
  width: 14px;
  height: 14px;
  cursor: pointer;
}

.range-value {
  width: 32px;
  text-align: right;
  font-size: 11px;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

.add-btn,
.clear-btn {
  width: 100%;
  height: 26px;
  margin-top: 4px;
  border-radius: var(--radius-sm);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.add-btn {
  background: var(--accent-muted);
  border: 1px solid var(--accent);
  color: var(--accent);
}

.add-btn:hover {
  background: var(--accent);
  color: var(--bg-primary);
}

.clear-btn {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  color: var(--text-muted);
}

.clear-btn:hover {
  background: var(--danger-muted);
  border-color: var(--danger);
  color: var(--danger);
}
</style>
