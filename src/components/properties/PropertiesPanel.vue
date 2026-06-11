<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from "vue";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";
import type { Element, TextElement, IconElement, ShapeElement, PathElement, GroupElement, FontInfo } from "@/types";
import AppIcon from "@/components/common/AppIcon.vue";

const project = useProjectStore();
const ui = useUiStore();

const element = computed(() => project.selectedElement);

const isLocked = computed(() => element.value?.locked === true);
const isText = computed(() => element.value?.type === "text");
const isIcon = computed(() => element.value?.type === "icon");
const isShape = computed(() => element.value?.type === "shape");
const isPath = computed(() => element.value?.type === "path");
const isGroup = computed(() => element.value?.type === "group");

// Font picker state
const fonts = ref<FontInfo[]>([]);
const fontSearch = ref("");
const fontDropdownOpen = ref(false);

onMounted(async () => {
  fonts.value = await project.listFonts();
});

async function searchFonts(keyword: string) {
  fontSearch.value = keyword;
  fonts.value = await project.listFonts(keyword || undefined);
}

function selectFont(family: string) {
  if (!element.value) return;
  updateProp("font_family", family);
  fontDropdownOpen.value = false;
  fontSearch.value = "";
}

function closeFontDropdown(e: MouseEvent) {
  if (fontDropdownOpen.value) {
    const target = e.target as HTMLElement;
    if (!target.closest('.font-dropdown-container')) {
      fontDropdownOpen.value = false;
    }
  }
}

onMounted(() => {
  document.addEventListener('click', closeFontDropdown);
});

onUnmounted(() => {
  document.removeEventListener('click', closeFontDropdown);
  if (batchTimer) { clearTimeout(batchTimer); batchTimer = null; }
});

const pendingProps = ref<Record<string, unknown>>({});
let batchTimer: ReturnType<typeof setTimeout> | null = null;

watch(element, () => {
  pendingProps.value = {};
  if (batchTimer) { clearTimeout(batchTimer); batchTimer = null; }
});

function updateProp(key: string, value: unknown) {
  if (!element.value) return;
  const id = element.value.id;
  pendingProps.value[key] = value;
  if (batchTimer) clearTimeout(batchTimer);
  batchTimer = setTimeout(() => {
    if (element.value && element.value.id === id) {
      project.updateElement(id, { ...pendingProps.value });
    }
    pendingProps.value = {};
  }, 200);
}

function typeLabel(el: Element): string {
  switch (el.type) {
    case "shape":
      return `Shape · ${(el as ShapeElement).shape_type}`;
    case "text":
      return `Text`;
    case "icon":
      return `Icon · ${(el as IconElement).name}`;
    case "image":
      return "Image";
    case "path":
      return "Path · Drawing";
    case "group":
      return `Group · ${(el as GroupElement).children.length} items`;
  }
}

// Alignment
function alignCenter() {
  if (!element.value) return;
  const cx = project.canvasWidth / 2 - element.value.width / 2;
  project.updateElement(element.value.id, { x: Math.round(cx) });
}
function alignMiddle() {
  if (!element.value) return;
  const cy = project.canvasHeight / 2 - element.value.height / 2;
  project.updateElement(element.value.id, { y: Math.round(cy) });
}
function sizeToCanvas() {
  if (!element.value) return;
  project.updateElement(element.value.id, {
    x: 0,
    y: 0,
    width: project.canvasWidth,
    height: project.canvasHeight,
  });
}

function ungroupElement() {
  if (!element.value || element.value.type !== "group") return;
  project.ungroup(element.value.id);
}

function handlePathBlur(event: FocusEvent) {
  if (!element.value || element.value.type !== "path") return;
  const newValue = (event.target as HTMLTextAreaElement).value;
  const el = element.value as PathElement;
  if (newValue !== el.d) {
    project.updateElement(el.id, { d: newValue });
  }
}

// Boolean operations
const booleanTargetId = ref("");
const isBooleanable = computed(() => isShape.value || isPath.value);
const booleanTargets = computed(() =>
  project.elements.filter(e => e.id !== element.value?.id && (e.type === "shape" || e.type === "path"))
);

