<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "@/stores/project";

const project = useProjectStore();

const element = computed(() => project.selectedElement);

interface StyleOption {
  value: string;
  label: string;
  icon: string;
}

const styles: StyleOption[] = [
  { value: "soft-3d", label: "Soft 3D", icon: "cube" },
  { value: "neumorphism", label: "Neumorphism", icon: "circle" },
  { value: "glassmorphism", label: "Glass", icon: "diamond" },
  { value: "flat", label: "Flat", icon: "square" },
];

const selectedStyle = ref("soft-3d");
const depth = ref(5);
const lightAngle = ref(135);
const highlight = ref(0.3);
const shadowSoftness = ref(8);

const applying = ref(false);

async function applyStyle() {
  if (!element.value || applying.value) return;
  applying.value = true;
  try {
    await invoke("apply_style_preset", {
      elementId: element.value.id,
      styleType: selectedStyle.value,
      params: {
        depth: depth.value,
        light_angle: lightAngle.value,
        highlight: highlight.value,
        shadow_softness: shadowSoftness.value,
      },
    });
    await project.refreshElements();
  } catch (e) {
    console.error("Failed to apply style:", e);
  } finally {
    applying.value = false;
  }
}
</script>

<template>
  <div v-if="element" class="section">
    <div class="section-header">
      <span class="section-title">Style Presets</span>
    </div>
    <div class="section-body">
      <div class="style-grid">
        <button
          v-for="style in styles"
          :key="style.value"
          class="style-card"
          :class="{ 'style-card--active': selectedStyle === style.value }"
          @click="selectedStyle = style.value"
        >
          <span class="style-card__icon">{{ style.icon }}</span>
          <span class="style-card__label">{{ style.label }}</span>
        </button>
      </div>

      <template v-if="selectedStyle !== 'flat'">
        <div class="prop-row">
          <label class="field-label">Depth</label>
          <input
            type="range"
            class="param-slider"
            min="1"
            max="20"
            step="1"
            v-model.number="depth"
          />
          <input type="number" class="num-input" min="1" max="20" step="1" v-model.number="depth" />
        </div>

        <div class="prop-row">
          <label class="field-label">Angle</label>
          <input
            type="range"
            class="param-slider"
            min="0"
            max="360"
            step="5"
            v-model.number="lightAngle"
          />
          <input type="number" class="num-input" min="0" max="360" step="5" v-model.number="lightAngle" />
        </div>

        <div class="prop-row">
          <label class="field-label">Highlight</label>
          <input
            type="range"
            class="param-slider"
            min="0.05"
            max="1"
            step="0.05"
            v-model.number="highlight"
          />
          <input type="number" class="num-input" min="0.05" max="1" step="0.05" v-model.number="highlight" />
        </div>

        <div class="prop-row">
          <label class="field-label">Softness</label>
          <input
            type="range"
            class="param-slider"
            min="1"
            max="30"
            step="1"
            v-model.number="shadowSoftness"
          />
          <input type="number" class="num-input" min="1" max="30" step="1" v-model.number="shadowSoftness" />
        </div>
      </template>

      <button class="btn-apply" :disabled="applying" @click="applyStyle">
        {{ applying ? "Applying..." : "Apply Style" }}
      </button>
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

.style-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.style-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px 4px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background: var(--input-bg);
  color: var(--text-muted);
  cursor: pointer;
  font-size: 10px;
  transition: border-color var(--transition-fast), background var(--transition-fast);
}

.style-card:hover {
  border-color: var(--accent);
}

.style-card--active {
  border-color: var(--accent);
  background: var(--bg-hover);
  color: var(--text-primary);
}

.style-card__icon {
  font-size: 16px;
  line-height: 1;
}

.style-card__label {
  font-weight: 500;
}

.prop-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.field-label {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 52px;
}

.param-slider {
  flex: 1;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--border-color);
  border-radius: 2px;
  outline: none;
}

.param-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  background: var(--accent);
  border-radius: 50%;
  cursor: pointer;
}

.num-input {
  width: 48px;
  height: 24px;
  padding: 0 6px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 11px;
  font-family: "JetBrains Mono", monospace;
  outline: none;
  text-align: right;
}

.num-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-muted);
}

.btn-apply {
  width: 100%;
  height: 28px;
  background: var(--accent);
  border: none;
  border-radius: var(--radius-md);
  color: var(--bg-primary);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity var(--transition-fast);
}

.btn-apply:hover:not(:disabled) {
  opacity: 0.85;
}

.btn-apply:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
