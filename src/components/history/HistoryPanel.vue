<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";
import AppIcon from "@/components/common/AppIcon.vue";

interface HistoryInfo {
  undo: string[];
  redo: string[];
}

const project = useProjectStore();
const ui = useUiStore();

const history = ref<HistoryInfo>({ undo: [], redo: [] });
let timer: ReturnType<typeof setInterval> | null = null;

async function refresh() {
  try {
    history.value = await invoke<HistoryInfo>("get_history");
  } catch {
    history.value = { undo: [], redo: [] };
  }
}

onMounted(() => {
  refresh();
  timer = setInterval(refresh, 5000);
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
});

async function handleUndo() {
  await project.performUndo();
  await refresh();
}

async function handleRedo() {
  await project.performRedo();
  await refresh();
}

async function undoTo(index: number) {
  const steps = history.value.undo.length - 1 - index;
  for (let i = 0; i < steps; i++) {
    await project.performUndo();
  }
  await refresh();
}

async function redoTo(index: number) {
  for (let i = 0; i <= index; i++) {
    await project.performRedo();
  }
  await refresh();
}
</script>

<template>
  <div class="history-panel">
    <div class="history-actions">
      <button class="action-btn" :disabled="!ui.canUndo" @click="handleUndo" title="Undo (Ctrl+Z)">
        <AppIcon name="undo" :size="14" />
        <span>Undo</span>
      </button>
      <button class="action-btn" :disabled="!ui.canRedo" @click="handleRedo" title="Redo (Ctrl+Y)">
        <AppIcon name="redo" :size="14" />
        <span>Redo</span>
      </button>
    </div>

    <div v-if="history.undo.length === 0 && history.redo.length === 0" class="empty-state">
      No actions yet
    </div>

    <div v-else class="history-list">
      <div
        v-for="(label, index) in history.undo"
        :key="'u-' + index"
        class="history-item undo-item"
        @click="undoTo(index)"
        :title="'Undo to: ' + label"
      >
        <span class="dot past" />
        <span class="label">{{ label }}</span>
        <span class="index">#{{ index + 1 }}</span>
      </div>

      <div class="current-marker">
        <span class="dot current" />
        <span class="current-label">Current State</span>
      </div>

      <div
        v-for="(label, i) in history.redo"
        :key="'r-' + i"
        class="history-item redo-item"
        @click="redoTo(i)"
        :title="'Redo to: ' + label"
      >
        <span class="dot future" />
        <span class="label">{{ label }}</span>
        <span class="index">redo</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.history-panel {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
  overflow: hidden;
}

.history-actions {
  display: flex;
  gap: 6px;
}

.action-btn {
  flex: 1;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.action-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.action-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.empty-state {
  color: var(--text-muted);
  font-size: 12px;
  text-align: center;
  padding: 24px 0;
}

.history-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.history-item:hover {
  background: var(--bg-hover);
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.dot.past {
  background: var(--accent);
}

.dot.current {
  background: var(--success);
  box-shadow: 0 0 6px var(--success);
}

.dot.future {
  background: var(--text-muted);
  opacity: 0.5;
}

.label {
  flex: 1;
  font-size: 12px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.redo-item .label {
  color: var(--text-muted);
}

.index {
  font-size: 10px;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
  flex-shrink: 0;
}

.current-marker {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  border-top: 1px solid var(--border-color);
  border-bottom: 1px solid var(--border-color);
  margin: 4px 0;
}

.current-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--success);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
</style>
