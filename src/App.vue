<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useKeyboard } from "./composables/useKeyboard";
import ActivityBar from "./components/layout/ActivityBar.vue";
import Sidebar from "./components/layout/Sidebar.vue";
import MainArea from "./components/layout/MainArea.vue";
import StatusBar from "./components/layout/StatusBar.vue";
import Toast from "./components/common/Toast.vue";
import AppIcon from "./components/common/AppIcon.vue";
import PropertiesPanel from "./components/properties/PropertiesPanel.vue";
import { useUiStore } from "./stores/ui";
import { useProjectStore } from "./stores/project";
import { useSettingsStore } from "./stores/settings";
import { useProjectSync } from "./composables/useProjectSync";
import { useWebSocketSync } from "./composables/useWebSocketSync";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { CanvasResult, IconProject } from "@/types";

const ui = useUiStore();
const project = useProjectStore();
const settings = useSettingsStore();

useProjectSync();
useWebSocketSync();
useKeyboard();

let unlisten: (() => void) | null = null;
let unlistenClose: (() => void) | null = null;

onMounted(async () => {
  project.debouncedFetchPreview();

  unlisten = await listen<string[]>("tauri://file-drop", async (event) => {
    const files = event.payload;
    if (files.length > 0) {
      const file = files[0];
      if (file.endsWith(".svg")) {
        try {
          const imported = await invoke<IconProject>("import_svg_file", { path: file });
          await project.syncCanvas(imported.canvas);
          ui.showToast("SVG imported", "success");
          await project.refreshElements();
          await ui.initUndoState();
        } catch (e) {
          ui.showToast(`Import failed: ${e}`, "error");
        }
      } else if (file.endsWith(".iconproject.json") || file.endsWith(".iconproject")) {
        try {
          const result = await invoke<IconProject>("open_project", { path: file });
          await project.syncCanvas(result.canvas);
          await project.refreshElements();
          await ui.initUndoState();
          ui.showToast("Project opened", "success");
        } catch (e) {
          ui.showToast(`Open failed: ${e}`, "error");
        }
      }
    }
  });

  // Handle window close: auto-export SVG then close
  unlistenClose = await getCurrentWindow().onCloseRequested(async (event) => {
    event.preventDefault();
    let closed = false;
    const forceTimer = setTimeout(() => {
      if (!closed) {
        try { getCurrentWindow().destroy(); } catch { /* last resort */ }
      }
    }, 3000);

    try {
      if (settings.autoExportOnClose && settings.autoExportDir && project.elements.length > 0) {
        await invoke("export_svg", {
          path: `${settings.autoExportDir}/icon.svg`,
        });
      }
    } catch (e) {
      console.error("Auto-export on close failed:", e);
    }
    try {
      await getCurrentWindow().destroy();
      closed = true;
      clearTimeout(forceTimer);
    } catch (e) {
      console.error("Failed to close window:", e);
    }
  });
});

onUnmounted(() => {
  if (unlisten) unlisten();
  if (unlistenClose) unlistenClose();
});
</script>

<template>
  <div class="app-container">
    <div class="main-row">
      <ActivityBar
        :active-panel="ui.activePanel"
        @select="(panel: string) => ui.setPanel(panel)"
      />
      <Sidebar />
      <div class="main-area-wrapper">
        <button
          v-if="ui.sidebarCollapsed"
          class="expand-sidebar-btn"
          @click="ui.toggleSidebar()"
          title="Expand sidebar"
        >
          <AppIcon name="chevronLeft" :size="14" />
        </button>
        <MainArea />
      </div>
      <div v-show="ui.selectedElementId && ui.activePanel !== 'properties'" class="right-panel">
        <PropertiesPanel />
      </div>
    </div>
    <StatusBar />
    <Toast />
  </div>
</template>

