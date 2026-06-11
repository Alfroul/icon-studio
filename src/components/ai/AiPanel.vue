<script setup lang="ts">
import { ref } from "vue";
import {
  useAiStore,
  type IconStyleType,
} from "@/stores/aiStore";
import AppIcon from "@/components/common/AppIcon.vue";
import { invoke } from "@tauri-apps/api/core";

const ai = useAiStore();

const prompt = ref("");
const selectedStyle = ref<IconStyleType>("flat");
const count = ref(1);
const hoveredIndex = ref<number | null>(null);

const styles: { id: IconStyleType; label: string }[] = [
  { id: "flat", label: "Flat" },
  { id: "outline", label: "Outline" },
  { id: "duotone", label: "Duotone" },
  { id: "gradient", label: "Gradient" },
  { id: "threeD", label: "3D" },
  { id: "minimal", label: "Minimal" },
  { id: "cartoon", label: "Cartoon" },
  { id: "pixelArt", label: "Pixel" },
  { id: "lineArt", label: "Line Art" },
  { id: "neon", label: "Neon" },
];

const providers: { id: string; label: string }[] = [
  { id: "openAi", label: "OpenAI" },
  { id: "recraft", label: "Recraft" },
  { id: "ollama", label: "Ollama" },
  { id: "custom", label: "Custom" },
];

async function handleGenerate() {
  if (!prompt.value.trim() || ai.generating) return;
  if (ai.provider !== "ollama" && !ai.apiKey) return;

  if (count.value <= 1) {
    await ai.generateIcon(prompt.value.trim(), selectedStyle.value);
  } else {
    const prompts = Array.from(
      { length: count.value },
      (_, i) => `${prompt.value.trim()} variation ${i + 1}`
    );
    await ai.generateSet(prompts, selectedStyle.value);
  }
}

function getSvgPreview(icon: { svg_content: string | null; image_data: number[] | null }): string {
  if (icon.svg_content) {
    return `data:image/svg+xml;base64,${btoa(unescape(encodeURIComponent(icon.svg_content)))}`;
  }
  if (icon.image_data) {
    const bytes = new Uint8Array(icon.image_data);
    let binary = "";
    bytes.forEach((b) => (binary += String.fromCharCode(b)));
    return `data:image/png;base64,${btoa(binary)}`;
  }
  return "";
}

async function addToCanvas(icon: { svg_content: string | null; image_data: number[] | null }) {
  try {
    if (icon.svg_content) {
      await invoke("add_path_from_svg", { svgContent: icon.svg_content });
    } else if (icon.image_data) {
      const bytes = new Uint8Array(icon.image_data);
      let binary = "";
      bytes.forEach((b) => (binary += String.fromCharCode(b)));
      const b64 = btoa(binary);
      await invoke("add_image_from_data", { imageData: `data:image/png;base64,${b64}`, width: 256, height: 256 });
    }
  } catch (e) {
    const { useUiStore } = await import("@/stores/ui");
    useUiStore().showToast(`Failed to add to canvas: ${e}`, "error");
  }
}

async function regenerate(index: number) {
  if (!ai.results[index] || ai.generating) return;
  const p = ai.results[index].prompt || prompt.value;
  await ai.generateIcon(p, selectedStyle.value);
}
</script>

