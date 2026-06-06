<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useExportStore, PNG_SIZES } from "@/stores/export";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import AppIcon from "@/components/common/AppIcon.vue";

const store = useExportStore();
const ui = useUiStore();
const project = useProjectStore();

// Animation export settings
const lottieFps = ref(30);
const gifFps = ref(15);
const gifSize = ref(512);

// Preview grid sizes - small subset for quick visual check
const previewSizes = [16, 32, 64, 128, 256];

function svgDataUrl(svgContent: string): string {
  return "data:image/svg+xml;charset=utf-8," + encodeURIComponent(svgContent);
}

const statusMessage = computed(() => {
  if (store.error) return store.error;
  if (store.exportResults.length > 0) return `已导出 ${store.exportResults.length} 个文件`;
  return "";
});

const statusType = computed(() => {
  if (store.error) return "error";
  if (store.exportResults.length > 0) return "success";
  return "";
});

const pngSizesForIco = [16, 32, 48, 256];

function toggleFormat(format: string) {
  const idx = store.selectedFormats.indexOf(format);
  if (idx >= 0) {
    store.selectedFormats.splice(idx, 1);
  } else {
    store.selectedFormats.push(format);
  }
}

function togglePngSize(size: number) {
  const idx = store.selectedPngSizes.indexOf(size);
  if (idx >= 0) {
    store.selectedPngSizes.splice(idx, 1);
  } else {
    store.selectedPngSizes.push(size);
  }
}

function selectAllPngSizes() {
  store.selectedPngSizes = [...PNG_SIZES];
}

function deselectAllPngSizes() {
  store.selectedPngSizes = [];
}

async function runExport(
  exportFn: () => Promise<string[]>,
  errorMsg: string = "导出失败"
) {
  if (!store.outputDir) {
    ui.showToast("Please select an output directory first", "warning");
    return;
  }
  if (store.exporting) return;
  store.exporting = true;
  store.error = null;
  try {
    store.exportResults = await exportFn();
  } catch (e: unknown) {
    store.error = e instanceof Error ? e.message : String(e);
  } finally {
    store.exporting = false;
  }
}

async function handleExportAll() {
  await runExport(() => store.exportAll(), "导出失败");
}

async function handleExportSvg() {
  await runExport(() => store.exportSvg(), "导出 SVG 失败");
}

async function handleExportPng() {
  if (store.selectedPngSizes.length === 0) {
    ui.showToast("Please select at least one PNG size", "warning");
    return;
  }
  await runExport(() => store.exportPng(), "导出 PNG 失败");
}

async function handleExportIco() {
  await runExport(() => store.exportIco(), "导出 ICO 失败");
}

async function handleAndroidIcons() {
  await runExport(() => store.exportAndroidIcons(), "导出 Android 图标失败");
}

async function handleIosIcons() {
  await runExport(() => store.exportIosIcons(), "导出 iOS 图标失败");
}

async function copySvgForFigma() {
  try {
    const svg = project.svgPreview;
    if (!svg) {
      ui.showToast('No SVG content to copy', 'warning');
      return;
    }
    await navigator.clipboard.writeText(svg);
    ui.showToast('SVG copied to clipboard — paste into Figma', 'success');
  } catch (e: unknown) {
    ui.showToast(`Failed: ${e instanceof Error ? e.message : String(e)}`, 'error');
  }
}

// Presets
const PRESETS_KEY = 'iconstudio-export-presets';

interface ExportPreset {
  name: string;
  formats: string[];
  pngSizes: number[];
}

const presetName = ref('');
const presets = ref<ExportPreset[]>([]);

function loadPresets() {
  try {
    const raw = localStorage.getItem(PRESETS_KEY);
    presets.value = raw ? JSON.parse(raw) : [];
  } catch { presets.value = []; }
}

function savePreset() {
  const name = presetName.value.trim();
  if (!name) return;
  const existing = presets.value.findIndex(p => p.name === name);
  const preset: ExportPreset = {
    name,
    formats: [...store.selectedFormats],
    pngSizes: [...store.selectedPngSizes],
  };
  if (existing >= 0) {
    presets.value[existing] = preset;
  } else {
    presets.value.push(preset);
  }
  localStorage.setItem(PRESETS_KEY, JSON.stringify(presets.value));
  presetName.value = '';
}

function applyPreset(preset: ExportPreset) {
  store.selectedFormats = [...preset.formats];
  store.selectedPngSizes = [...preset.pngSizes];
}

function deletePreset(index: number) {
  presets.value.splice(index, 1);
  localStorage.setItem(PRESETS_KEY, JSON.stringify(presets.value));
}

