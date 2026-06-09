import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as dialogOpen } from "@tauri-apps/plugin-dialog";
import type { Element, FontInfo } from "@/types";
import { useUiStore } from "@/stores/ui";
import { useCanvasStore } from "@/stores/canvas";
import { pickFilePath } from "@/utils/pickFilePath";
import { logError } from "@/utils/logger";


export const useElementsStore = defineStore("elements", () => {
  const ui = useUiStore();
  const canvas = useCanvasStore();
  const items = ref<Element[]>([]);
  const isDialogOpen = ref(false);
  const currentVersion = ref(0);

  let _onChanged: (() => Promise<void>) | null = null;

  function setOnChanged(fn: () => Promise<void>) {
    _onChanged = fn;
  }

  async function refreshElements() {
    try {
      const list = await invoke<Element[]>("list_elements");
      items.value = list;
      currentVersion.value++;
      if (_onChanged) await _onChanged();
    } catch (e) {
      logError("refreshElements", e);
      ui.showToast(`Failed to refresh elements: ${e}`, "error");
    }
  }

  async function addShape(shapeType: string, fill: string, size: number, x: number, y: number) {
    try {
      await invoke("add_shape", { shapeType, fill, size, x, y });
      await refreshElements();
    } catch (e) {
      logError("addShape", e);
      ui.showToast(`Failed to add shape: ${e}`, "error");
    }
  }

  async function addText(
    content: string,
    fontFamily: string,
    fontSize: number,
    fill: string,
    x: number,
    y: number
  ) {
    try {
      await invoke("add_text", { content, fontFamily, fontSize, fill, x, y });
      await refreshElements();
    } catch (e) {
      logError("addText", e);
      ui.showToast(`Failed to add text: ${e}`, "error");
    }
  }

  async function addIcon(iconName: string, fill: string, size: number, x: number, y: number) {
    try {
      await invoke("add_icon", { iconName, fill, size, x, y });
      await refreshElements();
    } catch (e) {
      logError("addIcon", e);
      ui.showToast(`Failed to add icon: ${e}`, "error");
    }
  }

  async function addImage(width?: number, height?: number, x?: number, y?: number) {
    try {
      if (isDialogOpen.value) return;
      isDialogOpen.value = true;
      const selected = await dialogOpen({
        multiple: false,
        filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "gif", "svg", "webp"] }],
      });
      const filePath = pickFilePath(selected);
      if (!filePath) { isDialogOpen.value = false; return; }

      const w = width ?? 200;
      const h = height ?? 200;
      const px = x ?? Math.round((canvas.width - w) / 2);
      const py = y ?? Math.round((canvas.height - h) / 2);

      await invoke("add_image", { filePath, width: w, height: h, x: px, y: py });
      await refreshElements();
    } catch (e) {
      logError("addImage", e);
      ui.showToast(`Add image failed: ${e}`, "error");
    } finally {
      isDialogOpen.value = false;
    }
  }

  async function addPath(d: string, stroke: string, strokeWidth: number, fill?: string) {
    try {
      await invoke("add_path", { d, stroke, strokeWidth, fill: fill || null });
      await refreshElements();
    } catch (e) {
      logError("addPath", e);
      ui.showToast(`Failed to add path: ${e}`, "error");
    }
  }

  async function removeElement(elementId: string) {
    try {
      await invoke("remove_element", { elementId });
      await refreshElements();
    } catch (e) {
      logError("removeElement", e);
      ui.showToast(`Failed to remove element: ${e}`, "error");
    }
  }

  async function updateElement(elementId: string, props: Record<string, unknown>) {
    try {
      await invoke("set_props", { elementId, props });
      await refreshElements();
    } catch (e) {
      logError("updateElement", e);
      ui.showToast(`Failed to update element: ${e}`, "error");
    }
  }

  async function reorderElement(elementId: string, newIndex: number) {
    try {
      await invoke("reorder_elements", { elementId, newIndex });
      await refreshElements();
    } catch (e) {
      logError("reorderElement", e);
      ui.showToast(`Failed to reorder element: ${e}`, "error");
    }
  }

  async function duplicateElement(elementId: string) {
    try {
      const newId = await invoke<string>("duplicate_element", { elementId });
      await refreshElements();
      ui.selectElement(newId);
      ui.showToast("Element duplicated", "success");
    } catch (e) {
      logError("duplicateElement", e);
      ui.showToast(`Duplication failed: ${e}`, "error");
    }
  }

  async function setGradient(elementId: string, gradientType: string, colors: string[], angle: number) {
    try {
      await invoke("set_gradient", { elementId, gradientType, colors, angle });
      await refreshElements();
    } catch (e) {
      logError("setGradient", e);
      ui.showToast(`Failed to set gradient: ${e}`, "error");
    }
  }

  async function clearGradient(elementId: string) {
    try {
      await invoke("clear_gradient", { elementId });
      await refreshElements();
    } catch (e) {
      logError("clearGradient", e);
      ui.showToast(`Failed to clear gradient: ${e}`, "error");
    }
  }

  async function setShadow(elementId: string, color: string, blur: number, offsetX: number, offsetY: number) {
    try {
      await invoke("set_shadow", { elementId, color, blur, offsetX, offsetY });
      await refreshElements();
    } catch (e) {
      logError("setShadow", e);
      ui.showToast(`Failed to set shadow: ${e}`, "error");
    }
  }

  async function clearShadow(elementId: string) {
    try {
      await invoke("clear_shadow", { elementId });
      await refreshElements();
    } catch (e) {
      logError("clearShadow", e);
      ui.showToast(`Failed to clear shadow: ${e}`, "error");
    }
  }

  async function setBlendMode(elementId: string, mode: string | null) {
    try {
      await invoke("set_blend_mode", { elementId, mode: mode || null });
      await refreshElements();
    } catch (e) {
      logError("setBlendMode", e);
      ui.showToast(`Failed to set blend mode: ${e}`, "error");
    }
  }

  async function setFilter(elementId: string, filterType: string, params: Record<string, number>) {
    try {
      await invoke("set_filter", { elementId, filterType, params });
      await refreshElements();
    } catch (e) {
      logError("setFilter", e);
      ui.showToast(`Failed to set filter: ${e}`, "error");
    }
  }

  async function clearFilter(elementId: string) {
    try {
      await invoke("clear_filter", { elementId });
      await refreshElements();
    } catch (e) {
      logError("clearFilter", e);
      ui.showToast(`Failed to clear filter: ${e}`, "error");
    }
  }

  async function setClip(elementId: string, clipElementId: string) {
    try {
      await invoke("set_clip", { elementId, clipElementId });
      await refreshElements();
    } catch (e) {
      logError("setClip", e);
      ui.showToast(`Failed to set clip: ${e}`, "error");
    }
  }

  async function clearClip(elementId: string) {
    try {
      await invoke("clear_clip", { elementId });
      await refreshElements();
    } catch (e) {
      logError("clearClip", e);
      ui.showToast(`Failed to clear clip: ${e}`, "error");
    }
  }

  async function setMask(elementId: string, maskElementId: string) {
    try {
      await invoke("set_mask", { elementId, maskElementId });
      await refreshElements();
    } catch (e) {
      logError("setMask", e);
      ui.showToast(`Failed to set mask: ${e}`, "error");
    }
  }

  async function clearMask(elementId: string) {
    try {
      await invoke("clear_mask", { elementId });
      await refreshElements();
    } catch (e) {
      logError("clearMask", e);
      ui.showToast(`Failed to clear mask: ${e}`, "error");
    }
  }

  async function booleanOp(elementAId: string, elementBId: string, operation: string) {
    try {
      const newId = await invoke<string>("boolean_operation", { elementAId, elementBId, operation });
      await refreshElements();
      ui.selectElement(newId);
      ui.showToast(`Boolean ${operation} completed`, "success");
    } catch (e) {
      logError("booleanOp", e);
      ui.showToast(`Boolean operation failed: ${e}`, "error");
    }
  }

  async function convertToPath(elementId: string) {
    try {
      const newId = await invoke<string>("convert_to_path", { elementId });
      await refreshElements();
      ui.selectElement(newId);
      ui.showToast("Converted to path", "success");
    } catch (e) {
      logError("convertToPath", e);
      ui.showToast(`Convert failed: ${e}`, "error");
    }
  }

  async function cleanSvg(svg: string, rules?: string[]) {
    try {
      const result = await invoke<{
        cleanedSvg: string;
        rulesApplied: string[];
        bytesBefore: number;
        bytesAfter: number;
      }>("optimize_svg", { svg, rules: rules || null });
      const saved = result.bytesBefore - result.bytesAfter;
      const pct = result.bytesBefore > 0
        ? Math.round((saved / result.bytesBefore) * 100)
        : 0;
      ui.showToast(
        `Cleaned: ${result.rulesApplied.length} rule(s), saved ${saved} bytes (${pct}%)`,
        "success",
      );
      return result;
    } catch (e) {
      logError("cleanSvg", e);
      ui.showToast(`Clean SVG failed: ${e}`, "error");
      return null;
    }
  }

  async function groupElements(elementIds: string[]) {
    try {
      await invoke("group_elements", { elementIds });
      await refreshElements();
    } catch (e) {
      logError("groupElements", e);
      ui.showToast(`Failed to group elements: ${e}`, "error");
    }
  }

  async function ungroup(groupId: string) {
    try {
      await invoke("ungroup", { groupId });
      await refreshElements();
    } catch (e) {
      logError("ungroup", e);
      ui.showToast(`Failed to ungroup: ${e}`, "error");
    }
  }

  async function addToGroup(groupId: string, elementId: string) {
    try {
      await invoke("add_to_group", { groupId, elementId });
      await refreshElements();
    } catch (e) {
      logError("addToGroup", e);
      ui.showToast(`Failed to add to group: ${e}`, "error");
    }
  }

  async function removeFromGroup(groupId: string, elementId: string) {
    try {
      await invoke("remove_from_group", { groupId, elementId });
      await refreshElements();
    } catch (e) {
      logError("removeFromGroup", e);
      ui.showToast(`Failed to remove from group: ${e}`, "error");
    }
  }

  async function setLayout(layoutType: string, gap: number, padding: number) {
    try {
      await invoke("set_layout", { layoutType, gap, padding });
      await refreshElements();
    } catch (e) {
      logError("setLayout", e);
      ui.showToast(`Failed to set layout: ${e}`, "error");
    }
  }

  async function listFonts(keyword?: string) {
    try {
      return await invoke<FontInfo[]>("list_fonts", { keyword: keyword || null });
    } catch (e) {
      logError("listFonts", e);
      ui.showToast(`Failed to list fonts: ${e}`, "error");
      return [];
    }
  }

  return {
    items,
    isDialogOpen,
    currentVersion,
    setOnChanged,
    refreshElements,
    addShape,
    addText,
    addIcon,
    addImage,
    addPath,
    removeElement,
    updateElement,
    reorderElement,
    duplicateElement,
    setGradient,
    clearGradient,
    setShadow,
    clearShadow,
    setBlendMode,
    setFilter,
    clearFilter,
    setLayout,
    listFonts,
    groupElements,
    ungroup,
    addToGroup,
    removeFromGroup,
    setClip,
    clearClip,
    setMask,
    clearMask,
    booleanOp,
    convertToPath,
    cleanSvg,
  };
});
