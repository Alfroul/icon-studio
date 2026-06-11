import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useUiStore } from "./ui";
import type { ThemeVariant, ThemeRule } from "@/types";

export interface PresetRuleSet {
  name: string;
  rules: ThemeRule[];
}

export const useVariantsStore = defineStore("variants", () => {
  const variants = ref<ThemeVariant[]>([]);
  const presets = ref<PresetRuleSet[]>([]);
  const loading = ref(false);

  async function fetchVariants(): Promise<void> {
    try {
      variants.value = await invoke<ThemeVariant[]>("list_variants");
    } catch {
      // silently ignore — panel may load before project
    }
  }

  async function fetchPresets(): Promise<void> {
    try {
      presets.value = await invoke<PresetRuleSet[]>("list_preset_rules");
    } catch {
      // ignore
    }
  }

  async function createVariant(name: string, rules: ThemeRule[]): Promise<ThemeVariant> {
    loading.value = true;
    try {
      const variant = await invoke<ThemeVariant>("create_variant", { name, rules });
      variants.value.push(variant);
      return variant;
    } catch (e) {
      const ui = useUiStore();
      ui.showToast(`Failed to create variant: ${e}`, "error");
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function deleteVariant(index: number): Promise<void> {
    loading.value = true;
    try {
      await invoke("delete_variant", { index });
      variants.value.splice(index, 1);
    } catch (e) {
      const ui = useUiStore();
      ui.showToast(`Failed to delete variant: ${e}`, "error");
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function previewVariant(index: number): Promise<string> {
    const svg = await invoke<string>("preview_variant", { index });
    return svg;
  }

  async function exportVariant(index: number, format: string, outputDir: string): Promise<string[]> {
    loading.value = true;
    try {
      const paths = await invoke<string[]>("export_variant", { index, format, outputDir });
      return paths;
    } catch (e) {
      const ui = useUiStore();
      ui.showToast(`Failed to export variant: ${e}`, "error");
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function exportAllVariants(format: string, outputDir: string): Promise<string[]> {
    loading.value = true;
    try {
      const paths = await invoke<string[]>("export_all_variants", { format, outputDir });
      return paths;
    } catch (e) {
      const ui = useUiStore();
      ui.showToast(`Failed to export variants: ${e}`, "error");
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function generateAllPresets(): Promise<void> {
    loading.value = true;
    try {
      const created = await invoke<ThemeVariant[]>("generate_all_presets");
      variants.value = created;
      const ui = useUiStore();
      ui.showToast(`Generated ${created.length} preset variants`, "success");
    } catch (e) {
      const ui = useUiStore();
      ui.showToast(`Failed to generate presets: ${e}`, "error");
      throw e;
    } finally {
      loading.value = false;
    }
  }

  return {
    variants,
    presets,
    loading,
    fetchVariants,
    fetchPresets,
    createVariant,
    deleteVariant,
    previewVariant,
    exportVariant,
    exportAllVariants,
    generateAllPresets,
  };
});
