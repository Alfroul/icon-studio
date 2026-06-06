<script setup lang="ts">
import { computed } from "vue";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { serializePath } from "@/composables/usePathEditor";
import type { PathNode } from "@/types";

const ui = useUiStore();
const project = useProjectStore();

const nodes = computed(() => ui.pathEditing?.nodes ?? []);
const selectedIndex = computed(() => ui.pathEditing?.selectedIndex ?? null);
const selectedNode = computed<PathNode | null>(() => {
  if (selectedIndex.value === null) return null;
  return nodes.value[selectedIndex.value] ?? null;
});

function addNodeAtSelected() {
  if (selectedIndex.value === null) return;
  const node = nodes.value[selectedIndex.value];
  const nextNode = nodes.value[selectedIndex.value + 1];
  const ax = nextNode ? (node.anchor.x + nextNode.anchor.x) / 2 : node.anchor.x + 20;
  const ay = nextNode ? (node.anchor.y + nextNode.anchor.y) / 2 : node.anchor.y;
  ui.addNode(selectedIndex.value + 1, { command: "L", anchor: { x: ax, y: ay } });
}

function deleteSelectedNode() {
  if (selectedIndex.value === null) return;
  ui.removeNode(selectedIndex.value);
}

function toggleCurve() {
  if (selectedIndex.value === null) return;
  const node = nodes.value[selectedIndex.value];
  if (node.command === "L") {
    const prev = selectedIndex.value > 0 ? nodes.value[selectedIndex.value - 1] : null;
    const px = prev ? prev.anchor.x : node.anchor.x - 30;
    const py = prev ? prev.anchor.y : node.anchor.y;
    ui.updateNode(selectedIndex.value, {
      command: "C",
      handleIn: { x: (px + node.anchor.x) / 2, y: py },
      handleOut: { x: (px + node.anchor.x) / 2, y: node.anchor.y },
    });
  } else if (node.command === "C") {
    ui.updateNode(selectedIndex.value, {
      command: "L",
      handleIn: undefined,
      handleOut: undefined,
    });
  }
}

function closePath() {
  if (nodes.value.length > 0 && nodes.value[nodes.value.length - 1].command !== "Z") {
    ui.addNode(nodes.value.length, { command: "Z", anchor: { x: 0, y: 0 } });
  }
}

function openPath() {
  if (nodes.value.length > 0 && nodes.value[nodes.value.length - 1].command === "Z") {
    ui.removeNode(nodes.value.length - 1);
  }
}

function commitAndExit() {
  if (ui.pathEditing) {
    const newD = serializePath(nodes.value);
    project.updateElement(ui.pathEditing.elementId, { d: newD });
  }
  ui.exitPathEdit();
}
</script>

<template>
  <div v-if="ui.pathEditing" class="path-toolbar">
    <div class="toolbar-group">
      <button
        class="toolbar-btn"
        :disabled="selectedIndex === null"
        @click="toggleCurve"
        :title="selectedNode?.command === 'C' ? 'Convert to line' : 'Convert to curve'"
      >
        {{ selectedNode?.command === "C" ? "ǀ" : "∿" }}
      </button>
      <button class="toolbar-btn" :disabled="selectedIndex === null" @click="addNodeAtSelected" title="Add node">
        +
      </button>
      <button class="toolbar-btn" :disabled="selectedIndex === null" @click="deleteSelectedNode" title="Delete node">
        −
      </button>
    </div>
    <div class="toolbar-separator" />
    <div class="toolbar-group">
      <button class="toolbar-btn" @click="closePath" title="Close path">Close</button>
      <button class="toolbar-btn" @click="openPath" title="Open path">Open</button>
    </div>
    <div class="toolbar-separator" />
    <div class="toolbar-group">
      <button class="toolbar-btn" @click="commitAndExit" title="Exit editing (Esc)">
        Done
      </button>
    </div>
    <span class="toolbar-info">
      {{ nodes.length }} nodes · {{ selectedIndex !== null ? `#${selectedIndex}` : "none selected" }}
    </span>
  </div>
</template>

<style scoped>
.path-toolbar {
  position: absolute;
  top: 12px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--bg-glass, rgba(20, 20, 22, 0.85));
  backdrop-filter: blur(8px);
  border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
  border-radius: 8px;
  padding: 4px 8px;
  z-index: 100;
  pointer-events: auto;
}

.toolbar-group {
  display: flex;
  gap: 2px;
}

.toolbar-btn {
  background: none;
  border: 1px solid transparent;
  color: var(--text-secondary, #ccc);
  font-size: 13px;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s;
  font-family: inherit;
}

.toolbar-btn:hover:not(:disabled) {
  background: var(--bg-hover, rgba(255, 255, 255, 0.08));
  color: var(--text-primary, #fff);
}

.toolbar-btn:disabled {
  opacity: 0.3;
  cursor: default;
}

.toolbar-separator {
  width: 1px;
  height: 20px;
  background: var(--border-color, rgba(255, 255, 255, 0.1));
}

.toolbar-info {
  font-size: 11px;
  color: var(--text-muted, #888);
  padding-left: 4px;
  white-space: nowrap;
  font-family: "JetBrains Mono", monospace;
}
</style>
