<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";
import AppIcon from "@/components/common/AppIcon.vue";
import type { SymbolDef, SymbolElement, Element } from "@/types";

const project = useProjectStore();
const ui = useUiStore();

const symbols = ref<SymbolDef[]>([]);
const editingName = ref("");
const showCreateDialog = ref(false);
const createTargetId = ref<string | null>(null);

onMounted(() => {
  refreshSymbols();
});

async function refreshSymbols() {
  try {
    symbols.value = await invoke<SymbolDef[]>("list_symbols");
  } catch (e) {
    console.error("Failed to list symbols:", e);
  }
}

function startCreateFromElement(elementId: string) {
  createTargetId.value = elementId;
  editingName.value = "";
  showCreateDialog.value = true;
}

async function confirmCreate() {
  if (!createTargetId.value || !editingName.value.trim()) return;
  try {
    await invoke("create_symbol", {
      elementId: createTargetId.value,
      name: editingName.value.trim(),
    });
    showCreateDialog.value = false;
    await project.refreshElements();
    await refreshSymbols();
  } catch (e) {
    console.error("Failed to create symbol:", e);
    ui.showToast(`Failed to create symbol: ${e}`, "error");
  }
}

async function detachElement(elementId: string) {
  try {
    await invoke("detach_symbol_cmd", { elementId });
    await project.refreshElements();
    await refreshSymbols();
  } catch (e) {
    console.error("Failed to detach symbol:", e);
    ui.showToast(`Failed to detach symbol: ${e}`, "error");
  }
}

async function updateMaster(symbolId: string, elementId: string) {
  try {
    await invoke("update_symbol", { symbolId, elementId });
    await project.refreshElements();
    await refreshSymbols();
  } catch (e) {
    console.error("Failed to update symbol:", e);
    ui.showToast(`Failed to update symbol: ${e}`, "error");
  }
}

async function addOverride(elementId: string, property: string, value: unknown) {
  try {
    await invoke("add_symbol_override", { elementId, property, value });
    await project.refreshElements();
  } catch (e) {
    console.error("Failed to add override:", e);
    ui.showToast(`Failed to add override: ${e}`, "error");
  }
}

async function removeOverride(elementId: string, property: string) {
  try {
    await invoke("remove_symbol_override", { elementId, property });
    await project.refreshElements();
  } catch (e) {
    console.error("Failed to remove override:", e);
    ui.showToast(`Failed to remove override: ${e}`, "error");
  }
}

const selectedElement = computed(() => {
  if (!ui.selectedElementId) return null;
  return project.elements.find(e => e.id === ui.selectedElementId) as Element | undefined;
});

const selectedSymbolInstance = computed((): SymbolElement | null => {
  const el = selectedElement.value;
  if (el && el.type === "symbol") return el as SymbolElement;
  return null;
});

const overridableProps = ["fill", "opacity", "x", "y", "width", "height", "rotation", "stroke", "content"];

const overrideValue = ref("");
const overrideProp = ref("fill");

function addOverrideForSelected() {
  if (!selectedSymbolInstance.value || !overrideProp.value) return;
  let parsed: unknown = overrideValue.value;
  try { parsed = JSON.parse(overrideValue.value); } catch { /* keep as string */ }
  addOverride(selectedSymbolInstance.value.id, overrideProp.value, parsed);
  overrideValue.value = "";
}
</script>

