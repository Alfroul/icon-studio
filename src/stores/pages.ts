import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { PageInfo } from "@/types";
import { useUiStore } from "@/stores/ui";
import { useCanvasStore } from "@/stores/canvas";
import { useElementsStore } from "@/stores/elements";
import { logError } from "@/utils/logger";

export const usePagesStore = defineStore("pages", () => {
  const ui = useUiStore();
  const canvasStore = useCanvasStore();
  const elementsStore = useElementsStore();

  const pages = ref<PageInfo[]>([]);
  const activePageIndex = ref(0);

  const activePage = computed(() => {
    if (pages.value.length === 0) return null;
    return pages.value[activePageIndex.value] ?? null;
  });

  const hasMultiplePages = computed(() => pages.value.length > 1);

  async function refreshPages() {
    try {
      pages.value = await invoke<PageInfo[]>("list_pages");
      const idx = await invoke<number>("get_active_page");
      activePageIndex.value = idx;
    } catch (e) {
      logError("refreshPages", e);
    }
  }

  async function addPage(name: string, width?: number, height?: number) {
    try {
      const info = await invoke<PageInfo>("add_page", { name, width, height });
      await refreshPages();
      await canvasStore.fetchAndSync();
      await elementsStore.refreshElements();
      ui.showToast(`Page "${name}" created`, "success");
      return info;
    } catch (e) {
      ui.showToast(`Failed to add page: ${e}`, "error");
    }
  }

  async function switchPage(pageId: string) {
    try {
      await invoke("switch_page", { pageId });
      await refreshPages();
      await canvasStore.fetchAndSync();
      await elementsStore.refreshElements();
      ui.selectElement(null);
    } catch (e) {
      ui.showToast(`Failed to switch page: ${e}`, "error");
    }
  }

  async function deletePage(pageId: string) {
    try {
      await invoke("delete_page", { pageId });
      await refreshPages();
      await canvasStore.fetchAndSync();
      await elementsStore.refreshElements();
      ui.selectElement(null);
      ui.showToast("Page deleted", "success");
    } catch (e) {
      ui.showToast(`Failed to delete page: ${e}`, "error");
    }
  }

  async function duplicatePage(pageId: string, name: string) {
    try {
      await invoke("duplicate_page", { pageId, name });
      await refreshPages();
      ui.showToast(`Page duplicated as "${name}"`, "success");
    } catch (e) {
      ui.showToast(`Failed to duplicate page: ${e}`, "error");
    }
  }

  async function renamePage(pageId: string, name: string) {
    try {
      await invoke("rename_page", { pageId, name });
      await refreshPages();
      ui.showToast("Page renamed", "success");
    } catch (e) {
      ui.showToast(`Failed to rename page: ${e}`, "error");
    }
  }

  return {
    pages,
    activePageIndex,
    activePage,
    hasMultiplePages,
    refreshPages,
    addPage,
    switchPage,
    deletePage,
    duplicatePage,
    renamePage,
  };
});
