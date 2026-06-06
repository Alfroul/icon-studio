<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  svgContent: string;
}>();

const svgDataUrl = computed(() => {
  if (!props.svgContent) return "";
  return "data:image/svg+xml;charset=utf-8," + encodeURIComponent(props.svgContent);
});
</script>

<template>
  <div class="svg-preview">
    <img v-if="svgContent" :src="svgDataUrl" class="svg-render" alt="Preview" />
  </div>
</template>

<style scoped>
.svg-preview {
  width: 100%;
  height: 100%;
}

.svg-render {
  width: 100%;
  height: 100%;
  object-fit: contain;
}
</style>