<template>
  <div class="symbols-panel">
    <div class="panel-header">
      <span class="panel-title">Symbols</span>
      <button
        class="header-btn"
        title="Create from selected element"
        :disabled="!ui.selectedElementId || selectedElement?.type === 'symbol'"
        @click="ui.selectedElementId && startCreateFromElement(ui.selectedElementId)"
      >
        <AppIcon name="plus" :size="14" />
      </button>
    </div>

    <!-- Symbol definitions list -->
    <div class="symbols-list">
      <div v-if="symbols.length === 0" class="empty-state">
        No symbols yet. Select an element and click + to create one.
      </div>
      <div
        v-for="sym in symbols"
        :key="sym.id"
        class="symbol-item"
      >
        <span class="symbol-icon"><AppIcon name="layers" :size="14" /></span>
        <span class="symbol-name">{{ sym.name }}</span>
        <span class="symbol-meta">{{ sym.instance_count }}x &middot; {{ sym.source_type }}</span>
      </div>
    </div>

    <!-- Selected instance details -->
    <div v-if="selectedSymbolInstance" class="instance-section">
      <div class="section-title">Instance Overrides</div>
      <div class="override-list">
        <div
          v-for="ov in selectedSymbolInstance.overrides"
          :key="ov.property"
          class="override-item"
        >
          <span class="override-prop">{{ ov.property }}</span>
          <span class="override-val">{{ JSON.stringify(ov.value) }}</span>
          <button class="remove-override-btn" @click="removeOverride(selectedSymbolInstance!.id, ov.property)">
            <AppIcon name="x" :size="10" />
          </button>
        </div>
        <div v-if="selectedSymbolInstance.overrides.length === 0" class="no-overrides">
          No overrides
        </div>
      </div>

      <div class="add-override-row">
        <select v-model="overrideProp" class="override-select">
          <option v-for="p in overridableProps" :key="p" :value="p">{{ p }}</option>
        </select>
        <input
          v-model="overrideValue"
          class="override-input"
          placeholder="value"
          @keydown.enter="addOverrideForSelected"
        />
        <button class="add-override-btn" :disabled="!overrideValue" @click="addOverrideForSelected">
          <AppIcon name="plus" :size="12" />
        </button>
      </div>

      <div class="instance-actions">
        <button class="action-btn" @click="detachElement(selectedSymbolInstance!.id)">
          Detach from Symbol
        </button>
        <button
          class="action-btn"
          @click="updateMaster(selectedSymbolInstance!.symbol_id, selectedSymbolInstance!.id)"
        >
          Update Master
        </button>
      </div>
    </div>

    <!-- Create dialog -->
    <Teleport to="body">
      <div v-if="showCreateDialog" class="dialog-overlay" @click.self="showCreateDialog = false">
        <div class="dialog">
          <div class="dialog-title">Create Symbol</div>
          <input
            v-model="editingName"
            class="dialog-input"
            placeholder="Symbol name"
            autofocus
            @keydown.enter="confirmCreate"
          />
          <div class="dialog-actions">
            <button class="dialog-btn cancel" @click="showCreateDialog = false">Cancel</button>
            <button class="dialog-btn confirm" :disabled="!editingName.trim()" @click="confirmCreate">Create</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.symbols-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-color);
}

.panel-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.header-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.header-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.header-btn:disabled {
  opacity: 0.3;
  cursor: default;
}

.symbols-list {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.empty-state {
  padding: 20px 14px;
  color: var(--text-muted);
  font-size: 12px;
  text-align: center;
}

.symbol-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.symbol-item:hover {
  background: var(--bg-hover);
}

.symbol-icon {
  color: var(--accent);
  flex-shrink: 0;
}

.symbol-name {
  flex: 1;
  font-size: 12px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.symbol-meta {
  font-size: 10px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.instance-section {
  border-top: 1px solid var(--border-color);
  padding: 10px 14px;
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
}

.override-list {
  margin-bottom: 8px;
}

.override-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 0;
  font-size: 11px;
}

.override-prop {
  color: var(--accent);
  font-weight: 500;
  min-width: 60px;
}

.override-val {
  flex: 1;
  color: var(--text-secondary);
  font-family: monospace;
  font-size: 10px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.remove-override-btn {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0;
  border-radius: var(--radius-sm);
}

.remove-override-btn:hover {
  color: var(--danger);
  background: var(--danger-muted);
}

.no-overrides {
  font-size: 11px;
  color: var(--text-muted);
  padding: 4px 0;
}

.add-override-row {
  display: flex;
  gap: 4px;
  margin-bottom: 8px;
}

.override-select {
  width: 70px;
  height: 24px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 11px;
  padding: 0 4px;
}

.override-input {
  flex: 1;
  height: 24px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 11px;
  padding: 0 6px;
}

.add-override-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-muted);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--accent);
  cursor: pointer;
}

.add-override-btn:disabled {
  opacity: 0.3;
  cursor: default;
}

.instance-actions {
  display: flex;
  gap: 6px;
}

.action-btn {
  flex: 1;
  height: 26px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 10px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 20px;
  width: 280px;
}

.dialog-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
}

.dialog-input {
  width: 100%;
  height: 32px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 13px;
  padding: 0 10px;
  margin-bottom: 12px;
}

.dialog-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.dialog-btn {
  height: 30px;
  padding: 0 16px;
  border-radius: var(--radius-md);
  font-size: 12px;
  cursor: pointer;
  border: 1px solid var(--border-color);
  transition: all var(--transition-fast);
}

.dialog-btn.cancel {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.dialog-btn.confirm {
  background: var(--accent);
  color: white;
  border-color: var(--accent);
}

.dialog-btn:disabled {
  opacity: 0.4;
  cursor: default;
}
</style>
