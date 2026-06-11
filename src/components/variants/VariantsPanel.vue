<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useVariantsStore } from "@/stores/variantsStore";
import { useUiStore } from "@/stores/ui";
import { useExportStore } from "@/stores/export";
import type { ThemeRule } from "@/types";

const store = useVariantsStore();
const ui = useUiStore();
const exportStore = useExportStore();

const showAddDialog = ref(false);
const newVariantName = ref("");
const selectedPreset = ref("");
const customRules = ref<ThemeRule[]>([]);
const previewSvgs = ref<Record<number, string>>({});

const hasCustomRules = computed(() => customRules.value.length > 0);

function svgDataUrl(svg: string): string {
  return "data:image/svg+xml;charset=utf-8," + encodeURIComponent(svg);
}

async function loadPreviews() {
  for (let i = 0; i < store.variants.length; i++) {
    try {
      if (!previewSvgs.value[i]) {
        previewSvgs.value[i] = await store.previewVariant(i);
      }
    } catch {
      // skip failed previews
    }
  }
}

async function handleGenerateAllPresets() {
  await store.generateAllPresets();
  previewSvgs.value = {};
  await loadPreviews();
}

function openAddDialog() {
  newVariantName.value = "";
  selectedPreset.value = "";
  customRules.value = [];
  showAddDialog.value = true;
}

async function handleCreateVariant() {
  const name = newVariantName.value.trim();
  if (!name) {
    ui.showToast("Please enter a variant name", "warning");
    return;
  }

  let rules: ThemeRule[];
  if (selectedPreset.value) {
    const preset = store.presets.find(p => p.name === selectedPreset.value);
    rules = preset ? preset.rules : [];
  } else if (hasCustomRules.value) {
    rules = customRules.value;
  } else {
    ui.showToast("Please select a preset or add custom rules", "warning");
    return;
  }

  await store.createVariant(name, rules);
  showAddDialog.value = false;
  previewSvgs.value = {};
  await loadPreviews();
}

async function handleDelete(index: number) {
  await store.deleteVariant(index);
  previewSvgs.value = {};
  await loadPreviews();
}

async function handleExportVariant(index: number) {
  if (!exportStore.outputDir) {
    ui.showToast("Please set an output directory in the Export panel first", "warning");
    return;
  }
  const paths = await store.exportVariant(index, "svg", exportStore.outputDir);
  if (paths.length > 0) {
    ui.showToast(`Exported ${paths.length} file(s)`, "success");
  }
}

async function handleExportAll() {
  if (!exportStore.outputDir) {
    ui.showToast("Please set an output directory in the Export panel first", "warning");
    return;
  }
  const paths = await store.exportAllVariants("svg", exportStore.outputDir);
  if (paths.length > 0) {
    ui.showToast(`Exported ${paths.length} variant file(s)`, "success");
  }
}

function addReplaceColorRule() {
  customRules.value.push({ replaceColor: { from: "#000000", to: "#FFFFFF" } });
}

function addAdjustOpacityRule() {
  customRules.value.push({ adjustOpacity: { factor: 0.5 } });
}

function addDesaturateRule() {
  customRules.value.push({ desaturate: { factor: 0.5 } });
}

function addCustomFillRule() {
  customRules.value.push({ customFill: { color: "#000000" } });
}

function addInvertRule() {
  customRules.value.push("invertColors");
}

function addGrayscaleRule() {
  customRules.value.push("grayscale");
}

function removeRule(index: number) {
  customRules.value.splice(index, 1);
}

onMounted(async () => {
  await Promise.all([store.fetchVariants(), store.fetchPresets()]);
  await loadPreviews();
});
</script>