<style>
:root,
[data-theme="dark"] {
  /* Background layers — Refined Dark (Obsidian depth system) */
  --bg-primary: #09090B;
  --bg-secondary: #111113;
  --bg-tertiary: #18181B;
  --bg-elevated: #27272A;
  --bg-hover: #2E2E32;
  --bg-active: #3F3F46;
  --bg-glass: rgba(17, 17, 19, 0.72);

  /* Text — high contrast for AA+ compliance */
  --text-primary: #FAFAFA;
  --text-secondary: #D4D4D8;
  --text-muted: #71717A;

  /* Borders */
  --border-color: #27272A;
  --border-subtle: #1A1A1D;

  /* Accent — Amber (Refined Dark signature) */
  --accent: #FBBF24;
  --accent-hover: #FCD34D;
  --accent-muted: rgba(251, 191, 36, 0.10);
  --accent-pressed: #F59E0B;
  --accent-glow: rgba(251, 191, 36, 0.20);

  /* Semantic */
  --danger: #F87171;
  --danger-muted: rgba(248, 113, 113, 0.10);
  --success: #34D399;
  --success-muted: rgba(52, 211, 153, 0.10);
  --warning: #FBBF24;
  --warning-muted: rgba(251, 191, 36, 0.10);

  /* Inputs */
  --input-bg: #09090B;
  --input-border: #27272A;
  --input-focus: #FBBF24;

  /* Scrollbar */
  --scrollbar-track: transparent;
  --scrollbar-thumb: #3F3F46;

  /* Shadows */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.4);
  --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.4);
  --shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 255, 255, 0.04);
  --shadow-glow: 0 0 12px rgba(251, 191, 36, 0.20);

  /* Transitions */
  --transition-fast: 120ms ease;
  --transition-normal: 200ms ease;
  --transition-slow: 350ms cubic-bezier(0.4, 0, 0.2, 1);

  /* Radius */
  --radius-sm: 6px;
  --radius-md: 8px;
  --radius-lg: 10px;

  /* Spacing */
  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 12px;
  --space-lg: 16px;
  --space-xl: 24px;
}

[data-theme="light"] {
  --bg-primary: #FAFAFA;
  --bg-secondary: #FFFFFF;
  --bg-tertiary: #F4F4F5;
  --bg-elevated: #E4E4E7;
  --bg-hover: #D4D4D8;
  --bg-active: #C4C4C8;
  --bg-glass: rgba(255, 255, 255, 0.72);

  --text-primary: #18181B;
  --text-secondary: #52525B;
  --text-muted: #A1A1AA;

  --border-color: #E4E4E7;
  --border-subtle: #F4F4F5;

  --accent: #D97706;
  --accent-hover: #B45309;
  --accent-muted: rgba(217, 119, 6, 0.08);
  --accent-pressed: #92400E;
  --accent-glow: rgba(217, 119, 6, 0.15);

  --danger: #DC2626;
  --danger-muted: rgba(220, 38, 38, 0.08);
  --success: #16A34A;
  --success-muted: rgba(22, 163, 74, 0.08);
  --warning: #D97706;
  --warning-muted: rgba(217, 119, 6, 0.08);

  --input-bg: #FFFFFF;
  --input-border: #E4E4E7;
  --input-focus: #D97706;

  --scrollbar-track: transparent;
  --scrollbar-thumb: #D4D4D8;

  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.06);
  --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.08);
  --shadow-lg: 0 8px 32px rgba(0, 0, 0, 0.12);
  --shadow-glow: 0 0 12px rgba(217, 119, 6, 0.15);
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html,
body,
#app {
  height: 100%;
  overflow: hidden;
  font-family: "Plus Jakarta Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-primary);
  background: var(--bg-primary);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* Scrollbar */
::-webkit-scrollbar {
  width: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted);
}

/* Focus ring */
:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 1px;
}

/* Selection */
::selection {
  background: var(--accent-muted);
  color: var(--text-primary);
}

.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.main-row {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.right-panel {
  width: 260px;
  flex-shrink: 0;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border-color);
  overflow-y: auto;
}

.main-area-wrapper {
  flex: 1;
  display: flex;
  position: relative;
  overflow: hidden;
}

.expand-sidebar-btn {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: 1px solid var(--border-color);
  background: var(--bg-secondary);
  color: var(--text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: color var(--transition-fast), background var(--transition-fast);
}

.expand-sidebar-btn:hover {
  color: var(--text-secondary);
  background: var(--bg-hover);
}
</style>