async function handleExportLottie() {
  if (!store.outputDir) {
    ui.showToast("Please select an output directory first", "warning");
    return;
  }
  if (store.exporting) return;
  store.exporting = true;
  store.error = null;
  try {
    const path = await invoke<string>("export_lottie", {
      outputPath: `${store.outputDir}/animation.json`,
    });
    store.exportResults = [path];
  } catch (e: unknown) {
    store.error = e instanceof Error ? e.message : String(e);
  } finally {
    store.exporting = false;
  }
}

async function handleExportGif() {
  if (!store.outputDir) {
    ui.showToast("Please select an output directory first", "warning");
    return;
  }
  if (store.exporting) return;
  store.exporting = true;
  store.error = null;
  try {
    const path = await invoke<string>("export_animated_gif", {
      outputPath: `${store.outputDir}/animation.gif`,
      fps: gifFps.value,
      width: gifSize.value,
      height: gifSize.value,
    });
    store.exportResults = [path];
  } catch (e: unknown) {
    store.error = e instanceof Error ? e.message : String(e);
  } finally {
    store.exporting = false;
  }
}

loadPresets();

</script>

<template>
  <div class="export-panel">
    <!-- Preview Grid -->
    <div class="section" v-if="project.svgPreview">
      <div class="section-label">Preview</div>
      <div class="preview-grid">
        <div v-for="size in previewSizes" :key="size" class="preview-item">
          <div class="preview-frame" :style="{ width: Math.min(size, 64) + 'px', height: Math.min(size, 64) + 'px' }">
            <img
              :src="svgDataUrl(project.svgPreview)"
              :style="{ width: Math.min(size, 64) + 'px', height: Math.min(size, 64) + 'px', imageRendering: size <= 32 ? 'pixelated' : 'auto' }"
            />
          </div>
          <span class="preview-label">{{ size }}×{{ size }}</span>
        </div>
      </div>
    </div>

    <!-- 导出格式 -->
    <div class="section">
      <div class="section-label">导出格式</div>
      <div class="format-list">
        <label class="checkbox-row">
          <input
            type="checkbox"
            :checked="store.selectedFormats.includes('svg')"
            @change="toggleFormat('svg')"
          />
          <span class="checkbox-label">SVG</span>
        </label>

        <label class="checkbox-row">
          <input
            type="checkbox"
            :checked="store.selectedFormats.includes('png')"
            @change="toggleFormat('png')"
          />
          <span class="checkbox-label">PNG</span>
        </label>

        <div v-if="store.selectedFormats.includes('png')" class="png-sizes">
          <div class="png-sizes-header">
            <span class="png-sizes-title">尺寸</span>
            <div class="png-sizes-actions">
              <button class="link-btn" @click="selectAllPngSizes">全选</button>
              <button class="link-btn" @click="deselectAllPngSizes">清除</button>
            </div>
          </div>
          <div class="size-chips">
            <label
              v-for="size in PNG_SIZES"
              :key="size"
              :class="['size-chip', { active: store.selectedPngSizes.includes(size) }]"
            >
              <input
                type="checkbox"
                :checked="store.selectedPngSizes.includes(size)"
                @change="togglePngSize(size)"
                class="hidden-checkbox"
              />
              {{ size }}
            </label>
          </div>
        </div>

        <label class="checkbox-row">
          <input
            type="checkbox"
            :checked="store.selectedFormats.includes('ico')"
            @change="toggleFormat('ico')"
          />
          <span class="checkbox-label">ICO</span>
          <span class="tag">16 · 32 · 48 · 256</span>
        </label>

        <label class="checkbox-row">
          <input
            type="checkbox"
            :checked="store.selectedFormats.includes('webp')"
            @change="toggleFormat('webp')"
          />
          <span class="checkbox-label">WebP</span>
        </label>

        <div class="anim-export-divider"></div>

        <div class="anim-export-row">
          <button
            class="pack-btn"
            :disabled="store.exporting"
            @click="handleExportLottie"
          >
            <AppIcon name="file-json" :size="14" />
            Lottie JSON
          </button>
        </div>
        <div v-if="true" class="anim-settings">
          <label class="setting-row">
            <span class="setting-label">FPS</span>
            <select v-model.number="lottieFps" class="setting-select">
              <option :value="24">24</option>
              <option :value="30">30</option>
              <option :value="60">60</option>
            </select>
          </label>
        </div>

        <div class="anim-export-row">
          <button
            class="pack-btn"
            :disabled="store.exporting"
            @click="handleExportGif"
          >
            <AppIcon name="film" :size="14" />
            Animated GIF
          </button>
        </div>
        <div class="anim-settings">
          <label class="setting-row">
            <span class="setting-label">FPS</span>
            <select v-model.number="gifFps" class="setting-select">
              <option :value="10">10</option>
              <option :value="15">15</option>
              <option :value="24">24</option>
              <option :value="30">30</option>
            </select>
          </label>
          <label class="setting-row">
            <span class="setting-label">Size</span>
            <select v-model.number="gifSize" class="setting-select">
              <option :value="128">128</option>
              <option :value="256">256</option>
              <option :value="512">512</option>
            </select>
          </label>
        </div>
      </div>
    </div>

    <!-- 输出目录 -->
    <div class="section">
      <div class="section-label">输出目录</div>
      <div class="output-dir-row">
        <span class="output-path">{{ store.outputDir || "未选择目录" }}</span>
        <button class="action-btn" @click="store.selectOutputDir">选择目录</button>
      </div>
    </div>

    <!-- Export Presets -->
    <div class="section">
      <div class="section-label">Export Presets</div>
      <div class="preset-save-row">
        <input
          v-model="presetName"
          class="preset-input"
          placeholder="Preset name..."
          @keydown.enter="savePreset"
        />
        <button class="action-btn" @click="savePreset" :disabled="!presetName.trim()">Save</button>
      </div>
      <div v-if="presets.length > 0" class="preset-list">
        <div v-for="(preset, idx) in presets" :key="preset.name" class="preset-item">
          <button class="preset-name" @click="applyPreset(preset)">{{ preset.name }}</button>
          <span class="preset-info">{{ preset.formats.join(', ') }} · {{ preset.pngSizes.length }} sizes</span>
          <button class="preset-delete" @click="deletePreset(idx)">×</button>
        </div>
      </div>
    </div>

    <!-- App 图标打包 -->
    <div class="section">
      <div class="section-label">App 图标打包</div>
      <div class="app-icon-buttons">
        <button
          class="pack-btn"
          :disabled="store.exporting"
          @click="handleAndroidIcons"
        >
          <span class="pack-icon"><AppIcon name="android" :size="16" /></span>
          Android 图标
        </button>
        <button
          class="pack-btn"
          :disabled="store.exporting"
          @click="handleIosIcons"
        >
          <span class="pack-icon"><AppIcon name="apple" :size="16" /></span>
          iOS 图标
        </button>
      </div>
    </div>

    <!-- 操作 -->
    <div class="section">
      <button
        class="export-all-btn"
        :disabled="store.exporting || store.selectedFormats.length === 0"
        @click="handleExportAll"
      >
        <span v-if="store.exporting" class="spinner"></span>
        <span v-else class="export-arrow"><AppIcon name="download" :size="14" /></span>
        {{ store.exporting ? "导出中…" : "一键导出" }}
      </button>
      <button class="figma-btn" @click="copySvgForFigma">
        <AppIcon name="copy" :size="14" />
        Copy SVG for Figma
      </button>
    </div>

    <!-- 状态消息 -->
    <div v-if="statusMessage" :class="['status-msg', statusType]">
      {{ statusMessage }}
    </div>
  </div>
