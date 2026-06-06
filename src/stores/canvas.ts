import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { CanvasResult, Gradient } from "@/types";

export const useCanvasStore = defineStore("canvas", () => {
  const width = ref(512);
  const height = ref(512);
  const background = ref("#FFFFFF");
  const cornerRadius = ref(0);
  const backgroundGradient = ref<Gradient | null>(null);

  function syncFromResult(canvas: CanvasResult) {
    width.value = canvas.width;
    height.value = canvas.height;
    background.value = canvas.background;
    cornerRadius.value = canvas.corner_radius;
    backgroundGradient.value = canvas.background_gradient ?? null;
  }

  async function pushCanvasUpdate(props: {
    width?: number;
    height?: number;
    background?: string;
    corner_radius?: number;
    background_gradient?: Gradient | null;
  }) {
    const invokeArgs: Record<string, unknown> = {};
    if (props.width !== undefined) invokeArgs.width = props.width;
    if (props.height !== undefined) invokeArgs.height = props.height;
    if (props.background !== undefined) invokeArgs.background = props.background;
    if (props.corner_radius !== undefined) invokeArgs.cornerRadius = props.corner_radius;
    if (props.background_gradient !== undefined) {
      if (props.background_gradient === null) {
        invokeArgs.clearBackgroundGradient = true;
      } else {
        invokeArgs.backgroundGradient = props.background_gradient;
      }
    }
    await invoke("update_canvas", invokeArgs);
    if (props.width !== undefined) width.value = props.width;
    if (props.height !== undefined) height.value = props.height;
    if (props.background !== undefined) background.value = props.background;
    if (props.corner_radius !== undefined) cornerRadius.value = props.corner_radius;
    if (props.background_gradient !== undefined) backgroundGradient.value = props.background_gradient;
  }

  async function fetchAndSync() {
    const canvas = await invoke<CanvasResult>("get_canvas");
    syncFromResult(canvas);
  }

  return { width, height, background, cornerRadius, backgroundGradient, syncFromResult, pushCanvasUpdate, fetchAndSync };
});
