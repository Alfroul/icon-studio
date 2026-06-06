<script setup lang="ts">
import { defineAsyncComponent } from "vue";
import { useUiStore } from "@/stores/ui";
import AppIcon from "@/components/common/AppIcon.vue";

const panels: Record<string, ReturnType<typeof defineAsyncComponent>> = {
  canvas: defineAsyncComponent(() => import("@/components/canvas/CanvasPanel.vue")),
  pages: defineAsyncComponent(() => import("@/components/pages/PagesPanel.vue")),
  elements: defineAsyncComponent(() => import("@/components/elements/ElementsPanel.vue")),
  symbols: defineAsyncComponent(() => import("@/components/symbols/SymbolsPanel.vue")),
  properties: defineAsyncComponent(() => import("@/components/properties/PropertiesPanel.vue")),
  style: defineAsyncComponent(() => import("@/components/style/StylePanel.vue")),
  analysis: defineAsyncComponent(() => import("@/components/analysis/AnalysisPanel.vue")),
  adaptive: defineAsyncComponent(() => import("@/components/adaptive/AdaptivePreview.vue")),
  brand: defineAsyncComponent(() => import("@/components/brand/BrandKitPanel.vue")),
  iconset: defineAsyncComponent(() => import("@/components/iconset/IconSetPanel.vue")),
  export: defineAsyncComponent(() => import("@/components/export/ExportPanel.vue")),
  library: defineAsyncComponent(() => import("@/components/library/LibraryPanel.vue")),
  templates: defineAsyncComponent(() => import("@/components/templates/TemplatesPanel.vue")),
  settings: defineAsyncComponent(() => import("@/components/settings/SettingsPanel.vue")),
};

const ui = useUiStore();
</script>

<template>
  <div
    class="sidebar"
    :class="{ 'sidebar--collapsed': ui.sidebarCollapsed }"
    :id="'panel-' + ui.activePanel"
    role="tabpanel"
    :aria-labelledby="'tab-' + ui.activePanel"
  >
    <div class="sidebar-header">
      <span class="sidebar-header__title">{{ ui.activePanel.charAt(0).toUpperCase() + ui.activePanel.slice(1) }}</span>
      <button class="collapse-btn" @click="ui.toggleSidebar()" :title="ui.sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'">
        <AppIcon name="chevronRight" :size="14" />
      </button>
    </div>
    <div class="sidebar-content">
      <component :is="panels[ui.activePanel]" v-if="panels[ui.activePanel]" />
      <p v-else>Select a panel from the Activity Bar.</p>
    </div>
  </div>
</template>

<style scoped>
.sidebar {
  width: 288px;
  background: var(--bg-glass);
  backdrop-filter: blur(20px);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  transition: width 200ms ease, border-color 200ms ease;
}

.sidebar--collapsed {
  width: 0;
  border-right-color: transparent;
}

.sidebar-header {
  padding: 12px 16px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  white-space: nowrap;
  min-height: 40px;
}

.sidebar--collapsed .sidebar-header {
  border-bottom-color: transparent;
}

.sidebar-header__title {
  overflow: hidden;
}

.collapse-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: none;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
  transition: color var(--transition-fast), background var(--transition-fast);
}

.collapse-btn:hover {
  color: var(--text-secondary);
  background: var(--bg-hover);
}

.sidebar-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  color: var(--text-muted);
  font-size: 12px;
  position: relative;
}

.sidebar-content > * {
  animation: panelFadeIn 200ms ease-out both;
}

@keyframes panelFadeIn {
  from {
    opacity: 0;
    transform: translateY(6px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.sidebar-content > p {
  padding: 14px;
}
</style>
