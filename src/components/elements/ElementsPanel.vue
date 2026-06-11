<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";
import { useDragReorder } from "@/composables/useDragReorder";
import type { Element, ShapeElement, TextElement, IconElement, PathElement, GroupElement, SymbolElement } from "@/types";
import AddShapeBar from "./AddShapeBar.vue";
import IconBrowser from "./IconBrowser.vue";
import AppIcon from "@/components/common/AppIcon.vue";

const project = useProjectStore();
const ui = useUiStore();

const { dragIndex, dropIndex, isDragging, onDragStart, onDragOver, onDrop, onDragEnd } = useDragReorder({
  onReorder(fromIndex, toIndex) {
    const el = project.elements[fromIndex];
    if (el) project.reorderElement(el.id, toIndex);
  },
});

const listboxRef = ref<HTMLElement | null>(null);

onMounted(() => {
  project.refreshElements();
});

function elementLabel(el: Element): string {
  switch (el.type) {
    case "shape": {
      const s = el as ShapeElement;
      return `${capitalize(s.shape_type)} · ${s.fill}`;
    }
    case "text": {
      const t = el as TextElement;
      const preview = t.content.length > 12 ? t.content.slice(0, 12) + "…" : t.content;
      return `Text · ${preview}`;
    }
    case "icon": {
      const i = el as IconElement;
      return `Icon · ${i.name}`;
    }
    case "image":
      return "Image";
    case "path":
      return "Path · Drawing";
    case "group": {
      const g = el as GroupElement;
      return `Group · ${g.children.length} items`;
    }
    case "symbol": {
      const s = el as SymbolElement;
      const overrides = s.overrides.length > 0 ? ` (${s.overrides.length} overrides)` : "";
      return `Symbol${overrides}`;
    }
  }
}

const shapeIconNames: Record<string, string> = {
  circle: "circle",
  rect: "square",
  "rounded-rect": "roundedSquare",
  hexagon: "hexagon",
  star: "star",
  shield: "shield",
  diamond: "diamond",
};

const typeIconNames: Record<string, string> = {
  text: "type",
  icon: "plusSquare",
  image: "imagePlus",
  path: "pathType",
  group: "layers",
  symbol: "box",
};

function elementIconName(el: Element): string {
  if (el.type === "shape") {
    return shapeIconNames[(el as ShapeElement).shape_type] ?? "square";
  }
  return typeIconNames[el.type] ?? "square";
}

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function selectElement(id: string) {
  ui.selectElement(id);
}

function toggleExpand(el: Element) {
  if (el.type === "group") {
    project.updateElement(el.id, { expanded: !(el as GroupElement).expanded });
  }
}

function moveUp(index: number) {
  if (index <= 0) return;
  const el = project.elements[index];
  project.reorderElement(el.id, index - 1);
}

function moveDown(index: number) {
  if (index >= project.elements.length - 1) return;
  const el = project.elements[index];
  project.reorderElement(el.id, index + 1);
}

function deleteSelected() {
  if (!ui.selectedElementId) return;
  const el = project.elements.find(e => e.id === ui.selectedElementId);
  if (el && !el.locked) project.removeElement(ui.selectedElementId);
}

const canGroup = computed(() => {
  const selected = project.elements.filter(e => ui.selectedElementIds.has(e.id));
  return selected.length >= 2;
});

const hasMultiSelection = computed(() => ui.selectedElementIds.size >= 2);

const isGroupSelected = computed(() => {
  if (!ui.selectedElementId) return false;
  const el = project.elements.find(e => e.id === ui.selectedElementId);
  return el?.type === "group";
});

function groupSelected() {
  const ids = project.elements
    .filter(e => ui.selectedElementIds.has(e.id))
    .map(e => e.id);
  if (ids.length >= 2) {
    project.groupElements(ids);
  }
}

function ungroupSelected() {
  if (!ui.selectedElementId) return;
  const el = project.elements.find(e => e.id === ui.selectedElementId);
  if (el?.type === "group") {
    project.ungroup(el.id);
  }
}

function focusItem(index: number) {
  if (!listboxRef.value) return;
  const items = listboxRef.value.querySelectorAll<HTMLElement>('[role="option"]');
  items[index]?.focus();
}

function onListKeydown(e: KeyboardEvent, index: number) {
  const len = project.elements.length;
  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (index < len - 1) focusItem(index + 1);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    if (index > 0) focusItem(index - 1);
  } else if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    selectElement(project.elements[index].id);
  } else if (e.key === "Delete" || e.key === "Backspace") {
    e.preventDefault();
    const el = project.elements[index];
    if (!el.locked) project.removeElement(el.id);
  }
}

function toggleVisible(el: Element) {
  project.updateElement(el.id, { visible: el.visible === false });
}

function toggleLocked(el: Element) {
  project.updateElement(el.id, { locked: !el.locked });
}

async function cleanSelectedSvg() {
  if (!project.svgPreview) {
    ui.showToast("No SVG to clean", "warning");
    return;
  }
  await project.cleanSvg(project.svgPreview);
}

