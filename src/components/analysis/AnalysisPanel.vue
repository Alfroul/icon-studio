<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useUiStore } from "@/stores/ui";
import type { ColorAnalysis, ConsistencyReport, FindResult } from "@/types";

const ui = useUiStore();

type TabId = "colors" | "consistency" | "find";
const activeTab = ref<TabId>("colors");

// --- Color Analysis State ---
const colorAnalysis = ref<ColorAnalysis | null>(null);
const colorLoading = ref(false);
const colorError = ref("");

async function analyzeColors() {
  colorLoading.value = true;
  colorError.value = "";
  try {
    colorAnalysis.value = await invoke<ColorAnalysis>("analyze_colors");
  } catch (e) {
    colorError.value = String(e);
  } finally {
    colorLoading.value = false;
  }
}

// --- Consistency State ---
const consistency = ref<ConsistencyReport | null>(null);
const consistencyLoading = ref(false);
const consistencyError = ref("");

async function checkConsistency() {
  consistencyLoading.value = true;
  consistencyError.value = "";
  try {
    consistency.value = await invoke<ConsistencyReport>("check_consistency");
  } catch (e) {
    consistencyError.value = String(e);
  } finally {
    consistencyLoading.value = false;
  }
}

const consistencyChecks = computed(() => {
  if (!consistency.value) return [];
  const c = consistency.value;
  return [
    { label: "Border Radius", pass: c.border_radius_consistent },
    { label: "Stroke Width", pass: c.stroke_width_consistent },
    { label: "Font Size", pass: c.font_size_consistent },
    { label: "Opacity", pass: c.opacity_consistent },
  ];
});

// --- Find Elements State ---
const filterType = ref<string>("");
const filterFill = ref("");
const filterMinWidth = ref<number | null>(null);
const filterMaxWidth = ref<number | null>(null);
const findResult = ref<FindResult | null>(null);
const findLoading = ref(false);
const findError = ref("");

async function findElements() {
  findLoading.value = true;
  findError.value = "";
  try {
    findResult.value = await invoke<FindResult>("find_elements", {
      elementType: filterType.value || null,
      fill: filterFill.value || null,
      minWidth: filterMinWidth.value ?? null,
      maxWidth: filterMaxWidth.value ?? null,
    });
  } catch (e) {
    findError.value = String(e);
  } finally {
    findLoading.value = false;
  }
}

function selectElement(id: string) {
  ui.selectElement(id);
  ui.setPanel("properties");
}
</script>