function runBoolean(op: string) {
  if (!element.value || !booleanTargetId.value) return;
  project.booleanOp(element.value.id, booleanTargetId.value, op);
  booleanTargetId.value = "";
}

// Overlay
const positions = [
  { value: "topLeft", label: "TL" },
  { value: "topRight", label: "TR" },
  { value: "bottomLeft", label: "BL" },
  { value: "bottomRight", label: "BR" },
];

function toggleOverlay(event: Event) {
  if (!element.value) return;
  const checked = (event.target as HTMLInputElement).checked;
  if (checked) {
    updateProp("overlay", { kind: "add", position: "bottomRight", color: "#FF0000", size_ratio: 0.4 });
  } else {
    updateProp("overlay", null);
  }
}

function updateOverlayProp(key: string, value: unknown) {
  if (!element.value?.overlay) return;
  updateProp("overlay", { ...element.value.overlay, [key]: value });
}
</script>

<template>
  <div class="properties-panel">
    <div v-if="element" class="properties-content">
      <div v-if="isLocked" class="locked-banner">
        <AppIcon name="lock" :size="14" />
        <span>This element is locked</span>
        <button class="unlock-btn" @click="updateProp('locked', false)">Unlock</button>
      </div>
      <div :class="{ 'controls-disabled': isLocked }">
        <div class="section-header">
          <span class="section-title">{{ typeLabel(element) }}</span>
          <span class="element-id">{{ element.id }}</span>
        </div>

      <!-- Position -->
      <div class="prop-group">
        <div class="prop-group-label">Position</div>
        <div class="prop-row">
          <label class="prop-label">X</label>
          <input
            type="number"
            class="prop-input"
            :value="element.x"
            @input="updateProp('x', Number(($event.target as HTMLInputElement).value))"
          />
        </div>
        <div class="prop-row">
          <label class="prop-label">Y</label>
          <input
            type="number"
            class="prop-input"
            :value="element.y"
            @input="updateProp('y', Number(($event.target as HTMLInputElement).value))"
          />
        </div>
        <div class="prop-row">
          <button class="align-btn" title="Center Horizontal" @click="alignCenter">H-Center</button>
          <button class="align-btn" title="Center Vertical" @click="alignMiddle">V-Center</button>
          <button class="align-btn" title="Fit to Canvas" @click="sizeToCanvas">Fill</button>
        </div>
      </div>

      <!-- Size -->
      <div class="prop-group">
        <div class="prop-group-label">Size</div>
        <div class="prop-row">
          <label class="prop-label">W</label>
          <input
            type="number"
            class="prop-input"
            :value="element.width"
            @input="updateProp('width', Number(($event.target as HTMLInputElement).value))"
          />
        </div>
        <div class="prop-row">
          <label class="prop-label">H</label>
          <input
            type="number"
            class="prop-input"
            :value="element.height"
            @input="updateProp('height', Number(($event.target as HTMLInputElement).value))"
          />
        </div>
      </div>

      <!-- Fill & Stroke -->
      <div class="prop-group" v-if="element.type !== 'image' && element.type !== 'group'">
        <div class="prop-group-label">Appearance</div>
        <div class="prop-row" v-if="!isPath">
          <label class="prop-label">Fill</label>
          <div class="color-input-wrap">
            <input
              type="color"
              class="color-swatch"
              :value="element.fill"
              @input="updateProp('fill', ($event.target as HTMLInputElement).value)"
            />
            <input
              type="text"
              class="color-text"
              :value="element.fill"
              @change="updateProp('fill', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </div>
        <div class="prop-row" v-if="isPath">
          <label class="prop-label">Fill</label>
          <input
            type="text"
            class="prop-input"
            :value="(element as PathElement).fill"
            @change="updateProp('fill', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="prop-row" v-if="'stroke' in element">
          <label class="prop-label">Stroke</label>
          <div class="color-input-wrap">
            <input
              type="color"
              class="color-swatch"
              :value="element.stroke || '#000000'"
              @input="updateProp('stroke', ($event.target as HTMLInputElement).value)"
            />
            <input
              type="text"
              class="color-text"
              :value="element.stroke || 'none'"
              @change="updateProp('stroke', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </div>
        <div class="prop-row" v-if="'stroke_width' in element">
          <label class="prop-label">Stroke W</label>
          <input
            type="number"
            class="prop-input"
            :value="element.stroke_width"
            min="0"
            step="1"
            @input="updateProp('stroke_width', Number(($event.target as HTMLInputElement).value))"
          />
        </div>
        <div class="prop-row" v-if="'stroke_dasharray' in element">
          <label class="prop-label">Dash</label>
          <input
            type="text"
            class="prop-input"
            :value="element.stroke_dasharray || ''"
            placeholder="e.g. 5,3"
            @change="updateProp('stroke_dasharray', ($event.target as HTMLInputElement).value || null)"
          />
        </div>
      </div>

      <!-- Transform -->
      <div class="prop-group">
        <div class="prop-group-label">Transform</div>
        <div class="prop-row">
          <label class="prop-label">Opacity</label>
          <input
            type="range"
            class="prop-range"
            :value="element.opacity"
            min="0"
            max="1"
            step="0.01"
            @input="updateProp('opacity', Number(($event.target as HTMLInputElement).value))"
          />
          <span class="range-value">{{ Math.round(element.opacity * 100) }}%</span>
        </div>
        <div class="prop-row">
          <label class="prop-label">Rotation</label>
          <input
            type="number"
            class="prop-input"
            :value="element.rotation"
            step="1"
            @input="updateProp('rotation', Number(($event.target as HTMLInputElement).value))"
          />
          <span class="range-value">deg</span>
        </div>
        <div class="prop-row">
          <label class="prop-label">Blend</label>
          <select
            class="prop-select"
            :value="element.blend_mode || ''"
            @change="updateProp('blend_mode', ($event.target as HTMLSelectElement).value || null)"
          >
            <option value="">Normal</option>
            <option value="multiply">Multiply</option>
            <option value="screen">Screen</option>
            <option value="overlay">Overlay</option>
            <option value="darken">Darken</option>
            <option value="lighten">Lighten</option>
            <option value="color-dodge">Color Dodge</option>
            <option value="color-burn">Color Burn</option>
            <option value="hard-light">Hard Light</option>
            <option value="soft-light">Soft Light</option>
            <option value="difference">Difference</option>
            <option value="exclusion">Exclusion</option>
          </select>
        </div>
      </div>

      <!-- Text-specific -->
      <div class="prop-group" v-if="isText">
        <div class="prop-group-label">Text</div>
        <div class="prop-row">
          <label class="prop-label">Content</label>
          <input
            type="text"
            class="prop-input"
            :value="(element as TextElement).content"
            @input="updateProp('content', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="prop-row font-row">
          <label class="prop-label">Font</label>
          <div class="font-picker font-dropdown-container">
            <button
              class="font-trigger"
              @click="fontDropdownOpen = !fontDropdownOpen"
            >
              {{ (element as TextElement).font_family }}
              <span class="font-arrow">▾</span>
            </button>
            <div v-if="fontDropdownOpen" class="font-dropdown">
              <input
                type="text"
                class="font-search"
                placeholder="Search fonts..."
                :value="fontSearch"
                @input="searchFonts(($event.target as HTMLInputElement).value)"
              />
              <div class="font-list">
                <button
                  v-for="font in fonts.slice(0, 50)"
                  :key="font.family"
                  :class="['font-option', { active: font.family === (element as TextElement).font_family }]"
                  @click="selectFont(font.family)"
                >
                  {{ font.family }}
                </button>
              </div>
            </div>
          </div>
        </div>
        <div class="prop-row">
          <label class="prop-label">Size</label>
          <input
            type="number"
            class="prop-input"
            :value="(element as TextElement).font_size"
            min="1"
            @input="updateProp('font_size', Number(($event.target as HTMLInputElement).value))"
          />
        </div>
        <div class="prop-row">
          <label class="prop-label">Weight</label>
          <select
            class="prop-select"
            :value="(element as TextElement).font_weight"
            @change="updateProp('font_weight', ($event.target as HTMLSelectElement).value)"
          >
            <option value="300">Light (300)</option>
            <option value="400">Regular (400)</option>
            <option value="500">Medium (500)</option>
            <option value="600">Semibold (600)</option>
            <option value="700">Bold (700)</option>
            <option value="900">Black (900)</option>
          </select>
        </div>
        <div class="prop-row">
          <label class="prop-label">Spacing</label>
          <input
            type="number"
            class="prop-input"
            :value="(element as TextElement).letter_spacing"
            step="0.5"
            @input="updateProp('letter_spacing', Number(($event.target as HTMLInputElement).value))"
          />
        </div>
      </div>

      <!-- Group-specific -->
      <div class="prop-group" v-if="isGroup">
        <div class="prop-group-label">Group</div>
        <div class="prop-row">
          <label class="prop-label">Children</label>
          <span class="range-value">{{ (element as GroupElement).children.length }}</span>
        </div>
        <button class="browse-icons-btn" @click="ungroupElement">
          Ungroup
        </button>
      </div>

      <!-- Path data editor -->
      <div class="prop-group" v-if="isPath">
        <div class="prop-group-label">Path Data</div>
        <textarea
          class="path-editor"
          :value="(element as PathElement).d"
          @blur="handlePathBlur($event)"
          rows="6"
          spellcheck="false"
          placeholder="M 0 0 L 100 0 L 100 100 Z"
        ></textarea>
      </div>

      <!-- Icon-specific -->
      <div class="prop-group" v-if="isIcon">
        <div class="prop-group-label">Icon</div>
        <div class="prop-row">
          <label class="prop-label">Name</label>
          <input
            type="text"
            class="prop-input"
            :value="(element as IconElement).name"
            @change="updateProp('name', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <button class="browse-icons-btn" @click="ui.toggleIconBrowser(true)">
          Browse Icons...
        </button>
      </div>

      <!-- Overlay -->
      <div class="prop-group">
        <div class="prop-group-label">Overlay</div>
        <div class="prop-row">
          <label class="prop-label">Enable</label>
          <input
            type="checkbox"
            :checked="element.overlay != null"
            @change="toggleOverlay($event)"
          />
        </div>
        <template v-if="element.overlay">
          <div class="prop-row">
            <label class="prop-label">Type</label>
            <select
              class="prop-select"
              :value="element.overlay.kind"
              @change="updateOverlayProp('kind', ($event.target as HTMLSelectElement).value)"
            >
              <option value="add">+ Add</option>
              <option value="remove">- Remove</option>
              <option value="check">✓ Check</option>
              <option value="info">i Info</option>
              <option value="warning">! Warning</option>
              <option value="error">✕ Error</option>
              <option value="star">★ Star</option>
              <option value="lock">🔒 Lock</option>
              <option value="new">New</option>
              <option value="custom">Custom</option>
            </select>
          </div>
          <div class="prop-row">
            <label class="prop-label">Position</label>
            <div class="position-grid">
              <button
                v-for="pos in positions"
                :key="pos.value"
                :class="['pos-btn', { active: element.overlay!.position === pos.value }]"
                :title="pos.label"
                @click="updateOverlayProp('position', pos.value)"
              >{{ pos.label.charAt(0) }}</button>
            </div>
          </div>
          <div class="prop-row">
            <label class="prop-label">Color</label>
            <div class="color-input-wrap">
              <input
                type="color"
                class="color-swatch"
                :value="element.overlay.color || '#FF0000'"
                @input="updateOverlayProp('color', ($event.target as HTMLInputElement).value)"
              />
              <input
                type="text"
                class="color-text"
                :value="element.overlay.color || '#FF0000'"
                @change="updateOverlayProp('color', ($event.target as HTMLInputElement).value)"
              />
            </div>
          </div>
          <div class="prop-row">
            <label class="prop-label">Size</label>
            <input
              type="range"
              class="prop-range"
              :value="element.overlay.size_ratio ?? 0.4"
              min="0.2"
              max="0.6"
              step="0.02"
              @input="updateOverlayProp('size_ratio', Number(($event.target as HTMLInputElement).value))"
            />
            <span class="range-value">{{ Math.round((element.overlay.size_ratio ?? 0.4) * 100) }}%</span>
          </div>
          <div class="prop-row" v-if="element.overlay.kind === 'custom'">
            <label class="prop-label">SVG Path</label>
            <input
              type="text"
              class="prop-input"
              :value="element.overlay.custom_path || ''"
              placeholder="M0,0 L10,10..."
              @change="updateOverlayProp('custom_path', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </template>
      </div>

      <!-- Clip / Mask -->
      <div class="prop-group" v-if="element.type !== 'group'">
        <div class="prop-group-label">Clip &amp; Mask</div>
        <div class="prop-row">
          <label class="prop-label">Clip</label>
          <span class="range-value">{{ element.clip_element_id || 'none' }}</span>
          <button v-if="element.clip_element_id" class="align-btn" @click="project.clearClip(element.id)">Clear</button>
        </div>
        <div class="prop-row" v-if="!element.clip_element_id">
          <select class="prop-select" :value="''" @change="($event.target as HTMLSelectElement).value && project.setClip(element.id, ($event.target as HTMLSelectElement).value)">
            <option value="">Select clip element...</option>
            <option v-for="el in project.elements.filter(e => e.id !== element.id)" :key="el.id" :value="el.id">
              {{ el.id }} ({{ el.type }})
            </option>
          </select>
        </div>
        <div class="prop-row">
          <label class="prop-label">Mask</label>
          <span class="range-value">{{ element.mask_element_id || 'none' }}</span>
          <button v-if="element.mask_element_id" class="align-btn" @click="project.clearMask(element.id)">Clear</button>
        </div>
        <div class="prop-row" v-if="!element.mask_element_id">
          <select class="prop-select" :value="''" @change="($event.target as HTMLSelectElement).value && project.setMask(element.id, ($event.target as HTMLSelectElement).value)">
            <option value="">Select mask element...</option>
            <option v-for="el in project.elements.filter(e => e.id !== element.id)" :key="el.id" :value="el.id">
              {{ el.id }} ({{ el.type }})
            </option>
          </select>
        </div>
      </div>

      <!-- Boolean Operations -->
      <div class="prop-group" v-if="isBooleanable">
        <div class="prop-group-label">Boolean</div>
        <div class="prop-row">
          <label class="prop-label">With</label>
          <select class="prop-select" v-model="booleanTargetId">
            <option value="">Select element...</option>
            <option v-for="el in booleanTargets" :key="el.id" :value="el.id">
              {{ el.id }} ({{ el.type }})
            </option>
          </select>
        </div>
        <div class="prop-row boolean-buttons" v-if="booleanTargetId">
          <button class="boolean-btn" @click="runBoolean('union')" title="Union: A + B merged">∪</button>
          <button class="boolean-btn" @click="runBoolean('subtract')" title="Subtract: A minus B">−</button>
          <button class="boolean-btn" @click="runBoolean('intersect')" title="Intersect: A ∩ B">∩</button>
          <button class="boolean-btn" @click="runBoolean('exclude')" title="Exclude: A XOR B">⊕</button>
        </div>
      </div>
      </div>
    </div>

    <div v-else class="properties-empty">
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
      <span>Select an element to edit properties</span>
    </div>
  </div>