// Batch overlay
const overlayModalOpen = ref(false);
const overlayKind = ref("add");
const overlayPosition = ref("bottomRight");
const overlayColor = ref("#FF0000");

function openOverlayModal() {
  overlayKind.value = "add";
  overlayPosition.value = "bottomRight";
  overlayColor.value = "#FF0000";
  overlayModalOpen.value = true;
}

async function applyBatchOverlay() {
  const ids = project.elements
    .filter(e => ui.selectedElementIds.has(e.id))
    .map(e => e.id);
  if (ids.length === 0) return;

  const overlay = {
    kind: overlayKind.value,
    position: overlayPosition.value,
    color: overlayColor.value,
    size_ratio: 0.4,
  };

  for (const id of ids) {
    project.updateElement(id, { overlay });
  }
  overlayModalOpen.value = false;
  ui.showToast(`Overlay applied to ${ids.length} element(s)`, "success");
}

async function removeBatchOverlay() {
  const ids = project.elements
    .filter(e => ui.selectedElementIds.has(e.id))
    .map(e => e.id);
  if (ids.length === 0) return;

  for (const id of ids) {
    project.updateElement(id, { overlay: null });
  }
  ui.showToast(`Overlay removed from ${ids.length} element(s)`, "success");
}
</script>

<template>
  <div class="elements-panel">
    <AddShapeBar />

    <div ref="listboxRef" class="elements-list" role="listbox" aria-label="Elements">
      <div v-if="project.elements.length === 0" class="empty-state">
        No elements yet. Add a shape above.
      </div>
      <template v-for="(el, index) in project.elements" :key="el.id">
        <!-- Top-level element -->
        <div
          :class="['element-item', {
            selected: ui.selectedElementId === el.id,
            'is-locked': el.locked,
            'is-hidden': el.visible === false,
            dragging: dragIndex === index,
            'drag-over-above': dropIndex === index && dragIndex !== null && dragIndex !== index,
            'drag-over-below': dropIndex === index + 1 && dragIndex !== null && dragIndex !== index,
          }]"
          :style="{ opacity: isDragging && dragIndex === index ? 0.5 : undefined }"
          :draggable="!el.locked"
          role="option"
          :aria-selected="ui.selectedElementId === el.id"
          tabindex="0"
          @click="selectElement(el.id)"
          @keydown="onListKeydown($event, index)"
          @dragstart="onDragStart(index, $event)"
          @dragover="onDragOver(index, $event)"
          @drop="onDrop(index, $event)"
          @dragend="onDragEnd"
        >
          <button v-if="el.type === 'group'" class="expand-btn" @click.stop="toggleExpand(el)">
            <AppIcon :name="(el as GroupElement).expanded ? 'chevronDown' : 'chevronRight'" :size="10" />
          </button>
          <span class="element-icon"><AppIcon :name="elementIconName(el)" :size="14" /></span>
          <span v-if="el.type === 'symbol'" class="symbol-badge">&#x1F517;</span>
          <span class="element-label">{{ elementLabel(el) }}</span>
          <div class="element-toggles">
            <button
              class="toggle-btn"
              :class="{ active: el.visible === false }"
              title="Toggle Visibility"
              @click.stop="toggleVisible(el)"
            >
              <AppIcon :name="el.visible === false ? 'eyeOff' : 'eye'" :size="12" />
            </button>
            <button
              class="toggle-btn"
              :class="{ active: el.locked }"
              title="Toggle Lock"
              @click.stop="toggleLocked(el)"
            >
              <AppIcon :name="el.locked ? 'lock' : 'unlock'" :size="12" />
            </button>
          </div>
          <div v-if="ui.selectedElementId === el.id" class="element-actions">
            <button
              class="reorder-btn"
              title="Move Up"
              :disabled="index === 0"
              @click.stop="moveUp(index)"
            >
              <AppIcon name="chevronUp" :size="10" />
            </button>
            <button
              class="reorder-btn"
              title="Move Down"
              :disabled="index === project.elements.length - 1"
              @click.stop="moveDown(index)"
            >
              <AppIcon name="chevronDown" :size="10" />
            </button>
          </div>
        </div>

        <!-- Group children (indented, shown when expanded) -->
        <template v-if="el.type === 'group' && (el as GroupElement).expanded">
          <div
            v-for="child in (el as GroupElement).children"
            :key="child.id"
            :class="['element-item group-child', { selected: ui.selectedElementId === child.id }]"
            @click="selectElement(child.id)"
          >
            <span class="element-icon"><AppIcon :name="elementIconName(child)" :size="14" /></span>
            <span class="element-label">{{ elementLabel(child) }}</span>
          </div>
        </template>
      </template>
    </div>

    <div class="elements-footer">
      <div class="footer-row">
        <button
          class="footer-btn group-btn"
          :disabled="!canGroup"
          @click="groupSelected"
        >
          Group
        </button>
        <button
          class="footer-btn group-btn"
          :disabled="!isGroupSelected"
          @click="ungroupSelected"
        >
          Ungroup
        </button>
      </div>
      <div class="footer-row" v-if="hasMultiSelection">
        <button class="footer-btn" @click="openOverlayModal">
          + Overlay
        </button>
        <button class="footer-btn" @click="removeBatchOverlay">
          - Overlay
        </button>
      </div>
      <button
        class="delete-btn"
        :disabled="!ui.selectedElementId"
        aria-label="Delete selected element"
        @click="deleteSelected"
      >
        Delete Selected
      </button>
      <button
        class="footer-btn"
        :disabled="!ui.selectedElementId"
        @click="cleanSelectedSvg"
      >
        Clean Up SVG
      </button>
    </div>

    <IconBrowser />

    <!-- Batch Overlay Modal -->
    <div v-if="overlayModalOpen" class="modal-overlay" @click.self="overlayModalOpen = false">
      <div class="modal-content">
        <div class="modal-title">Apply Overlay</div>
        <div class="modal-row">
          <label class="modal-label">Type</label>
          <select class="prop-select" v-model="overlayKind">
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
        <div class="modal-row">
          <label class="modal-label">Position</label>
          <select class="prop-select" v-model="overlayPosition">
            <option value="topLeft">Top Left</option>
            <option value="topRight">Top Right</option>
            <option value="bottomLeft">Bottom Left</option>
            <option value="bottomRight">Bottom Right</option>
          </select>
        </div>
        <div class="modal-row">
          <label class="modal-label">Color</label>
          <input type="color" v-model="overlayColor" class="color-swatch" />
        </div>
        <div class="modal-actions">
          <button class="modal-btn cancel" @click="overlayModalOpen = false">Cancel</button>
          <button class="modal-btn confirm" @click="applyBatchOverlay">Apply</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.elements-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.elements-list {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.empty-state {
  padding: 20px 12px;
  color: var(--text-muted);
  font-size: 12px;
  text-align: center;
}

.element-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  cursor: pointer;
  transition: background var(--transition-fast), box-shadow var(--transition-fast);
}

