import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as dialogOpen } from "@tauri-apps/plugin-dialog";
import type { CanvasResult, Gradient, IconProject } from "@/types";
import { useUiStore } from "@/stores/ui";
import { useCanvasStore } from "@/stores/canvas";
import { useElementsStore } from "@/stores/elements";
import { pickFilePath } from "@/utils/pickFilePath";
import { logError, logWarn } from "@/utils/logger";

export const useProjectStore = defineStore("project", () => {
  const ui = useUiStore();
  const canvasStore = useCanvasStore();
  const elementsStore = useElementsStore();

  const svgPreview = ref("");

  let _timer: ReturnType<typeof setTimeout> | null = null;

  async function fetchPreview() {
    try {
      svgPreview.value = await invoke<string>("render_preview");
    } catch (e) {
      logError("fetchPreview", e);
      ui.showToast(`Failed to render preview: ${e}`, "error");
    }
  }

  function debouncedFetchPreview(delay = 200) {
    if (_timer) clearTimeout(_timer);
    _timer = setTimeout(fetchPreview, delay);
  }

  elementsStore.setOnChanged(async () => {
    debouncedFetchPreview();
  });

  async function updateCanvas(props: {
    width?: number;
    height?: number;
    background?: string;
    corner_radius?: number;
    background_gradient?: Gradient | null;
  }) {
    try {
      await canvasStore.pushCanvasUpdate(props);
      await elementsStore.refreshElements();
      debouncedFetchPreview();
    } catch (e) {
      logError("updateCanvas", e);
      ui.showToast(`Failed to update canvas: ${e}`, "error");
    }
  }

  async function syncCanvas(canvas: CanvasResult) {
    canvasStore.syncFromResult(canvas);
  }

  async function fetchStatus() {
    try {
      return await invoke<string>("get_status");
    } catch (e) {
      logError("fetchStatus", e);
      ui.showToast(`Failed to get status: ${e}`, "error");
      return "";
    }
  }

  async function newProject() {
    try {
      const result = await invoke<CanvasResult>("new_canvas");
      await syncCanvas(result);
      elementsStore.items = [];
      ui.selectElement(null);
      await ui.initUndoState();
      await fetchPreview();
      ui.showToast("New project created", "success");
    } catch (e) {
      logError("newProject", e);
      ui.showToast(`Failed to create new project: ${e}`, "error");
    }
  }

  async function saveProject(path?: string) {
    try {
      const saved = await invoke<string>("save_project", { path: path || null });
      ui.showToast(`Saved to ${saved}`, "success");
    } catch (e) {
      ui.showToast(`Save failed: ${e}`, "error");
    }
  }

  async function importSvg() {
    if (elementsStore.isDialogOpen) return;
    try {
      elementsStore.isDialogOpen = true;
      const selected = await dialogOpen({
        multiple: false,
        filters: [{ name: "SVG", extensions: ["svg"] }],
      });
      const path = pickFilePath(selected);
      if (!path) return;

      const result = await invoke<IconProject>("import_svg_file", { path });
      await syncCanvas(result.canvas);
      await elementsStore.refreshElements();
      await ui.initUndoState();
      ui.selectElement(null);
      ui.showToast("SVG imported", "success");
    } catch (e) {
      ui.showToast(`Import failed: ${e}`, "error");
    } finally {
      elementsStore.isDialogOpen = false;
    }
  }

  async function openProject(path?: string) {
    if (elementsStore.isDialogOpen && !path) return;
    try {
      let filePath = path;
      if (!filePath) {
        elementsStore.isDialogOpen = true;
        const selected = await dialogOpen({
          multiple: false,
          filters: [
            { name: "IconStudio Project", extensions: ["iconproject.json"] },
            { name: "SVG", extensions: ["svg"] },
          ],
        });
        const picked = pickFilePath(selected);
        if (!picked) return;
        filePath = picked;
      }

      if (filePath.toLowerCase().endsWith(".svg")) {
        const result = await invoke<IconProject>("import_svg_file", { path: filePath });
        await syncCanvas(result.canvas);
        await elementsStore.refreshElements();
        await ui.initUndoState();
        ui.selectElement(null);
        ui.showToast("SVG imported", "success");
      } else {
        const result = await invoke<IconProject>("open_project", { path: filePath });
        await syncCanvas(result.canvas);
        await elementsStore.refreshElements();
        await ui.initUndoState();
        ui.selectElement(null);
        ui.showToast("Project opened", "success");
      }
    } catch (e) {
      ui.showToast(`Open failed: ${e}`, "error");
    } finally {
      elementsStore.isDialogOpen = false;
    }
  }

  async function performUndo() {
    try {
      const ok = await invoke<boolean>("undo");
      if (ok) {
        ui.canUndo = await invoke<boolean>("can_undo");
        ui.canRedo = await invoke<boolean>("can_redo");
        await elementsStore.refreshElements();
        await canvasStore.fetchAndSync();
      }
    } catch (e) {
      logWarn("performUndo", e);
      ui.showToast(`Undo failed: ${e}`, "error");
    }
  }

  async function performRedo() {
    try {
      const ok = await invoke<boolean>("redo");
      if (ok) {
        ui.canUndo = await invoke<boolean>("can_undo");
        ui.canRedo = await invoke<boolean>("can_redo");
        await elementsStore.refreshElements();
        await canvasStore.fetchAndSync();
      }
    } catch (e) {
      logWarn("performRedo", e);
      ui.showToast(`Redo failed: ${e}`, "error");
    }
  }

  // Backward-compat re-exports — components still use project.canvasWidth etc.
  const canvasWidth = computed(() => canvasStore.width);
  const canvasHeight = computed(() => canvasStore.height);
  const canvasBackground = computed(() => canvasStore.background);
  const canvasCornerRadius = computed(() => canvasStore.cornerRadius);
  const canvasBackgroundGradient = computed(() => canvasStore.backgroundGradient);
  const elements = computed(() => elementsStore.items);
  const isDialogOpen = computed(() => elementsStore.isDialogOpen);
  function findElementDeep(elements: Element[], id: string): Element | null {
    for (const el of elements) {
      if (el.id === id) return el;
      if (el.type === "group" && "children" in el) {
        const found = findElementDeep((el as { children: Element[] }).children, id);
        if (found) return found;
      }
    }
    return null;
  }

  const selectedElement = computed(() => {
    if (!ui.selectedElementId) return null;
    return findElementDeep(elementsStore.items, ui.selectedElementId) ?? null;
  });
  const currentVersion = computed(() => elementsStore.currentVersion);

  return {
    canvasWidth,
    canvasHeight,
    canvasBackground,
    canvasCornerRadius,
    canvasBackgroundGradient,
    elements,
    svgPreview,
    isDialogOpen,
    selectedElement,
    currentVersion,
    fetchPreview,
    debouncedFetchPreview,
    fetchStatus,
    updateCanvas,
    refreshElements: elementsStore.refreshElements,
    addShape: elementsStore.addShape,
    addText: elementsStore.addText,
    addIcon: elementsStore.addIcon,
    addImage: elementsStore.addImage,
    addPath: elementsStore.addPath,
    removeElement: elementsStore.removeElement,
    updateElement: elementsStore.updateElement,
    reorderElement: elementsStore.reorderElement,
    duplicateElement: elementsStore.duplicateElement,
    newProject,
    saveProject,
    importSvg,
    openProject,
    setGradient: elementsStore.setGradient,
    clearGradient: elementsStore.clearGradient,
    setShadow: elementsStore.setShadow,
    clearShadow: elementsStore.clearShadow,
    setFilter: elementsStore.setFilter,
    clearFilter: elementsStore.clearFilter,
    setLayout: elementsStore.setLayout,
    listFonts: elementsStore.listFonts,
    groupElements: elementsStore.groupElements,
    ungroup: elementsStore.ungroup,
    addToGroup: elementsStore.addToGroup,
    removeFromGroup: elementsStore.removeFromGroup,
    setClip: elementsStore.setClip,
    clearClip: elementsStore.clearClip,
    setMask: elementsStore.setMask,
    clearMask: elementsStore.clearMask,
    booleanOp: elementsStore.booleanOp,
    cleanSvg: elementsStore.cleanSvg,
    syncCanvas,
    performUndo,
    performRedo,
  };
});
