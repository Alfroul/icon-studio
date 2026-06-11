<script setup lang="ts">
const props = defineProps<{
  svgContent: string;
  appName: string;
}>();
</script>

<template>
  <div class="browser">
    <div class="browser-chrome">
      <!-- Traffic lights -->
      <div class="traffic-lights">
        <span class="light light--close" />
        <span class="light light--minimize" />
        <span class="light light--maximize" />
      </div>

      <!-- Tab -->
      <div class="tab">
        <span class="tab-favicon">
          <span class="favicon-embed" v-if="svgContent" v-html="svgContent" />
        </span>
        <span class="tab-title">{{ appName || 'My App' }}</span>
        <span class="tab-close">&times;</span>
      </div>

      <!-- Address bar -->
      <div class="address-bar">
        <span class="lock-icon">&#128274;</span>
        <span class="url">https://{{ (appName || 'myapp').toLowerCase().replace(/\s+/g, '') }}.example.com</span>
      </div>
    </div>

    <!-- Content area -->
    <div class="browser-content">
      <div class="content-header">
        <div class="app-icon-large">
          <div class="icon-embed" v-if="svgContent" v-html="svgContent" />
        </div>
        <div class="app-info">
          <h2 class="app-name">{{ appName || 'My App' }}</h2>
          <div class="app-rating">&#9733;&#9733;&#9733;&#9733;&#9734;</div>
          <button class="download-btn">Download</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.browser {
  width: 280px;
  margin: 0 auto;
  border-radius: 8px;
  overflow: hidden;
  border: 1.5px solid #4a4a4a;
  background: #2d2d2d;
}

.browser-chrome {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.traffic-lights {
  display: flex;
  gap: 5px;
  padding: 8px 10px 4px;
}

.light {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.light--close { background: #ff5f56; }
.light--minimize { background: #ffbd2e; }
.light--maximize { background: #27c93f; }

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: #2d2d2d;
  font-size: 10px;
  color: #e0e0e0;
  font-family: system-ui, sans-serif;
}

.tab-favicon {
  width: 12px;
  height: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.favicon-embed :deep(svg) {
  width: 100%;
  height: 100%;
}

.tab-title { flex: 1; }

.tab-close {
  color: #888;
  cursor: pointer;
  font-size: 12px;
}

.address-bar {
  display: flex;
  align-items: center;
  gap: 5px;
  margin: 2px 8px 6px;
  padding: 4px 8px;
  background: #1c1c1e;
  border-radius: 4px;
  font-size: 10px;
  color: #888;
  font-family: system-ui, sans-serif;
}

.lock-icon { font-size: 8px; }

.browser-content {
  background: #fff;
  min-height: 140px;
}

.content-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
}

.app-icon-large {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  background: #f0f0f0;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  flex-shrink: 0;
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

.app-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.app-name {
  font-size: 14px;
  font-weight: 700;
  color: #333;
  margin: 0;
  font-family: system-ui, sans-serif;
}

.app-rating {
  color: #fbbc05;
  font-size: 12px;
}

.download-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 4px 14px;
  background: #007aff;
  color: #fff;
  border: none;
  border-radius: 12px;
  font-size: 10px;
  font-weight: 600;
  cursor: pointer;
  width: fit-content;
  font-family: system-ui, sans-serif;
}

.download-btn:hover { background: #0066d6; }
</style>
