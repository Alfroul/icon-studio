<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import {
  useIconSetStore,
  type SetEntry,
  type ConsistencyIssue,
} from "@/stores/iconSetStore";

const store = useIconSetStore();

const newSetName = ref("");
const newEntryName = ref("");
const newEntryTags = ref("");
const searchQuery = ref("");
const filterTag = ref<string | null>(null);
const exportFormat = ref("png");
const exportDir = ref("");
const showExportDialog = ref(false);
const showAddDialog = ref(false);

onMounted(() => {
  store.loadSets();
});

const displayedEntries = computed(() => {
  if (!store.activeSet) return [];
  let entries = store.activeSet.entries;

  if (filterTag.value) {
    entries = entries.filter((e) =>
      e.tags.some((t) => t.toLowerCase() === filterTag.value!.toLowerCase())
    );
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase();
    entries = entries.filter(
      (e) =>
        e.name.toLowerCase().includes(q) ||
        e.tags.some((t) => t.toLowerCase().includes(q))
    );
  }

  return entries;
});

async function createSet() {
  if (!newSetName.value.trim()) return;
  await store.createSet(newSetName.value.trim());
  newSetName.value = "";
}

async function selectSet(id: string) {
  await store.selectSet(id);
  filterTag.value = null;
  searchQuery.value = "";
}

async function addToSet() {
  if (!store.activeSetId) return;
  const tags = newEntryTags.value
    .split(",")
    .map((t) => t.trim())
    .filter(Boolean);
  await store.addCurrentToSet(newEntryName.value || undefined, tags);
  newEntryName.value = "";
  newEntryTags.value = "";
  showAddDialog.value = false;
}

async function removeEntry(id: string) {
  await store.removeFromSet(id);
}

async function runConsistency() {
  await store.checkConsistency();
}

async function runExport() {
  if (!exportDir.value.trim()) return;
  await store.exportSet(exportFormat.value, undefined, exportDir.value.trim());
  showExportDialog.value = false;
}

function toggleTag(tag: string) {
  filterTag.value = filterTag.value === tag ? null : tag;
}

function issueSeverity(issue: ConsistencyIssue): string {
  const expected = parseFloat(issue.expected);
  const actual = parseFloat(issue.actual);
  if (isNaN(expected) || expected === 0) return "warn";
  const deviation = Math.abs(actual - expected) / expected;
  return deviation > 0.5 ? "high" : "warn";
}
</script>