</template>

<style scoped>
.export-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.section-label {
  padding: 0 12px 2px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--accent);
}

/* Preview grid */
.preview-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  padding: 0 8px;
  align-items: flex-end;
}

.preview-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.preview-frame {
  display: flex;
  align-items: center;
  justify-content: center;
  background: repeating-conic-gradient(var(--bg-tertiary) 0% 25%, var(--bg-secondary) 0% 50%) 50% / 8px 8px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.preview-frame img {
  display: block;
}

.preview-label {
  font-size: 10px;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

/* Format checkboxes */
.format-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 0 8px;
}

.checkbox-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 4px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: background var(--transition-fast) ease;
}

.checkbox-row:hover {
  background: var(--bg-hover);
}

.checkbox-row.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.checkbox-row input[type="checkbox"] {
  accent-color: var(--accent);
  width: 14px;
  height: 14px;
  cursor: pointer;
}

.checkbox-row.disabled input[type="checkbox"] {
  cursor: not-allowed;
}

.checkbox-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.tag {
  font-size: 10px;
  color: var(--text-muted);
  margin-left: auto;
  font-family: "JetBrains Mono", monospace;
}

.tag.upcoming {
  color: var(--warning);
  background: var(--warning-muted);
  padding: 1px 5px;
  border-radius: 4px;
}

/* PNG size chips */
.png-sizes {
  margin-left: 22px;
  padding: 4px 0 4px 0;
}

.png-sizes-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
}

