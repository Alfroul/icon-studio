<script setup lang="ts">
import { useSettingsStore, type ThemeMode } from "@/stores/settings";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { open } from "@tauri-apps/plugin-dialog";
import ToggleSwitch from "@/components/common/ToggleSwitch.vue";

const settings = useSettingsStore();
const ui = useUiStore();
const project = useProjectStore();

const themeOptions: { value: ThemeMode; label: string }[] = [
  { value: "dark", label: "Dark" },
  { value: "light", label: "Light" },
  { value: "system", label: "System" },
];

async function selectAutoExportDir() {
  try {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") {
      settings.setAutoExportDir(selected);
    }
  } catch (e) {
    console.error("Failed to select directory:", e);
  }
}
</script>

<template>
  <div class="settings-panel">
    <div class="section">
      <div class="section-title">Theme</div>
      <div class="theme-options">
        <button
          v-for="opt in themeOptions"
          :key="opt.value"
          :class="['theme-btn', { active: settings.theme === opt.value }]"
          @click="settings.applyTheme(opt.value)"
        >
          {{ opt.label }}
        </button>
      </div>
    </div>

    <div class="section">
      <div class="section-title">MCP Server</div>
      <div class="setting-row">
        <span class="setting-label">Enable MCP Server</span>
        <ToggleSwitch
          :model-value="settings.mcpEnabled"
          @update:model-value="settings.setMcpEnabled($event)"
        />
      </div>
      <div class="setting-row">
        <span class="setting-label">MCP Status</span>
        <span :class="['status-badge', settings.mcpStatus]">
          {{ settings.mcpStatus }}
        </span>
      </div>
    </div>

    <div class="section">
      <div class="section-title">Defaults</div>
      <div class="setting-row">
        <span class="setting-label">Default Font</span>
        <span class="setting-value">{{ settings.defaultFontFamily }}</span>
      </div>
      <div class="setting-row">
        <span class="setting-label">Export Formats</span>
        <span class="setting-value">{{ settings.defaultExportFormats.join(", ") }}</span>
      </div>
    </div>

    <div class="section">
      <div class="section-title">Auto Export on Close</div>
      <div class="setting-row">
        <span class="setting-label">关闭时自动导出 SVG</span>
        <ToggleSwitch
          :model-value="settings.autoExportOnClose"
          @update:model-value="settings.setAutoExportOnClose($event)"
        />
      </div>
      <div v-if="settings.autoExportOnClose" class="setting-row column">
        <span class="setting-label">导出目录</span>
        <div class="dir-row">
          <span class="setting-value dir-path">{{ settings.autoExportDir || "未选择" }}</span>
          <button class="dir-btn" @click="selectAutoExportDir">选择目录</button>
        </div>
      </div>
    </div>

    <div class="section">
      <div class="section-title">Keyboard Shortcuts</div>
      <div class="shortcut-list">
        <div class="shortcut-item"><kbd>Ctrl+Z</kbd> Undo</div>
        <div class="shortcut-item"><kbd>Ctrl+Y</kbd> / <kbd>Ctrl+Shift+Z</kbd> Redo</div>
        <div class="shortcut-item"><kbd>Ctrl+N</kbd> New Project</div>
        <div class="shortcut-item"><kbd>Ctrl+S</kbd> Save Project</div>
        <div class="shortcut-item"><kbd>Ctrl+E</kbd> Export Panel</div>
        <div class="shortcut-item"><kbd>Delete</kbd> / <kbd>Backspace</kbd> Delete Element</div>
        <div class="shortcut-item"><kbd>Ctrl+D</kbd> Duplicate Element</div>
        <div class="shortcut-item"><kbd>Ctrl+A</kbd> Select All</div>
        <div class="shortcut-item"><kbd>Ctrl+G</kbd> Group Selected</div>
        <div class="shortcut-item"><kbd>Ctrl+Shift+G</kbd> Ungroup Selected</div>
        <div class="shortcut-item"><kbd>Arrow Keys</kbd> Move 1px</div>
        <div class="shortcut-item"><kbd>Shift+Arrow Keys</kbd> Move 10px</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-panel {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  overflow-y: auto;
  height: 100%;
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--accent);
  margin-bottom: 8px;
}

.theme-options {
  display: flex;
  gap: 4px;
}

.theme-btn {
  flex: 1;
  height: 32px;
  padding: 0 6px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-muted);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.theme-btn:hover {
  color: var(--text-primary);
  border-color: var(--accent);
}
.theme-btn.active {
  background: var(--accent);
  color: var(--bg-primary);
  border-color: var(--accent);
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
}

.setting-row.column {
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
}

.dir-row {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.dir-path {
  flex: 1;
  font-size: 11px;
  font-family: "JetBrains Mono", monospace;
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  padding: 4px 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.dir-btn {
  flex-shrink: 0;
  height: 28px;
  padding: 0 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.dir-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.setting-label {
  font-size: 12px;
  color: var(--text-primary);
}

.setting-value {
  font-size: 12px;
  color: var(--text-muted);
}

.status-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  text-transform: uppercase;
}
.status-badge.off {
  background: var(--bg-hover);
  color: var(--text-muted);
}
.status-badge.starting {
  background: var(--warning-muted);
  color: var(--warning);
}
.status-badge.running {
  background: var(--success-muted);
  color: var(--success);
}
.status-badge.error {
  background: var(--danger-muted);
  color: var(--danger);
}

.shortcut-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.shortcut-item {
  font-size: 12px;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  gap: 6px;
}

kbd {
  display: inline-block;
  padding: 1px 5px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-family: "JetBrains Mono", monospace;
  font-size: 11px;
  color: var(--text-secondary);
  min-width: fit-content;
}
</style>