<template>
  <div class="variants-panel">
    <div class="panel-header">
      <h3>Theme Variants</h3>
      <div class="header-actions">
        <button class="icon-btn" @click="handleGenerateAllPresets" :disabled="store.loading" title="Generate all presets">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v4m0 12v4M4.93 4.93l2.83 2.83m8.48 8.48l2.83 2.83M2 12h4m12 0h4M4.93 19.07l2.83-2.83m8.48-8.48l2.83-2.83"/></svg>
        </button>
        <button class="icon-btn" @click="openAddDialog" title="Add variant">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        </button>
      </div>
    </div>

    <!-- Variant list -->
    <div v-if="store.variants.length > 0" class="variant-list">
      <div v-for="(variant, idx) in store.variants" :key="idx" class="variant-card">
        <div class="variant-preview">
          <img
            v-if="previewSvgs[idx]"
            :src="svgDataUrl(previewSvgs[idx])"
            class="preview-img"
          />
          <div v-else class="preview-placeholder">?</div>
        </div>
        <div class="variant-info">
          <span class="variant-name">{{ variant.name }}</span>
          <span class="variant-rules">{{ variant.rules.length }} rule(s)</span>
        </div>
        <div class="variant-actions">
          <button class="card-btn" @click="handleExportVariant(idx)" title="Export">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
          </button>
          <button class="card-btn danger" @click="handleDelete(idx)" title="Delete">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>
      </div>
    </div>

    <div v-else class="empty-state">
      <p>No variants yet</p>
      <button class="link-btn" @click="handleGenerateAllPresets">Generate all presets</button>
    </div>

    <!-- Batch export -->
    <div v-if="store.variants.length > 1" class="batch-section">
      <button class="action-btn full" @click="handleExportAll" :disabled="store.loading">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        Export All Variants
      </button>
    </div>

    <!-- Add variant dialog -->
    <div v-if="showAddDialog" class="dialog-overlay" @click.self="showAddDialog = false">
      <div class="dialog">
        <div class="dialog-header">
          <h4>Add Variant</h4>
          <button class="icon-btn" @click="showAddDialog = false">×</button>
        </div>

        <div class="dialog-body">
          <div class="field">
            <label>Name</label>
            <input v-model="newVariantName" placeholder="e.g. Dark Mode" class="input" />
          </div>

          <div class="field">
            <label>Preset</label>
            <select v-model="selectedPreset" class="input">
              <option value="">— Custom —</option>
              <option v-for="p in store.presets" :key="p.name" :value="p.name">{{ p.name }}</option>
            </select>
          </div>

          <div v-if="!selectedPreset" class="custom-rules">
            <div class="rules-header">
              <label>Custom Rules</label>
              <div class="rule-add-btns">
                <button class="tag-btn" @click="addInvertRule">Invert</button>
                <button class="tag-btn" @click="addGrayscaleRule">Gray</button>
                <button class="tag-btn" @click="addReplaceColorRule">Replace</button>
                <button class="tag-btn" @click="addAdjustOpacityRule">Opacity</button>
                <button class="tag-btn" @click="addDesaturateRule">Desat</button>
                <button class="tag-btn" @click="addCustomFillRule">Fill</button>
              </div>
            </div>

            <div v-for="(rule, ri) in customRules" :key="ri" class="rule-row">
              <span class="rule-label">{{ typeof rule === 'string' ? rule : Object.keys(rule)[0] }}</span>
              <template v-if="typeof rule !== 'string' && 'replaceColor' in rule">
                <input v-model="rule.replaceColor.from" class="mini-input" placeholder="#from" />
                <span class="arrow">→</span>
                <input v-model="rule.replaceColor.to" class="mini-input" placeholder="#to" />
              </template>
              <template v-if="typeof rule !== 'string' && 'adjustOpacity' in rule">
                <input type="range" v-model.number="rule.adjustOpacity.factor" min="0" max="2" step="0.1" class="slider" />
                <span class="val">{{ rule.adjustOpacity.factor.toFixed(1) }}</span>
              </template>
              <template v-if="typeof rule !== 'string' && 'desaturate' in rule">
                <input type="range" v-model.number="rule.desaturate.factor" min="0" max="1" step="0.1" class="slider" />
                <span class="val">{{ rule.desaturate.factor.toFixed(1) }}</span>
              </template>
              <template v-if="typeof rule !== 'string' && 'customFill' in rule">
                <input type="color" v-model="rule.customFill.color" class="color-input" />
              </template>
              <button class="remove-btn" @click="removeRule(ri)">×</button>
            </div>
          </div>
        </div>

        <div class="dialog-footer">
          <button class="action-btn" @click="showAddDialog = false">Cancel</button>
          <button class="action-btn primary" @click="handleCreateVariant" :disabled="!newVariantName.trim() || (!selectedPreset && customRules.length === 0)">Create</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.variants-panel {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.panel-header h3 {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 4px;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast) ease;
  font-size: 16px;
  line-height: 1;
}