<template>
  <div class="analysis-panel">
    <div class="tab-bar">
      <button
        v-for="tab in (['colors', 'consistency', 'find'] as TabId[])"
        :key="tab"
        :class="['tab-btn', { active: activeTab === tab }]"
        @click="activeTab = tab"
      >
        {{ tab === 'colors' ? 'Colors' : tab === 'consistency' ? 'Consistency' : 'Find' }}
      </button>
    </div>

    <!-- Color Analysis Tab -->
    <div v-if="activeTab === 'colors'" class="tab-content">
      <button class="action-btn" :disabled="colorLoading" @click="analyzeColors">
        {{ colorLoading ? 'Analyzing...' : 'Analyze Colors' }}
      </button>

      <div v-if="colorError" class="error-msg">{{ colorError }}</div>

      <div v-if="colorAnalysis" class="color-results">
        <!-- Primary Color -->
        <div v-if="colorAnalysis.primary" class="section-block">
          <div class="section-label">Primary</div>
          <div class="primary-color-row">
            <span
              class="color-swatch color-swatch--lg"
              :style="{ background: colorAnalysis.primary.hex }"
            />
            <div class="color-detail">
              <span class="color-hex">{{ colorAnalysis.primary.hex }}</span>
              <span class="color-count">{{ colorAnalysis.primary.usage_count }} uses</span>
            </div>
          </div>
        </div>

        <!-- All Colors -->
        <div v-if="colorAnalysis.all_colors.length" class="section-block">
          <div class="section-label">All Colors ({{ colorAnalysis.all_colors.length }})</div>
          <div class="color-list">
            <div
              v-for="c in colorAnalysis.all_colors"
              :key="c.hex"
              class="color-row"
              :title="c.element_ids.join(', ')"
            >
              <span class="color-swatch" :style="{ background: c.hex }" />
              <span class="color-hex">{{ c.hex }}</span>
              <span class="color-count">{{ c.usage_count }}</span>
            </div>
          </div>
        </div>

        <!-- Secondary Colors -->
        <div v-if="colorAnalysis.secondary.length" class="section-block">
          <div class="section-label">Secondary</div>
          <div class="color-list">
            <div
              v-for="c in colorAnalysis.secondary"
              :key="'sec-' + c.hex"
              class="color-row"
              :title="c.element_ids.join(', ')"
            >
              <span class="color-swatch" :style="{ background: c.hex }" />
              <span class="color-hex">{{ c.hex }}</span>
              <span class="color-count">{{ c.usage_count }}</span>
            </div>
          </div>
        </div>

        <!-- Accent Colors -->
        <div v-if="colorAnalysis.accent.length" class="section-block">
          <div class="section-label">Accent</div>
          <div class="color-list">
            <div
              v-for="c in colorAnalysis.accent"
              :key="'acc-' + c.hex"
              class="color-row"
              :title="c.element_ids.join(', ')"
            >
              <span class="color-swatch" :style="{ background: c.hex }" />
              <span class="color-hex">{{ c.hex }}</span>
              <span class="color-count">{{ c.usage_count }}</span>
            </div>
          </div>
        </div>
      </div>

      <div v-if="!colorAnalysis && !colorError && !colorLoading" class="empty-hint">
        Click "Analyze Colors" to inspect the color palette used in this project.
      </div>
    </div>

    <!-- Consistency Tab -->
    <div v-if="activeTab === 'consistency'" class="tab-content">
      <button class="action-btn" :disabled="consistencyLoading" @click="checkConsistency">
        {{ consistencyLoading ? 'Checking...' : 'Check Consistency' }}
      </button>

      <div v-if="consistencyError" class="error-msg">{{ consistencyError }}</div>

      <div v-if="consistency" class="consistency-results">
        <div class="check-grid">
          <div
            v-for="check in consistencyChecks"
            :key="check.label"
            :class="['check-row', check.pass ? 'check-pass' : 'check-fail']"
          >
            <span class="check-indicator">{{ check.pass ? '\u2713' : '\u2717' }}</span>
            <span class="check-label">{{ check.label }}</span>
          </div>
        </div>

        <div v-if="consistency.issues.length" class="section-block">
          <div class="section-label">Issues ({{ consistency.issues.length }})</div>
          <div class="issue-list">
            <div
              v-for="(issue, i) in consistency.issues"
              :key="i"
              class="issue-row"
            >
              <span class="issue-id">{{ issue.element_id }}</span>
              <span class="issue-prop">{{ issue.property }}</span>
              <span class="issue-values">
                <span class="issue-expected">{{ issue.expected }}</span>
                <span class="issue-arrow">&rarr;</span>
                <span class="issue-actual">{{ issue.actual }}</span>
              </span>
            </div>
          </div>
        </div>

        <div v-if="!consistency.issues.length" class="all-pass-msg">
          All consistency checks passed.
        </div>
      </div>

      <div v-if="!consistency && !consistencyError && !consistencyLoading" class="empty-hint">
        Click "Check Consistency" to verify design uniformity across elements.
      </div>
    </div>

    <!-- Find Elements Tab -->
    <div v-if="activeTab === 'find'" class="tab-content">
      <div class="filter-group">
        <label class="filter-label">Element Type</label>
        <select v-model="filterType" class="filter-select">
          <option value="">Any</option>
          <option value="shape">Shape</option>
          <option value="text">Text</option>
          <option value="icon">Icon</option>
          <option value="image">Image</option>
          <option value="path">Path</option>
          <option value="group">Group</option>
        </select>
      </div>

      <div class="filter-group">
        <label class="filter-label">Fill Color</label>
        <div class="filter-color-row">
          <input
            v-model="filterFill"
            type="text"
            class="filter-input"
            placeholder="#000000"
          />
          <span
            v-if="filterFill"
            class="color-swatch color-swatch--inline"
            :style="{ background: filterFill }"
          />
        </div>
      </div>

      <div class="filter-row">
        <div class="filter-group filter-group--half">
          <label class="filter-label">Min Width</label>
          <input
            v-model.number="filterMinWidth"
            type="number"
            class="filter-input"
            placeholder="0"
            min="0"
          />
        </div>
        <div class="filter-group filter-group--half">
          <label class="filter-label">Max Width</label>
          <input
            v-model.number="filterMaxWidth"
            type="number"
            class="filter-input"
            placeholder="512"
            min="0"
          />
        </div>
      </div>

      <button class="action-btn" :disabled="findLoading" @click="findElements">
        {{ findLoading ? 'Searching...' : 'Search' }}
      </button>

      <div v-if="findError" class="error-msg">{{ findError }}</div>

      <div v-if="findResult" class="find-results">
        <div class="section-label">{{ findResult.count }} element(s) found</div>
        <div v-if="findResult.matching_ids.length" class="find-list">
          <button
            v-for="id in findResult.matching_ids"
            :key="id"
            class="find-item"
            @click="selectElement(id)"
          >
            {{ id }}
          </button>
        </div>
      </div>

      <div v-if="!findResult && !findError && !findLoading" class="empty-hint">
        Set filter criteria and click "Search" to find matching elements.
      </div>
    </div>
  </div>