<template>
  <div class="icon-set-panel">
    <!-- Create new set -->
    <div class="section">
      <div class="section-header">
        <span class="section-title">Icon Sets</span>
      </div>
      <div class="section-body">
        <div class="form-row">
          <input
            v-model="newSetName"
            class="input"
            placeholder="New set name..."
            @keyup.enter="createSet"
          />
          <button class="btn-primary btn-sm" @click="createSet">+</button>
        </div>
      </div>
    </div>

    <!-- Set list -->
    <div class="section">
      <div class="section-body">
        <div v-if="store.sets.length === 0" class="empty-msg">
          No icon sets yet
        </div>
        <div
          v-for="set in store.sets"
          :key="set.id"
          class="set-item"
          :class="{ active: set.id === store.activeSetId }"
          @click="selectSet(set.id)"
        >
          <span class="set-name">{{ set.name }}</span>
          <span class="set-count">{{ set.entry_count }}</span>
        </div>
      </div>
    </div>

    <!-- Active set content -->
    <template v-if="store.activeSet">
      <div class="section">
        <div class="section-header">
          <span class="section-title">{{ store.activeSet.name }}</span>
          <div class="section-actions">
            <button class="btn-secondary btn-sm" @click="showAddDialog = true">
              Add
            </button>
            <button
              class="btn-secondary btn-sm"
              @click="runConsistency"
            >
              Check
            </button>
            <button
              class="btn-secondary btn-sm"
              @click="showExportDialog = true"
            >
              Export
            </button>
          </div>
        </div>
      </div>

      <!-- Search -->
      <div class="section">
        <div class="section-body">
          <input
            v-model="searchQuery"
            class="input"
            placeholder="Search icons..."
          />
        </div>
      </div>

      <!-- Tag cloud -->
      <div v-if="store.allTags.length > 0" class="section">
        <div class="section-body">
          <div class="tag-cloud">
            <button
              v-for="tag in store.allTags"
              :key="tag"
              class="tag-chip"
              :class="{ active: filterTag === tag }"
              @click="toggleTag(tag)"
            >
              {{ tag }}
            </button>
          </div>
        </div>
      </div>

      <!-- Icon grid -->
      <div class="section icon-grid-section">
        <div class="section-body">
          <div v-if="displayedEntries.length === 0" class="empty-msg">
            No icons{{ filterTag ? " with this tag" : "" }}
          </div>
          <div class="icon-grid">
            <div
              v-for="entry in displayedEntries"
              :key="entry.id"
              class="icon-card"
            >
              <div class="icon-thumb">
                <img
                  v-if="entry.thumbnail"
                  :src="entry.thumbnail"
                  :alt="entry.name"
                />
                <span v-else class="icon-placeholder">?</span>
              </div>
              <span class="icon-name">{{ entry.name }}</span>
              <button
                class="btn-icon btn-danger icon-remove"
                @click="removeEntry(entry.id)"
              >
                x
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Consistency report -->
      <div
        v-if="store.consistencyReport"
        class="section"
      >
        <div class="section-header">
          <span class="section-title">Consistency Report</span>
          <span
            class="badge"
            :class="store.consistencyReport.consistent ? 'badge-ok' : 'badge-warn'"
          >
            {{ store.consistencyReport.consistent ? "OK" : "Issues" }}
          </span>
        </div>
        <div class="section-body">
          <p class="report-summary">{{ store.consistencyReport.summary }}</p>
          <div
            v-if="store.consistencyReport.issues.length > 0"
            class="issues-list"
          >
            <div
              v-for="(issue, idx) in store.consistencyReport.issues"
              :key="idx"
              class="issue-item"
              :class="issueSeverity(issue)"
            >
              <span class="issue-prop">{{ issue.property }}</span>
              <span class="issue-detail">
                {{ issue.project_path }}/{{ issue.element_id }}: expected
                {{ issue.expected }}, got {{ issue.actual }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- Add dialog -->
    <div v-if="showAddDialog" class="dialog-overlay" @click.self="showAddDialog = false">
      <div class="dialog">
        <h4>Add Current Icon to Set</h4>
        <input
          v-model="newEntryName"
          class="input"
          placeholder="Icon name (optional)"
        />
        <input
          v-model="newEntryTags"
          class="input"
          placeholder="Tags (comma-separated)"
        />
        <div class="dialog-actions">
          <button class="btn-secondary" @click="showAddDialog = false">
            Cancel
          </button>
          <button class="btn-primary" @click="addToSet">Add</button>
        </div>
      </div>
    </div>

    <!-- Export dialog -->
    <div v-if="showExportDialog" class="dialog-overlay" @click.self="showExportDialog = false">
      <div class="dialog">
        <h4>Export Set</h4>
        <div class="form-row">
          <select v-model="exportFormat" class="select-sm">
            <option value="png">PNG</option>
            <option value="svg">SVG</option>
            <option value="all">All</option>
          </select>
        </div>
        <input
          v-model="exportDir"
          class="input"
          placeholder="Output directory..."
        />
        <div class="dialog-actions">
          <button class="btn-secondary" @click="showExportDialog = false">
            Cancel
          </button>
          <button class="btn-primary" @click="runExport">Export</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.icon-set-panel {
  display: flex;
  flex-direction: column;
  gap: 0;
  height: 100%;
  overflow-y: auto;
}

.section {
  border-top: 1px solid var(--bg-tertiary);
  padding: 10px 0;
}

.section:first-child {
  border-top: none;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px 6px;
}

.section-title {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--accent);
}

.section-actions {
  display: flex;
  gap: 4px;
}

