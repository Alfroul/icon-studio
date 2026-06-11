<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { useExportStore } from "@/stores/export";
import type { CanvasResult, Element, TextElement, ThemePreset } from "@/types";

const ui = useUiStore();
const project = useProjectStore();
const exportStore = useExportStore();

interface BuiltinTemplate {
  meta: { name: string; description: string; is_builtin: boolean };
  preview_svg: string;
}

interface UserTemplate {
  meta: { name: string; description: string; is_builtin: boolean };
}

const activeTab = ref<"templates" | "presets">("templates");
const builtinTemplates = ref<BuiltinTemplate[]>([]);
const userTemplates = ref<UserTemplate[]>([]);
const themePresets = ref<ThemePreset[]>([]);
const saving = ref(false);
const templateName = ref("");
const showSaveDialog = ref(false);
const loading = ref(true);

onMounted(async () => {
  await loadTemplates();
});

async function loadTemplates() {
  loading.value = true;
  try {
    builtinTemplates.value = await invoke<BuiltinTemplate[]>("list_builtin_templates");
    userTemplates.value = await invoke<UserTemplate[]>("list_user_templates_cmd");
    themePresets.value = await invoke<ThemePreset[]>("list_theme_presets");
  } catch (e) {
    console.error("Failed to load templates:", e);
  } finally {
    loading.value = false;
  }
}

async function applyBuiltin(index: number) {
  if (project.elements.length > 0) {
    if (!confirm("Applying a template will replace the current project. Continue?")) return;
  }
  try {
    const canvas = await invoke<CanvasResult>("apply_builtin_template", { index });
    await project.syncCanvas(canvas);
    await project.refreshElements();
    await ui.initUndoState();
    ui.showToast("Template applied", "success");
  } catch (e) {
    ui.showToast(`Failed: ${e}`, "error");
  }
}

async function applyUser(name: string) {
  if (project.elements.length > 0) {
    if (!confirm("Applying a template will replace the current project. Continue?")) return;
  }
  try {
    const canvas = await invoke<CanvasResult>("apply_user_template", { name });
    await project.syncCanvas(canvas);
    await project.refreshElements();
    await ui.initUndoState();
    ui.showToast("Template applied", "success");
  } catch (e) {
    ui.showToast(`Failed: ${e}`, "error");
  }
}

async function deleteUser(name: string) {
  if (!confirm(`Delete template "${name}"?`)) return;
  try {
    await invoke("delete_user_template", { name });
    ui.showToast("Template deleted", "success");
    await loadTemplates();
  } catch (e) {
    ui.showToast(`Failed: ${e}`, "error");
  }
}

function startSave() {
  templateName.value = "";
  showSaveDialog.value = true;
}

async function confirmSave() {
  if (!templateName.value.trim()) return;
  saving.value = true;
  try {
    await invoke("save_as_template", {
      name: templateName.value.trim(),
      description: null,
    });
    ui.showToast("Template saved", "success");
    showSaveDialog.value = false;
    await loadTemplates();
  } catch (e) {
    ui.showToast(`Failed: ${e}`, "error");
  } finally {
    saving.value = false;
  }
}

// Theme presets
const customPresetName = ref("");
const customPresetShape = ref("squircle");
const customPresetRadius = ref(10);
const customPresetBg = ref("#FFFFFF");
const showCustomPresetDialog = ref(false);

async function applyThemePreset(presetId: string) {
  try {
    await invoke("apply_theme_preset", { presetId });
    await project.refreshElements();
    ui.showToast("Theme preset applied", "success");
  } catch (e) {
    ui.showToast(`Failed: ${e}`, "error");
  }
}

function startCustomPreset() {
  customPresetName.value = "";
  customPresetShape.value = "squircle";
  customPresetRadius.value = 10;
  customPresetBg.value = "#FFFFFF";
  showCustomPresetDialog.value = true;
}

async function confirmSaveCustomPreset() {
  if (!customPresetName.value.trim()) return;
  try {
    await invoke("save_custom_theme_preset", {
      name: customPresetName.value.trim(),
      shape: customPresetShape.value,
      cornerRadius: customPresetRadius.value,
      background: customPresetBg.value,
      paddingRatio: 0.10,
    });
    ui.showToast("Custom preset saved", "success");
    showCustomPresetDialog.value = false;
    await loadTemplates();
  } catch (e) {
    ui.showToast(`Failed: ${e}`, "error");
  }
}

// Batch generation
const batchLabels = ref("");
const batchFill = ref("");
const batchRunning = ref(false);
const batchProgress = ref("");
const batchResults = ref<string[]>([]);

