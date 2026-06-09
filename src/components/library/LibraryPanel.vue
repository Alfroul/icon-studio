<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { usePacksStore, type PackIcon } from "@/stores/packs";

const ui = useUiStore();
const project = useProjectStore();
const packs = usePacksStore();

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

// Pack browsing state
const activeTab = ref<"library" | "packs">("library");
const packSearchQuery = ref("");
const packPage = ref(0);
const PACK_PAGE_SIZE = 100;

// Computed: filtered/paginated icons for current pack
const displayedPackIcons = computed(() => {
  const icons = packSearchQuery.value
    ? packs.searchResults
    : packs.currentPackIcons;
  const start = packPage.value * PACK_PAGE_SIZE;
  return icons.slice(start, start + PACK_PAGE_SIZE);
});

const totalPackPages = computed(() => {
  const total = packSearchQuery.value
    ? packs.searchResults.length
    : packs.currentPackIcons.length;
  return Math.ceil(total / PACK_PAGE_SIZE);
});

// Inline SVG cache for grid previews
const iconSvgMap = ref<Map<string, string>>(new Map());

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
  await packs.loadPacks();
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

// --- Pack functions ---

async function importPack() {
  try {
    const selected = await open({ directory: true, multiple: false });
    if (!selected) return;
    const dir = typeof selected === "string" ? selected : selected[0] ?? "";
    if (!dir) return;
    const name = dir.split(/[/\\]/).filter(Boolean).pop() || "Untitled Pack";
    await packs.importPack(dir, name);
  } catch (e) {
    ui.showToast(`Failed: ${e}`, "error");
  }
}

async function openPack(packId: string) {
  await packs.loadPackIcons(packId);
  packSearchQuery.value = "";
  packPage.value = 0;
  iconSvgMap.value.clear();
  // Preload SVG for visible icons
  preloadIcons();
}

async function preloadIcons() {
  const toLoad = displayedPackIcons.value.filter(icon => {
    const key = `${packs.currentPackId}/${icon.name}`;
    return !iconSvgMap.value.has(key);
  });
  if (!packs.currentPackId || toLoad.length === 0) return;
  const pid = packs.currentPackId;
  await Promise.allSettled(
    toLoad.map(async (icon) => {
      const svg = await packs.loadIconSvg(pid, icon.name);
      iconSvgMap.value.set(`${pid}/${icon.name}`, svg);
    }),
  );
}

let searchTimer: ReturnType<typeof setTimeout> | null = null;

function searchPackIcons() {
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = setTimeout(async () => {
    if (!packs.currentPackId) return;
    if (!packSearchQuery.value.trim()) {
      packs.searchResults = [];
      return;
    }
    await packs.searchIcons(packs.currentPackId, packSearchQuery.value);
    packPage.value = 0;
  }, 300);
}

async function addPackIcon(icon: PackIcon) {
  if (!packs.currentPackId) return;
  await packs.addIconToCanvas(packs.currentPackId, icon.name);
}

async function deletePack(packId: string, e: MouseEvent) {
  e.stopPropagation();
  await packs.removePack(packId);
}

function backToPackList() {
  packs.clearCurrentPack();
  iconSvgMap.value.clear();
  packSearchQuery.value = "";
  packPage.value = 0;
}

interface HighlightSegment {
  text: string;
  highlight: boolean;
}

function splitHighlight(text: string, query: string): HighlightSegment[] {
  if (!query) return [{ text, highlight: false }];
  const idx = text.toLowerCase().indexOf(query.toLowerCase());
  if (idx === -1) return [{ text, highlight: false }];
  return [
    { text: text.slice(0, idx), highlight: false },
    { text: text.slice(idx, idx + query.length), highlight: true },
    { text: text.slice(idx + query.length), highlight: false },
  ];
}
</script>

