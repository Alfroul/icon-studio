<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as dialogOpen } from "@tauri-apps/plugin-dialog";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import AppIcon from "@/components/common/AppIcon.vue";

const ui = useUiStore();
const project = useProjectStore();

const selectedShape = ref("circle");
const shapes = [
  { id: "circle", label: "Circle" },
  { id: "squircle", label: "Squircle" },
  { id: "rounded-rect", label: "Rounded Rect" },
  { id: "pill", label: "Pill" },
  { id: "square", label: "Square" },
];

interface SafeZoneViolation {
  element_id: string;
  overshoot_px: number;
}

interface SafeZoneResult {
  safe: boolean;
  violations: SafeZoneViolation[];
}

const previewImages = ref<Record<string, string>>({});
const previewLoading = ref(false);
const safeZoneResult = ref<SafeZoneResult | null>(null);
const safeZoneMargin = ref(34);
const fgIds = ref<string[]>([]);
const bgIds = ref<string[]>([]);
const exporting = ref(false);
const exportResults = ref<string[]>([]);

const elements = computed(() => project.elements);

const fgIdsText = computed({
  get: () => fgIds.value.join(", "),
  set: (v: string) => {
    fgIds.value = v.split(",").map(s => s.trim()).filter(Boolean);
  },
});
const bgIdsText = computed({
  get: () => bgIds.value.join(", "),
  set: (v: string) => {
    bgIds.value = v.split(",").map(s => s.trim()).filter(Boolean);
  },
});

async function loadPreviews() {
  previewLoading.value = true;
  try {
    const results: Record<string, string> = {};
    for (const shape of shapes) {
      try {
        const b64 = await invoke<string>("preview_adaptive_icon", {
          shape: shape.id,
          size: 256,
        });
        results[shape.id] = `data:image/png;base64,${b64}`;
      } catch (e) {
        console.warn(`Preview failed for ${shape.id}:`, e);
      }
    }
    previewImages.value = results;
  } finally {
    previewLoading.value = false;
  }
}

async function checkSafeZone() {
  try {
    const result = await invoke<string>("check_adaptive_safe_zone", {
      margin: safeZoneMargin.value,
    });
    safeZoneResult.value = JSON.parse(result);
  } catch (e) {
    ui.showToast(`Safe zone check failed: ${e}`, "error");
  }
}

async function setForeground() {
  try {
    await invoke("set_adaptive_foreground", { ids: fgIds.value });
    ui.showToast("Foreground layer updated", "success");
  } catch (e) {
    ui.showToast(`Failed: ${e}`, "error");
  }
}

async function setBackground() {
  try {
    await invoke("set_adaptive_background", { ids: bgIds.value });
    ui.showToast("Background layer updated", "success");
  } catch (e) {
    ui.showToast(`Failed: ${e}`, "error");
  }
}

async function exportAndroid() {
  if (exporting.value) return;
  try {
    const selected = await dialogOpen({
      directory: true,
      title: "Select output directory",
    });
    const dir = typeof selected === "string" ? selected : null;
    if (!dir) return;

    exporting.value = true;
    exportResults.value = await invoke<string[]>("export_adaptive_android", {
      outputDir: dir,
    });
    ui.showToast(`Exported ${exportResults.value.length} files`, "success");
  } catch (e) {
    ui.showToast(`Export failed: ${e}`, "error");
  } finally {
    exporting.value = false;
  }
}

watch(
  () => project.currentVersion,
  () => loadPreviews(),
  { immediate: true }
);
</script>