.element-item.dragging {
  opacity: 0.5;
}

.element-item.drag-over-above {
  box-shadow: inset 0 2px 0 0 var(--accent);
}

.element-item.drag-over-below {
  box-shadow: inset 0 -2px 0 0 var(--accent);
}

.element-item.group-child {
  padding-left: 38px;
}

.element-item:hover {
  background: var(--bg-hover);
}

.element-item.selected {
  background: var(--accent-muted);
  box-shadow: inset 0 0 0 1px var(--accent-glow);
}

.element-icon {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  flex-shrink: 0;
}

.element-item.selected .element-icon {
  color: var(--accent);
}

.symbol-badge {
  font-size: 10px;
  flex-shrink: 0;
}

.element-label {
  flex: 1;
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.element-item.selected .element-label {
  color: var(--text-primary);
}

.element-item.is-locked .element-label {
  color: var(--text-muted);
}

.element-item.is-hidden .element-label {
  opacity: 0.5;
}

.element-toggles {
  display: flex;
  gap: 1px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.element-item:hover .element-toggles,
.element-toggles:has(.toggle-btn.active) {
  opacity: 1;
}

.toggle-btn {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0;
  border-radius: var(--radius-sm);
  transition: background var(--transition-fast), color var(--transition-fast);
}

.toggle-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.toggle-btn.active {
  color: var(--accent);
}

.element-actions {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
}

.reorder-btn {
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  cursor: pointer;
  padding: 0;
  transition: background var(--transition-fast), color var(--transition-fast);
}

.reorder-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.reorder-btn:disabled {
  opacity: 0.3;
  cursor: default;
}

.expand-btn {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0;
  flex-shrink: 0;
}

.expand-btn:hover {
  color: var(--text-primary);
}

.elements-footer {
  padding: 8px 12px;
  border-top: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.footer-row {
  display: flex;
  gap: 6px;
}

.footer-btn {
  flex: 1;
  height: 28px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.footer-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.footer-btn:disabled {
  opacity: 0.4;
  cursor: default;
}

.delete-btn {
  width: 100%;
  height: 32px;
  background: var(--danger-muted);
  border: 1px solid var(--danger-muted);
  border-radius: var(--radius-md);
  color: var(--danger);
  font-size: 12px;
  cursor: pointer;
  transition: background var(--transition-fast), color var(--transition-fast);
}

.delete-btn:hover:not(:disabled) {
  background: var(--danger);
  color: white;
}

.delete-btn:disabled {
  opacity: 0.4;
  cursor: default;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.modal-content {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 16px;
  width: 280px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
}

.modal-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
}

.modal-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.modal-label {
  width: 52px;
  flex-shrink: 0;
  font-size: 11px;
  color: var(--text-muted);
}

.modal-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
  justify-content: flex-end;
}

.modal-btn {
  padding: 6px 16px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  cursor: pointer;
  border: 1px solid var(--border-color);
  transition: all var(--transition-fast) ease;
}

.modal-btn.cancel {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.modal-btn.confirm {
  background: var(--accent);
  color: white;
  border-color: var(--accent);
}

.modal-btn:hover {
  opacity: 0.85;
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
</style>