</template>

<style scoped>
.properties-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.properties-content {
  overflow-y: auto;
  padding: 8px 0;
}

.properties-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 8px;
  color: var(--text-muted);
}

.properties-empty svg {
  opacity: 0.4;
}

.properties-empty span {
  font-size: 12px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 12px 8px;
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-primary);
}

.element-id {
  font-size: 10px;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

.prop-group {
  padding: 4px 12px 6px;
  border-bottom: 1px solid var(--bg-tertiary);
}

.prop-group:last-child {
  border-bottom: none;
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
  transition: border-color var(--transition-fast) ease, box-shadow var(--transition-fast) ease;
}

.prop-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-muted);
}

.prop-input[type="number"] {
  -moz-appearance: textfield;
}

.prop-input[type="number"]::-webkit-inner-spin-button,
.prop-input[type="number"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
}

.color-input-wrap {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 4px;
}

.color-swatch {
  width: 28px;
  height: 28px;
  padding: 0;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: none;
  cursor: pointer;
}

.color-swatch::-webkit-color-swatch-wrapper {
  padding: 2px;
}

.color-swatch::-webkit-color-swatch {
  border: none;
  border-radius: 2px;
}

.color-text {
  flex: 1;
  height: 26px;
  padding: 0 8px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 11px;
  font-family: "JetBrains Mono", monospace;
  outline: none;
  transition: border-color var(--transition-fast) ease, box-shadow var(--transition-fast) ease;
}

