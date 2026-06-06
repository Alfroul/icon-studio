<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useBrandStore, type BrandKitInfo } from "@/stores/brandStore";

const brand = useBrandStore();

const newKitName = ref("");
const newKitPrimary = ref("#FF6B35");
const newKitSecondary = ref("");
const newKitAccent = ref("");
const newKitNeutral = ref("");
const suggestInput = ref("");
const suggestResult = ref<BrandKitInfo | null>(null);
const activeKitId = ref<string | null>(null);
const guideVisible = ref(false);
const applyMode = ref<"closest" | "exact">("closest");

const colorRoles = ["primary", "secondary", "accent", "neutral", "surface", "error"];

const roleLabels: Record<string, string> = {
  primary: "Primary",
  secondary: "Secondary",
  accent: "Accent",
  neutral: "Neutral",
  surface: "Surface",
  error: "Error",
};

onMounted(() => {
  brand.fetchKits();
});

const activeKit = () => brand.kits.find((k) => k.id === activeKitId.value) ?? null;

async function createKit() {
  if (!newKitName.value.trim()) return;
  try {
    await brand.createKit(
      newKitName.value.trim(),
      newKitPrimary.value,
      newKitSecondary.value || undefined,
      newKitAccent.value || undefined,
      newKitNeutral.value || undefined,
    );
    newKitName.value = "";
    newKitSecondary.value = "";
    newKitAccent.value = "";
    newKitNeutral.value = "";
  } catch (e) {
    console.error("Create kit failed:", e);
  }
}

async function suggestBrand() {
  if (!suggestInput.value.trim()) return;
  try {
    suggestResult.value = await brand.suggest(suggestInput.value.trim());
  } catch (e) {
    console.error("Suggest failed:", e);
  }
}

async function applySuggested() {
  if (!suggestResult.value) return;
  try {
    const kit = await brand.createKit(
      suggestResult.value.name,
      suggestResult.value.colors.primary,
    );
    suggestResult.value = null;
    suggestInput.value = "";
    activeKitId.value = kit.id;
  } catch (e) {
    console.error("Apply suggestion failed:", e);
  }
}

async function generateVariant(type: string) {
  if (!activeKitId.value) return;
  try {
    const kit = await brand.generateVariant(activeKitId.value, type);
    activeKitId.value = kit.id;
  } catch (e) {
    console.error("Generate variant failed:", e);
  }
}

async function applyToCanvas() {
  if (!activeKitId.value) return;
  try {
    await brand.applyKit(activeKitId.value, applyMode.value);
  } catch (e) {
    console.error("Apply brand failed:", e);
  }
}

async function exportGuide() {
  if (!activeKitId.value) return;
  try {
    await brand.exportGuide(activeKitId.value);
    guideVisible.value = true;
  } catch (e) {
    console.error("Export guide failed:", e);
  }
}

async function deleteKit(id: string) {
  try {
    await brand.deleteKit(id);
    if (activeKitId.value === id) activeKitId.value = null;
  } catch (e) {
    console.error("Delete kit failed:", e);
  }
}

async function updateColor(role: string, event: Event) {
  if (!activeKitId.value) return;
  const input = event.target as HTMLInputElement;
  try {
    await brand.updateColor(activeKitId.value, role, input.value);
  } catch (e) {
    console.error("Update color failed:", e);
  }
}

function copyGuide() {
  navigator.clipboard.writeText(brand.guideText);
}
</script>

