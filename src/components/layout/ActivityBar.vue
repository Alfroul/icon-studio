<script setup lang="ts">
import { ref } from "vue";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import AppIcon from "@/components/common/AppIcon.vue";

const props = defineProps<{
  activePanel: string;
}>();

const emit = defineEmits<{
  select: [panel: string];
}>();

const ui = useUiStore();
const project = useProjectStore();

const panels = [
  { id: "canvas", icon: "layoutGrid", label: "Canvas" },
  { id: "pages", icon: "copy", label: "Pages" },
  { id: "elements", icon: "layers", label: "Elements" },
  { id: "symbols", icon: "box", label: "Symbols" },
  { id: "properties", icon: "settings", label: "Properties" },
  { id: "style", icon: "droplet", label: "Style" },
  { id: "analysis", icon: "search", label: "Analysis" },
  { id: "adaptive", icon: "smartphone", label: "Adaptive" },
  { id: "brand", icon: "palette", label: "Brand" },
  { id: "iconset", icon: "grid2x2", label: "Icon Sets" },
  { id: "library", icon: "grid3x3", label: "Library" },
  { id: "templates", icon: "layoutDashboard", label: "Templates" },
  { id: "export", icon: "upload", label: "Export" },
  { id: "settings", icon: "settings", label: "Settings" },
  { id: "history", icon: "clock", label: "History" },
];

const tabRefs = ref<HTMLElement[]>([]);

function focusTab(index: number) {
  tabRefs.value[index]?.focus();
}

function onTabKeydown(e: KeyboardEvent, index: number) {
  let nextIndex = index;
  if (e.key === "ArrowDown" || e.key === "ArrowRight") {
    e.preventDefault();
    nextIndex = (index + 1) % panels.length;
  } else if (e.key === "ArrowUp" || e.key === "ArrowLeft") {
    e.preventDefault();
    nextIndex = (index - 1 + panels.length) % panels.length;
  } else if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    emit("select", panels[index].id);
    return;
  } else {
    return;
  }
  focusTab(nextIndex);
}

async function handleUndo() {
  await project.performUndo();
}

async function handleRedo() {
  await project.performRedo();
}
</script>

<template>
  <div class="activity-bar" role="tablist" aria-label="Navigation panels">
    <button
      v-for="(panel, index) in panels"
      :ref="(el) => { if (el) tabRefs[index] = el as HTMLElement }"
      :key="panel.id"
      :class="['activity-btn', { active: activePanel === panel.id }]"
      :title="panel.label"
      role="tab"
      :id="'tab-' + panel.id"
      :aria-selected="activePanel === panel.id"
      :aria-controls="'panel-' + panel.id"
      :tabindex="activePanel === panel.id ? 0 : -1"
      @click="$emit('select', panel.id)"
      @keydown="onTabKeydown($event, index)"
    >
      <AppIcon :name="panel.icon" :size="18" />
    </button>
    <div class="spacer" />
    <div class="history-buttons">
      <button
        class="activity-btn icon-btn"
        :disabled="!ui.canUndo"
        title="Undo (Ctrl+Z)"
        @click="handleUndo"
      >
        <AppIcon name="undo" :size="18" />
      </button>
      <button
        class="activity-btn icon-btn"
        :disabled="!ui.canRedo"
        title="Redo (Ctrl+Y)"
        @click="handleRedo"
      >
        <AppIcon name="redo" :size="18" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.activity-bar {
  width: 52px;
  background: var(--bg-secondary);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 4px;
  border-right: 1px solid var(--border-color);
}

.activity-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 8px;
  margin-bottom: 2px;
  transition: all var(--transition-fast);
}

.activity-btn:hover {
  color: var(--text-secondary);
  background: var(--bg-hover);
}

.activity-btn.active {
  color: var(--accent);
  background: var(--accent-muted);
  box-shadow: inset 0 0 0 1px var(--accent-glow);
}

.icon-btn {
  height: 28px;
  width: 28px;
}

.icon-btn:disabled {
  opacity: 0.25;
  cursor: not-allowed;
}
.icon-btn:disabled:hover {
  background: none;
  color: var(--text-muted);
}

.spacer {
  flex: 1;
}

.history-buttons {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-bottom: 12px;
  border-top: 1px solid var(--border-color);
}
</style>
