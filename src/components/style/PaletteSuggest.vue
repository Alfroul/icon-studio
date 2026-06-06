<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits<{
  "apply-color": [color: string];
}>();

const baseColor = ref("#4488FF");
const scheme = ref("complementary");
const count = ref(5);
const palette = ref<string[]>([]);
const loading = ref(false);

const schemes = [
  { value: "complementary", label: "互补" },
  { value: "analogous", label: "类似" },
  { value: "triadic", label: "三角" },
  { value: "split-complementary", label: "分裂互补" },
  { value: "monochromatic", label: "单色" },
];

async function generate() {
  loading.value = true;
  try {
    const result = await invoke<string[]>("suggest_palette", {
      baseColor: baseColor.value,
      scheme: scheme.value,
      count: count.value,
    });
    palette.value = result;
  } catch (e) {
    console.error("Failed to generate palette:", e);
    palette.value = [];
  } finally {
    loading.value = false;
  }
}

function applyColor(color: string) {
  emit("apply-color", color);
}
</script>

<template>
  <div class="section">
    <div class="section-header">
      <span class="section-title">配色建议</span>
    </div>

    <div class="section-body">
      <!-- Base color -->
      <div class="prop-row">
        <label class="field-label">基色</label>
        <input type="color" class="color-input" v-model="baseColor" />
        <span class="hex-label">{{ baseColor.toUpperCase() }}</span>
      </div>

      <!-- Scheme -->
      <div class="prop-row">
        <label class="field-label">方案</label>
        <select class="select-input" v-model="scheme">
          <option v-for="s in schemes" :key="s.value" :value="s.value">
            {{ s.label }}
          </option>
        </select>
      </div>

      <!-- Count -->
      <div class="prop-row">
        <label class="field-label">数量</label>
        <input
          type="number"
          class="num-input"
          min="2"
          max="8"
          v-model.number="count"
        />
      </div>

      <!-- Generate button -->
      <button class="btn-generate" :disabled="loading" @click="generate">
        {{ loading ? "生成中…" : "生成配色" }}
      </button>

      <!-- Palette swatches -->
      <div v-if="palette.length" class="swatches">
        <button
          v-for="(color, i) in palette"
          :key="i"
          class="swatch"
          :title="color"
          @click="applyColor(color)"
        >
          <span class="swatch-color" :style="{ background: color }" />
          <span class="swatch-hex">{{ color.toUpperCase() }}</span>
        </button>
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
  padding: 10px 14px;
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--accent);
}

.section-body {
  padding: 0 14px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.prop-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.field-label {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 28px;
}

.hex-label {
  font-size: 11px;
  color: var(--text-muted);
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

.select-input {
  flex: 1;
  height: 22px;
  padding: 0 4px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
  cursor: pointer;
  transition: border-color var(--transition-fast);
}

.select-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-muted);
}

.select-input option {
  background: var(--bg-elevated);
  color: var(--text-primary);
}

.num-input {
  flex: 1;
  height: 22px;
  padding: 0 6px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
  transition: border-color var(--transition-fast);
}

.num-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-muted);
}

.btn-generate {
  padding: 5px 0;
  border: 1px solid var(--accent);
  background: transparent;
  color: var(--accent);
  font-size: 11px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.btn-generate:hover:not(:disabled) {
  background: var(--accent);
  color: var(--bg-primary);
}

.btn-generate:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Swatches */
.swatches {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.swatch {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 6px;
  border: 1px solid transparent;
  background: none;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
  width: 100%;
  text-align: left;
}

.swatch:hover {
  border-color: var(--accent);
  background: var(--bg-hover);
  box-shadow: 0 0 8px var(--accent-glow);
}

.swatch-color {
  width: 18px;
  height: 18px;
  border-radius: var(--radius-sm);
  border: 1px solid rgba(255, 255, 255, 0.1);
  flex-shrink: 0;
}

.swatch-hex {
  font-size: 11px;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}
</style>
