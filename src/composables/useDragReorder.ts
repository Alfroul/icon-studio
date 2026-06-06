import { ref } from "vue";

export function useDragReorder(options: {
  onReorder: (fromIndex: number, toIndex: number) => void;
}) {
  const dragIndex = ref<number | null>(null);
  const dropIndex = ref<number | null>(null);
  const isDragging = ref(false);

  function onDragStart(index: number, e: DragEvent) {
    if (!e.dataTransfer) return;
    dragIndex.value = index;
    isDragging.value = true;
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", String(index));
  }

  function onDragOver(index: number, e: DragEvent) {
    if (dragIndex.value === null || !e.dataTransfer) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";

    const target = e.currentTarget as HTMLElement;
    if (!target) return;

    const rect = target.getBoundingClientRect();
    const midY = rect.top + rect.height / 2;

    // Top half → drop above (dropIndex = index), bottom half → drop below (dropIndex = index + 1)
    dropIndex.value = e.clientY < midY ? index : index + 1;
  }

  function onDrop(_index: number, e: DragEvent) {
    e.preventDefault();
    if (dragIndex.value !== null && dropIndex.value !== null) {
      options.onReorder(dragIndex.value, dropIndex.value);
    }
    resetState();
  }

  function onDragEnd() {
    resetState();
  }

  function resetState() {
    dragIndex.value = null;
    dropIndex.value = null;
    isDragging.value = false;
  }

  return {
    dragIndex,
    dropIndex,
    isDragging,
    onDragStart,
    onDragOver,
    onDrop,
    onDragEnd,
  };
}