<template>
  <div class="brand-kit-panel">
    <!-- Create new kit -->
    <div class="section">
      <div class="section-header">
        <span class="section-title">Create Brand Kit</span>
      </div>
      <div class="section-body">
        <div class="form-row">
          <input v-model="newKitName" class="input" placeholder="Brand name..." />
        </div>
        <div class="color-row">
          <label class="color-label">Primary</label>
          <input v-model="newKitPrimary" type="color" class="color-picker" />
          <span class="color-hex">{{ newKitPrimary }}</span>
        </div>
        <div class="color-row">
          <label class="color-label">Secondary</label>
          <input v-model="newKitSecondary" type="color" class="color-picker" />
          <span class="color-hex">{{ newKitSecondary || "auto" }}</span>
        </div>
        <div class="color-row">
          <label class="color-label">Accent</label>
          <input v-model="newKitAccent" type="color" class="color-picker" />
          <span class="color-hex">{{ newKitAccent || "auto" }}</span>
        </div>
        <button class="btn-primary" @click="createKit">Create</button>
      </div>
    </div>

    <!-- Suggest from description -->
    <div class="section">
      <div class="section-header">
        <span class="section-title">Suggest from Description</span>
      </div>
      <div class="section-body">
        <div class="form-row">
          <input
            v-model="suggestInput"
            class="input"
            placeholder="e.g. tech, 自然, luxury..."
            @keyup.enter="suggestBrand"
          />
        </div>
        <button class="btn-secondary" @click="suggestBrand">Suggest</button>
        <div v-if="suggestResult" class="suggest-result">
          <div class="suggest-colors">
            <div
              v-for="role in colorRoles"
              :key="role"
              class="color-swatch"
              :style="{ background: suggestResult.colors[role] || '#ccc' }"
              :title="`${roleLabels[role]}: ${suggestResult.colors[role] || ''}`"
            ></div>
          </div>
          <button class="btn-primary btn-sm" @click="applySuggested">Use This</button>
        </div>
      </div>
    </div>

    <!-- Saved kits list -->
    <div class="section">
      <div class="section-header">
        <span class="section-title">Saved Kits ({{ brand.kits.length }})</span>
      </div>
      <div class="section-body">
        <div v-if="brand.kits.length === 0" class="empty-msg">
          No brand kits yet
        </div>
        <div v-for="kit in brand.kits" :key="kit.id" class="kit-item" :class="{ active: kit.id === activeKitId }">
          <div class="kit-header" @click="activeKitId = kit.id">
            <div class="kit-swatches">
              <div
                v-for="role in colorRoles.slice(0, 4)"
                :key="role"
                class="mini-swatch"
                :style="{ background: kit.colors[role] || '#ccc' }"
              ></div>
            </div>
            <span class="kit-name">{{ kit.name }}</span>
            <button class="btn-icon btn-danger" @click.stop="deleteKit(kit.id)">×</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Active kit editor -->
    <div v-if="activeKit()" class="section">
      <div class="section-header">
        <span class="section-title">Edit: {{ activeKit()!.name }}</span>
      </div>
      <div class="section-body">
        <div v-for="role in colorRoles" :key="role" class="color-edit-row">
          <label class="color-label">{{ roleLabels[role] }}</label>
          <input
            type="color"
            class="color-picker"
            :value="activeKit()!.colors[role] || '#000000'"
            @change="updateColor(role, $event)"
          />
          <span class="color-hex">{{ activeKit()!.colors[role] || "—" }}</span>
        </div>

        <div class="action-group">
          <span class="action-label">Variants</span>
          <div class="btn-row">
            <button class="btn-secondary btn-sm" @click="generateVariant('dark')">Dark</button>
            <button class="btn-secondary btn-sm" @click="generateVariant('light')">Light</button>
            <button class="btn-secondary btn-sm" @click="generateVariant('high-contrast')">Hi-Con</button>
          </div>
        </div>

        <div class="action-group">
          <span class="action-label">Apply to Canvas</span>
          <div class="btn-row">
            <select v-model="applyMode" class="select-sm">
              <option value="closest">Closest</option>
              <option value="exact">Exact</option>
            </select>
            <button class="btn-primary btn-sm" @click="applyToCanvas">Apply</button>
          </div>
        </div>

        <div class="action-group">
          <button class="btn-secondary" @click="exportGuide">Export Brand Guide</button>
        </div>
      </div>
    </div>

    <!-- Guide preview -->
    <div v-if="guideVisible" class="section guide-section">
      <div class="section-header">
        <span class="section-title">Brand Guide</span>
        <div class="section-actions">
          <button class="btn-secondary btn-sm" @click="copyGuide">Copy</button>
          <button class="btn-icon" @click="guideVisible = false">×</button>
        </div>
      </div>
      <div class="section-body">
        <pre class="guide-text">{{ brand.guideText }}</pre>
      </div>
    </div>
  </div>
</template>

<style scoped>
.brand-kit-panel {
  display: flex;
  flex-direction: column;
  gap: 0;
  height: 100%;
  overflow-y: auto;
}

.section {
  border-top: 1px solid var(--bg-tertiary);
  padding: 12px 0;
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
  align-items: center;
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

.color-row,
.color-edit-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.color-label {
  font-size: 11px;
  color: var(--text-secondary);
  width: 70px;
  flex-shrink: 0;
}

.color-picker {
  width: 28px;
  height: 28px;
  padding: 0;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: none;
  cursor: pointer;
}

.color-picker::-webkit-color-swatch-wrapper {
  padding: 2px;
}

.color-picker::-webkit-color-swatch {
  border: none;
  border-radius: 2px;
}

.color-hex {
  font-size: 10px;
  font-family: monospace;
  color: var(--text-muted);
}

.btn-primary {
  width: 100%;
  height: 30px;
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

.btn-secondary {
  width: 100%;
  height: 28px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-secondary:hover {
  border-color: var(--accent);
  color: var(--text-primary);
}

.btn-sm {
  width: auto;
  padding: 0 10px;
  height: 24px;
  font-size: 10px;
}

.btn-icon {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 14px;
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
}

.btn-icon:hover {
  color: var(--text-primary);
}

.btn-danger:hover {
  color: var(--danger);
}

.suggest-result {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
}

.suggest-colors {
  display: flex;
  gap: 3px;
  flex: 1;
}

.color-swatch {
  width: 24px;
  height: 24px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-color);
}

.kit-item {
  padding: 6px 8px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.kit-item:hover {
  background: var(--bg-tertiary);
}

.kit-item.active {
  background: var(--bg-tertiary);
  border: 1px solid var(--accent);
}

.kit-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.kit-swatches {
  display: flex;
  gap: 2px;
}

.mini-swatch {
  width: 14px;
  height: 14px;
  border-radius: 2px;
  border: 1px solid var(--border-color);
}

.kit-name {
  flex: 1;
  font-size: 11px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.action-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 4px;
}

.action-label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.btn-row {
  display: flex;
  gap: 4px;
}

.select-sm {
  height: 24px;
  padding: 0 6px;
  font-size: 10px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
}

.guide-section .section-body {
  max-height: 300px;
  overflow-y: auto;
}

.guide-text {
  font-size: 10px;
  line-height: 1.5;
  color: var(--text-secondary);
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
}

.empty-msg {
  text-align: center;
  font-size: 11px;
  color: var(--text-muted);
  padding: 8px;
}
</style>