.section-body {
  padding: 0 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-row {
  display: flex;
  gap: 6px;
}

.input {
  flex: 1;
  height: 28px;
  padding: 0 8px;
  font-size: 11px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
}

.input:focus {
  border-color: var(--accent);
}

.input::placeholder {
  color: var(--text-muted);
}

.btn-primary {
  height: 28px;
  background: var(--accent);
  border: none;
  border-radius: var(--radius-md);
  color: var(--bg-primary);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity var(--transition-fast);
}

.btn-primary:hover {
  opacity: 0.85;
}

.btn-primary.btn-sm {
  width: 28px;
  padding: 0;
}

.btn-secondary {
  height: 26px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  padding: 0 8px;
  transition: all var(--transition-fast);
}

.btn-secondary:hover {
  border-color: var(--accent);
  color: var(--text-primary);
}

.btn-sm {
  height: 24px;
  font-size: 10px;
  padding: 0 8px;
}

.btn-icon {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 12px;
  cursor: pointer;
  padding: 0 2px;
  line-height: 1;
}

.btn-icon:hover {
  color: var(--text-primary);
}

.btn-danger:hover {
  color: var(--danger);
}

.empty-msg {
  text-align: center;
  font-size: 11px;
  color: var(--text-muted);
  padding: 8px;
}

.set-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.set-item:hover {
  background: var(--bg-tertiary);
}

.set-item.active {
  background: var(--bg-tertiary);
  border: 1px solid var(--accent);
}

.set-name {
  font-size: 11px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.set-count {
  font-size: 10px;
  color: var(--text-muted);
  min-width: 20px;
  text-align: center;
}

.tag-cloud {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tag-chip {
  height: 20px;
  padding: 0 6px;
  font-size: 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.tag-chip:hover {
  border-color: var(--accent);
}

.tag-chip.active {
  background: var(--accent);
  color: var(--bg-primary);
  border-color: var(--accent);
}

.icon-grid-section {
  flex: 1;
  min-height: 0;
}

.icon-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(64px, 1fr));
  gap: 6px;
}

.icon-card {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 4px;
  border-radius: var(--radius-md);
  background: var(--bg-tertiary);
  border: 1px solid transparent;
  transition: border-color var(--transition-fast);
}

.icon-card:hover {
  border-color: var(--border-color);
}

.icon-card:hover .icon-remove {
  opacity: 1;
}

.icon-thumb {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.icon-thumb img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.icon-placeholder {
  font-size: 16px;
  color: var(--text-muted);
}

.icon-name {
  font-size: 9px;
  color: var(--text-secondary);
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 60px;
}

.icon-remove {
  position: absolute;
  top: 2px;
  right: 2px;
  opacity: 0;
  transition: opacity var(--transition-fast);
  font-size: 10px;
}

.report-summary {
  font-size: 11px;
  color: var(--text-secondary);
  margin: 0;
}

.issues-list {
  display: flex;
  flex-direction: column;
  gap: 3px;
  max-height: 200px;
  overflow-y: auto;
}

.issue-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 6px;
  border-radius: var(--radius-sm);
  font-size: 10px;
}

.issue-item.warn {
  background: rgba(255, 193, 7, 0.1);
}

.issue-item.high {
  background: rgba(244, 67, 54, 0.1);
}

.issue-prop {
  font-weight: 600;
  color: var(--text-primary);
  min-width: 60px;
}

.issue-detail {
  color: var(--text-muted);
}

.badge {
  font-size: 9px;
  padding: 2px 6px;
  border-radius: 8px;
  font-weight: 600;
  text-transform: uppercase;
}

.badge-ok {
  background: rgba(76, 175, 80, 0.15);
  color: #4caf50;
}

.badge-warn {
  background: rgba(255, 152, 0, 0.15);
  color: #ff9800;
}

.select-sm {
  height: 28px;
  padding: 0 8px;
  font-size: 11px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
  width: 100%;
}

/* Dialog */
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.dialog {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 16px;
  width: 280px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.dialog h4 {
  margin: 0;
  font-size: 13px;
  color: var(--text-primary);
}

.dialog-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.dialog-actions .btn-secondary,
.dialog-actions .btn-primary {
  width: auto;
  padding: 0 16px;
}
</style>