async function runBatchGenerate() {
  const labels = batchLabels.value.trim().split("\n").filter((l) => l.trim());
  if (labels.length === 0) return;

  const outputDir = exportStore.outputDir;
  if (!outputDir) {
    ui.showToast("Please select an output directory in the Export panel first", "warning");
    return;
  }

  batchRunning.value = true;
  batchResults.value = [];
  batchProgress.value = `0/${labels.length}`;

  // Save full project state for restoration
  let savedProject: string | null = null;

  try {
    savedProject = await invoke<string>("save_project_to_string");

    for (let i = 0; i < labels.length; i++) {
      const label = labels[i].trim();

      // Find first text element and update its content
      const elements = await invoke<Element[]>("list_elements");
      const textEl = elements.find((e) => e.type === "text");
      if (textEl) {
        const props: Record<string, unknown> = { content: label };
        if (batchFill.value) props.fill = batchFill.value;
        await invoke("set_props", { elementId: textEl.id, props });
      }

      // Export PNG to a label-named subdirectory to avoid overwriting
      const labelDir = `${outputDir}/${label}`;
      const paths = await invoke<string[]>("export_png", {
        sizes: [512],
        outputDir: labelDir,
      });
      const pathStr = paths.length > 0 ? paths[paths.length - 1] : "ok";
      batchResults.value.push(`${label}: ${pathStr}`);
      batchProgress.value = `${i + 1}/${labels.length}`;
    }
    ui.showToast(`Batch complete: ${labels.length} icons`, "success");
  } catch (e) {
    ui.showToast(`Batch failed: ${e}`, "error");
  } finally {
    // Restore full project state from saved snapshot
    if (savedProject) {
      try {
        await invoke("restore_project_from_string", { data: savedProject });
        await project.refreshElements();
        await ui.initUndoState();
      } catch {
        ui.showToast("Failed to restore project state after batch", "error");
      }
    }
    batchRunning.value = false;
  }
}
</script>

<template>
  <div class="templates-panel">
    <div v-if="loading" class="loading">Loading templates...</div>

    <template v-else>
      <div class="tab-bar">
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'templates' }"
          @click="activeTab = 'templates'"
        >Templates</button>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'presets' }"
          @click="activeTab = 'presets'"
        >Theme Presets</button>
      </div>

      <template v-if="activeTab === 'presets'">
        <div class="section">
          <div class="section-header">
            <span>Built-in Presets</span>
          </div>
          <div class="template-grid">
            <div
              v-for="preset in themePresets"
              :key="preset.id"
              class="template-card preset-card"
              @click="applyThemePreset(preset.id)"
            >
              <img
                v-if="preset.previewSvg"
                class="template-preview"
                :src="'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(preset.previewSvg)"
                :alt="preset.name"
              />
              <div v-else class="preset-placeholder">?</div>
              <div class="template-name">{{ preset.name }}</div>
            </div>
          </div>
        </div>

        <div class="save-section">
          <button v-if="!showCustomPresetDialog" class="btn-primary" @click="startCustomPreset">
            Save Custom Preset
          </button>
          <div v-else class="save-dialog">
            <input
              v-model="customPresetName"
              placeholder="Preset name"
              class="input"
            />
            <div class="preset-options-row">
              <select v-model="customPresetShape" class="input">
                <option value="squircle">Squircle</option>
                <option value="circle">Circle</option>
                <option value="roundedRect">Rounded Rect</option>
                <option value="square">Square</option>
                <option value="hexagon">Hexagon</option>
                <option value="shield">Shield</option>
              </select>
              <input type="number" v-model.number="customPresetRadius" class="input" placeholder="Radius %" min="0" max="50" />
              <input type="color" v-model="customPresetBg" />
            </div>
            <div class="save-dialog-actions">
              <button class="btn-primary" :disabled="!customPresetName.trim()" @click="confirmSaveCustomPreset">Save</button>
              <button class="btn-secondary" @click="showCustomPresetDialog = false">Cancel</button>
            </div>
          </div>
        </div>
      </template>

      <template v-else>
      <div class="section">
        <div class="section-header">
          <span>Built-in Templates</span>
        </div>
        <div class="template-grid">
          <div
            v-for="(tmpl, i) in builtinTemplates"
            :key="i"
            class="template-card"
            @click="applyBuiltin(i)"
          >
            <img
              class="template-preview"
              :src="'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(tmpl.preview_svg)"
              :alt="tmpl.meta.name"
            />
            <div class="template-name">{{ tmpl.meta.name }}</div>
          </div>
        </div>
      </div>

      <div class="section">
        <div class="section-header">
          <span>User Templates</span>
        </div>
        <div v-if="userTemplates.length === 0" class="empty">
          No saved templates yet.
        </div>
        <div
          v-for="tmpl in userTemplates"
          :key="tmpl.meta.name"
          class="user-template-item"
        >
          <span class="user-template-name">{{ tmpl.meta.name }}</span>
          <div class="user-template-actions">
            <button class="btn-small" @click="applyUser(tmpl.meta.name)">Apply</button>
            <button class="btn-small btn-danger" @click="deleteUser(tmpl.meta.name)">Delete</button>
          </div>
        </div>
      </div>

      <div class="save-section">
        <button v-if="!showSaveDialog" class="btn-primary" @click="startSave">
          Save Current as Template
        </button>
        <div v-else class="save-dialog">
          <input
            v-model="templateName"
            placeholder="Template name"
            class="input"
            @keyup.enter="confirmSave"
          />
          <div class="save-dialog-actions">
            <button class="btn-primary" :disabled="saving || !templateName.trim()" @click="confirmSave">
              {{ saving ? "Saving..." : "Save" }}
            </button>
            <button class="btn-secondary" @click="showSaveDialog = false">Cancel</button>
          </div>
        </div>
      </div>

      <!-- Batch Generation -->
      <div class="section">
        <div class="section-header">
          <span>Batch Generate</span>
        </div>
        <textarea
          class="batch-input"
          v-model="batchLabels"
          rows="4"
          placeholder="One label per line&#10;Home&#10;Search&#10;Settings"
          :disabled="batchRunning"
        ></textarea>
        <div class="batch-color-row">
          <label class="batch-label">Fill</label>
          <input type="color" class="batch-color" v-model="batchFill" />
          <button
            class="batch-btn"
            @click="runBatchGenerate"
            :disabled="batchRunning || !batchLabels.trim()"
          >
            {{ batchRunning ? batchProgress : "Generate" }}
          </button>
        </div>
        <div v-if="batchResults.length" class="batch-results">
          <div v-for="r in batchResults" :key="r" class="batch-result">{{ r }}</div>
        </div>
      </div>
      </template>
    </template>
  </div>
