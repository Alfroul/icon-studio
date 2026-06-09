import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useUiStore } from "@/stores/ui";
import { logError } from "@/utils/logger";

export interface IconPackMeta {
  id: string;
  name: string;
  version: string;
  iconCount: number;
  categories: string[];
  sourcePath: string;
  importedAt: string;
}

export interface PackIcon {
  name: string;
  category: string;
  tags: string[];
  svgPath: string;
}

export const usePacksStore = defineStore("packs", () => {
  const ui = useUiStore();

  const packs = ref<IconPackMeta[]>([]);
  const currentPackIcons = ref<PackIcon[]>([]);
  const currentPackId = ref<string | null>(null);
  const loading = ref(false);
  const searchResults = ref<PackIcon[]>([]);

  // Icon SVG cache: packId/iconName -> svg content
  const svgCache = ref<Map<string, string>>(new Map());

  async function loadPacks(): Promise<void> {
    try {
      packs.value = await invoke<IconPackMeta[]>("list_icon_packs");
    } catch (e) {
      logError("loadPacks", e);
    }
  }

  async function importPack(dir: string, name: string): Promise<void> {
    try {
      const result = await invoke<{ pack: IconPackMeta; iconsImported: number }>(
        "import_icon_pack",
        { dir, packName: name },
      );
      ui.showToast(`Imported ${result.iconsImported} icons`, "success");
      await loadPacks();
    } catch (e) {
      logError("importPack", e);
      ui.showToast(`Import failed: ${e}`, "error");
    }
  }

  async function loadPackIcons(packId: string): Promise<void> {
    loading.value = true;
    currentPackId.value = packId;
    try {
      currentPackIcons.value = await invoke<PackIcon[]>("list_pack_icons", {
        packId,
      });
    } catch (e) {
      logError("loadPackIcons", e);
      currentPackIcons.value = [];
    } finally {
      loading.value = false;
    }
  }

  async function searchIcons(
    packId: string,
    query: string,
  ): Promise<void> {
    if (!query.trim()) {
      searchResults.value = [];
      return;
    }
    try {
      searchResults.value = await invoke<PackIcon[]>("search_pack_icons", {
        packId,
        query,
      });
    } catch (e) {
      logError("searchIcons", e);
      searchResults.value = [];
    }
  }

  async function loadIconSvg(packId: string, iconName: string): Promise<string> {
    const cacheKey = `${packId}/${iconName}`;
    const cached = svgCache.value.get(cacheKey);
    if (cached) return cached;

    const svg = await invoke<string>("load_pack_icon", { packId, iconName });
    svgCache.value.set(cacheKey, svg);
    return svg;
  }

  async function addIconToCanvas(packId: string, iconName: string): Promise<void> {
    try {
      const svg = await invoke<string>("add_pack_icon_to_canvas", { packId, iconName });
      // TODO: parse SVG elements and add to project via a future import_svg_content command
      // For now, copy SVG to clipboard for manual use
      await navigator.clipboard.writeText(svg);
      ui.showToast("Icon SVG copied to clipboard", "success");
    } catch (e) {
      logError("addIconToCanvas", e);
      ui.showToast(`Failed to add icon: ${e}`, "error");
    }
  }

  async function removePack(packId: string): Promise<void> {
    try {
      await invoke("remove_icon_pack", { packId });
      if (currentPackId.value === packId) {
        currentPackId.value = null;
        currentPackIcons.value = [];
      }
      await loadPacks();
      ui.showToast("Pack removed", "success");
    } catch (e) {
      logError("removePack", e);
      ui.showToast(`Failed to remove pack: ${e}`, "error");
    }
  }

  function clearCurrentPack(): void {
    currentPackId.value = null;
    currentPackIcons.value = [];
    searchResults.value = [];
  }

  return {
    packs,
    currentPackIcons,
    currentPackId,
    loading,
    searchResults,
    svgCache,
    loadPacks,
    importPack,
    loadPackIcons,
    searchIcons,
    loadIconSvg,
    addIconToCanvas,
    removePack,
    clearCurrentPack,
  };
});