.icon-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.icon-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Variant list */
.variant-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.variant-card {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  transition: border-color var(--transition-fast) ease;
}

.variant-card:hover {
  border-color: var(--accent);
}

.variant-preview {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-sm);
  background: repeating-conic-gradient(var(--bg-tertiary) 0% 25%, var(--bg-secondary) 0% 50%) 50% / 6px 6px;
  overflow: hidden;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.preview-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.preview-placeholder {
  font-size: 14px;
  color: var(--text-muted);
}

.variant-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.variant-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.variant-rules {
  font-size: 10px;
  color: var(--text-muted);
}

.variant-actions {
  display: flex;
  gap: 2px;
}

.card-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: none;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.card-btn:hover {
  border-color: var(--border-color);
  color: var(--text-primary);
  background: var(--bg-tertiary);
}

.card-btn.danger:hover {
  color: var(--danger);
  border-color: var(--danger);
}

/* Empty state */
.empty-state {
  text-align: center;
  padding: 20px 0;
  color: var(--text-muted);
}

.empty-state p {
  font-size: 12px;
  margin: 0 0 8px;
}

.link-btn {
  background: none;
  border: none;
  color: var(--accent);
  font-size: 11px;
  cursor: pointer;
  padding: 0;
}

.link-btn:hover {
  text-decoration: underline;
}

/* Batch export */
.batch-section {
  padding-top: 4px;
  border-top: 1px solid var(--border-color);
}

.action-btn {
  height: 30px;
  padding: 0 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.action-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn.primary {
  background: var(--accent);
  color: var(--bg-primary);
  border-color: var(--accent);
}

.action-btn.primary:hover:not(:disabled) {
  background: var(--accent-hover);
}

.action-btn.full {
  width: 100%;
  justify-content: center;
}

/* Dialog */
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.dialog {
  width: 340px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
}

.dialog-header h4 {
  margin: 0;
  font-size: 13px;
  color: var(--text-primary);
}

.dialog-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.input {
  height: 28px;
  padding: 0 8px;
  font-size: 11px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
}

.input:focus {
  border-color: var(--accent);
}

/* Custom rules */
.custom-rules {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.rules-header {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.rules-header label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
}

.rule-add-btns {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tag-btn {
  height: 22px;
  padding: 0 8px;
  font-size: 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.tag-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.rule-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
}

.rule-label {
  font-size: 10px;
  font-weight: 500;
  color: var(--accent);
  min-width: 52px;
}

.mini-input {
  width: 64px;
  height: 22px;
  padding: 0 4px;
  font-size: 10px;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
}

.arrow {
  color: var(--text-muted);
  font-size: 10px;
}

.slider {
  flex: 1;
  accent-color: var(--accent);
}

.val {
  font-size: 10px;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
  min-width: 24px;
}

.color-input {
  width: 28px;
  height: 22px;
  padding: 0;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  cursor: pointer;
  background: none;
}

.remove-btn {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 14px;
  border-radius: var(--radius-sm);
}

.remove-btn:hover {
  color: var(--danger);
  background: var(--danger-muted);
}
</style>
