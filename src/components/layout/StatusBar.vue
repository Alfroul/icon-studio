<script setup lang="ts">
import { computed } from "vue";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";
import { useSettingsStore } from "@/stores/settings";
import { usePagesStore } from "@/stores/pages";

const project = useProjectStore();
const ui = useUiStore();
const settings = useSettingsStore();
const pagesStore = usePagesStore();

const canvasSize = computed(() => `${project.canvasWidth}×${project.canvasHeight}`);
const elementCount = computed(() => project.elements.length);
const zoom = computed(() => ui.displayZoom);
const mcpStatus = computed(() => settings.mcpStatus);
const mcpClass = computed(() => `mcp-${settings.mcpStatus}`);
const wsClass = computed(() => `ws-${settings.wsStatus}`);
const wsLabel = computed(() => {
  const labels: Record<string, string> = {
    disconnected: "disconnected",
    connecting: "connecting",
    connected: "connected",
    error: "error",
  };
  return labels[settings.wsStatus] || "disconnected";
});
const pageInfo = computed(() => {
  if (pagesStore.pages.length > 1) {
    const idx = pagesStore.activePageIndex + 1;
    return `Page ${idx}/${pagesStore.pages.length}`;
  }
  return "";
});
</script>

<template>
  <div class="status-bar">
    <span v-if="pageInfo" class="status-item">{{ pageInfo }}</span>
    <span v-if="pageInfo" class="status-separator">·</span>
    <span class="status-item">{{ canvasSize }}</span>
    <span class="status-separator">·</span>
    <span class="status-item">{{ elementCount }} elements</span>
    <span class="status-separator">·</span>
    <span class="status-item">{{ zoom }}%</span>
    <span class="status-spacer"></span>
    <span :class="['status-item', 'mcp-status', mcpClass]">
      <span class="status-dot"></span>
      MCP {{ mcpStatus }}
    </span>
    <span class="status-separator">·</span>
    <span :class="['status-item', 'ws-status', wsClass]">
      <span class="status-dot"></span>
      WS {{ wsLabel }}
    </span>
  </div>
</template>

<style scoped>
.status-bar {
  height: 32px;
  background: var(--bg-glass);
  backdrop-filter: blur(12px);
  color: var(--text-muted);
  border-top: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  padding: 0 var(--space-md);
  font-size: 11px;
  gap: 12px;
  letter-spacing: 0.02em;
}

.status-item {
  white-space: nowrap;
}

.status-separator {
  display: inline-block;
  width: 1px;
  height: 14px;
  background: var(--border-color);
}

.status-spacer {
  flex: 1;
}

.mcp-status {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-muted);
}

.mcp-running .status-dot {
  background: var(--success);
  animation: pulse 2s ease infinite;
}

.mcp-off .status-dot {
  background: var(--text-muted);
}

.mcp-error .status-dot {
  background: var(--danger);
}

.mcp-starting .status-dot {
  background: var(--warning);
}

.ws-status {
  display: flex;
  align-items: center;
  gap: 6px;
}

.ws-connected .status-dot {
  background: var(--success);
  animation: pulse 2s ease infinite;
}

.ws-disconnected .status-dot {
  background: var(--text-muted);
}

.ws-error .status-dot {
  background: var(--danger);
}

.ws-connecting .status-dot {
  background: var(--warning);
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
</style>
