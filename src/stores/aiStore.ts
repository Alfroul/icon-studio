import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useUiStore } from "./ui";

export type AiProviderType = "openAi" | "recraft" | "custom" | "ollama";
export type IconStyleType =
  | "flat"
  | "outline"
  | "duotone"
  | "gradient"
  | "threeD"
  | "minimal"
  | "cartoon"
  | "pixelArt"
  | "lineArt"
  | "neon";

export interface GeneratedIconResult {
  svg_content: string | null;
  image_data: number[] | null;
  prompt: string;
}

const PROVIDER_MODELS: Record<AiProviderType, string[]> = {
  openAi: ["gpt-4o", "dall-e-3", "gpt-4o-mini"],
  recraft: ["recraft-v3", "recraft-vector"],
  custom: [],
  ollama: ["llava", "llama3", "codellama", "mistral"],
};

const DEFAULT_MODELS: Record<AiProviderType, string> = {
  openAi: "gpt-4o",
  recraft: "recraft-v3",
  custom: "",
  ollama: "llava",
};

export const useAiStore = defineStore("ai", () => {
  const provider = ref<AiProviderType>(
    (localStorage.getItem("iconstudio-ai-provider") as AiProviderType) || "openAi"
  );
  const apiKey = ref(localStorage.getItem("iconstudio-ai-api-key") || "");
  const model = ref(localStorage.getItem("iconstudio-ai-model") || "gpt-4o");
  const endpoint = ref(localStorage.getItem("iconstudio-ai-endpoint") || "");
  const generating = ref(false);
  const results = ref<GeneratedIconResult[]>([]);
  const error = ref<string | null>(null);
  const configExpanded = ref(false);

  const availableModels = computed(() => PROVIDER_MODELS[provider.value]);

  function setProvider(p: AiProviderType) {
    provider.value = p;
    model.value = DEFAULT_MODELS[p];
    localStorage.setItem("iconstudio-ai-provider", p);
    localStorage.setItem("iconstudio-ai-model", model.value);
  }

  function setApiKey(key: string) {
    apiKey.value = key;
    localStorage.setItem("iconstudio-ai-api-key", key);
  }

  function setModel(m: string) {
    model.value = m;
    localStorage.setItem("iconstudio-ai-model", m);
  }

  function setEndpoint(e: string) {
    endpoint.value = e;
    localStorage.setItem("iconstudio-ai-endpoint", e);
  }

  async function generateIcon(
    prompt: string,
    style: IconStyleType
  ): Promise<GeneratedIconResult[]> {
    generating.value = true;
    error.value = null;
    try {
      const icons = await invoke<GeneratedIconResult[]>("generate_icon", {
        task: "textToIcon",
        style,
        prompt,
        provider: provider.value,
        apiKey: apiKey.value,
        model: model.value,
        endpoint: endpoint.value || null,
      });
      results.value = icons;
      return icons;
    } catch (e) {
      error.value = String(e);
      const ui = useUiStore();
      ui.showToast(`AI generation failed: ${e}`, "error");
      throw e;
    } finally {
      generating.value = false;
    }
  }

  async function generateSet(
    prompts: string[],
    style: IconStyleType
  ): Promise<GeneratedIconResult[]> {
    generating.value = true;
    error.value = null;
    try {
      const icons = await invoke<GeneratedIconResult[]>("generate_icon_set", {
        prompts,
        style,
        provider: provider.value,
        apiKey: apiKey.value,
        model: model.value,
      });
      results.value = icons;
      return icons;
    } catch (e) {
      error.value = String(e);
      const ui = useUiStore();
      ui.showToast(`AI batch generation failed: ${e}`, "error");
      throw e;
    } finally {
      generating.value = false;
    }
  }

  async function removeBackground(imageData: string): Promise<string> {
    const result = await invoke<string>("ai_remove_background", {
      imageData,
      provider: provider.value,
      apiKey: apiKey.value,
      model: model.value,
    });
    return result;
  }

  function clearResults() {
    results.value = [];
    error.value = null;
  }

  return {
    provider,
    apiKey,
    model,
    endpoint,
    generating,
    results,
    error,
    configExpanded,
    availableModels,
    setProvider,
    setApiKey,
    setModel,
    setEndpoint,
    generateIcon,
    generateSet,
    removeBackground,
    clearResults,
  };
});