</template>

<style scoped>
.analysis-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

/* Tab Bar */
.tab-bar {
  display: flex;
  border-bottom: 1px solid var(--border-color);
  padding: 0 8px;
}

.tab-btn {
  flex: 1;
  padding: 8px 0;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.tab-btn:hover {
  color: var(--text-secondary);
}

.tab-btn.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

/* Tab Content */
.tab-content {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* Action Button */
.action-btn {
  width: 100%;
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--bg-primary);
  background: var(--accent);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.action-btn:hover:not(:disabled) {
  background: var(--accent-hover);
}

.action-btn:active:not(:disabled) {
  background: var(--accent-pressed);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Section Blocks */
.section-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-muted);
}

/* Color Swatches */
.color-swatch {
  display: inline-block;
  width: 14px;
  height: 14px;
  border-radius: 3px;
  border: 1px solid var(--border-color);
  flex-shrink: 0;
}

.color-swatch--lg {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
}

.color-swatch--inline {
  width: 18px;
  height: 18px;
  border-radius: 3px;
  margin-left: 4px;
}

/* Primary Color */
.primary-color-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-color);
}

.color-detail {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

/* Color List */
.color-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.color-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 6px;
  border-radius: 4px;
  transition: background var(--transition-fast);
  cursor: default;
}

.color-row:hover {
  background: var(--bg-hover);
}

.color-hex {
  font-size: 11px;
  font-family: "Cascadia Code", "Fira Code", monospace;
  color: var(--text-secondary);
}

.color-count {
  font-size: 10px;
  color: var(--text-muted);
  margin-left: auto;
}

/* Consistency Checks */
.check-grid {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.check-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: var(--radius-sm);
  font-size: 12px;
}

.check-pass {
  background: var(--success-muted);
}

.check-fail {
  background: var(--danger-muted);
}

.check-indicator {
  font-size: 13px;
  font-weight: 700;
  width: 16px;
  text-align: center;
}

.check-pass .check-indicator {
  color: var(--success);
}

.check-fail .check-indicator {
  color: var(--danger);
}

.check-label {
  color: var(--text-secondary);
}

/* Issue List */
.issue-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.issue-row {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 6px 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  border-left: 2px solid var(--danger);
}

.issue-id {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
}

.issue-prop {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
}

.issue-values {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-family: "Cascadia Code", "Fira Code", monospace;
}

.issue-expected {
  color: var(--success);
}

.issue-arrow {
  color: var(--text-muted);
}

.issue-actual {
  color: var(--danger);
}

.all-pass-msg {
  text-align: center;
  padding: 12px;
  color: var(--success);
  font-size: 12px;
}

/* Filter Controls */
.filter-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.filter-group--half {
  flex: 1;
}

.filter-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
}

.filter-row {
  display: flex;
  gap: 8px;
}

.filter-input,
.filter-select {
  width: 100%;
  padding: 6px 8px;
  font-size: 12px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  outline: none;
  transition: border-color var(--transition-fast);
}

.filter-input:focus,
.filter-select:focus {
  border-color: var(--input-focus);
}

.filter-select {
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%2371717A' stroke-width='2'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
  padding-right: 24px;
}

.filter-color-row {
  display: flex;
  align-items: center;
}

/* Find Results */
.find-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.find-item {
  display: block;
  width: 100%;
  padding: 6px 8px;
  font-size: 11px;
  font-family: "Cascadia Code", "Fira Code", monospace;
  color: var(--text-secondary);
  background: var(--bg-tertiary);
  border: none;
  border-radius: 4px;
  cursor: pointer;
  text-align: left;
  transition: all var(--transition-fast);
}

.find-item:hover {
  background: var(--accent-muted);
  color: var(--accent);
}

/* Error / Empty States */
.error-msg {
  padding: 8px;
  font-size: 11px;
  color: var(--danger);
  background: var(--danger-muted);
  border-radius: var(--radius-sm);
}

.empty-hint {
  text-align: center;
  padding: 16px 8px;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.6;
}
</style>
