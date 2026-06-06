<script setup lang="ts">
import { ref } from "vue";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";
import AppIcon from "@/components/common/AppIcon.vue";

const project = useProjectStore();
const ui = useUiStore();

const brushColor = ref("#333333");
const brushWidth = ref(3);
const brushFill = ref("none");

const shapes = [
  { type: "circle", label: "Circle", icon: "circle" },
  { type: "rect", label: "Rectangle", icon: "square" },
  { type: "rounded-rect", label: "Rounded Rect", icon: "roundedSquare" },
  { type: "hexagon", label: "Hexagon", icon: "hexagon" },
  { type: "star", label: "Star", icon: "star" },
  { type: "shield", label: "Shield", icon: "shield" },
  { type: "diamond", label: "Diamond", icon: "diamond" },
  { type: "triangle", label: "Triangle", icon: "triangle" },
  { type: "arrow-right", label: "Arrow", icon: "arrowRight" },
  { type: "cross", label: "Cross", icon: "cross" },
  { type: "heart", label: "Heart", icon: "heart" },
  { type: "pentagon", label: "Pentagon", icon: "pentagon" },
  { type: "octagon", label: "Octagon", icon: "octagon" },
  { type: "wave", label: "Wave", icon: "wave" },
] as const;

function addShape(shapeType: string) {
  const size = 200;
  const x = Math.round(project.canvasWidth / 2 - size / 2);
  const y = Math.round(project.canvasHeight / 2 - size / 2);
  project.addShape(shapeType, "#4A90D9", size, x, y);
}

function addText() {
  const x = Math.round(project.canvasWidth / 2 - 50);
  const y = Math.round(project.canvasHeight / 2 - 24);
  project.addText("Hello", "Arial", 48, "#333333", x, y);
}

function openIconBrowser() {
  ui.toggleIconBrowser(true);
}

function toggleDrawing() {
  ui.isDrawing = !ui.isDrawing;
}

function finishDrawing() {
  if (!ui.currentPath || ui.currentPath.length < 2) {
    ui.isDrawing = false;
    ui.currentPath = [];
    return;
  }
  const points = ui.currentPath;
  let d = `M${points[0].x.toFixed(1)},${points[0].y.toFixed(1)}`;
  for (let i = 1; i < points.length; i++) {
    d += ` L${points[i].x.toFixed(1)},${points[i].y.toFixed(1)}`;
  }
  project.addPath(d, brushColor.value, brushWidth.value, brushFill.value === "none" ? undefined : brushFill.value);
  ui.isDrawing = false;
  ui.currentPath = [];
}

async function importImage() {
  const size = 200;
  const x = Math.round((project.canvasWidth - size) / 2);
  const y = Math.round((project.canvasHeight - size) / 2);
  await project.addImage(size, size, x, y);
}
</script>

<template>
  <div class="add-shape-bar">
    <div class="section-label">Add Shape</div>
    <div class="shape-buttons">
      <button
        v-for="shape in shapes"
        :key="shape.type"
        class="shape-btn"
        :title="shape.label"
        @click="addShape(shape.type)"
      >
        <AppIcon :name="shape.icon" :size="16" />
      </button>
    </div>
    <div class="section-label">Add Element</div>
    <div class="action-buttons">
      <button class="action-btn" title="Add Text" @click="addText">
        <span class="action-icon"><AppIcon name="type" :size="14" /></span>
        <span class="action-label">Text</span>
      </button>
      <button class="action-btn" title="Add Icon" @click="openIconBrowser">
        <span class="action-icon"><AppIcon name="plusSquare" :size="14" /></span>
        <span class="action-label">Icon</span>
      </button>
      <button class="action-btn" title="Import Image" @click="importImage">
        <span class="action-icon"><AppIcon name="imagePlus" :size="14" /></span>
        <span class="action-label">Image</span>
      </button>
      <button
        :class="['action-btn', { active: ui.isDrawing }]"
        title="Draw Path"
        @click="toggleDrawing"
      >
        <span class="action-icon"><AppIcon name="paintbrush" :size="14" /></span>
        <span class="action-label">{{ ui.isDrawing ? 'Drawing' : 'Brush' }}</span>
      </button>
    </div>
    <div v-if="ui.isDrawing" class="brush-settings">
      <div class="brush-row">
        <label class="brush-label">Color</label>
        <input type="color" v-model="brushColor" class="brush-color" />
        <label class="brush-label">Width</label>
        <input type="number" v-model.number="brushWidth" min="1" max="20" class="brush-input" />
      </div>
      <div class="brush-row">
        <label class="brush-label">Fill</label>
        <select v-model="brushFill" class="brush-select">
          <option value="none">None</option>
          <option value="#333333">Dark</option>
          <option value="#FFFFFF">White</option>
        </select>
        <button class="brush-done-btn" @click="finishDrawing">Done</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.add-shape-bar {
  padding: 8px 0;
  border-bottom: 1px solid var(--border-color);
}

.section-label {
  padding: 0 12px 4px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-muted);
}

.shape-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 0 8px 8px;
}

.shape-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: background var(--transition-fast), border-color var(--transition-fast), color var(--transition-fast);
}

.shape-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.shape-btn:active {
  background: var(--accent);
  color: var(--bg-primary);
}

.action-buttons {
  display: flex;
  gap: 6px;
  padding: 0 8px;
}

.action-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 30px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: background var(--transition-fast), border-color var(--transition-fast), color var(--transition-fast);
}

.action-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.action-btn:active {
  background: var(--accent);
  color: var(--bg-primary);
}

.action-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 14px;
  height: 14px;
}

.action-label {
  font-size: 10px;
}

.action-btn.active {
  background: var(--accent-muted);
  color: var(--accent);
  border-color: var(--accent);
  box-shadow: inset 0 0 0 1px var(--accent-glow);
}

.brush-settings {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 6px 8px;
  border-top: 1px solid var(--border-color);
}

.brush-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.brush-label {
  font-size: 10px;
  color: var(--text-muted);
  width: 32px;
}

.brush-color {
  width: 24px;
  height: 20px;
  padding: 0;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: none;
  cursor: pointer;
}

.brush-input {
  width: 48px;
  height: 20px;
  padding: 0 4px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 10px;
  outline: none;
}

.brush-select {
  flex: 1;
  height: 20px;
  padding: 0 4px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 10px;
  outline: none;
}

.brush-done-btn {
  height: 20px;
  padding: 0 8px;
  background: var(--accent);
  border: none;
  border-radius: var(--radius-sm);
  color: var(--bg-primary);
  font-size: 10px;
  font-weight: 600;
  cursor: pointer;
}
</style>
