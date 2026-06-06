import { onMounted, onUnmounted } from "vue";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";

export function useKeyboard() {
  const ui = useUiStore();
  const project = useProjectStore();

  function handleKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    // Guard: skip when typing in inputs
    if (target.matches("input, textarea, select, [contenteditable]")) return;

    const ctrl = e.ctrlKey || e.metaKey;
    const shift = e.shiftKey;

    // Undo
    if (ctrl && e.key === "z" && !shift) {
      e.preventDefault();
      project.performUndo();
      return;
    }

    // Redo
    if ((ctrl && e.key === "y") || (ctrl && shift && e.key === "Z")) {
      e.preventDefault();
      project.performRedo();
      return;
    }

    // New project
    if (ctrl && e.key === "n") {
      e.preventDefault();
      project.newProject();
      return;
    }

    // Save project
    if (ctrl && e.key === "s") {
      e.preventDefault();
      project.saveProject();
      return;
    }

    // Export panel
    if (ctrl && e.key === "e") {
      e.preventDefault();
      ui.setPanel("export");
      return;
    }

    // Select all
    if (ctrl && e.key === "a") {
      e.preventDefault();
      const allIds = project.elements.map((el) => el.id);
      ui.selectElements(allIds);
      return;
    }

    // Group selected (Ctrl+G, need ≥2 selected)
    if (ctrl && e.key === "g" && !shift) {
      e.preventDefault();
      const selectedIds = Array.from(ui.selectedElementIds);
      if (selectedIds.length >= 2) {
        project.groupElements(selectedIds);
      }
      return;
    }

    // Ungroup selected (Ctrl+Shift+G, selected must be group)
    if (ctrl && shift && e.key === "G") {
      e.preventDefault();
      if (ui.selectedElementId) {
        const el = project.selectedElement;
        if (el?.type === "group") {
          project.ungroup(el.id);
        }
      }
      return;
    }

    // Delete selected (locked check)
    if ((e.key === "Delete" || e.key === "Backspace") && ui.selectedElementId) {
      const el = project.selectedElement;
      if (el && !el.locked) {
        e.preventDefault();
        project.removeElement(ui.selectedElementId);
        ui.selectElement(null);
      }
      return;
    }

    // Duplicate selected (Ctrl+D)
    if (ctrl && e.key === "d" && ui.selectedElementId) {
      e.preventDefault();
      project.duplicateElement(ui.selectedElementId);
      return;
    }

    // Arrow keys — nudge selected element (locked check)
    if (
      ui.selectedElementId &&
      ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(e.key)
    ) {
      const el = project.selectedElement;
      if (!el || el.locked) return;
      e.preventDefault();
      const step = shift ? 10 : 1;
      const dx = e.key === "ArrowLeft" ? -step : e.key === "ArrowRight" ? step : 0;
      const dy = e.key === "ArrowUp" ? -step : e.key === "ArrowDown" ? step : 0;
      project.updateElement(el.id, { x: el.x + dx, y: el.y + dy });
      return;
    }
  }

  onMounted(() => {
    document.addEventListener("keydown", handleKeydown);
  });

  onUnmounted(() => {
    document.removeEventListener("keydown", handleKeydown);
  });
}
