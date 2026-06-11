import { defineStore } from "pinia";
import { ref } from "vue";

export type ThemeMode = "dark" | "light" | "system";

export const useSettingsStore = defineStore("settings", () => {
  const theme = ref<ThemeMode>(
    (localStorage.getItem("iconstudio-theme") as ThemeMode) || "dark"
  );
  const mcpEnabled = ref(
    localStorage.getItem("iconstudio-mcp-enabled") !== "false"
  );
  const defaultFontFamily = ref(
    localStorage.getItem("iconstudio-default-font") || "sans-serif"
  );
  const defaultExportFormats = ref<string[]>(
    (() => {
      try {
        return JSON.parse(localStorage.getItem("iconstudio-default-export-formats") || '["svg","png"]');
      } catch {
        return ["svg", "png"];
      }
    })()
  );
  const mcpStatus = ref<"off" | "starting" | "running" | "error">("off");
  const wsStatus = ref<"disconnected" | "connecting" | "connected" | "error">("disconnected");
  const autoExportOnClose = ref(
    localStorage.getItem("iconstudio-auto-export-on-close") === "true"
  );
  const autoExportDir = ref(
    localStorage.getItem("iconstudio-auto-export-dir") || ""
  );
  const aiProvider = ref<string>(
    localStorage.getItem("iconstudio-ai-provider") || "openAi"
  );
  const aiApiKey = ref(
    localStorage.getItem("iconstudio-ai-api-key") || ""
  );
  const aiModel = ref(
    localStorage.getItem("iconstudio-ai-model") || "gpt-4o"
  );

  function applyTheme(mode: ThemeMode) {
    theme.value = mode;
    localStorage.setItem("iconstudio-theme", mode);

    const root = document.documentElement;
    if (
      mode === "dark" ||
      (mode === "system" &&
        window.matchMedia("(prefers-color-scheme: dark)").matches)
    ) {
      root.setAttribute("data-theme", "dark");
    } else {
      root.setAttribute("data-theme", "light");
    }
  }

  function setMcpEnabled(enabled: boolean) {
    mcpEnabled.value = enabled;
    localStorage.setItem("iconstudio-mcp-enabled", String(enabled));
  }

  function setDefaultFont(font: string) {
    defaultFontFamily.value = font;
    localStorage.setItem("iconstudio-default-font", font);
  }

  function setDefaultExportFormats(formats: string[]) {
    defaultExportFormats.value = formats;
    localStorage.setItem(
      "iconstudio-default-export-formats",
      JSON.stringify(formats)
    );
  }

  function setAutoExportOnClose(enabled: boolean) {
    autoExportOnClose.value = enabled;
    localStorage.setItem("iconstudio-auto-export-on-close", String(enabled));
  }

  function setAutoExportDir(dir: string) {
    autoExportDir.value = dir;
    localStorage.setItem("iconstudio-auto-export-dir", dir);
  }

  function setWsStatus(status: "disconnected" | "connecting" | "connected" | "error") {
    wsStatus.value = status;
  }

  function setAiProvider(provider: string) {
    aiProvider.value = provider;
    localStorage.setItem("iconstudio-ai-provider", provider);
  }

  function setAiApiKey(key: string) {
    aiApiKey.value = key;
    localStorage.setItem("iconstudio-ai-api-key", key);
  }

  function setAiModel(model: string) {
    aiModel.value = model;
    localStorage.setItem("iconstudio-ai-model", model);
  }

  if (typeof window !== "undefined") {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const onMediaChange = () => {
      if (theme.value === "system") {
        applyTheme("system");
      }
    };
    mql.addEventListener("change", onMediaChange);
  }

  applyTheme(theme.value);

  return {
    theme,
    mcpEnabled,
    defaultFontFamily,
    defaultExportFormats,
    mcpStatus,
    wsStatus,
    autoExportOnClose,
    autoExportDir,
    aiProvider,
    aiApiKey,
    aiModel,
    applyTheme,
    setMcpEnabled,
    setDefaultFont,
    setDefaultExportFormats,
    setAutoExportOnClose,
    setAutoExportDir,
    setWsStatus,
    setAiProvider,
    setAiApiKey,
    setAiModel,
  };
});