<template>
  <div class="library-panel">
    <!-- Tab switcher -->
    <div class="tab-switcher">
      <button
        :class="['switch-btn', { active: activeTab === 'library' }]"
        @click="activeTab = 'library'"
      >
        Library
      </button>
      <button
        :class="['switch-btn', { active: activeTab === 'packs' }]"
        @click="activeTab = 'packs'"
      >
        Icon Packs
      </button>
    </div>

    <!-- Library tab -->
    <template v-if="activeTab === 'library'">
      <div v-if="loading && assets.length === 0" class="loading">Loading library...</div>
      <template v-else>
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

        <div class="search-row">
          <input
            v-model="searchKeyword"
            class="search-input"
            placeholder="Search assets..."
            @keyup.enter="search"
          />
        </div>

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

        <div v-else class="empty">
          <template v-if="searchKeyword">No assets match "{{ searchKeyword }}"</template>
          <template v-else>No assets in this category</template>
        </div>
      </template>
    </template>

    <!-- Packs tab -->
    <template v-if="activeTab === 'packs'">
      <!-- Pack list view -->
      <template v-if="!packs.currentPackId">
        <button class="import-pack-btn" @click="importPack">
          Import Icon Pack
        </button>

        <div v-if="packs.packs.length === 0" class="empty">
          No icon packs imported yet
        </div>

        <div v-else class="pack-list">
          <div
            v-for="pack in packs.packs"
            :key="pack.id"
            class="pack-card"
            @click="openPack(pack.id)"
          >
            <div class="pack-info">
              <div class="pack-name">{{ pack.name }}</div>
              <div class="pack-meta">{{ pack.iconCount }} icons</div>
            </div>
            <div class="pack-categories">
              <span v-for="cat in pack.categories.slice(0, 3)" :key="cat" class="chip">
                {{ cat }}
              </span>
            </div>
            <button
              class="pack-delete-btn"
              title="Remove pack"
              @click="deletePack(pack.id, $event)"
            >
              ×
            </button>
          </div>
        </div>
      </template>

      <!-- Pack detail (icons) view -->
      <template v-else>
        <button class="back-btn" @click="backToPackList">← Back to packs</button>

        <div class="search-row">
          <input
            v-model="packSearchQuery"
            class="search-input"
            placeholder="Search icons..."
            @input="searchPackIcons"
          />
        </div>

        <div v-if="packs.loading" class="loading">Loading icons...</div>

        <div v-else-if="displayedPackIcons.length > 0" class="asset-grid">
          <div
            v-for="icon in displayedPackIcons"
            :key="icon.name"
            class="asset-card"
            @click="addPackIcon(icon)"
            :title="icon.name"
          >
            <img
              class="asset-preview"
              :src="iconSvgMap.get(`${packs.currentPackId}/${icon.name}`)
                ? 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(iconSvgMap.get(`${packs.currentPackId}/${icon.name}`)!)
                : ''"
              :alt="icon.name"
            />
            <div class="asset-label">
              <template v-if="packSearchQuery">
                <span
                  v-for="(seg, si) in splitHighlight(icon.name, packSearchQuery)"
                  :key="si"
                  :class="{ highlight: seg.highlight }"
                >{{ seg.text }}</span>
              </template>
              <template v-else>{{ icon.name }}</template>
            </div>
            <span class="chip chip-small">{{ icon.category }}</span>
          </div>
        </div>

        <div v-else class="empty">
          <template v-if="packSearchQuery">No icons match "{{ packSearchQuery }}"</template>
          <template v-else>No icons in this pack</template>
        </div>

        <!-- Pagination -->
        <div v-if="totalPackPages > 1" class="pagination">
          <button
            class="page-btn"
            :disabled="packPage === 0"
            @click="packPage--; preloadIcons()"
          >
            ←
          </button>
          <span class="page-info">{{ packPage + 1 }} / {{ totalPackPages }}</span>
          <button
            class="page-btn"
            :disabled="packPage >= totalPackPages - 1"
            @click="packPage++; preloadIcons()"
          >
            →
          </button>
        </div>
      </template>
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

/* Tab switcher */
.tab-switcher {
  display: flex;
  gap: 2px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  padding: 2px;
}

.switch-btn {
  flex: 1;
  padding: 5px 0;
  font-size: 11px;
  background: transparent;
  color: var(--text-muted);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.switch-btn.active {
  background: var(--accent-muted);
  color: var(--accent);
}

.switch-btn:hover:not(.active) {
  color: var(--text-secondary);
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

.asset-label :deep(mark) {
  background: var(--accent-muted);
  color: var(--accent);
  border-radius: 2px;
  padding: 0 1px;
}

/* Import Pack button */
.import-pack-btn {
  width: 100%;
  height: 28px;
  background: var(--accent-muted);
  border: 1px solid var(--accent);
  border-radius: var(--radius-md);
  color: var(--accent);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.import-pack-btn:hover {
  background: var(--accent);
  color: #fff;
}

/* Pack list */
.pack-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.pack-card {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
}

.pack-card:hover {
  border-color: var(--accent);
  box-shadow: 0 0 12px var(--accent-glow);
}

.pack-info {
  flex: 1;
  min-width: 0;
}

.pack-name {
  font-size: 12px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pack-meta {
  font-size: 10px;
  color: var(--text-muted);
}

.pack-categories {
  display: flex;
  gap: 3px;
  flex-wrap: wrap;
}

.pack-delete-btn {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 18px;
  height: 18px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 14px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.pack-card:hover .pack-delete-btn {
  opacity: 1;
}

.pack-delete-btn:hover {
  color: #e55;
  background: var(--bg-hover);
}

/* Chip */
.chip {
  display: inline-block;
  padding: 1px 6px;
  font-size: 9px;
  background: var(--bg-tertiary);
  color: var(--text-muted);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  white-space: nowrap;
}

.chip-small {
  font-size: 8px;
  padding: 0 4px;
}

/* Back button */
.back-btn {
  background: none;
  border: none;
  color: var(--accent);
  font-size: 11px;
  cursor: pointer;
  padding: 2px 0;
  text-align: left;
}

.back-btn:hover {
  text-decoration: underline;
}

/* Pagination */
.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 4px 0;
}

.page-btn {
  width: 24px;
  height: 24px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  font-size: 11px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.page-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.page-btn:not(:disabled):hover {
  border-color: var(--accent);
  color: var(--accent);
}

.page-info {
  font-size: 11px;
  color: var(--text-muted);
}
</style>
