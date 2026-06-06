<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";

const ui = useUiStore();
const project = useProjectStore();

interface LibraryAsset {
  name: string;
  label: string;
  category: string;
  tags: string[];
  svg: string;
}

const categories = ref<string[]>([]);
const activeCategory = ref("");
const assets = ref<LibraryAsset[]>([]);
const searchKeyword = ref("");
const loading = ref(true);

onMounted(async () => {
  try {
    categories.value = await invoke<string[]>("list_library_categories");
    if (categories.value.length > 0) {
      activeCategory.value = categories.value[0];
    }
    await loadAssets();
  } catch (e) {
    console.error("Failed to load library:", e);
  } finally {
    loading.value = false;
  }
});

async function loadAssets() {
  loading.value = true;
  try {
    assets.value = await invoke<LibraryAsset[]>("list_library_assets", {
      category: activeCategory.value || null,
      keyword: searchKeyword.value || null,
    });
  } catch (e) {
    console.error("Failed to load assets:", e);
    assets.value = [];
  } finally {
    loading.value = false;
  }
}

async function selectCategory(cat: string) {
  activeCategory.value = cat;
  await loadAssets();
}

async function search() {
  await loadAssets();
}

async function addAsset(assetName: string) {
  try {
    await invoke("add_library_asset", { assetName });
    await project.refreshElements();
    ui.showToast("Asset added", "success");
  } catch (e) {
    ui.showToast(`Failed: ${e}`, "error");
  }
}

function onDragStart(e: DragEvent, asset: LibraryAsset) {
  e.dataTransfer?.setData(
    "application/json",
    JSON.stringify({ type: "library-asset", name: asset.name }),
  );
  e.dataTransfer!.effectAllowed = "copy";
}
</script>

<template>
  <div class="library-panel">
    <div v-if="loading && assets.length === 0" class="loading">Loading library...</div>

    <template v-else>
      <!-- Category tabs -->
      <div class="category-tabs">
        <button
          v-for="cat in categories"
          :key="cat"
          :class="['tab-btn', { active: activeCategory === cat }]"
          @click="selectCategory(cat)"
        >
          {{ cat }}
        </button>
      </div>

      <!-- Search -->
      <div class="search-row">
        <input
          v-model="searchKeyword"
          class="search-input"
          placeholder="Search assets..."
          @keyup.enter="search"
        />
      </div>

      <!-- Asset grid -->
      <div v-if="assets.length > 0" class="asset-grid">
        <div
          v-for="asset in assets"
          :key="asset.name"
          class="asset-card"
          draggable="true"
          @dragstart="onDragStart($event, asset)"
          @click="addAsset(asset.name)"
          :title="asset.label"
        >
          <img
            class="asset-preview"
            :src="'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(asset.svg)"
            :alt="asset.label"
          />
          <div class="asset-label">{{ asset.label }}</div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-else class="empty">
        <template v-if="searchKeyword">No assets match "{{ searchKeyword }}"</template>
        <template v-else>No assets in this category</template>
      </div>
    </template>
  </div>
</template>

<style scoped>
.library-panel {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  overflow-y: auto;
  height: 100%;
}

.loading,
.empty {
  color: var(--text-muted);
  font-size: 12px;
  padding: 8px 0;
  text-align: center;
}

/* Category tabs */
.category-tabs {
  display: flex;
  gap: 4px;
  overflow-x: auto;
  padding-bottom: 4px;
  scrollbar-width: none;
}

.category-tabs::-webkit-scrollbar {
  display: none;
}

.tab-btn {
  flex-shrink: 0;
  padding: 4px 10px;
  font-size: 11px;
  background: var(--bg-tertiary);
  color: var(--text-muted);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  cursor: pointer;
  white-space: nowrap;
  transition: all var(--transition-fast);
}

.tab-btn:hover {
  color: var(--text-secondary);
  border-color: var(--text-muted);
}

.tab-btn.active {
  background: var(--accent-muted);
  color: var(--accent);
  border-color: var(--accent);
}

/* Search */
.search-row {
  display: flex;
}

.search-input {
  width: 100%;
  height: 28px;
  padding: 0 10px;
  background: var(--input-bg);
  color: var(--text-primary);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  font-size: 12px;
  outline: none;
  transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-muted);
}

/* Asset grid — 3 columns */
.asset-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
}

.asset-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: 6px 4px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  user-select: none;
}

.asset-card:hover {
  border-color: var(--accent);
  box-shadow: 0 0 12px var(--accent-glow);
  transform: translateY(-1px);
}

.asset-card:active {
  transform: translateY(0);
  background: var(--bg-hover);
}

/* Drag ghost styling */
.asset-card:drag {
  opacity: 0.6;
}

.asset-preview {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  object-fit: contain;
}

.asset-label {
  font-size: 9px;
  color: var(--text-muted);
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
  line-height: 1.2;
}
</style>