</template>

<style scoped>
.templates-panel {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow-y: auto;
  height: 100%;
}

.loading,
.empty {
  color: var(--text-muted);
  font-size: 12px;
  padding: 8px 0;
}

.section-header {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--accent);
  margin-bottom: 8px;
}

.template-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.template-card {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 8px;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.template-card:hover {
  border-color: var(--accent);
  box-shadow: 0 0 12px var(--accent-glow);
  transform: translateY(-1px);
}

.template-preview {
  width: 100%;
  aspect-ratio: 1;
  overflow: hidden;
  border-radius: 4px;
}
.template-preview :deep(svg) {
  width: 100%;
  height: 100%;
}

.template-name {
  font-size: 10px;
  color: var(--text-muted);
  margin-top: 4px;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.user-template-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  margin-bottom: 4px;
}

.user-template-name {
  font-size: 12px;
  color: var(--text-primary);
}

.user-template-actions {
  display: flex;
  gap: 4px;
}

.btn-small {
  padding: 2px 8px;
  font-size: 11px;
  background: var(--bg-hover);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}
.btn-small:hover {
  background: var(--bg-active);
}
.btn-danger {
  color: var(--danger);
}

.save-section {
  padding-top: 8px;
  border-top: 1px solid var(--border-color);
}

.save-dialog {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.save-dialog-actions {
  display: flex;
  gap: 8px;
}

.input {
  height: 28px;
  padding: 0 10px;
  background: var(--input-bg);
  color: var(--text-primary);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-md);
  font-size: 12px;
  outline: none;
  transition: all var(--transition-fast);
}
.input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-muted);
}

.btn-primary {
  padding: 8px 16px;
  background: var(--accent);
  color: var(--bg-primary);
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  transition: all var(--transition-fast);
}
.btn-primary:hover {
  background: var(--accent-hover);
}
.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  padding: 8px 16px;
  background: var(--bg-hover);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: 12px;
  transition: all var(--transition-fast);
}
.btn-secondary:hover {
  background: var(--bg-active);
}

.batch-input {
  width: calc(100% - 24px);
  margin: 0 12px 8px;
  padding: 8px;
  font-size: 11px;
  line-height: 1.5;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  resize: vertical;
  outline: none;
  font-family: inherit;
}
.batch-input:focus {
  border-color: var(--accent);
}

.batch-color-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px 8px;
}

.batch-label {
  font-size: 11px;
  color: var(--text-muted);
}

.batch-color {
  width: 28px;
  height: 28px;
  padding: 0;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: none;
  cursor: pointer;
}

.batch-btn {
  margin-left: auto;
  height: 28px;
  padding: 0 12px;
  background: var(--accent);
  border: none;
  border-radius: var(--radius-md);
  color: var(--bg-primary);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}
.batch-btn:hover:not(:disabled) {
  background: var(--accent-hover);
}
.batch-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.batch-results {
  padding: 4px 12px;
  font-size: 10px;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

.batch-result {
  padding: 2px 0;
}

.tab-bar {
  display: flex;
  gap: 2px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  padding: 2px;
  margin-bottom: 4px;
}

.tab-btn {
  flex: 1;
  padding: 6px 8px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.tab-btn.active {
  background: var(--accent);
  color: var(--bg-primary);
}

.tab-btn:hover:not(.active) {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.preset-card {
  text-align: center;
}

.preset-placeholder {
  width: 100%;
  aspect-ratio: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-hover);
  border-radius: 4px;
  color: var(--text-muted);
  font-size: 18px;
}

.preset-options-row {
  display: flex;
  gap: 4px;
  align-items: center;
}

.preset-options-row select.input {
  flex: 1;
}

.preset-options-row input[type="number"] {
  width: 60px;
}

.preset-options-row input[type="color"] {
  width: 28px;
  height: 28px;
  padding: 0;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: none;
  cursor: pointer;
}
</style>
