<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";

const project = useProjectStore();
const ui = useUiStore();

const searchQuery = ref("");
const icons = ref<{ name: string; path: string }[]>([]);
const loading = ref(false);
const dialogRef = ref<HTMLElement | null>(null);
const searchInputRef = ref<HTMLInputElement | null>(null);

onMounted(() => {
  searchIcons("");
});

let searchSeq = 0;

async function searchIcons(keyword: string) {
  const seq = ++searchSeq;
  loading.value = true;
  try {
    const result = await invoke<{ name: string; path: string }[]>("list_icons", {
      keyword,
    });
    if (seq !== searchSeq) return;
    icons.value = result;
  } catch (e) {
    if (seq !== searchSeq) return;
    console.error("Failed to list icons:", e);
    icons.value = [];
  } finally {
    if (seq === searchSeq) loading.value = false;
  }
}

let debounceTimer: ReturnType<typeof setTimeout> | null = null;
function onSearchInput() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    searchIcons(searchQuery.value);
  }, 300);
}

function selectIcon(iconName: string) {
  const size = 200;
  const x = Math.round(project.canvasWidth / 2 - size / 2);
  const y = Math.round(project.canvasHeight / 2 - size / 2);
  project.addIcon(iconName, "#333333", size, x, y);
  ui.toggleIconBrowser(false);
}

function close() {
  ui.toggleIconBrowser(false);
}

function getFocusableElements(): HTMLElement[] {
  if (!dialogRef.value) return [];
  const selectors = 'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';
  return Array.from(dialogRef.value.querySelectorAll<HTMLElement>(selectors));
}

function onDialogKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    close();
    return;
  }
  if (e.key !== "Tab") return;

  const focusable = getFocusableElements();
  if (focusable.length === 0) return;

  const first = focusable[0];
  const last = focusable[focusable.length - 1];

  if (e.shiftKey) {
    if (document.activeElement === first) {
      e.preventDefault();
      last.focus();
    }
  } else {
    if (document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }
}

// Focus search input when dialog opens
watch(() => ui.iconBrowserOpen, async (open) => {
  if (open) {
    await nextTick();
    searchInputRef.value?.focus();
  }
});

onUnmounted(() => {
  if (debounceTimer) clearTimeout(debounceTimer);
});
</script>

<template>
  <Teleport to="body">
    <div v-if="ui.iconBrowserOpen" class="icon-browser-overlay" @click.self="close">
      <div
        ref="dialogRef"
        class="icon-browser"
        role="dialog"
        aria-modal="true"
        aria-label="Browse icons"
        @keydown="onDialogKeydown"
      >
        <div class="icon-browser-header">
          <span class="icon-browser-title">Select Icon</span>
          <button class="close-btn" aria-label="Close" @click="close">✕</button>
        </div>
        <div class="icon-browser-search">
          <input
            ref="searchInputRef"
            v-model="searchQuery"
            type="text"
            placeholder="Search icons..."
            class="search-input"
            aria-label="Search icons"
            @input="onSearchInput"
          />
        </div>
        <div class="icon-browser-grid">
          <div v-if="loading" class="icon-browser-empty">Loading...</div>
          <div v-else-if="icons.length === 0" class="icon-browser-empty">No icons found</div>
          <button
            v-for="icon in icons"
            :key="icon.name"
            class="icon-card"
            :title="icon.name"
            @click="selectIcon(icon.name)"
          >
            <svg
              viewBox="0 0 24 24"
              width="24"
              height="24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path :d="icon.path" />
            </svg>
            <span class="icon-card-name">{{ icon.name }}</span>
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.icon-browser-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.icon-browser {
  width: 520px;
  max-height: 70vh;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.icon-browser-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-color);
}

.icon-browser-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 14px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.icon-browser-search {
  padding: 8px 14px;
  border-bottom: 1px solid var(--border-color);
}

.search-input {
  width: 100%;
  height: 28px;
  padding: 0 8px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
  transition: border-color var(--transition-fast);
}

.search-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-muted);
}

.search-input::placeholder {
  color: var(--text-muted);
}

.icon-browser-grid {
  flex: 1;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 4px;
  padding: 10px;
}

.icon-browser-empty {
  grid-column: 1 / -1;
  text-align: center;
  padding: 24px;
  color: var(--text-muted);
  font-size: 12px;
}

.icon-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 8px 4px;
  background: var(--bg-tertiary);
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.icon-card:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
  box-shadow: 0 0 8px var(--accent-glow);
}

.icon-card-name {
  font-size: 8px;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 80px;
}
</style>