.color-text:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-muted);
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

.range-value {
  width: 32px;
  text-align: right;
  font-size: 11px;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

.prop-select {
  flex: 1;
  height: 26px;
  padding: 0 6px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
  transition: border-color var(--transition-fast) ease, box-shadow var(--transition-fast) ease;
}

.prop-select:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-muted);
}

.browse-icons-btn {
  width: 100%;
  height: 28px;
  margin-top: 4px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.browse-icons-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* Alignment buttons */
.align-btn {
  flex: 1;
  height: 24px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  font-size: 10px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.align-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

/* Font picker */
.font-row {
  position: relative;
}

.font-picker {
  flex: 1;
  position: relative;
}

.font-trigger {
  width: 100%;
  height: 26px;
  padding: 0 8px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  text-align: left;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
  transition: border-color var(--transition-fast) ease;
}

.font-trigger:hover {
  border-color: var(--accent);
}

.font-arrow {
  font-size: 10px;
  color: var(--text-muted);
}

.font-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: 100;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 255, 255, 0.06);
  max-height: 200px;
  display: flex;
  flex-direction: column;
}

.font-search {
  width: 100%;
  height: 26px;
  padding: 0 8px;
  background: var(--input-bg);
  border: none;
  border-bottom: 1px solid var(--border-color);
  border-radius: var(--radius-md) var(--radius-md) 0 0;
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
}

