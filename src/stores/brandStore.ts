import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { logError } from "@/utils/logger";

export interface BrandKitInfo {
  id: string;
  name: string;
  colors: Record<string, string>;
  variant_count: number;
}

export const useBrandStore = defineStore("brand", () => {
  const kits = ref<BrandKitInfo[]>([]);
  const loading = ref(false);
  const guideText = ref("");

  async function fetchKits() {
    loading.value = true;
    try {
      kits.value = await invoke<BrandKitInfo[]>("list_brand_kits");
    } catch (e) {
      logError("fetchKits", e);
    } finally {
      loading.value = false;
    }
  }

  async function createKit(
    name: string,
    primary: string,
    secondary?: string,
    accent?: string,
    neutral?: string,
  ): Promise<BrandKitInfo> {
    const kit = await invoke<BrandKitInfo>("create_brand_kit", {
      name,
      primary,
      secondary: secondary || null,
      accent: accent || null,
      neutral: neutral || null,
    });
    await fetchKits();
    return kit;
  }

  async function applyKit(kitId: string, mode?: string) {
    await invoke("apply_brand", { kitId, mode: mode || null });
  }

  async function generateVariant(kitId: string, variantType: string): Promise<BrandKitInfo> {
    const kit = await invoke<BrandKitInfo>("generate_brand_variant", {
      kitId,
      variantType,
    });
    await fetchKits();
    return kit;
  }

  async function exportGuide(kitId: string): Promise<string> {
    const text = await invoke<string>("export_brand_guide", { kitId });
    guideText.value = text;
    return text;
  }

  async function suggest(description: string): Promise<BrandKitInfo> {
    return invoke<BrandKitInfo>("suggest_brand", { description });
  }

  async function deleteKit(kitId: string) {
    await invoke("delete_brand_kit", { kitId });
    await fetchKits();
  }

  async function updateColor(kitId: string, role: string, color: string) {
    await invoke("update_brand_kit_color", { kitId, role, color });
    await fetchKits();
  }

  return {
    kits,
    loading,
    guideText,
    fetchKits,
    createKit,
    applyKit,
    generateVariant,
    exportGuide,
    suggest,
    deleteKit,
    updateColor,
  };
});
