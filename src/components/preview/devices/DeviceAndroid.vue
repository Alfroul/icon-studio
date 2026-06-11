<script setup lang="ts">
const props = defineProps<{
  svgContent: string;
  wallpaper: string;
}>();

const placeholderColors = [
  "#4285f4", "#ea4335", "#34a853", "#fbbc05",
  "#673ab7", "#ff5722", "#009688", "#e91e63",
  "#3f51b5", "#ff9800", "#00bcd4", "#8bc34a",
  "#9c27b0", "#795548", "#607d8b", "#cddc39",
  "#2196f3", "#f44336", "#4caf50", "#ffc107",
];
const dockColors = ["#4285f4", "#34a853", "#fbbc05", "#ea4335"];
</script>

<template>
  <div class="android">
    <div class="android-frame">
      <!-- Status bar -->
      <div class="status-bar">
        <span class="time">12:00</span>
        <div class="status-icons">
          <svg width="24" height="11" viewBox="0 0 24 11"><rect x="0" y="1" width="20" height="9" rx="2" fill="none" stroke="#fff" stroke-width="1"/><rect x="1.5" y="2.5" width="15" height="6" rx="1" fill="#34a853"/></svg>
        </div>
      </div>

      <!-- Search bar -->
      <div class="search-bar">
        <span class="search-text">Search</span>
      </div>

      <!-- App grid -->
      <div class="app-grid">
        <div v-for="(_, idx) in 20" :key="idx" class="grid-slot">
          <div v-if="idx === 0" class="app-icon app-icon--active">
            <div class="icon-embed" v-if="svgContent" v-html="svgContent" />
          </div>
          <div v-else class="app-icon" :style="{ background: placeholderColors[idx % placeholderColors.length] }" />
        </div>
      </div>

      <!-- Dock -->
      <div class="dock">
        <div v-for="(color, i) in dockColors" :key="i" class="dock-icon" :style="{ background: color, opacity: 0.85 }" />
      </div>
    </div>
    <!-- Navigation bar -->
    <div class="nav-bar" />
  </div>
</template>

<style scoped>
.android {
  width: 170px;
  background: #1c1c1e;
  border-radius: 24px;
  border: 2px solid #3a3a3c;
  padding: 6px;
  display: flex;
  flex-direction: column;
  position: relative;
  margin: 0 auto;
}

.android-frame {
  border-radius: 20px;
  overflow: hidden;
  flex: 1;
  display: flex;
  flex-direction: column;
  background: v-bind(wallpaper);
  min-height: 340px;
}

.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px 2px;
  color: #fff;
  font-size: 10px;
  font-weight: 500;
}

.time { font-family: system-ui, sans-serif; }

.search-bar {
  margin: 6px 10px;
  padding: 8px 14px;
  background: rgba(255, 255, 255, 0.15);
  border-radius: 20px;
}

.search-text {
  color: rgba(255, 255, 255, 0.5);
  font-size: 11px;
  font-family: system-ui, sans-serif;
}

.app-grid {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 14px 6px;
  padding: 10px 8px;
  align-content: start;
}

.grid-slot {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.app-icon {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.app-icon--active {
  background: #333;
}

.icon-embed {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-embed :deep(svg) {
  width: 100%;
  height: 100%;
}

.dock {
  display: flex;
  justify-content: center;
  gap: 10px;
  padding: 8px 20px;
  margin: 0 8px 4px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 20px;
}

.dock-icon {
  width: 26px;
  height: 26px;
  border-radius: 50%;
}

.nav-bar {
  width: 40%;
  height: 3px;
  background: rgba(255, 255, 255, 0.4);
  border-radius: 2px;
  margin: 5px auto 3px;
}
</style>
