<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";

const project = useProjectStore();
const ui = useUiStore();

async function startBlank() {
  await project.newProject();
}

function openTemplates() {
  ui.setPanel("templates");
}

function openAi() {
  ui.setPanel("ai");
}

async function importImage() {
  const input = document.createElement("input");
  input.type = "file";
  input.accept = "image/png,image/jpeg,image/webp,image/svg+xml";
  input.onchange = async () => {
    const file = input.files?.[0];
    if (!file) return;
    await handleImageFile(file);
  };
  input.click();
}

async function handleImageFile(file: File) {
  try {
    if (file.type === "image/svg+xml") {
      const text = await file.text();
      await invoke("add_path_from_svg", { svgContent: text });
    } else {
      const dataUrl = await readFileAsDataUrl(file);
      const img = new Image();
      await new Promise<void>((resolve, reject) => {
        img.onload = () => resolve();
        img.onerror = () => reject(new Error("Failed to load image"));
        img.src = dataUrl;
      });
      await invoke("add_image_from_data", {
        imageData: dataUrl,
        width: img.naturalWidth,
        height: img.naturalHeight,
      });
    }
    await project.refreshElements();
    ui.showToast("Image imported", "success");
  } catch (e) {
    ui.showToast(`Import failed: ${e}`, "error");
  }
}

function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error("Failed to read file"));
    reader.readAsDataURL(file);
  });
}

defineExpose({ handleImageFile });
</script>

<template>
  <div class="quickstart-overlay">
    <div class="quickstart-card">
      <h2 class="quickstart-title">Create an Icon</h2>
      <p class="quickstart-subtitle">Choose how to get started</p>
      <div class="quickstart-options">
        <button class="option-btn" @click="importImage">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="m21 15-5-5L5 21"/></svg>
          <span>Drop Image</span>
        </button>
        <button class="option-btn" @click="openTemplates">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
          <span>From Template</span>
        </button>
        <button class="option-btn" @click="openAi">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2a4 4 0 0 0-4 4v2H6a2 2 0 0 0-2 2v10h16V10a2 2 0 0 0-2-2h-2V6a4 4 0 0 0-4-4z"/><circle cx="9" cy="15" r="1"/><circle cx="15" cy="15" r="1"/></svg>
          <span>AI Generate</span>
        </button>
        <button class="option-btn" @click="startBlank">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5v14M5 12h14"/></svg>
          <span>Blank Canvas</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.quickstart-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(9, 9, 11, 0.85);
  backdrop-filter: blur(4px);
  z-index: 10;
}

.quickstart-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 32px 40px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
}

.quickstart-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.quickstart-subtitle {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0 0 8px;
}

.quickstart-options {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.option-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 20px 24px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
  min-width: 120px;
}

.option-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.option-btn:active {
  background: var(--accent);
  color: var(--bg-primary);
}

.option-btn span {
  font-size: 12px;
  font-weight: 500;
}
</style>
