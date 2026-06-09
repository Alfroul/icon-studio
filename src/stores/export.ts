import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useUiStore } from "@/stores/ui";
import { logError } from "@/utils/logger";

export const PNG_SIZES = [16, 32, 64, 128, 256, 512, 1024] as const;

export const useExportStore = defineStore("export", () => {
  const ui = useUiStore();
  const exporting = ref(false);
  const outputDir = ref("");
  const selectedFormats = ref<string[]>(["svg", "png", "ico"]);
  const selectedPngSizes = ref<number[]>([16, 32, 64, 128, 256, 512]);
  const exportResults = ref<string[]>([]);
  const error = ref<string | null>(null);

  async function selectOutputDir(): Promise<void> {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (selected == null) return;
      if (typeof selected === "string") {
        outputDir.value = selected;
      } else if (Array.isArray(selected)) {
        outputDir.value = selected[0] ?? "";
      }
    } catch (e) {
      logError("selectOutputDir", e);
      ui.showToast(`Failed to select directory: ${e}`, "error");
    }
  }

  async function exportSvg(): Promise<string[]> {
    if (!outputDir.value) return [];
    const svgPath = await invoke<string>("export_svg", {
      path: `${outputDir.value}/icon.svg`,
    });
    return [svgPath];
  }

  async function exportPng(sizes?: number[]): Promise<string[]> {
    const targetSizes = sizes ?? selectedPngSizes.value;
    if (!outputDir.value) return [];
    const results = await invoke<string[]>("export_png", {
      sizes: targetSizes,
      outputDir: outputDir.value,
    });
    return results;
  }

  async function exportIco(sizes?: number[]): Promise<string[]> {
    const targetSizes = sizes ?? [16, 32, 48, 256];
    if (!outputDir.value) return [];
    const result = await invoke<string>("export_ico", {
      sizes: targetSizes,
      path: `${outputDir.value}/icon.ico`,
    });
    return [result];
  }

  async function exportAndroidIcons(): Promise<string[]> {
    if (!outputDir.value) return [];
    const results = await invoke<string[]>("export_android_icons", {
      outputDir: outputDir.value,
    });
    return results;
  }

  async function exportIosIcons(): Promise<string[]> {
    if (!outputDir.value) return [];
    const results = await invoke<string[]>("export_ios_icons", {
      outputDir: outputDir.value,
    });
    return results;
  }

  async function exportAll(): Promise<string[]> {
    if (!outputDir.value) return [];
    const results = await invoke<string[]>("export_all", {
      outputDir: outputDir.value,
      formats: selectedFormats.value,
      pngSizes: selectedPngSizes.value,
    });
    return results;
  }

  async function exportWebp(size: number = 512): Promise<string[]> {
    if (!outputDir.value) return [];
    await invoke("export_webp", {
      size,
      path: `${outputDir.value}/icon.webp`,
    });
    return [`${outputDir.value}/icon.webp`];
  }

  async function exportCode(
    svgContent: string,
    componentName: string,
    format: string,
    size: number,
    parametrizeFill: boolean,
  ): Promise<{ code: string; format: string; filename: string } | null> {
    try {
      return await invoke<{ code: string; format: string; filename: string }>("export_code", {
        svgContent,
        options: {
          componentName,
          format,
          size,
          parametrizeFill,
        },
      });
    } catch (e) {
      logError("exportCode", e);
      ui.showToast(`Code export failed: ${e}`, "error");
      return null;
    }
  }

  // Stage 4: design token export
  async function exportTokens(
    format: string,
  ): Promise<{ content: string; format: string; filename: string } | null> {
    try {
      return await invoke<{ content: string; format: string; filename: string }>("export_tokens", {
        format,
      });
    } catch (e) {
      logError("exportTokens", e);
      ui.showToast(`Token export failed: ${e}`, "error");
      return null;
    }
  }

  // Stage 6: icon font export
  async function exportIconFont(
    glyphs: Array<{ iconName: string; unicode: string; svgPathData: string }>,
    fontName: string,
    formats: string[],
    includeCss: boolean = true,
    includeDemo: boolean = true,
    unicodeStart: number = 0xe000,
  ): Promise<{ files: Array<[string, number[]]> } | null> {
    try {
      return await invoke<{ files: Array<[string, number[]]> }>("export_icon_font", {
        glyphs,
        options: {
          fontName,
          formats,
          includeCss,
          includeDemo,
          unicodeStart,
        },
      });
    } catch (e) {
      logError("exportIconFont", e);
      ui.showToast(`Icon font export failed: ${e}`, "error");
      return null;
    }
  }

  return {
    PNG_SIZES,
    exporting,
    outputDir,
    selectedFormats,
    selectedPngSizes,
    exportResults,
    error,
    selectOutputDir,
    exportSvg,
    exportPng,
    exportIco,
    exportAndroidIcons,
    exportIosIcons,
    exportAll,
    exportWebp,
    exportCode,
    exportTokens,
    exportIconFont,
  };
});
