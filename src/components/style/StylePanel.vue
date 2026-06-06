<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "@/stores/project";
import GradientEditor from "./GradientEditor.vue";
import ShadowEditor from "./ShadowEditor.vue";
import FilterPanel from "./FilterPanel.vue";
import PaletteSuggest from "./PaletteSuggest.vue";
import AnimationPanel from "./AnimationPanel.vue";
import StylePresetPanel from "./StylePresetPanel.vue";
import type { Gradient, Shadow, SvgFilter } from "@/types";

const project = useProjectStore();

const element = computed(() => project.selectedElement);

const hasGradient = computed(() => {
  if (!element.value) return false;
  return "gradient" in element.value;
});

const hasShadow = computed(() => {
  if (!element.value) return false;
  return "shadows" in element.value;
});

const elementGradient = computed(() => {
  if (!element.value || !("gradient" in element.value)) return null;
  return element.value.gradient;
});

const elementShadow = computed(() => {
  if (!element.value || !("shadows" in element.value)) return null;
  const shadows = (element.value as { shadows: Shadow[] }).shadows;
  return shadows.length > 0 ? shadows[0] : null;
});

const elementFilter = computed(() => {
  if (!element.value) return null;
  return (element.value as { svg_filter?: SvgFilter | null }).svg_filter ?? null;
});

function onGradientUpdate(gradient: Gradient | null) {
  if (!element.value) return;
  if (gradient) {
    project.setGradient(element.value.id, gradient.type, gradient.colors, gradient.angle);
  } else {
    project.clearGradient(element.value.id);
  }
}

function onShadowUpdate(shadow: Shadow | null) {
  if (!element.value) return;
  if (shadow) {
    project.setShadow(element.value.id, shadow.color, shadow.blur, shadow.offset_x, shadow.offset_y);
  } else {
    project.clearShadow(element.value.id);
  }
}

function onFilterUpdate(filter: SvgFilter | null) {
  if (!element.value) return;
  if (filter) {
    project.setFilter(element.value.id, filter.filter_type, filter.params);
  } else {
    project.clearFilter(element.value.id);
  }
}

function onApplyColor(color: string) {
  if (!element.value) return;
  project.updateElement(element.value.id, { fill: color });
}

// Color Variants — generate palettes from all schemes using the project's primary color
interface PaletteVariant {
  name: string;
  colors: string[];
}

const paletteVariants = ref<PaletteVariant[]>([]);
const loadingPalettes = ref(false);

const schemes = [
  { value: "complementary", label: "互补" },
  { value: "analogous", label: "类似" },
  { value: "triadic", label: "三角" },
  { value: "split-complementary", label: "分裂互补" },
  { value: "monochromatic", label: "单色" },
];

async function generatePalettes() {
  loadingPalettes.value = true;
  paletteVariants.value = [];
  try {
    // Get primary color from analysis
    const analysis = await invoke<{
      all_colors: { hex: string; usage_count: number; element_ids: string[] }[];
      primary: { hex: string; usage_count: number; element_ids: string[] } | null;
    }>("analyze_colors");
    const baseColor = analysis.primary?.hex || "#4488FF";

    // Generate palettes for all schemes in parallel
    const results = await Promise.all(
      schemes.map(async (s) => {
        try {
          const colors = await invoke<string[]>("suggest_palette", {
            baseColor: baseColor,
            scheme: s.value,
            count: 5,
          });
          return { name: s.label, colors };
        } catch {
          return null;
        }
      })
    );

    paletteVariants.value = results.filter((r): r is PaletteVariant => r !== null);
  } catch (e) {
    console.error("Failed to generate palettes:", e);
  } finally {
    loadingPalettes.value = false;
  }
}

async function applyPalette(colors: string[]) {
  try {
    const colorCount = colors.length;
    const elementsList = project.elements;
    if (!elementsList || elementsList.length === 0) return;

    await Promise.all(
      elementsList
        .filter((el) => el.type !== "image" && el.type !== "group")
        .map((el, i, arr) => {
          const color = colors[i % colorCount];
          return invoke("set_props", { elementId: el.id, props: { fill: color } });
        })
    );
    await project.refreshElements();
  } catch (e) {
    console.error("Failed to apply palette:", e);
  }
}

// Saved Palettes
const SAVED_PALETTES_KEY = 'iconstudio-saved-palettes';

interface SavedPalette {
  name: string;
  colors: string[];
}

const savedPalettes = ref<SavedPalette[]>([]);
const newPaletteName = ref('');

function loadSavedPalettes() {
  try {
    const raw = localStorage.getItem(SAVED_PALETTES_KEY);
    savedPalettes.value = raw ? JSON.parse(raw) : [];
  } catch { savedPalettes.value = []; }
}

function saveCurrentAsPalette(colors: string[]) {
  const name = newPaletteName.value.trim() || `Palette ${savedPalettes.value.length + 1}`;
  savedPalettes.value.push({ name, colors: [...colors] });
  localStorage.setItem(SAVED_PALETTES_KEY, JSON.stringify(savedPalettes.value));
  newPaletteName.value = '';
}

function deleteSavedPalette(index: number) {
  savedPalettes.value.splice(index, 1);
  localStorage.setItem(SAVED_PALETTES_KEY, JSON.stringify(savedPalettes.value));
}

