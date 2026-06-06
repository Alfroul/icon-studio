<script setup lang="ts">
import { ref, watch, computed, onUnmounted } from "vue";
import type { SvgFilter, FilterType } from "@/types";
import ToggleSwitch from "@/components/common/ToggleSwitch.vue";

const props = defineProps<{
  elementId: string;
  filter: SvgFilter | null;
}>();

const emit = defineEmits<{
  update: [filter: SvgFilter | null];
}>();

interface FilterOption {
  value: FilterType;
  label: string;
}

const filterOptions: FilterOption[] = [
  { value: "noise", label: "噪点" },
  { value: "blur", label: "模糊" },
  { value: "pixelate", label: "像素化" },
  { value: "emboss", label: "浮雕" },
  { value: "posterize", label: "色调分离" },
  { value: "turbulence", label: "湍流" },
];

const enabled = ref(!!props.filter);
const selectedType = ref<FilterType>(props.filter?.filter_type ?? "blur");
const params = ref<Record<string, number>>(props.filter?.params ? { ...props.filter.params } : {});

let emitTimer: ReturnType<typeof setTimeout> | null = null;
let skipWatch = false;

onUnmounted(() => {
  if (emitTimer) { clearTimeout(emitTimer); emitTimer = null; }
});

function debouncedEmit() {
  if (emitTimer) clearTimeout(emitTimer);
  emitTimer = setTimeout(emitFilter, 200);
}

watch(
  () => props.filter,
  (f) => {
    if (skipWatch) { skipWatch = false; return; }
    enabled.value = !!f;
    if (f) {
      selectedType.value = f.filter_type;
      params.value = { ...f.params };
    } else {
      selectedType.value = "blur";
      params.value = {};
    }
  }
);

const paramDefs = computed(() => {
  switch (selectedType.value) {
    case "noise":
      return [
        { key: "baseFrequency", label: "频率", min: 0.001, max: 0.1, step: 0.001, default: 0.05 },
        { key: "numOctaves", label: "层数", min: 1, max: 5, step: 1, default: 3 },
      ];
    case "blur":
      return [
        { key: "stdDeviation", label: "模糊度", min: 0, max: 20, step: 0.5, default: 3 },
      ];
    case "pixelate":
      return [
        { key: "size", label: "像素尺寸", min: 2, max: 20, step: 1, default: 4 },
      ];
    case "emboss":
      return [
        { key: "strength", label: "强度", min: 0.5, max: 3, step: 0.1, default: 1.0 },
      ];
    case "posterize":
      return [
        { key: "steps", label: "色阶", min: 2, max: 10, step: 1, default: 4 },
      ];
    case "turbulence":
      return [
        { key: "baseFrequency", label: "频率", min: 0.001, max: 0.1, step: 0.001, default: 0.05 },
        { key: "numOctaves", label: "层数", min: 1, max: 5, step: 1, default: 3 },
      ];
    default:
      return [];
  }
});

function getParam(key: string, fallback: number): number {
  return params.value[key] ?? fallback;
}

function setParam(key: string, value: number) {
  params.value = { ...params.value, [key]: value };
  debouncedEmit();
}

function onTypeChange() {
  const defaults: Record<string, number> = {};
  for (const def of paramDefs.value) {
    defaults[def.key] = def.default;
  }
  params.value = { ...defaults, ...params.value };
  debouncedEmit();
}

function emitFilter() {
  if (!enabled.value) {
    emit("update", null);
    return;
  }
  emit("update", {
    filter_type: selectedType.value,
    params: { ...params.value },
  });
}

function toggleEnabled() {
  enabled.value = !enabled.value;
  skipWatch = true;
  emitFilter();
}
</script>

<template>
  <div class="section">
    <div class="section-header">
      <span class="section-title">滤镜</span>
      <ToggleSwitch :model-value="enabled" @update:model-value="toggleEnabled" />
    </div>

    <div v-if="enabled" class="section-body">
      <div class="prop-row">
        <label class="field-label">类型</label>
        <select
          class="type-select"
          :value="selectedType"
          @change="selectedType = ($event.target as HTMLSelectElement).value as FilterType; onTypeChange()"
        >
          <option v-for="opt in filterOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
      </div>

      <div v-for="def in paramDefs" :key="def.key" class="prop-row">
        <label class="field-label">{{ def.label }}</label>
        <input
          type="range"
          class="param-slider"
          :min="def.min"
          :max="def.max"
          :step="def.step"
          :value="getParam(def.key, def.default)"
          @input="setParam(def.key, Number(($event.target as HTMLInputElement).value))"
        />
        <input
          type="number"
          class="num-input"
          :min="def.min"
          :max="def.max"
          :step="def.step"
          :value="getParam(def.key, def.default)"
          @input="setParam(def.key, Number(($event.target as HTMLInputElement).value))"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.section {
  border-bottom: 1px solid var(--border-color);
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
}

.section-body {
  padding: 0 16px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.prop-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.field-label {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 42px;
}

.type-select {
  flex: 1;
  height: 24px;
  padding: 0 6px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 11px;
  outline: none;
  cursor: pointer;
}

.type-select:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-muted);
}

.param-slider {
  flex: 1;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--border-color);
  border-radius: 2px;
  outline: none;
}

.param-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  background: var(--accent);
  border-radius: 50%;
  cursor: pointer;
}

.num-input {
  width: 52px;
  height: 24px;
  padding: 0 6px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 11px;
  font-family: "JetBrains Mono", monospace;
  outline: none;
  text-align: right;
}

.num-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-muted);
}

.num-input::-webkit-inner-spin-button {
  opacity: 0.3;
}
</style>
