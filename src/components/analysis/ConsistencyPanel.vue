<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import type { ConsistencyReport, ConsistencyIssue, IssueSeverity } from "@/types";

const ui = useUiStore();
const project = useProjectStore();

const report = ref<ConsistencyReport | null>(null);
const loading = ref(false);
const error = ref("");
const fixing = ref(false);

async function runAnalysis() {
  loading.value = true;
  error.value = "";
  try {
    report.value = await invoke<ConsistencyReport>("check_consistency");
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

const checks = computed(() => {
  if (!report.value) return [];
  const r = report.value;
  return [
    { label: "Border Radius", pass: r.border_radius_consistent },
    { label: "Stroke Width", pass: r.stroke_width_consistent },
    { label: "Font Size", pass: r.font_size_consistent },
    { label: "Opacity", pass: r.opacity_consistent },
    { label: "Stroke Weight", pass: r.stroke_weight_consistent ?? true },
    { label: "Fill Style", pass: r.fill_style_consistent ?? true },
    { label: "Proportions", pass: r.proportions_consistent ?? true },
  ];
});

const issueCount = computed(() => report.value?.issues.length ?? 0);

function severityClass(severity?: IssueSeverity) {
  switch (severity) {
    case "error": return "severity-error";
    case "warning": return "severity-warning";
    case "info": return "severity-info";
    default: return "severity-info";
  }
}

function severityIcon(severity?: IssueSeverity) {
  switch (severity) {
    case "error": return "✗";
    case "warning": return "!";
    case "info": return "i";
    default: return "i";
  }
}

async function fixIssue(issue: ConsistencyIssue) {
  fixing.value = true;
  try {
    await invoke<string>("fix_consistency", { elementIds: [issue.element_id] });
    await project.refreshElements();
    await runAnalysis();
  } catch (e) {
    error.value = String(e);
  } finally {
    fixing.value = false;
  }
}

async function fixAll() {
  if (!report.value || report.value.issues.length === 0) return;
  fixing.value = true;
  try {
    const ids = [...new Set(report.value.issues.map(i => i.element_id))];
    await invoke<string>("fix_consistency", { elementIds: ids });
    await project.refreshElements();
    await runAnalysis();
  } catch (e) {
    error.value = String(e);
  } finally {
    fixing.value = false;
  }
}
</script>

<template>
  <div class="consistency-panel">
    <button class="action-btn" :disabled="loading" @click="runAnalysis">
      {{ loading ? 'Analyzing...' : 'Run Consistency Check' }}
    </button>

    <div v-if="error" class="error-msg">{{ error }}</div>

    <div v-if="report" class="report-content">
      <!-- Check badges -->
      <div class="check-grid">
        <div
          v-for="check in checks"
          :key="check.label"
          :class="['check-row', check.pass ? 'check-pass' : 'check-fail']"
        >
          <span class="check-indicator">{{ check.pass ? '✓' : '✗' }}</span>
          <span class="check-label">{{ check.label }}</span>
        </div>
      </div>

      <!-- Visual center drift -->
      <div v-if="report.visual_center_drift != null" class="drift-row">
        <span class="drift-label">Visual Center Drift</span>
        <span :class="['drift-value', report.visual_center_drift > 0.2 ? 'drift-warn' : 'drift-ok']">
          {{ (report.visual_center_drift * 100).toFixed(1) }}%
        </span>
      </div>

      <!-- Issues list -->
      <div v-if="issueCount > 0" class="section-block">
        <div class="section-header">
          <span class="section-label">Issues ({{ issueCount }})</span>
          <button class="fix-all-btn" :disabled="fixing" @click="fixAll">
            {{ fixing ? 'Fixing...' : 'Fix All' }}
          </button>
        </div>
        <div class="issue-list">
          <div
            v-for="(issue, i) in report.issues"
            :key="i"
            :class="['issue-row', severityClass(issue.severity)]"
          >
            <div class="issue-top">
              <span :class="['severity-badge', severityClass(issue.severity)]">
                {{ severityIcon(issue.severity) }}
              </span>
              <span class="issue-prop">{{ issue.property.replace('_', ' ') }}</span>
              <span class="issue-id">{{ issue.element_id }}</span>
            </div>
            <div class="issue-values">
              <span class="issue-expected">{{ issue.expected }}</span>
              <span class="issue-arrow">&rarr;</span>
              <span class="issue-actual">{{ issue.actual }}</span>
            </div>
            <button class="fix-btn" :disabled="fixing" @click="fixIssue(issue)">
              Fix
            </button>
          </div>
        </div>
      </div>

      <div v-if="issueCount === 0" class="all-pass-msg">
        All consistency checks passed.
      </div>
    </div>

    <div v-if="!report && !error && !loading" class="empty-hint">
      Click "Run Consistency Check" to verify design uniformity across all elements.
    </div>
  </div>
</template>

<style scoped>
.consistency-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px;
  height: 100%;
}

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

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.error-msg {
  padding: 8px;
  font-size: 11px;
  color: var(--danger);
  background: var(--danger-muted);
  border-radius: var(--radius-sm);
}

.report-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 1;
  overflow-y: auto;
}

/* Check badges */
.check-grid {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.check-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
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
  font-size: 12px;
  font-weight: 700;
  width: 14px;
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

/* Visual center drift */
.drift-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-color);
}

.drift-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
}

.drift-value {
  font-size: 12px;
  font-family: "Cascadia Code", "Fira Code", monospace;
  font-weight: 600;
}

.drift-ok {
  color: var(--success);
}

.drift-warn {
  color: var(--warning, #F59E0B);
}

/* Section blocks */
.section-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-muted);
}

/* Fix All button */
.fix-all-btn {
  padding: 3px 8px;
  font-size: 10px;
  font-weight: 600;
  color: var(--accent);
  background: var(--accent-muted);
  border: 1px solid var(--accent-glow, var(--accent));
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.fix-all-btn:hover:not(:disabled) {
  background: var(--accent);
  color: var(--bg-primary);
}

.fix-all-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Issue list */
.issue-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.issue-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  border-left: 3px solid var(--text-muted);
  position: relative;
}

.issue-row.severity-info {
  border-left-color: #3B82F6;
}

.issue-row.severity-warning {
  border-left-color: #F59E0B;
}

.issue-row.severity-error {
  border-left-color: var(--danger);
}

.issue-top {
  display: flex;
  align-items: center;
  gap: 6px;
}

.severity-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  font-size: 10px;
  font-weight: 700;
  flex-shrink: 0;
}

.severity-badge.severity-info {
  background: rgba(59, 130, 246, 0.2);
  color: #3B82F6;
}

.severity-badge.severity-warning {
  background: rgba(245, 158, 11, 0.2);
  color: #F59E0B;
}

.severity-badge.severity-error {
  background: rgba(239, 68, 68, 0.2);
  color: var(--danger);
}

.issue-prop {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
}

.issue-id {
  font-size: 11px;
  font-family: "Cascadia Code", "Fira Code", monospace;
  color: var(--text-secondary);
  margin-left: auto;
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

.fix-btn {
  align-self: flex-end;
  padding: 2px 8px;
  font-size: 10px;
  font-weight: 600;
  color: var(--accent);
  background: none;
  border: 1px solid var(--accent-glow, var(--accent));
  border-radius: 4px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.fix-btn:hover:not(:disabled) {
  background: var(--accent);
  color: var(--bg-primary);
}

.fix-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.all-pass-msg {
  text-align: center;
  padding: 12px;
  color: var(--success);
  font-size: 12px;
}

.empty-hint {
  text-align: center;
  padding: 16px 8px;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.6;
}
</style>