<template>
  <div class="adaptive-preview">
    <!-- Shape Preview Grid -->
    <div class="panel-section">
      <h3 class="section-title">Shape Preview</h3>
      <div class="preview-grid">
        <div
          v-for="shape in shapes"
          :key="shape.id"
          class="preview-card"
          :class="{ 'preview-card--active': selectedShape === shape.id }"
          @click="selectedShape = shape.id"
        >
          <div class="preview-frame">
            <img
              v-if="previewImages[shape.id]"
              :src="previewImages[shape.id]"
              class="preview-img"
            />
            <span v-else class="preview-placeholder">--</span>
          </div>
          <span class="preview-label">{{ shape.label }}</span>
        </div>
      </div>
    </div>

    <!-- Safe Zone Check -->
    <div class="panel-section">
      <h3 class="section-title">Safe Zone</h3>
      <div class="safe-zone-row">
        <label class="field-label">
          Margin %
          <input
            v-model.number="safeZoneMargin"
            type="number"
            class="field-input"
            min="0"
            max="50"
          />
        </label>
        <button class="action-btn" @click="checkSafeZone">Check</button>
      </div>
      <div v-if="safeZoneResult" class="safe-zone-result">
        <div :class="['status-badge', safeZoneResult.safe ? 'status-ok' : 'status-warn']">
          {{ safeZoneResult.safe ? 'All elements within safe zone' : `${safeZoneResult.violations.length} element(s) outside` }}
        </div>
        <div v-if="!safeZoneResult.safe" class="violation-list">
          <div
            v-for="v in safeZoneResult.violations"
            :key="v.element_id"
            class="violation-item"
          >
            <AppIcon name="alertTriangle" :size="12" />
            <span>{{ v.element_id }}</span>
            <span class="overshoot">+{{ v.overshoot_px.toFixed(1) }}px</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Layer Assignment -->
    <div class="panel-section">
      <h3 class="section-title">Foreground / Background</h3>
      <div class="layer-assign">
        <div class="layer-field">
          <label class="field-label-sm">Foreground IDs (comma-separated)</label>
          <input
            v-model="fgIdsText"
            class="field-input"
            placeholder="e.g. shape-1, icon-2"
            @change="setForeground"
          />
        </div>
        <div class="layer-field">
          <label class="field-label-sm">Background IDs (comma-separated)</label>
          <input
            v-model="bgIdsText"
            class="field-input"
            placeholder="e.g. shape-3"
            @change="setBackground"
          />
        </div>
      </div>
      <div v-if="elements.length > 0" class="element-layer-list">
        <div v-for="el in elements" :key="el.id" class="element-layer-row">
          <span class="el-id">{{ el.id }}</span>
          <span class="el-type">{{ el.type }}</span>
          <span
            :class="['layer-tag', fgIds.includes(el.id) ? 'tag-fg' : bgIds.includes(el.id) ? 'tag-bg' : 'tag-none']"
          >
            {{ fgIds.includes(el.id) ? 'FG' : bgIds.includes(el.id) ? 'BG' : '--' }}
          </span>
        </div>
      </div>
    </div>

    <!-- Export -->
    <div class="panel-section">
      <h3 class="section-title">Export</h3>
      <button class="export-btn" :disabled="exporting" @click="exportAndroid">
        <span v-if="exporting" class="spinner"></span>
        <AppIcon v-else name="download" :size="14" />
        {{ exporting ? "Exporting..." : "Export Android Adaptive Icon" }}
      </button>
      <div v-if="exportResults.length > 0" class="export-results">
        <span class="result-count">{{ exportResults.length }} files exported</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.adaptive-preview {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.panel-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-secondary);
  margin: 0;
}

.preview-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.preview-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px 4px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: border-color var(--transition-fast), background var(--transition-fast);
}

.preview-card:hover {
  border-color: var(--accent-color);
}

.preview-card--active {
  border-color: var(--accent-color);
  background: var(--bg-hover);
}

.preview-frame {
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: repeating-conic-gradient(var(--bg-tertiary) 0% 25%, var(--bg-secondary) 0% 50%) 50% / 8px 8px;
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.preview-img {
  width: 64px;
  height: 64px;
  object-fit: contain;
}

.preview-placeholder {
  font-size: 11px;
  color: var(--text-muted);
}

.preview-label {
  font-size: 10px;
  color: var(--text-muted);
  text-align: center;
}

.safe-zone-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.field-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-secondary);
  flex: 1;
}

.field-input {
  flex: 1;
  height: 28px;
  padding: 0 8px;
  font-size: 11px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
  transition: border-color var(--transition-fast);
}

.field-input:focus {
  border-color: var(--accent);
}

.action-btn {
  flex-shrink: 0;
  height: 28px;
  padding: 0 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.action-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.status-badge {
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  font-size: 11px;
  line-height: 1.4;
}

.status-ok {
  color: var(--success);
  background: var(--success-muted);
}

.status-warn {
  color: var(--warning);
  background: var(--warning-muted);
}

.violation-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 4px;
}

.violation-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--danger);
  padding: 3px 6px;
  background: var(--danger-muted);
  border-radius: var(--radius-sm);
}

.overshoot {
  margin-left: auto;
  font-family: "JetBrains Mono", monospace;
  font-size: 10px;
}

.layer-assign {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.layer-field {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.field-label-sm {
  font-size: 10px;
  color: var(--text-muted);
}

.element-layer-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 120px;
  overflow-y: auto;
}

.element-layer-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 3px 6px;
  border-radius: var(--radius-sm);
  font-size: 11px;
}

.element-layer-row:hover {
  background: var(--bg-hover);
}

.el-id {
  color: var(--text-secondary);
  font-family: "JetBrains Mono", monospace;
  font-size: 10px;
}

.el-type {
  color: var(--text-muted);
  font-size: 10px;
}

.layer-tag {
  margin-left: auto;
  font-size: 9px;
  font-weight: 600;
  padding: 1px 5px;
  border-radius: 3px;
}

.tag-fg {
  color: var(--success);
  background: var(--success-muted);
}

.tag-bg {
  color: var(--accent);
  background: var(--accent-muted);
}

.tag-none {
  color: var(--text-muted);
  background: var(--bg-tertiary);
}

.export-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  height: 34px;
  background: var(--accent);
  border: 1px solid var(--accent);
  border-radius: var(--radius-md);
  color: var(--bg-primary);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.export-btn:hover:not(:disabled) {
  background: var(--accent-hover);
}

.export-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.export-results {
  margin-top: 4px;
}

.result-count {
  font-size: 10px;
  color: var(--success);
}

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--accent-muted);
  border-top-color: var(--bg-primary);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