.font-list {
  flex: 1;
  overflow-y: auto;
  max-height: 170px;
}

.font-option {
  width: 100%;
  padding: 5px 8px;
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 11px;
  text-align: left;
  cursor: pointer;
  transition: background var(--transition-fast) ease;
}

.font-option:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.font-option.active {
  background: var(--accent-muted);
  color: var(--accent);
}

.path-editor {
  width: calc(100% - 16px);
  margin: 0 8px;
  padding: 8px;
  font-family: "JetBrains Mono", "Consolas", monospace;
  font-size: 11px;
  line-height: 1.5;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  resize: vertical;
  outline: none;
  transition: border-color var(--transition-fast) ease;
}

.path-editor:focus {
  border-color: var(--accent);
}

.boolean-buttons {
  gap: 4px;
}

.boolean-btn {
  flex: 1;
  height: 28px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 14px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.boolean-btn:hover {
  background: var(--accent-muted);
  border-color: var(--accent);
  color: var(--accent);
}

.locked-banner {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  margin: 4px 8px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-muted);
  font-size: 11px;
}

.unlock-btn {
  margin-left: auto;
  padding: 2px 8px;
  background: var(--accent-muted);
  border: 1px solid var(--accent);
  border-radius: var(--radius-sm);
  color: var(--accent);
  font-size: 10px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.unlock-btn:hover {
  background: var(--accent);
  color: white;
}

.controls-disabled {
  pointer-events: none;
  opacity: 0.5;
}

.position-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 3px;
  flex: 1;
}

.pos-btn {
  height: 22px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  font-size: 10px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.pos-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.pos-btn.active {
  background: var(--accent-muted);
  border-color: var(--accent);
  color: var(--accent);
}
</style>
