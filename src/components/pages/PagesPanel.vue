<script setup lang="ts">
import { ref, onMounted } from "vue";
import { usePagesStore } from "@/stores/pages";
import { useUiStore } from "@/stores/ui";
import AppIcon from "@/components/common/AppIcon.vue";

const pagesStore = usePagesStore();
const ui = useUiStore();

const newName = ref("");
const renamingId = ref<string | null>(null);
const contextMenu = ref<{ x: number; y: number; pageId: string } | null>(null);

onMounted(() => {
  pagesStore.refreshPages();
});

async function handleAdd() {
  const name = newName.value.trim() || `Page ${pagesStore.pages.length + 1}`;
  await pagesStore.addPage(name);
  newName.value = "";
}

async function handleClick(pageId: string) {
  if (renamingId.value) return;
  await pagesStore.switchPage(pageId);
}

function startRename(pageId: string, currentName: string) {
  renamingId.value = pageId;
  newName.value = currentName;
  contextMenu.value = null;
}

async function confirmRename() {
  if (!renamingId.value || !newName.value.trim()) return;
  await pagesStore.renamePage(renamingId.value, newName.value.trim());
  renamingId.value = null;
  newName.value = "";
}

function cancelRename() {
  renamingId.value = null;
  newName.value = "";
}

function onContextMenu(e: MouseEvent, pageId: string) {
  e.preventDefault();
  contextMenu.value = { x: e.clientX, y: e.clientY, pageId };
}

function closeContextMenu() {
  contextMenu.value = null;
}

async function handleDuplicate(pageId: string) {
  const name = `Copy of ${pagesStore.pages.find(p => p.id === pageId)?.name ?? "Page"}`;
  await pagesStore.duplicatePage(pageId, name);
  contextMenu.value = null;
}

async function handleDelete(pageId: string) {
  await pagesStore.deletePage(pageId);
  contextMenu.value = null;
}
</script>

<template>
  <div class="pages-panel" @click="closeContextMenu">
    <div class="pages-header">
      <span class="section-label">Pages</span>
      <button class="add-btn" @click="handleAdd" title="Add page">
        <AppIcon name="plus" :size="14" />
      </button>
    </div>

    <div class="pages-list">
      <div
        v-for="(page, index) in pagesStore.pages"
        :key="page.id"
        :class="['page-item', { active: page.active }]"
        @click="handleClick(page.id)"
        @contextmenu="onContextMenu($event, page.id)"
      >
        <div class="page-info">
          <span v-if="renamingId === page.id" class="rename-input-wrap">
            <input
              v-model="newName"
              class="rename-input"
              autofocus
              @keydown.enter="confirmRename"
              @keydown.escape="cancelRename"
              @blur="confirmRename"
            />
          </span>
          <span v-else class="page-name">{{ page.name }}</span>
          <span class="page-meta">{{ page.width }}x{{ page.height }} · {{ page.element_count }} el</span>
        </div>
        <div v-if="page.active" class="active-indicator" />
      </div>
    </div>

    <!-- Context menu -->
    <Teleport to="body">
      <div
        v-if="contextMenu"
        class="context-menu"
        :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
        @click.stop
      >
        <button @click="startRename(contextMenu!.pageId, pagesStore.pages.find(p => p.id === contextMenu!.pageId)?.name ?? '')">
          Rename
        </button>
        <button @click="handleDuplicate(contextMenu!.pageId)">
          Duplicate
        </button>
        <button
          v-if="pagesStore.hasMultiplePages"
          class="danger"
          @click="handleDelete(contextMenu!.pageId)"
        >
          Delete
        </button>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.pages-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.pages-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
}

.section-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-secondary);
}

.add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: 1px solid var(--border-color);
  background: var(--bg-elevated);
  color: var(--text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.add-btn:hover {
  color: var(--text-secondary);
  background: var(--bg-hover);
}

.pages-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px;
}

.page-item {
  display: flex;
  align-items: center;
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
  margin-bottom: 2px;
}

.page-item:hover {
  background: var(--bg-hover);
}

.page-item.active {
  background: var(--accent-muted);
}

.page-info {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
}

.page-name {
  font-size: 12px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.page-item.active .page-name {
  color: var(--accent);
  font-weight: 500;
}

.page-meta {
  font-size: 10px;
  color: var(--text-muted);
  margin-top: 2px;
}

.active-indicator {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent);
  flex-shrink: 0;
  margin-left: 8px;
}

.rename-input-wrap {
  width: 100%;
}

.rename-input {
  width: 100%;
  background: var(--input-bg);
  border: 1px solid var(--accent);
  border-radius: 3px;
  color: var(--text-primary);
  font-size: 12px;
  padding: 2px 6px;
  outline: none;
}

.context-menu {
  position: fixed;
  z-index: 1000;
  background: var(--bg-elevated);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  padding: 4px;
  min-width: 140px;
}

.context-menu button {
  display: block;
  width: 100%;
  text-align: left;
  padding: 6px 12px;
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  border-radius: 4px;
  transition: all var(--transition-fast);
}

.context-menu button:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.context-menu button.danger:hover {
  color: var(--danger);
  background: var(--danger-muted);
}
</style>