.png-sizes-title {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.png-sizes-actions {
  display: flex;
  gap: 6px;
}

.link-btn {
  background: none;
  border: none;
  color: var(--accent);
  font-size: 10px;
  cursor: pointer;
  padding: 0;
  transition: color var(--transition-fast) ease;
}

.link-btn:hover {
  color: var(--accent-hover);
  text-decoration: underline;
}

.size-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.size-chip {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 42px;
  height: 26px;
  padding: 0 6px;
  font-size: 11px;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast) ease;
  user-select: none;
}

.size-chip:hover {
  border-color: var(--accent);
  color: var(--text-secondary);
}

.size-chip.active {
  background: var(--accent-muted);
  border-color: var(--accent);
  color: var(--accent);
}

.hidden-checkbox {
  display: none;
}

/* Output directory */
.output-dir-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 8px;
}

.output-path {
  flex: 1;
  font-size: 11px;
  color: var(--text-muted);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  padding: 5px 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.action-btn {
  flex-shrink: 0;
  height: 30px;
  padding: 0 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
  white-space: nowrap;
}

.action-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.action-btn:active {
  background: var(--accent);
  color: var(--bg-primary);
}

/* Presets */
.preset-save-row {
  display: flex;
  gap: 6px;
  padding: 0 8px;
}

.preset-input {
  flex: 1;
  height: 28px;
  padding: 0 8px;
  font-size: 11px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
  transition: border-color var(--transition-fast) ease;
}

.preset-input:focus {
  border-color: var(--accent);
}

.preset-input::placeholder {
  color: var(--text-muted);
}

.preset-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 4px 8px 0;
}

.preset-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 6px;
  border-radius: var(--radius-sm);
  transition: background var(--transition-fast) ease;
}

.preset-item:hover {
  background: var(--bg-hover);
}

.preset-name {
  background: none;
  border: none;
  color: var(--accent);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  padding: 0;
  transition: color var(--transition-fast) ease;
}

.preset-name:hover {
  color: var(--accent-hover);
  text-decoration: underline;
}

.preset-info {
  flex: 1;
  font-size: 10px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preset-delete {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 14px;
  cursor: pointer;
  padding: 0 2px;
  line-height: 1;
  transition: color var(--transition-fast) ease;
}

.preset-delete:hover {
  color: var(--danger);
}

/* App icon pack buttons */
.app-icon-buttons {
  display: flex;
  gap: 6px;
  padding: 0 8px;
}

.pack-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 34px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.pack-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.pack-btn:active:not(:disabled) {
  background: var(--accent);
  color: var(--bg-primary);
}

.pack-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pack-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  line-height: 0;
}

.pack-icon svg {
  width: 16px;
  height: 16px;
}

/* Export all button */
.export-all-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: calc(100% - 16px);
  margin: 0 8px;
  height: 38px;
  background: var(--accent);
  border: 1px solid var(--accent);
  border-radius: var(--radius-md);
  color: var(--bg-primary);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

/* Copy for Figma button */
.figma-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: calc(100% - 16px);
  margin: 0 8px;
  height: 32px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.figma-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.figma-btn:active {
  background: var(--accent);
  color: var(--bg-primary);
}

.export-all-btn:hover:not(:disabled) {
  background: var(--accent-hover);
  border-color: var(--accent-hover);
}

.export-all-btn:active:not(:disabled) {
  background: var(--accent-pressed);
  border-color: var(--accent-pressed);
}

.export-all-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.export-arrow {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 0;
}

.export-arrow svg {
  width: 14px;
  height: 14px;
}

/* Spinner */
.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--accent-muted);
  border-top-color: var(--bg-primary);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* Status message */
.status-msg {
  margin: 0 8px;
  padding: 6px 10px;
  font-size: 11px;
  border-radius: var(--radius-sm);
  line-height: 1.4;
}

.status-msg.success {
  color: var(--success);
  background: var(--success-muted);
  border: 1px solid var(--success-muted);
}

.status-msg.error {
  color: var(--danger);
  background: var(--danger-muted);
  border: 1px solid var(--danger-muted);
}

/* Animation export */
.anim-export-divider {
  height: 1px;
  background: var(--border-color);
  margin: 4px 12px;
}

.anim-export-row {
  padding: 0 8px;
}

.anim-settings {
  margin-left: 22px;
  display: flex;
  gap: 8px;
  padding: 2px 0;
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 4px;
}

.setting-label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.setting-select {
  height: 24px;
  padding: 0 4px;
  font-size: 11px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
  cursor: pointer;
}

.setting-select:focus {
  border-color: var(--accent);
}
</style>
