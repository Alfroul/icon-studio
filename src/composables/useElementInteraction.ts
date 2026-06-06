import { ref, type Ref } from "vue";
import type { Element } from "@/types";

export function useElementInteraction(options: {
  elements: () => Element[];
  canvasWidth: () => number;
  canvasHeight: () => number;
  effectiveScale: () => number;
  panX: () => number;
  panY: () => number;
  onSelect: (id: string | null) => void;
  onMove: (id: string, x: number, y: number) => void;
  onResize?: (id: string, x: number, y: number, width: number, height: number) => void;
  containerRef: Ref<HTMLElement | null>;
}) {
  const isDragging = ref(false);
  const isResizing = ref(false);

  let dragStartScreenX = 0;
  let dragStartScreenY = 0;
  let dragElementStartX = 0;
  let dragElementStartY = 0;
  let dragElementId: string | null = null;

  let resizeHandleIndex = -1;
  let resizeStartScreenX = 0;
  let resizeStartScreenY = 0;
  let resizeElementStartBounds = { x: 0, y: 0, width: 0, height: 0 };
  let resizeElementId: string | null = null;

  const MIN_SIZE = 10;

  function screenToCanvas(clientX: number, clientY: number): { x: number; y: number } {
    const el = options.containerRef.value;
    if (!el) return { x: 0, y: 0 };
    const rect = el.getBoundingClientRect();
    const scale = options.effectiveScale();
    const panX = options.panX();
    const panY = options.panY();
    const containerCenterX = rect.left + rect.width / 2;
    const containerCenterY = rect.top + rect.height / 2;
    const x = ((clientX - containerCenterX) / scale + options.canvasWidth() / 2) - panX / scale;
    const y = ((clientY - containerCenterY) / scale + options.canvasHeight() / 2) - panY / scale;
    return { x, y };
  }

  function hitTest(canvasX: number, canvasY: number): Element | null {
    const els = options.elements();
    // Reverse order: top-most element first
    for (let i = els.length - 1; i >= 0; i--) {
      const el = els[i];
      if (
        canvasX >= el.x &&
        canvasX <= el.x + el.width &&
        canvasY >= el.y &&
        canvasY <= el.y + el.height
      ) {
        if (el.type === "group") {
          return el;
        }
        return el;
      }
    }
    return null;
  }

  function handleResizeMouseDown(e: MouseEvent, handleIndex: number, elementId: string) {
    if (e.button !== 0) return;
    if (!options.onResize) return;

    const el = options.elements().find((el) => el.id === elementId);
    if (!el) return;

    resizeHandleIndex = handleIndex;
    resizeStartScreenX = e.clientX;
    resizeStartScreenY = e.clientY;
    resizeElementStartBounds = { x: el.x, y: el.y, width: el.width, height: el.height };
    resizeElementId = el.id;
    isResizing.value = true;
    e.preventDefault();
    e.stopPropagation();
  }

  function handleOverlayMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    const { x, y } = screenToCanvas(e.clientX, e.clientY);
    const hit = hitTest(x, y);

    if (hit) {
      options.onSelect(hit.id);
      isDragging.value = true;
      dragStartScreenX = e.clientX;
      dragStartScreenY = e.clientY;
      dragElementStartX = hit.x;
      dragElementStartY = hit.y;
      dragElementId = hit.id;
      e.preventDefault();
    } else {
      options.onSelect(null);
    }
  }

  function handleOverlayMouseMove(e: MouseEvent) {
    if (isResizing.value && resizeElementId) {
      const el = options.containerRef.value;
      if (!el) return;

      const current = screenToCanvas(e.clientX, e.clientY);
      const start = screenToCanvas(resizeStartScreenX, resizeStartScreenY);
      const dx = current.x - start.x;
      const dy = current.y - start.y;

      const b = resizeElementStartBounds;
      let newX = b.x;
      let newY = b.y;
      let newW = b.width;
      let newH = b.height;

      // Handle indices: 0=top-left, 1=top-right, 2=bottom-left, 3=bottom-right
      switch (resizeHandleIndex) {
        case 0: // top-left: anchor is bottom-right
          newX = b.x + dx;
          newY = b.y + dy;
          newW = b.width - dx;
          newH = b.height - dy;
          break;
        case 1: // top-right: anchor is bottom-left
          newY = b.y + dy;
          newW = b.width + dx;
          newH = b.height - dy;
          break;
        case 2: // bottom-left: anchor is top-right
          newX = b.x + dx;
          newW = b.width - dx;
          newH = b.height + dy;
          break;
        case 3: // bottom-right: anchor is top-left
          newW = b.width + dx;
          newH = b.height + dy;
          break;
      }

      // Enforce minimum size
      if (newW < MIN_SIZE) {
        if (resizeHandleIndex === 0 || resizeHandleIndex === 2) {
          newX = b.x + b.width - MIN_SIZE;
        }
        newW = MIN_SIZE;
      }
      if (newH < MIN_SIZE) {
        if (resizeHandleIndex === 0 || resizeHandleIndex === 1) {
          newY = b.y + b.height - MIN_SIZE;
        }
        newH = MIN_SIZE;
      }

      options.onResize!(resizeElementId, Math.round(newX), Math.round(newY), Math.round(newW), Math.round(newH));
      return;
    }

    if (!isDragging.value || !dragElementId) return;

    const el = options.containerRef.value;
    if (!el) return;

    const current = screenToCanvas(e.clientX, e.clientY);
    const start = screenToCanvas(dragStartScreenX, dragStartScreenY);
    const dx = current.x - start.x;
    const dy = current.y - start.y;

    const newX = Math.round(dragElementStartX + dx);
    const newY = Math.round(dragElementStartY + dy);

    options.onMove(dragElementId, newX, newY);
  }

  function handleOverlayMouseUp() {
    isDragging.value = false;
    dragElementId = null;
    isResizing.value = false;
    resizeElementId = null;
    resizeHandleIndex = -1;
  }

  return {
    isDragging,
    isResizing,
    handleOverlayMouseDown,
    handleOverlayMouseMove,
    handleOverlayMouseUp,
    handleResizeMouseDown,
  };
}
