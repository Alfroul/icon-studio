<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "@/stores/project";
import AppIcon from "@/components/common/AppIcon.vue";

const project = useProjectStore();

const isPlaying = ref(false);
const currentTimeMs = ref(0);
const durationMs = ref(2000);
const fps = ref(30);
const previewDataUrl = ref("");
let playInterval: ReturnType<typeof setInterval> | null = null;

const totalFrames = computed(() => Math.ceil((durationMs.value / 1000) * fps.value));
const currentFrame = computed(() => Math.floor((currentTimeMs.value / 1000) * fps.value));

function formatTime(ms: number): string {
  const s = (ms / 1000).toFixed(1);
  return `${s}s`;
}

async function fetchFrame(timeMs: number) {
  try {
    const dataUrl = await invoke<string>("preview_animation_frame", {
      timeMs,
    });
    previewDataUrl.value = dataUrl;
  } catch {
    // ignore — frame fetch failures are non-critical
  }
}

function play() {
  if (isPlaying.value) return;
  isPlaying.value = true;
  const stepMs = 1000 / fps.value;
  playInterval = setInterval(() => {
    currentTimeMs.value += stepMs;
    if (currentTimeMs.value >= durationMs.value) {
      currentTimeMs.value = 0;
    }
    fetchFrame(currentTimeMs.value);
  }, stepMs);
}

function pause() {
  isPlaying.value = false;
  if (playInterval) {
    clearInterval(playInterval);
    playInterval = null;
  }
}

function togglePlay() {
  if (isPlaying.value) pause();
  else play();
}

function seekTo(ms: number) {
  currentTimeMs.value = Math.max(0, Math.min(ms, durationMs.value));
  fetchFrame(currentTimeMs.value);
}

function onSliderInput(e: Event) {
  const val = Number((e.target as HTMLInputElement).value);
  seekTo(val);
}

onUnmounted(() => {
  pause();
});

// Load initial frame when preview becomes available
watch(
  () => project.svgPreview,
  () => {
    fetchFrame(0);
  },
  { immediate: true }
);
</script>

<template>
  <div class="animation-preview">
    <div class="panel-section">
      <h3 class="section-title">Animation Preview</h3>

      <div class="preview-area">
        <img
          v-if="previewDataUrl"
          :src="previewDataUrl"
          class="preview-image"
          alt="Animation frame"
        />
        <p v-else class="placeholder-text">No animation to preview.</p>
      </div>

      <div class="controls" v-if="previewDataUrl">
        <button class="play-btn" @click="togglePlay">
          <AppIcon :name="isPlaying ? 'pause' : 'play'" :size="16" />
        </button>

        <input
          type="range"
          class="timeline"
          :min="0"
          :max="durationMs"
          :value="currentTimeMs"
          step="33"
          @input="onSliderInput"
        />

        <div class="info">
          <span class="fps-label">{{ fps }} fps</span>
          <span class="frame-counter">{{ currentFrame }} / {{ totalFrames }}</span>
          <span class="time-label">{{ formatTime(currentTimeMs) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.animation-preview {
  padding: 12px;
}
.panel-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-secondary);
  margin: 0;
}
.preview-area {
  min-height: 120px;
  max-height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px dashed var(--border-color);
  border-radius: var(--radius-md);
  overflow: hidden;
  background: repeating-conic-gradient(var(--bg-tertiary) 0% 25%, var(--bg-secondary) 0% 50%) 50% / 8px 8px;
}
.preview-image {
  max-width: 100%;
  max-height: 180px;
  image-rendering: auto;
}
.placeholder-text {
  color: var(--text-muted);
  font-size: 12px;
}
.controls {
  display: flex;
  align-items: center;
  gap: 8px;
}
.play-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast) ease;
  flex-shrink: 0;
}
.play-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}
.play-btn:active {
  background: var(--accent);
  color: var(--bg-primary);
}
.timeline {
  flex: 1;
  height: 4px;
  appearance: none;
  background: var(--bg-tertiary);
  border-radius: 2px;
  cursor: pointer;
  outline: none;
}
.timeline::-webkit-slider-thumb {
  appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--accent);
  cursor: pointer;
  border: none;
}
.timeline::-webkit-slider-runnable-track {
  height: 4px;
  border-radius: 2px;
}
.info {
  display: flex;
  gap: 6px;
  align-items: center;
  flex-shrink: 0;
}
.fps-label,
.frame-counter,
.time-label {
  font-size: 10px;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
  white-space: nowrap;
}
</style>
