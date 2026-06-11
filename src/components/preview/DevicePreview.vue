<script setup lang="ts">
import { ref, computed } from "vue";
import { useProjectStore } from "@/stores/project";
import { wallpapers } from "./wallpapers";
import DeviceIPhone from "./devices/DeviceIPhone.vue";
import DeviceAndroid from "./devices/DeviceAndroid.vue";
import DeviceMacBook from "./devices/DeviceMacBook.vue";
import DeviceBrowser from "./devices/DeviceBrowser.vue";

const project = useProjectStore();

const devices = [
  { id: "iphone", label: "iPhone" },
  { id: "android", label: "Android" },
  { id: "macbook", label: "MacBook" },
  { id: "browser", label: "Browser" },
] as const;

type DeviceId = typeof devices[number]["id"];
const activeDevice = ref<DeviceId>("iphone");
const activeWallpaper = ref(wallpapers[0].id);
const appName = ref("My App");

const currentWallpaperStyle = computed(
  () => wallpapers.find((w) => w.id === activeWallpaper.value)?.style ?? wallpapers[0].style,
);

const currentSvg = computed(() => project.svgPreview);
</script>

<template>
  <div class="device-preview">
    <!-- Device tab bar -->
    <div class="device-tabs">
      <button
        v-for="device in devices"
        :key="device.id"
        :class="['device-tab', { 'device-tab--active': activeDevice === device.id }]"
        @click="activeDevice = device.id"
      >
        {{ device.label }}
      </button>
    </div>

    <!-- Device preview area -->
    <div class="preview-area">
      <DeviceIPhone
        v-if="activeDevice === 'iphone'"
        :svg-content="currentSvg"
        :wallpaper="currentWallpaperStyle"
      />
      <DeviceAndroid
        v-if="activeDevice === 'android'"
        :svg-content="currentSvg"
        :wallpaper="currentWallpaperStyle"
      />
      <DeviceMacBook
        v-if="activeDevice === 'macbook'"
        :svg-content="currentSvg"
        :wallpaper="currentWallpaperStyle"
      />
      <DeviceBrowser
        v-if="activeDevice === 'browser'"
        :svg-content="currentSvg"
        :app-name="appName"
      />
    </div>

    <!-- Wallpaper selector -->
    <div class="wallpaper-section">
      <span class="section-label">Wallpaper</span>
      <div class="wallpaper-grid">
        <button
          v-for="wp in wallpapers"
          :key="wp.id"
          :class="['wallpaper-swatch', { 'wallpaper-swatch--active': activeWallpaper === wp.id }]"
          :style="{ background: wp.style }"
          :title="wp.name"
          @click="activeWallpaper = wp.id"
        />
      </div>
    </div>

    <!-- App name (for browser preview) -->
    <div v-if="activeDevice === 'browser'" class="field-row">
      <label class="section-label">App Name</label>
      <input
        v-model="appName"
        class="field-input"
        placeholder="My App"
      />
    </div>
  </div>
</template>

<style scoped>
.device-preview {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
  height: 100%;
}

.device-tabs {
  display: flex;
  gap: 2px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  padding: 2px;
}

.device-tab {
  flex: 1;
  padding: 5px 0;
  border: none;
  background: none;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
  font-family: system-ui, sans-serif;
}

.device-tab:hover {
  color: var(--text-secondary);
  background: var(--bg-hover);
}

.device-tab--active {
  color: var(--text-primary);
  background: var(--bg-primary);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
}

.preview-area {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 12px 0;
  min-height: 200px;
}

.wallpaper-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.section-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-secondary);
}

.wallpaper-grid {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.wallpaper-swatch {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: 2px solid transparent;
  cursor: pointer;
  transition: border-color var(--transition-fast), transform var(--transition-fast);
}

.wallpaper-swatch:hover {
  transform: scale(1.1);
}

.wallpaper-swatch--active {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent);
}

.field-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field-input {
  height: 28px;
  padding: 0 8px;
  font-size: 11px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
  transition: border-color var(--transition-fast);
}

.field-input:focus {
  border-color: var(--accent);
}
</style>
