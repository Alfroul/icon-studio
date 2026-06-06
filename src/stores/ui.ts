import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { PathNode } from "@/types";
import { parsePath } from "@/composables/usePathEditor";

export interface ToastMessage {
  id: number;
  text: string;
  type: "info" | "success" | "error" | "warning";
}

export interface Point {
  x: number;
  y: number;
}

export const useUiStore = defineStore("ui", () => {
  const activePanel = ref("canvas");
  const selectedElementId = ref<string | null>(null);
  const selectedElementIds = ref<Set<string>>(new Set());
  const zoom = ref(100);
  const displayZoom = ref(100);
  const panX = ref(0);
  const panY = ref(0);
  const iconBrowserOpen = ref(false);
  const canUndo = ref(false);
  const canRedo = ref(false);
  const toasts = ref<ToastMessage[]>([]);
  const isDrawing = ref(false);
  const currentPath = ref<Point[]>([]);
  const sidebarCollapsed = ref(false);

  const pathEditing = ref<{
    elementId: string;
    nodes: PathNode[];
    selectedIndex: number | null;
  } | null>(null);

  let toastCounter = 0;

  function setPanel(panel: string) {
    activePanel.value = panel;
  }

  function selectElement(id: string | null) {
    selectedElementId.value = id;
    if (id) {
      selectedElementIds.value = new Set([id]);
    } else {
      selectedElementIds.value = new Set();
    }
  }

  function selectElements(ids: string[]) {
    selectedElementIds.value = new Set(ids);
    selectedElementId.value = ids.length === 1 ? ids[0] : null;
  }

  function toggleElementSelection(id: string) {
    const newSet = new Set(selectedElementIds.value);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    selectedElementIds.value = newSet;
    if (newSet.size === 1) {
      selectedElementId.value = newSet.values().next().value ?? null;
    } else {
      selectedElementId.value = null;
    }
  }

  function clearSelection() {
    selectedElementIds.value = new Set();
    selectedElementId.value = null;
  }

  function toggleIconBrowser(open?: boolean) {
    iconBrowserOpen.value = open ?? !iconBrowserOpen.value;
  }

  function showToast(text: string, type: ToastMessage["type"] = "info", duration = 3000) {
    const id = ++toastCounter;
    toasts.value.push({ id, text, type });
    setTimeout(() => {
      toasts.value = toasts.value.filter((t) => t.id !== id);
    }, duration);
  }

  function setZoom(value: number) {
    zoom.value = Math.max(25, Math.min(400, Math.round(value / 5) * 5));
    displayZoom.value = zoom.value;
  }

  function setPan(x: number, y: number) {
    panX.value = x;
    panY.value = y;
  }

  function resetView() {
    zoom.value = 100;
    displayZoom.value = 100;
    panX.value = 0;
    panY.value = 0;
  }

  async function initUndoState() {
    try {
      canUndo.value = await invoke<boolean>("can_undo");
      canRedo.value = await invoke<boolean>("can_redo");
    } catch { /* ignore */ }
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  function enterPathEdit(elementId: string, d: string) {
    pathEditing.value = {
      elementId,
      nodes: parsePath(d),
      selectedIndex: null,
    };
  }

  function exitPathEdit() {
    pathEditing.value = null;
  }

  function updateNode(index: number, partial: Partial<PathNode>) {
    if (!pathEditing.value) return;
    if (index < 0 || index >= pathEditing.value.nodes.length) return;
    const nodes = [...pathEditing.value.nodes];
    nodes[index] = { ...nodes[index], ...partial };
    pathEditing.value = { ...pathEditing.value, nodes };
  }

  function addNode(index: number, node: PathNode) {
    if (!pathEditing.value) return;
    const nodes = [...pathEditing.value.nodes];
    nodes.splice(index, 0, node);
    pathEditing.value = { ...pathEditing.value, nodes };
  }

  function removeNode(index: number) {
    if (!pathEditing.value) return;
    if (pathEditing.value.nodes.length <= 1) return;
    const nodes = [...pathEditing.value.nodes];
    nodes.splice(index, 1);
    const sel = pathEditing.value.selectedIndex;
    let newSel: number | null = sel;
    if (sel === index) {
      newSel = null;
    } else if (sel !== null && sel > index) {
      newSel = sel - 1;
    }
    pathEditing.value = { ...pathEditing.value, nodes, selectedIndex: newSel };
  }

  function setSelectedNode(index: number) {
    if (!pathEditing.value) return;
    pathEditing.value = { ...pathEditing.value, selectedIndex: index };
  }

  function setDragging(_value: boolean) {
    // no-op: dragging state tracked locally by PathEditorOverlay
  }

  return {
    activePanel,
    selectedElementId,
    selectedElementIds,
    zoom,
    displayZoom,
    panX,
    panY,
    iconBrowserOpen,
    canUndo,
    canRedo,
    toasts,
    isDrawing,
    currentPath,
    sidebarCollapsed,
    pathEditing,
    setPanel,
    selectElement,
    selectElements,
    toggleElementSelection,
    clearSelection,
    toggleIconBrowser,
    showToast,
    setZoom,
    setPan,
    resetView,
    initUndoState,
    toggleSidebar,
    enterPathEdit,
    exitPathEdit,
    updateNode,
    addNode,
    removeNode,
    setSelectedNode,
    setDragging,
  };
});
