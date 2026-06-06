import { onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { useProjectStore } from "@/stores/project";
import { useCanvasStore } from "@/stores/canvas";
import { usePagesStore } from "@/stores/pages";

export function useProjectSync() {
  const project = useProjectStore();
  const canvas = useCanvasStore();
  const pagesStore = usePagesStore();
  let unlisten: (() => void) | null = null;

  onMounted(async () => {
    unlisten = await listen("project-changed", async () => {
      await project.refreshElements();
      project.debouncedFetchPreview();
      await canvas.fetchAndSync();
      await pagesStore.refreshPages();
    });
  });

  onUnmounted(() => {
    if (unlisten) unlisten();
  });
}