// Load on mount
loadSavedPalettes();
</script>

<template>
  <div class="style-panel">
    <div v-if="!element" class="empty-state">
      <span class="empty-icon">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 22a7 7 0 0 0 7-7c0-2-1-3.9-3-5.5s-3.5-4-4-6.5c-.5 2.5-2 4.9-4 6.5C6 11.1 5 13 5 15a7 7 0 0 0 7 7z"/></svg>
      </span>
      <p>选择一个元素以编辑样式</p>
    </div>
    <div v-else class="style-content">
      <StylePresetPanel />
      <GradientEditor
        v-if="hasGradient"
        :element-id="element.id"
        :gradient="elementGradient"
        @update="onGradientUpdate"
      />
      <ShadowEditor
        v-if="hasShadow"
        :element-id="element.id"
        :shadow="elementShadow"
        @update="onShadowUpdate"
      />
      <FilterPanel
        :element-id="element.id"
        :filter="elementFilter"
        @update="onFilterUpdate"
      />
      <AnimationPanel />
      <PaletteSuggest @apply-color="onApplyColor" />

      <!-- Color Variants -->
      <div class="section">
        <div class="section-header">
          <span class="section-title">Apply Palette</span>
        </div>
        <div class="section-body">
          <button class="btn-generate" :disabled="loadingPalettes" @click="generatePalettes">
            {{ loadingPalettes ? "Generating…" : "Generate Variants" }}
          </button>
          <div v-if="paletteVariants.length" class="palette-list">
            <div v-for="palette in paletteVariants" :key="palette.name" class="palette-item">
              <div class="palette-header">
                <span class="palette-name">{{ palette.name }}</span>
                <div class="palette-actions">
                  <button class="apply-btn" @click="applyPalette(palette.colors)">Apply</button>
                  <button class="apply-btn save-btn" @click="saveCurrentAsPalette(palette.colors)">Save</button>
                </div>
              </div>
              <div class="palette-colors">
                <div
                  v-for="(color, idx) in palette.colors"
                  :key="idx"
                  class="color-swatch"
                  :style="{ background: color }"
                  :title="color"
                ></div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Saved Palettes -->
      <div class="section">
        <div class="section-header">
          <span class="section-title">Saved Palettes</span>
        </div>
        <div class="section-body">
          <div class="palette-save-row">
            <input
              v-model="newPaletteName"
              class="palette-name-input"
              placeholder="Palette name..."
            />
          </div>
          <div v-if="savedPalettes.length" class="palette-list">
            <div v-for="(palette, idx) in savedPalettes" :key="idx" class="palette-item">
              <div class="palette-header">
                <span class="palette-name">{{ palette.name }}</span>
                <div class="palette-actions">
                  <button class="apply-btn" @click="applyPalette(palette.colors)">Apply</button>
                  <button class="save-btn" @click="saveCurrentAsPalette(palette.colors)">Dup</button>
                  <button class="delete-btn" @click="deleteSavedPalette(idx)">×</button>
                </div>
              </div>
              <div class="palette-colors">
                <div
                  v-for="(color, ci) in palette.colors"
                  :key="ci"
                  class="color-swatch"
                  :style="{ background: color }"
                  :title="color"
                ></div>
              </div>
            </div>
          </div>
          <div v-else class="empty-palettes">
            <span>Generate and save palettes above</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.style-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 12px;
  color: var(--text-muted);
}

.empty-icon {
  opacity: 0.3;
}

.empty-state p {
  font-size: 13px;
  margin: 0;
  text-align: center;
}

.style-content {
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  gap: 0;
}

/* Section (used by Color Variants) */
.section {
  border-top: 1px solid var(--bg-tertiary);
  padding: 12px 0;
}

.section-header {
  padding: 0 12px 6px;
}

.section-title {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--accent);
}

.section-body {
  padding: 0 12px;
}

.btn-generate {
  width: 100%;
  height: 30px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.btn-generate:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--text-primary);
}

.btn-generate:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.palette-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 10px;
}

.palette-item {
  padding: 8px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
}

.palette-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.palette-name {
  font-size: 11px;
  color: var(--text-secondary);
  font-weight: 500;
}

.apply-btn {
  background: none;
  border: 1px solid var(--accent);
  color: var(--accent);
  font-size: 10px;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.apply-btn:hover {
  background: var(--accent);
  color: var(--bg-primary);
}

.palette-colors {
  display: flex;
  gap: 4px;
}

.color-swatch {
  width: 24px;
  height: 24px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-color);
}

.palette-save-row {
  display: flex;
  gap: 6px;
  margin-bottom: 8px;
}

.palette-name-input {
  flex: 1;
  height: 26px;
  padding: 0 8px;
  font-size: 11px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
}

.palette-name-input:focus {
  border-color: var(--accent);
}

.palette-name-input::placeholder {
  color: var(--text-muted);
}

.palette-actions {
  display: flex;
  gap: 4px;
}

.save-btn {
  font-size: 9px;
  padding: 2px 6px;
}

.delete-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  padding: 0 2px;
  line-height: 1;
  transition: color var(--transition-fast) ease;
}

.delete-btn:hover {
  color: var(--danger);
}

.empty-palettes {
  text-align: center;
  padding: 8px;
  font-size: 11px;
  color: var(--text-muted);
}
</style>
