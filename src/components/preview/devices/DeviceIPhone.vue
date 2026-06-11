<script setup lang="ts">
const props = defineProps<{
  svgContent: string;
  wallpaper: string;
}>();

const placeholderColors = [
  "#4a9eff", "#ff6b6b", "#51cf66", "#ffd43b",
  "#845ef7", "#ff922b", "#20c997", "#f06595",
  "#339af0", "#ff8787", "#69db7c", "#ffe066",
  "#9775fa", "#ffa94d", "#38d9a9", "#f783ac",
  "#74c0fc", "#ff6b6b", "#8ce99a", "#ffd43b",
  "#b197fc", "#ffb347", "#63e6be", "#e64980",
];
const dockColors = ["#34c759", "#007aff", "#ff9500", "#ff2d55"];
</script>

<template>
  <div class="iphone">
    <div class="iphone-frame">
      <!-- Status bar -->
      <div class="status-bar">
        <span class="time">9:41</span>
        <div class="notch" />
        <div class="status-icons">
          <svg width="16" height="12" viewBox="0 0 16 12"><rect x="0" y="8" width="3" height="4" rx="0.5" fill="#fff"/><rect x="4" y="6" width="3" height="6" rx="0.5" fill="#fff"/><rect x="8" y="4" width="3" height="8" rx="0.5" fill="#fff"/><rect x="12" y="2" width="3" height="10" rx="0.5" fill="#fff"/></svg>
          <svg width="24" height="11" viewBox="0 0 24 11"><rect x="0" y="1" width="20" height="9" rx="2" fill="none" stroke="#fff" stroke-width="1"/><rect x="21" y="3.5" width="2" height="4" rx="1" fill="#fff"/><rect x="1.5" y="2.5" width="15" height="6" rx="1" fill="#34c759"/></svg>
        </div>
      </div>

      <!-- App grid -->
      <div class="app-grid">
        <div v-for="(_, idx) in 24" :key="idx" class="grid-slot">
          <div v-if="idx === 0" class="app-icon app-icon--active">
            <div class="icon-embed" v-if="svgContent" v-html="svgContent" />
          </div>
          <div v-else class="app-icon" :style="{ background: placeholderColors[idx % placeholderColors.length] }" />
          <span v-if="idx === 0" class="icon-label">My Icon</span>
        </div>
      </div>

      <!-- Dock -->
      <div class="dock">
        <div v-for="(color, i) in dockColors" :key="i" class="app-icon dock-icon" :style="{ background: color, opacity: 0.85 }" />
      </div>
    </div>
    <!-- Home indicator -->
    <div class="home-indicator" />
  </div>
</template>

<style scoped>
.iphone {
  width: 180px;
  background: #1c1c1e;
  border-radius: 28px;
  border: 2px solid #3a3a3c;
  padding: 8px;
  display: flex;
  flex-direction: column;
  position: relative;
  margin: 0 auto;
}

.iphone-frame {
  border-radius: 22px;
  overflow: hidden;
  flex: 1;
  display: flex;
  flex-direction: column;
  background: v-bind(wallpaper);
  min-height: 360px;
}

.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px 4px;
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  position: relative;
}

.time { font-family: system-ui, sans-serif; }

.notch {
  width: 80px;
  height: 22px;
  background: #000;
  border-radius: 11px;
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  top: 4px;
}

.status-icons {
  display: flex;
  gap: 4px;
  align-items: center;
}

.app-grid {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px 8px;
  padding: 20px 10px 8px;
  align-content: start;
}

.grid-slot {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
}

.app-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
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

.icon-label {
  font-size: 7px;
  color: #fff;
  text-align: center;
  opacity: 0.85;
  font-family: system-ui, sans-serif;
  max-width: 44px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dock {
  display: flex;
  justify-content: center;
  gap: 12px;
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.12);
  margin: 0 -0px;
}

.dock-icon { width: 32px; height: 32px; }

.home-indicator {
  width: 50%;
  height: 3px;
  background: rgba(255, 255, 255, 0.5);
  border-radius: 2px;
  margin: 6px auto 4px;
}
</style>