<template>
  <div class="ai-panel">
    <!-- Config Section (collapsible) -->
    <div class="section">
      <button class="section-header" @click="ai.configExpanded = !ai.configExpanded">
        <AppIcon :name="ai.configExpanded ? 'chevronDown' : 'chevronRight'" :size="12" />
        <span>Configuration</span>
      </button>
      <div v-if="ai.configExpanded" class="section-body">
        <label class="field-label">Provider</label>
        <select
          :value="ai.provider"
          class="field-select"
          @change="ai.setProvider(($event.target as HTMLSelectElement).value as any)"
        >
          <option v-for="p in providers" :key="p.id" :value="p.id">{{ p.label }}</option>
        </select>

        <label class="field-label">
          API Key
          <span v-if="ai.provider === 'ollama'" class="hint">(not needed)</span>
        </label>
        <input
          v-if="ai.provider !== 'ollama'"
          type="password"
          class="field-input"
          placeholder="Enter API key..."
          :value="ai.apiKey"
          @input="ai.setApiKey(($event.target as HTMLInputElement).value)"
        />

        <label class="field-label">Model</label>
        <input
          class="field-input"
          :list="'model-list-' + ai.provider"
          :value="ai.model"
          placeholder="Model name"
          @input="ai.setModel(($event.target as HTMLInputElement).value)"
        />
        <datalist :id="'model-list-' + ai.provider">
          <option v-for="m in ai.availableModels" :key="m" :value="m" />
        </datalist>

        <template v-if="ai.provider === 'custom' || ai.provider === 'ollama'">
          <label class="field-label">Endpoint</label>
          <input
            class="field-input"
            :placeholder="ai.provider === 'ollama' ? 'http://localhost:11434' : 'https://...'"
            :value="ai.endpoint"
            @input="ai.setEndpoint(($event.target as HTMLInputElement).value)"
          />
        </template>
      </div>
    </div>

    <!-- Generation Section -->
    <div class="section">
      <div class="section-header static">
        <span>Generate</span>
      </div>
      <div class="section-body">
        <textarea
          v-model="prompt"
          class="field-textarea"
          rows="3"
          placeholder="Describe the icon you want to generate..."
          @keydown.ctrl.enter="handleGenerate"
        />

        <label class="field-label">Style</label>
        <div class="style-grid">
          <button
            v-for="s in styles"
            :key="s.id"
            :class="['style-btn', { active: selectedStyle === s.id }]"
            @click="selectedStyle = s.id"
            :title="s.label"
          >
            {{ s.label }}
          </button>
        </div>

        <label class="field-label">Count</label>
        <div class="count-row">
          <button
            v-for="n in [1, 2, 3, 4, 5, 6]"
            :key="n"
            :class="['count-btn', { active: count === n }]"
            @click="count = n"
          >
            {{ n }}
          </button>
        </div>

        <button
          class="generate-btn"
          :disabled="!prompt.trim() || ai.generating || (ai.provider !== 'ollama' && !ai.apiKey)"
          @click="handleGenerate"
        >
          <AppIcon v-if="ai.generating" name="loader" :size="16" class="spin" />
          <AppIcon v-else name="sparkles" :size="16" />
          {{ ai.generating ? "Generating..." : "Generate" }}
        </button>
      </div>
    </div>

    <!-- Error -->
    <div v-if="ai.error" class="error-banner">
      {{ ai.error }}
    </div>

    <!-- Results Section -->
    <div v-if="ai.results.length > 0" class="section">
      <div class="section-header static">
        <span>Results ({{ ai.results.length }})</span>
        <button class="clear-btn" @click="ai.clearResults()" title="Clear results">Clear</button>
      </div>
      <div class="results-grid">
        <div
          v-for="(icon, i) in ai.results"
          :key="i"
          class="result-card"
          @mouseenter="hoveredIndex = i"
          @mouseleave="hoveredIndex = null"
        >
          <div class="result-thumb">
            <img
              v-if="getSvgPreview(icon)"
              :src="getSvgPreview(icon)"
              alt="Generated icon"
            />
            <div v-else class="result-empty">No preview</div>
          </div>
          <div class="result-actions" v-if="hoveredIndex === i">
            <button class="action-btn" @click="addToCanvas(icon)" title="Add to canvas">
              <AppIcon name="plus" :size="14" />
            </button>
            <button class="action-btn" @click="regenerate(i)" title="Regenerate">
              <AppIcon name="refreshCw" :size="14" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ai-panel {
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0;
  height: 100%;
  overflow-y: auto;
}

.section {
  border-bottom: 1px solid var(--border-color);
}

.section-header {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 10px 14px;
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  cursor: pointer;
}

.section-header.static {
  cursor: default;
  padding-bottom: 6px;
}

.section-body {
  padding: 0 14px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field-label {
  font-size: 11px;
  color: var(--text-secondary);
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 4px;
}

.hint {
  font-weight: 400;
  color: var(--text-muted);
  font-size: 10px;
}

.field-input,
.field-select,
.field-textarea {
  width: 100%;
  padding: 6px 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
  transition: border-color var(--transition-fast);
  box-sizing: border-box;
}

.field-input:focus,
.field-select:focus,
.field-textarea:focus {
  border-color: var(--accent);
}

.field-textarea {
  resize: vertical;
  min-height: 60px;
  font-family: inherit;
}

.style-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 4px;
}

.style-btn {
  padding: 4px 2px;
  font-size: 10px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.style-btn:hover {
  color: var(--text-secondary);
  border-color: var(--text-muted);
}

.style-btn.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-muted);
}

.count-row {
  display: flex;
  gap: 4px;
}

.count-btn {
  flex: 1;
  padding: 4px;
  font-size: 11px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.count-btn:hover {
  color: var(--text-secondary);
}

.count-btn.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-muted);
}

.generate-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  padding: 8px;
  background: var(--accent);
  color: var(--bg-primary);
  border: none;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity var(--transition-fast);
}

.generate-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.generate-btn:not(:disabled):hover {
  opacity: 0.9;
}

.error-banner {
  margin: 8px 14px;
  padding: 8px 10px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: var(--radius-sm);
  color: #ef4444;
  font-size: 11px;
}

.results-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
  padding: 0 14px 14px;
}

.result-card {
  position: relative;
  aspect-ratio: 1;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  overflow: hidden;
  cursor: pointer;
}

.result-thumb {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 12px;
  box-sizing: border-box;
}

.result-thumb img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.result-empty {
  color: var(--text-muted);
  font-size: 10px;
}

.result-actions {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  gap: 4px;
  padding: 6px;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.7));
  justify-content: center;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: var(--radius-sm);
  background: rgba(255, 255, 255, 0.15);
  color: white;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.clear-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 10px;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: var(--radius-sm);
}

.clear-btn:hover {
  color: var(--text-secondary);
  background: var(--bg-hover);
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
