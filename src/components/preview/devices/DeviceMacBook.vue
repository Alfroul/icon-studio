<script setup lang="ts">
const props = defineProps<{
  svgContent: string;
  wallpaper: string;
}>();

const dockItems = [
  { name: "Finder", color: "#333" },
  { name: "Safari", color: "#007aff" },
  { name: "Messages", color: "#34c759" },
  { name: "Music", color: "#ff9500" },
  { name: "Photos", color: "#ff2d55" },
  { name: "Trash", color: "#8e8e93" },
];
</script>

<template>
  <div class="macbook">
    <div class="macbook-screen">
      <div class="screen-content">
        <!-- Menu bar -->
        <div class="menu-bar">
          <span class="menu-app">Finder</span>
          <span class="menu-time">12:00</span>
        </div>

        <!-- Desktop icons -->
        <div class="desktop-icons">
          <div class="desktop-icon">
            <div class="icon-embed" v-if="svgContent" v-html="svgContent" />
            <span class="icon-name">MyIcon</span>
          </div>
        </div>

        <!-- Dock -->
        <div class="dock">
          <div
            v-for="(item, i) in dockItems"
            :key="item.name"
            class="dock-item"
            :class="{ 'dock-item--active': i === 0 }"
          >
            <div v-if="i === 0 && svgContent" class="dock-icon-wrap">
              <div class="icon-embed" v-html="svgContent" />
            </div>
            <div v-else class="dock-icon-wrap" :style="{ background: item.color }" />
          </div>
        </div>
      </div>
    </div>
    <!-- MacBook base -->
    <div class="macbook-base">
      <div class="trackpad" />
    </div>
  </div>
</template>

<style scoped>
.macbook {
  width: 280px;
  margin: 0 auto;
}

.macbook-screen {
  background: #2d2d2d;
  border: 1.5px solid #4a4a4a;
  border-radius: 8px;
  padding: 3px;
}

.screen-content {
  border-radius: 4px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: v-bind(wallpaper);
  height: 170px;
}

.menu-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 3px 10px;
  background: rgba(0, 0, 0, 0.5);
  color: #fff;
  font-size: 10px;
  font-family: system-ui, sans-serif;
}

.menu-app { font-weight: 600; }

.desktop-icons {
  flex: 1;
  padding: 12px;
  display: flex;
  flex-wrap: wrap;
  align-content: start;
  gap: 16px;
}

.desktop-icon {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  width: 44px;
}

.icon-embed {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  overflow: hidden;
}

.desktop-icon .icon-embed {
  background: rgba(0, 0, 0, 0.3);
}

.icon-embed :deep(svg) {
  width: 100%;
  height: 100%;
}

.icon-name {
  font-size: 8px;
  color: #fff;
  text-align: center;
  font-family: system-ui, sans-serif;
  opacity: 0.9;
  max-width: 50px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dock {
  display: flex;
  justify-content: center;
  gap: 4px;
  padding: 4px 8px;
  margin: 0 30px 4px;
  background: rgba(255, 255, 255, 0.15);
  border-radius: 6px;
}

.dock-item { display: flex; flex-direction: column; align-items: center; }

.dock-icon-wrap {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.dock-item--active .dock-icon-wrap { background: #333; }

.dock-icon-wrap .icon-embed :deep(svg) {
  width: 100%;
  height: 100%;
}

.macbook-base {
  background: #3a3a3c;
  border-radius: 0 0 4px 4px;
  padding: 4px 0;
  display: flex;
  justify-content: center;
}

.trackpad {
  width: 60px;
  height: 8px;
  background: #2d2d2d;
  border-radius: 2px;
  border: 0.5px solid #4a4a4a;
}
</style>
