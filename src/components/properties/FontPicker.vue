<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useProjectStore } from "@/stores/project";
import type { FontInfo } from "@/types";

const props = defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const project = useProjectStore();

const fonts = ref<FontInfo[]>([]);
const fontSearch = ref("");
const fontDropdownOpen = ref(false);
let searchTimer: ReturnType<typeof setTimeout> | null = null;

onMounted(async () => {
  fonts.value = await project.listFonts();
  document.addEventListener("click", closeFontDropdown);
});

onUnmounted(() => {
  document.removeEventListener("click", closeFontDropdown);
  if (searchTimer) clearTimeout(searchTimer);
});

function onSearchInput(e: Event) {
  const keyword = (e.target as HTMLInputElement).value;
  fontSearch.value = keyword;
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = setTimeout(async () => {
    fonts.value = await project.listFonts(keyword || undefined);
  }, 200);
}

function selectFont(family: string) {
  emit("update:modelValue", family);
  fontDropdownOpen.value = false;
  fontSearch.value = "";
}

function closeFontDropdown(e: MouseEvent) {
  if (fontDropdownOpen.value) {
    const target = e.target as HTMLElement;
    if (!target.closest(".font-dropdown-container")) {
      fontDropdownOpen.value = false;
    }
  }
}
</script>

<template>
  <div class="font-picker font-dropdown-container">
    <button
      class="font-trigger"
      @click="fontDropdownOpen = !fontDropdownOpen"
    >
      {{ props.modelValue }}
      <span class="font-arrow">&#9662;</span>
    </button>
    <div v-if="fontDropdownOpen" class="font-dropdown">
      <input
        type="text"
        class="font-search"
        placeholder="Search fonts..."
        :value="fontSearch"
        @input="onSearchInput"
      />
      <div class="font-list">
        <button
          v-for="font in fonts.slice(0, 50)"
          :key="font.family"
          :class="['font-option', { active: font.family === props.modelValue }]"
          @click="selectFont(font.family)"
        >
          {{ font.family }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.font-picker {
  position: relative;
  flex: 1;
}

.font-trigger {
  width: 100%;
  height: 26px;
  padding: 0 8px;
  background: var(--input-bg);
  border: 1px solid var(--input-border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  text-align: left;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
  transition: border-color var(--transition-fast) ease;
}

.font-trigger:hover {
  border-color: var(--accent);
}

.font-arrow {
  font-size: 10px;
  color: var(--text-muted);
}

.font-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: 100;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 255, 255, 0.06);
  max-height: 200px;
  display: flex;
  flex-direction: column;
}

.font-search {
  width: 100%;
  height: 26px;
  padding: 0 8px;
  background: var(--input-bg);
  border: none;
  border-bottom: 1px solid var(--border-color);
  border-radius: var(--radius-md) var(--radius-md) 0 0;
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
}

.font-list {
  flex: 1;
  overflow-y: auto;
  max-height: 170px;
}

.font-option {
  width: 100%;
  padding: 5px 8px;
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 11px;
  text-align: left;
  cursor: pointer;
  transition: background var(--transition-fast) ease;
}

.font-option:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.font-option.active {
  background: var(--accent-muted);
  color: var(--accent);
}
</style>
