<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useExportStore, PNG_SIZES } from "@/stores/export";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { useVariantsStore } from "@/stores/variantsStore";
import AppIcon from "@/components/common/AppIcon.vue";

const store = useExportStore();
const ui = useUiStore();
const project = useProjectStore();
const variantsStore = useVariantsStore();

// Code export settings
const codeFormat = ref<string>('reactTs');
const codeComponentName = ref<string>('MyIcon');
const codeParametrizeFill = ref(true);
const codePreview = ref<string>('');
const codePreviewVisible = ref(false);

const CODE_FORMATS = [
  { group: 'Web', formats: [
    { value: 'reactTs', label: 'React TS', ext: '.tsx' },
    { value: 'vueTs', label: 'Vue TS', ext: '.vue' },
    { value: 'svelte', label: 'Svelte', ext: '.svelte' },
    { value: 'svgSymbol', label: 'SVG Symbol', ext: '.svg' },
    { value: 'svgMinified', label: 'SVG Minified', ext: '.min.svg' },
  ]},
  { group: 'Mobile', formats: [
    { value: 'swiftUI', label: 'SwiftUI', ext: '.swift' },
    { value: 'flutter', label: 'Flutter', ext: '.dart' },
  ]},
  { group: 'Desktop', formats: [
    { value: 'xaml', label: 'XAML', ext: '.xaml' },
    { value: 'cpp', label: 'C++', ext: '.hpp' },
  ]},
  { group: 'Android', formats: [
    { value: 'vectorDrawable', label: 'VectorDrawable', ext: '.xml' },
  ]},
];

async function generateCodePreview() {
  const svg = project.svgPreview;
  if (!svg) {
    ui.showToast('No SVG content', 'warning');
    return;
  }
  const result = await store.exportCode(
    svg,
    codeComponentName.value,
    codeFormat.value,
    24,
    codeParametrizeFill.value,
  );
  if (result) {
    codePreview.value = result.code;
    codePreviewVisible.value = true;
  }
}

async function copyCode() {
  if (!codePreview.value) {
    await generateCodePreview();
  }
  if (codePreview.value) {
    try {
      await navigator.clipboard.writeText(codePreview.value);
      ui.showToast('Code copied to clipboard', 'success');
    } catch (e: unknown) {
      ui.showToast(`Failed: ${e instanceof Error ? e.message : String(e)}`, 'error');
    }
  }
}

async function downloadCodeFile() {
  const svg = project.svgPreview;
  if (!svg) {
    ui.showToast('No SVG content', 'warning');
    return;
  }
  const result = await store.exportCode(
    svg,
    codeComponentName.value,
    codeFormat.value,
    24,
    codeParametrizeFill.value,
  );
  if (result) {
    const blob = new Blob([result.code], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = result.filename;
    a.click();
    URL.revokeObjectURL(url);
  }
}

// Animation export settings
const lottieFps = ref(30);
const gifFps = ref(15);
const gifSize = ref(512);

// Preview grid sizes - small subset for quick visual check
const previewSizes = [16, 32, 64, 128, 256];

function svgDataUrl(svgContent: string): string {
  return "data:image/svg+xml;charset=utf-8," + encodeURIComponent(svgContent);
}

const statusMessage = computed(() => {
  if (store.error) return store.error;
  if (store.exportResults.length > 0) return `已导出 ${store.exportResults.length} 个文件`;
  return "";
});

const statusType = computed(() => {
  if (store.error) return "error";
  if (store.exportResults.length > 0) return "success";
  return "";
});

const pngSizesForIco = [16, 32, 48, 256];

function toggleFormat(format: string) {
  const idx = store.selectedFormats.indexOf(format);
  if (idx >= 0) {
    store.selectedFormats.splice(idx, 1);
  } else {
    store.selectedFormats.push(format);
  }
}

function togglePngSize(size: number) {
  const idx = store.selectedPngSizes.indexOf(size);
  if (idx >= 0) {
    store.selectedPngSizes.splice(idx, 1);
  } else {
    store.selectedPngSizes.push(size);
  }
}

function selectAllPngSizes() {
  store.selectedPngSizes = [...PNG_SIZES];
}

function deselectAllPngSizes() {
  store.selectedPngSizes = [];
}

async function runExport(
  exportFn: () => Promise<string[]>,
  errorMsg: string = "导出失败"
) {
  if (!store.outputDir) {
    ui.showToast("Please select an output directory first", "warning");
    return;
  }
  if (store.exporting) return;
  store.exporting = true;
  store.error = null;
  try {
    store.exportResults = await exportFn();
  } catch (e: unknown) {
    store.error = e instanceof Error ? e.message : String(e);
  } finally {
    store.exporting = false;
  }
}

async function handleExportAll() {
  await runExport(() => store.exportAll(), "导出失败");
}

async function handleExportSvg() {
  await runExport(() => store.exportSvg(), "导出 SVG 失败");
}

async function handleExportPng() {
  if (store.selectedPngSizes.length === 0) {
    ui.showToast("Please select at least one PNG size", "warning");
    return;
  }
  await runExport(() => store.exportPng(), "导出 PNG 失败");
}

async function handleExportIco() {
  await runExport(() => store.exportIco(), "导出 ICO 失败");
}

async function handleAndroidIcons() {
  await runExport(() => store.exportAndroidIcons(), "导出 Android 图标失败");
}

async function handleIosIcons() {
  await runExport(() => store.exportIosIcons(), "导出 iOS 图标失败");
}

async function copySvgForFigma() {
  try {
    const svg = project.svgPreview;
    if (!svg) {
      ui.showToast('No SVG content to copy', 'warning');
      return;
    }
    await navigator.clipboard.writeText(svg);
    ui.showToast('SVG copied to clipboard — paste into Figma', 'success');
  } catch (e: unknown) {
    ui.showToast(`Failed: ${e instanceof Error ? e.message : String(e)}`, 'error');
  }
}

// Token export settings
const tokenFormat = ref<string>('cssVariables');
const tokenPreview = ref<string>('');
const tokenVisible = ref(false);

// Icon Font export settings
const fontVisible = ref(false);
const fontName = ref('MyIcons');
const fontFormats = ref<string[]>(['ttf', 'woff']);
const fontIncludeCss = ref(true);
const fontIncludeDemo = ref(true);
const fontUnicodeStart = ref('E000');

const FONT_FORMAT_OPTIONS = [
  { value: 'ttf', label: 'TTF' },
  { value: 'woff', label: 'WOFF' },
];

function toggleFontFormat(fmt: string) {
  const idx = fontFormats.value.indexOf(fmt);
  if (idx >= 0) fontFormats.value.splice(idx, 1);
  else fontFormats.value.push(fmt);
}

async function handleExportIconFont() {
  if (!project.svgPreview) {
    ui.showToast('No SVG content', 'warning');
    return;
  }
  const unicodeVal = parseInt(fontUnicodeStart.value, 16) || 0xe000;
  const glyphs = [{
    iconName: 'icon',
    unicode: String.fromCodePoint(unicodeVal),
    svgPathData: project.svgPreview,
  }];
  const result = await store.exportIconFont(
    glyphs,
    fontName.value,
    fontFormats.value,
    fontIncludeCss.value,
    fontIncludeDemo.value,
    unicodeVal,
  );
  if (result && result.files.length > 0) {
    // Download each file
    for (const [filename, data] of result.files) {
      const blob = new Blob([new Uint8Array(data)], { type: 'application/octet-stream' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      a.click();
      URL.revokeObjectURL(url);
    }
    ui.showToast(`Exported ${result.files.length} font files`, 'success');
  }
}

const TOKEN_FORMATS = [
  { value: 'cssVariables', label: 'CSS Variables', ext: '.css' },
  { value: 'jsonDtcg', label: 'JSON (DTCG)', ext: '.json' },
  { value: 'scssVariables', label: 'SCSS Variables', ext: '.scss' },
  { value: 'tailwindConfig', label: 'Tailwind Config', ext: '.js' },
];

async function generateTokenPreview() {
  const result = await store.exportTokens(tokenFormat.value);
  if (result) {
    tokenPreview.value = result.content;
  }
}

async function copyTokens() {
  if (!tokenPreview.value) {
    await generateTokenPreview();
  }
  if (tokenPreview.value) {
    try {
      await navigator.clipboard.writeText(tokenPreview.value);
      ui.showToast('Tokens copied to clipboard', 'success');
    } catch (e: unknown) {
      ui.showToast(`Failed: ${e instanceof Error ? e.message : String(e)}`, 'error');
    }
  }
}

async function downloadTokenFile() {
  const result = await store.exportTokens(tokenFormat.value);
  if (result) {
    const blob = new Blob([result.content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = result.filename;
    a.click();
    URL.revokeObjectURL(url);
  }
}

// Presets
const PRESETS_KEY = 'iconstudio-export-presets';

interface ExportPreset {
  name: string;
  formats: string[];
  pngSizes: number[];
}

const presetName = ref('');
const presets = ref<ExportPreset[]>([]);

function loadPresets() {
  try {
    const raw = localStorage.getItem(PRESETS_KEY);
    presets.value = raw ? JSON.parse(raw) : [];
  } catch { presets.value = []; }
}

function savePreset() {
  const name = presetName.value.trim();
  if (!name) return;
  const existing = presets.value.findIndex(p => p.name === name);
  const preset: ExportPreset = {
    name,
    formats: [...store.selectedFormats],
    pngSizes: [...store.selectedPngSizes],
  };
  if (existing >= 0) {
    presets.value[existing] = preset;
  } else {
    presets.value.push(preset);
  }
  localStorage.setItem(PRESETS_KEY, JSON.stringify(presets.value));
  presetName.value = '';
}

function applyPreset(preset: ExportPreset) {
  store.selectedFormats = [...preset.formats];
  store.selectedPngSizes = [...preset.pngSizes];
}

function deletePreset(index: number) {
  presets.value.splice(index, 1);
  localStorage.setItem(PRESETS_KEY, JSON.stringify(presets.value));
}

async function handleExportLottie() {
  if (!store.outputDir) {
    ui.showToast("Please select an output directory first", "warning");
    return;
  }
  if (store.exporting) return;
  store.exporting = true;
  store.error = null;
  try {
    const path = await invoke<string>("export_lottie", {
      outputPath: `${store.outputDir}/animation.json`,
      fps: lottieFps.value,
    });
    store.exportResults = [path];
  } catch (e: unknown) {
    store.error = e instanceof Error ? e.message : String(e);
  } finally {
    store.exporting = false;
  }
}

async function handleExportGif() {
  if (!store.outputDir) {
    ui.showToast("Please select an output directory first", "warning");
    return;
  }
  if (store.exporting) return;
  store.exporting = true;
  store.error = null;
  try {
    const path = await invoke<string>("export_animated_gif", {
      outputPath: `${store.outputDir}/animation.gif`,
      fps: gifFps.value,
      width: gifSize.value,
      height: gifSize.value,
    });
    store.exportResults = [path];
  } catch (e: unknown) {
    store.error = e instanceof Error ? e.message : String(e);
  } finally {
    store.exporting = false;
  }
}

loadPresets();

// Variant export settings
const exportVariants = ref(false);
const selectedVariantIndices = ref<number[]>([]);

// Weight variants
const weightVisible = ref(false);
const selectedWeights = ref<string[]>(['thin', 'light', 'regular', 'medium', 'bold', 'fill']);
const weightResults = ref<Array<{ weight: string; svg: string }>>([]);
const weightGenerating = ref(false);

const WEIGHT_OPTIONS = [
  { value: 'thin', label: 'Thin', factor: '0.5x' },
  { value: 'light', label: 'Light', factor: '0.75x' },
  { value: 'regular', label: 'Regular', factor: '1.0x' },
  { value: 'medium', label: 'Medium', factor: '1.25x' },
  { value: 'bold', label: 'Bold', factor: '1.5x' },
  { value: 'fill', label: 'Fill', factor: 'stroke→fill' },
];

function toggleWeight(w: string) {
  const idx = selectedWeights.value.indexOf(w);
  if (idx >= 0) selectedWeights.value.splice(idx, 1);
  else selectedWeights.value.push(w);
}

async function handleGenerateWeightVariants() {
  if (selectedWeights.value.length === 0) {
    ui.showToast('Select at least one weight', 'warning');
    return;
  }
  weightGenerating.value = true;
  try {
    weightResults.value = await store.generateWeightVariants(selectedWeights.value);
    if (weightResults.value.length === 0) {
      ui.showToast('No variants generated', 'warning');
    }
  } finally {
    weightGenerating.value = false;
  }
}

function svgThumb(svg: string): string {
  return "data:image/svg+xml;charset=utf-8," + encodeURIComponent(svg);
}

async function exportAllWeightVariants() {
  if (weightResults.value.length === 0) return;
  for (const v of weightResults.value) {
    const blob = new Blob([v.svg], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `icon-${v.weight}.svg`;
    a.click();
    URL.revokeObjectURL(url);
  }
  ui.showToast(`Exported ${weightResults.value.length} weight variant(s)`, 'success');
}

// Favicon snippet
const faviconAppName = ref('My App');
const faviconSnippetText = ref('');
const faviconSnippetVisible = ref(false);

// Sprite Sheet
const spriteSheetVisible = ref(false);
const SPRITE_SIZES = [16, 24, 32, 48, 64];
const spriteSheetSize = ref(24);
const spriteSheetLayout = ref<'horizontal' | 'grid'>('horizontal');
const spriteSheetColumns = ref(4);
const spriteSheetPadding = ref(0);
const spriteSheetResult = ref<import('@/types').SpriteSheetResult | null>(null);

const spriteSheetJsonPreview = computed(() => {
  if (!spriteSheetResult.value) return '';
  return JSON.stringify(spriteSheetResult.value.icons, null, 2);
});

async function handleExportSpriteSheet() {
  if (!store.outputDir) {
    ui.showToast("Please select an output directory first", "warning");
    return;
  }
  const cols = spriteSheetLayout.value === 'horizontal' ? 0 : spriteSheetColumns.value;
  const result = await store.exportSpriteSheet(
    `${store.outputDir}/sprite-sheet.png`,
    cols,
    spriteSheetSize.value,
    spriteSheetPadding.value,
  );
  if (result) {
    spriteSheetResult.value = result;
    ui.showToast(`Sprite sheet: ${result.total_width}×${result.total_height}`, 'success');
  }
}

// All-platforms export
const showAllPlatformsModal = ref(false);
const allPlatformsAppName = ref('My App');
const allPlatformsThemeColor = ref('#000000');
const allPlatformsBgColor = ref('#FFFFFF');

async function loadFaviconSnippet() {
  if (faviconSnippetText.value) return;
  faviconSnippetText.value = await store.getFaviconHtmlSnippet(faviconAppName.value);
}

async function refreshFaviconSnippet() {
  faviconSnippetText.value = await store.getFaviconHtmlSnippet(faviconAppName.value);
}

async function copyFaviconSnippet() {
  await loadFaviconSnippet();
  if (faviconSnippetText.value) {
    try {
      await navigator.clipboard.writeText(faviconSnippetText.value);
      ui.showToast('HTML snippet copied to clipboard', 'success');
    } catch (e: unknown) {
      ui.showToast(`Failed: ${e instanceof Error ? e.message : String(e)}`, 'error');
    }
  }
}

function toggleVariantExport() {
  exportVariants.value = !exportVariants.value;
  if (exportVariants.value && selectedVariantIndices.value.length === 0) {
    // Select all by default
    selectedVariantIndices.value = variantsStore.variants.map((_, i) => i);
  }
}

function toggleVariantIndex(idx: number) {
  const i = selectedVariantIndices.value.indexOf(idx);
  if (i >= 0) selectedVariantIndices.value.splice(i, 1);
  else selectedVariantIndices.value.push(idx);
}

async function handleExportSelectedVariants() {
  if (!store.outputDir) {
    ui.showToast("Please select an output directory first", "warning");
    return;
  }
  store.exporting = true;
  store.error = null;
  try {
    let allPaths: string[] = [];
    for (const idx of selectedVariantIndices.value) {
      const paths = await variantsStore.exportVariant(idx, "svg", store.outputDir);
      allPaths = allPaths.concat(paths);
    }
    store.exportResults = allPaths;
    ui.showToast(`Exported ${allPaths.length} variant file(s)`, "success");
  } catch (e: unknown) {
    store.error = e instanceof Error ? e.message : String(e);
  } finally {
    store.exporting = false;
  }
}

async function handlePwaIcons() {
  if (!store.outputDir) {
    ui.showToast("Please select an output directory first", "warning");
    return;
  }
  store.exporting = true;
  store.error = null;
  try {
    const paths = await store.exportPwaIcons(
      store.outputDir,
      faviconAppName.value,
      '#000000',
      '#FFFFFF',
    );
    store.exportResults = paths;
    ui.showToast(`Exported ${paths.length} PWA file(s)`, 'success');
  } catch (e: unknown) {
    store.error = e instanceof Error ? e.message : String(e);
  } finally {
    store.exporting = false;
  }
}

async function handleAllPlatformsExport() {
  if (!store.outputDir) {
    ui.showToast("Please select an output directory first", "warning");
    showAllPlatformsModal.value = false;
    return;
  }
  showAllPlatformsModal.value = false;
  store.exporting = true;
  store.error = null;
  try {
    const result = await store.exportAllPlatforms(
      store.outputDir,
      allPlatformsAppName.value,
      allPlatformsThemeColor.value,
      allPlatformsBgColor.value,
    );
    if (result) {
      const total = result.ios_paths.length + result.android_paths.length + result.pwa_paths.length + result.favicon_paths.length;
      store.exportResults = [...result.ios_paths, ...result.android_paths, ...result.pwa_paths, ...result.favicon_paths];
      ui.showToast(`Exported ${total} files across all platforms`, 'success');
    }
  } catch (e: unknown) {
    store.error = e instanceof Error ? e.message : String(e);
  } finally {
    store.exporting = false;
  }
}

</script>

<template>
  <div class="export-panel">
    <!-- Preview Grid -->
    <div class="section" v-if="project.svgPreview">
      <div class="section-label">Preview</div>
      <div class="preview-grid">
        <div v-for="size in previewSizes" :key="size" class="preview-item">
          <div class="preview-frame" :style="{ width: Math.min(size, 64) + 'px', height: Math.min(size, 64) + 'px' }">
            <img
              :src="svgDataUrl(project.svgPreview)"
              :style="{ width: Math.min(size, 64) + 'px', height: Math.min(size, 64) + 'px', imageRendering: size <= 32 ? 'pixelated' : 'auto' }"
            />
          </div>
          <span class="preview-label">{{ size }}×{{ size }}</span>
        </div>
      </div>
    </div>

    <!-- 导出格式 -->
    <div class="section">
      <div class="section-label">导出格式</div>
      <div class="format-list">
        <label class="checkbox-row">
          <input
            type="checkbox"
            :checked="store.selectedFormats.includes('svg')"
            @change="toggleFormat('svg')"
          />
          <span class="checkbox-label">SVG</span>
        </label>

        <label class="checkbox-row">
          <input
            type="checkbox"
            :checked="store.selectedFormats.includes('png')"
            @change="toggleFormat('png')"
          />
          <span class="checkbox-label">PNG</span>
        </label>

        <div v-if="store.selectedFormats.includes('png')" class="png-sizes">
          <div class="png-sizes-header">
            <span class="png-sizes-title">尺寸</span>
            <div class="png-sizes-actions">
              <button class="link-btn" @click="selectAllPngSizes">全选</button>
              <button class="link-btn" @click="deselectAllPngSizes">清除</button>
            </div>
          </div>
          <div class="size-chips">
            <label
              v-for="size in PNG_SIZES"
              :key="size"
              :class="['size-chip', { active: store.selectedPngSizes.includes(size) }]"
            >
              <input
                type="checkbox"
                :checked="store.selectedPngSizes.includes(size)"
                @change="togglePngSize(size)"
                class="hidden-checkbox"
              />
              {{ size }}
            </label>
          </div>
        </div>

        <label class="checkbox-row">
          <input
            type="checkbox"
            :checked="store.selectedFormats.includes('ico')"
            @change="toggleFormat('ico')"
          />
          <span class="checkbox-label">ICO</span>
          <span class="tag">16 · 32 · 48 · 256</span>
        </label>

        <label class="checkbox-row">
          <input
            type="checkbox"
            :checked="store.selectedFormats.includes('webp')"
            @change="toggleFormat('webp')"
          />
          <span class="checkbox-label">WebP</span>
        </label>

        <div class="anim-export-divider"></div>

        <label class="checkbox-row code-toggle" @click.prevent="codePreviewVisible = !codePreviewVisible">
          <input type="checkbox" :checked="codePreviewVisible" @click.prevent />
          <span class="checkbox-label">Code</span>
          <span class="tag">10 formats</span>
        </label>

        <div v-if="codePreviewVisible" class="code-export-section">
          <div class="code-field">
            <label class="code-label">Component Name</label>
            <input v-model="codeComponentName" class="code-input" placeholder="MyIcon" />
          </div>
          <div class="code-field">
            <label class="code-label">Format</label>
            <select v-model="codeFormat" class="code-select">
              <optgroup v-for="group in CODE_FORMATS" :key="group.group" :label="group.group">
                <option v-for="fmt in group.formats" :key="fmt.value" :value="fmt.value">{{ fmt.label }} ({{ fmt.ext }})</option>
              </optgroup>
            </select>
          </div>
          <label class="checkbox-row code-option">
            <input type="checkbox" v-model="codeParametrizeFill" />
            <span class="checkbox-label">Parametrize fill → currentColor</span>
          </label>
          <div class="code-actions">
            <button class="action-btn" @click="generateCodePreview">Preview</button>
            <button class="action-btn" @click="copyCode">Copy Code</button>
            <button class="action-btn" @click="downloadCodeFile">Download</button>
          </div>
          <div v-if="codePreview" class="code-preview">
            <pre><code>{{ codePreview }}</code></pre>
          </div>
        </div>

        <label class="checkbox-row code-toggle" @click.prevent="tokenVisible = !tokenVisible">
          <input type="checkbox" :checked="tokenVisible" @click.prevent />
          <span class="checkbox-label">Tokens</span>
          <span class="tag">CSS / JSON / SCSS / TW</span>
        </label>

        <div v-if="tokenVisible" class="code-export-section">
          <div class="code-field">
            <label class="code-label">Format</label>
            <select v-model="tokenFormat" class="code-select">
              <option v-for="fmt in TOKEN_FORMATS" :key="fmt.value" :value="fmt.value">{{ fmt.label }} ({{ fmt.ext }})</option>
            </select>
          </div>
          <div class="code-actions">
            <button class="action-btn" @click="generateTokenPreview">Preview</button>
            <button class="action-btn" @click="copyTokens">Copy</button>
            <button class="action-btn" @click="downloadTokenFile">Download</button>
          </div>
          <div v-if="tokenPreview" class="code-preview">
            <pre><code>{{ tokenPreview }}</code></pre>
          </div>
        </div>

        <label class="checkbox-row code-toggle" @click.prevent="fontVisible = !fontVisible">
          <input type="checkbox" :checked="fontVisible" @click.prevent />
          <span class="checkbox-label">Icon Font</span>
          <span class="tag">TTF / WOFF</span>
        </label>

        <div v-if="fontVisible" class="code-export-section">
          <div class="code-field">
            <label class="code-label">Font Name</label>
            <input v-model="fontName" class="code-input" placeholder="MyIcons" />
          </div>
          <div class="code-field">
            <label class="code-label">Formats</label>
            <div class="font-format-chips">
              <label
                v-for="fmt in FONT_FORMAT_OPTIONS"
                :key="fmt.value"
                :class="['size-chip', { active: fontFormats.includes(fmt.value) }]"
                @click="toggleFontFormat(fmt.value)"
              >
                {{ fmt.label }}
              </label>
            </div>
          </div>
          <div class="code-field">
            <label class="code-label">Unicode Start</label>
            <input v-model="fontUnicodeStart" class="code-input" placeholder="E000" style="width:80px;flex:none" />
          </div>
          <label class="checkbox-row code-option">
            <input type="checkbox" v-model="fontIncludeCss" />
            <span class="checkbox-label">Include CSS</span>
          </label>
          <label class="checkbox-row code-option">
            <input type="checkbox" v-model="fontIncludeDemo" />
            <span class="checkbox-label">Include Demo HTML</span>
          </label>
          <div class="code-actions">
            <button class="action-btn" @click="handleExportIconFont">Export Font</button>
          </div>
        </div>

        <div class="anim-export-divider"></div>

        <div class="anim-export-row">
          <button
            class="pack-btn"
            :disabled="store.exporting"
            @click="handleExportLottie"
          >
            <AppIcon name="file-json" :size="14" />
            Lottie JSON
          </button>
        </div>
        <div class="anim-settings">
          <label class="setting-row">
            <span class="setting-label">FPS</span>
            <select v-model.number="lottieFps" class="setting-select">
              <option :value="24">24</option>
              <option :value="30">30</option>
              <option :value="60">60</option>
            </select>
          </label>
        </div>

        <div class="anim-export-row">
          <button
            class="pack-btn"
            :disabled="store.exporting"
            @click="handleExportGif"
          >
            <AppIcon name="film" :size="14" />
            Animated GIF
          </button>
        </div>
        <div class="anim-settings">
          <label class="setting-row">
            <span class="setting-label">FPS</span>
            <select v-model.number="gifFps" class="setting-select">
              <option :value="10">10</option>
              <option :value="15">15</option>
              <option :value="24">24</option>
              <option :value="30">30</option>
            </select>
          </label>
          <label class="setting-row">
            <span class="setting-label">Size</span>
            <select v-model.number="gifSize" class="setting-select">
              <option :value="128">128</option>
              <option :value="256">256</option>
              <option :value="512">512</option>
            </select>
          </label>
        </div>
      </div>
    </div>

    <!-- 输出目录 -->
    <div class="section">
      <div class="section-label">输出目录</div>
      <div class="output-dir-row">
        <span class="output-path">{{ store.outputDir || "未选择目录" }}</span>
        <button class="action-btn" @click="store.selectOutputDir">选择目录</button>
      </div>
    </div>

    <!-- Export Presets -->
    <div class="section">
      <div class="section-label">Export Presets</div>
      <div class="preset-save-row">
        <input
          v-model="presetName"
          class="preset-input"
          placeholder="Preset name..."
          @keydown.enter="savePreset"
        />
        <button class="action-btn" @click="savePreset" :disabled="!presetName.trim()">Save</button>
      </div>
      <div v-if="presets.length > 0" class="preset-list">
        <div v-for="(preset, idx) in presets" :key="preset.name" class="preset-item">
          <button class="preset-name" @click="applyPreset(preset)">{{ preset.name }}</button>
          <span class="preset-info">{{ preset.formats.join(', ') }} · {{ preset.pngSizes.length }} sizes</span>
          <button class="preset-delete" @click="deletePreset(idx)">×</button>
        </div>
      </div>
    </div>

    <!-- Variant Export -->
    <div v-if="variantsStore.variants.length > 0" class="section">
      <label class="checkbox-row" @click.prevent="toggleVariantExport">
        <input type="checkbox" :checked="exportVariants" @click.prevent />
        <span class="checkbox-label">Export Variants</span>
        <span class="tag">{{ variantsStore.variants.length }} variant(s)</span>
      </label>
      <div v-if="exportVariants" class="variant-export-list">
        <div v-for="(v, idx) in variantsStore.variants" :key="idx" class="variant-export-row">
          <label class="checkbox-row compact">
            <input
              type="checkbox"
              :checked="selectedVariantIndices.includes(idx)"
              @change="toggleVariantIndex(idx)"
            />
            <span class="checkbox-label">{{ v.name }}</span>
            <span class="tag">icon.{{ v.name.toLowerCase().replace(/\s+/g, '-') }}.svg</span>
          </label>
        </div>
        <button class="action-btn" @click="handleExportSelectedVariants" :disabled="store.exporting || selectedVariantIndices.length === 0">
          Export Selected Variants
        </button>
      </div>
    </div>

    <!-- Weight Variants -->
    <div class="section">
      <label class="checkbox-row" @click.prevent="weightVisible = !weightVisible">
        <input type="checkbox" :checked="weightVisible" @click.prevent />
        <span class="checkbox-label">Weight Variants</span>
        <span class="tag">6 weights</span>
      </label>
      <div v-if="weightVisible" class="code-export-section">
        <div class="weight-chips">
          <label
            v-for="w in WEIGHT_OPTIONS"
            :key="w.value"
            :class="['size-chip', { active: selectedWeights.includes(w.value) }]"
            @click="toggleWeight(w.value)"
          >
            {{ w.label }}
          </label>
        </div>
        <div class="code-actions">
          <button class="action-btn" @click="handleGenerateWeightVariants" :disabled="weightGenerating || selectedWeights.length === 0">
            {{ weightGenerating ? 'Generating...' : 'Generate Variants' }}
          </button>
        </div>
        <div v-if="weightResults.length > 0" class="weight-results">
          <div class="weight-results-grid">
            <div v-for="v in weightResults" :key="v.weight" class="weight-result-item">
              <div class="preview-frame weight-preview">
                <img :src="svgThumb(v.svg)" style="width: 48px; height: 48px" />
              </div>
              <span class="preview-label">{{ v.weight }}</span>
            </div>
          </div>
          <button class="action-btn" @click="exportAllWeightVariants" style="margin-top: 6px">
            Export All
          </button>
        </div>
      </div>
    </div>

    <!-- Sprite Sheet -->
    <div class="section">
      <label class="checkbox-row" @click.prevent="spriteSheetVisible = !spriteSheetVisible">
        <input type="checkbox" :checked="spriteSheetVisible" @click.prevent />
        <span class="checkbox-label">Sprite Sheet</span>
        <span class="tag">Image Strip</span>
      </label>
      <div v-if="spriteSheetVisible" class="code-export-section">
        <div class="code-field">
          <label class="code-label">Icon Size</label>
          <div class="size-chips">
            <label
              v-for="s in SPRITE_SIZES"
              :key="s"
              :class="['size-chip', { active: spriteSheetSize === s }]"
              @click="spriteSheetSize = s"
            >
              {{ s }}
            </label>
          </div>
        </div>
        <div class="code-field">
          <label class="code-label">Layout</label>
          <select v-model="spriteSheetLayout" class="code-select">
            <option value="horizontal">Horizontal (1 row)</option>
            <option value="grid">Grid (columns)</option>
          </select>
        </div>
        <div v-if="spriteSheetLayout === 'grid'" class="code-field">
          <label class="code-label">Columns</label>
          <input v-model.number="spriteSheetColumns" type="number" min="1" max="64" class="code-input" style="width:60px;flex:none" />
        </div>
        <div class="code-field">
          <label class="code-label">Padding</label>
          <input v-model.number="spriteSheetPadding" type="number" min="0" max="32" class="code-input" style="width:60px;flex:none" />
          <span class="setting-label">px</span>
        </div>
        <div class="code-actions">
          <button class="action-btn" @click="handleExportSpriteSheet" :disabled="store.exporting">
            Export Sprite Sheet
          </button>
        </div>
        <div v-if="spriteSheetResult" class="code-preview">
          <pre><code>{{ spriteSheetResult.image_path }} ({{ spriteSheetResult.total_width }}×{{ spriteSheetResult.total_height }})

icons.json:
{{ spriteSheetJsonPreview }}</code></pre>
        </div>
      </div>
    </div>

    <!-- App 图标打包 -->
    <div class="section">
      <div class="section-label">App 图标打包</div>
      <div class="app-icon-buttons">
        <button
          class="pack-btn"
          :disabled="store.exporting"
          @click="handleAndroidIcons"
        >
          <span class="pack-icon"><AppIcon name="android" :size="16" /></span>
          Android 图标
        </button>
        <button
          class="pack-btn"
          :disabled="store.exporting"
          @click="handleIosIcons"
        >
          <span class="pack-icon"><AppIcon name="apple" :size="16" /></span>
          iOS 图标
        </button>
      </div>
      <div class="app-icon-buttons" style="margin-top: 6px">
        <button
          class="pack-btn"
          :disabled="store.exporting"
          @click="handlePwaIcons"
        >
          <span class="pack-icon"><AppIcon name="globe" :size="16" /></span>
          PWA
        </button>
        <button
          class="pack-btn"
          :disabled="store.exporting"
          @click="showAllPlatformsModal = true"
        >
          <span class="pack-icon"><AppIcon name="layers" :size="16" /></span>
          全平台
        </button>
      </div>
    </div>

    <!-- All Platforms Modal -->
    <div v-if="showAllPlatformsModal" class="modal-overlay" @click.self="showAllPlatformsModal = false">
      <div class="modal-content">
        <div class="modal-header">全平台导出配置</div>
        <div class="modal-body">
          <div class="code-field">
            <label class="code-label">App Name</label>
            <input v-model="allPlatformsAppName" class="code-input" placeholder="My App" />
          </div>
          <div class="code-field">
            <label class="code-label">Theme Color</label>
            <input v-model="allPlatformsThemeColor" class="code-input" placeholder="#000000" />
          </div>
          <div class="code-field">
            <label class="code-label">Background</label>
            <input v-model="allPlatformsBgColor" class="code-input" placeholder="#FFFFFF" />
          </div>
        </div>
        <div class="modal-footer">
          <button class="action-btn" @click="showAllPlatformsModal = false">取消</button>
          <button class="action-btn" style="background: var(--accent); color: var(--bg-primary); border-color: var(--accent)" @click="handleAllPlatformsExport" :disabled="store.exporting">导出</button>
        </div>
      </div>
    </div>

    <!-- Favicon HTML Snippet -->
    <div class="section">
      <label class="checkbox-row" @click.prevent="faviconSnippetVisible = !faviconSnippetVisible">
        <input type="checkbox" :checked="faviconSnippetVisible" @click.prevent />
        <span class="checkbox-label">Favicon HTML Snippet</span>
        <span class="tag">Copy &amp; paste</span>
      </label>
      <div v-if="faviconSnippetVisible" class="code-export-section">
        <div class="code-field">
          <label class="code-label">App Name</label>
          <input v-model="faviconAppName" class="code-input" placeholder="My App" @input="refreshFaviconSnippet" />
        </div>
        <div class="code-actions">
          <button class="action-btn" @click="loadFaviconSnippet">Preview</button>
          <button class="action-btn" @click="copyFaviconSnippet">Copy to Clipboard</button>
        </div>
        <div v-if="faviconSnippetText" class="code-preview">
          <pre><code>{{ faviconSnippetText }}</code></pre>
        </div>
      </div>
    </div>

    <!-- Pixel Snap Option -->
    <div class="section">
      <label class="checkbox-row">
        <input type="checkbox" v-model="store.pixelSnap" />
        <span class="checkbox-label">Pixel Snap</span>
        <span class="tag">≤32px</span>
      </label>
    </div>

    <!-- 操作 -->
    <div class="section">
      <button
        class="export-all-btn"
        :disabled="store.exporting || store.selectedFormats.length === 0"
        @click="handleExportAll"
      >
        <span v-if="store.exporting" class="spinner"></span>
        <span v-else class="export-arrow"><AppIcon name="download" :size="14" /></span>
        {{ store.exporting ? "导出中…" : "一键导出" }}
      </button>
      <button class="figma-btn" @click="copySvgForFigma">
        <AppIcon name="copy" :size="14" />
        Copy SVG for Figma
      </button>
    </div>

    <!-- 状态消息 -->
    <div v-if="statusMessage" :class="['status-msg', statusType]">
      {{ statusMessage }}
    </div>
  </div>
</template>

<style scoped>
.export-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.section-label {
  padding: 0 12px 2px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--accent);
}

/* Preview grid */
.preview-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  padding: 0 8px;
  align-items: flex-end;
}

.preview-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.preview-frame {
  display: flex;
  align-items: center;
  justify-content: center;
  background: repeating-conic-gradient(var(--bg-tertiary) 0% 25%, var(--bg-secondary) 0% 50%) 50% / 8px 8px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.preview-frame img {
  display: block;
}

.preview-label {
  font-size: 10px;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

/* Format checkboxes */
.format-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 0 8px;
}

.checkbox-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 4px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: background var(--transition-fast) ease;
}

.checkbox-row:hover {
  background: var(--bg-hover);
}

.checkbox-row.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.checkbox-row input[type="checkbox"] {
  accent-color: var(--accent);
  width: 14px;
  height: 14px;
  cursor: pointer;
}

.checkbox-row.disabled input[type="checkbox"] {
  cursor: not-allowed;
}

.checkbox-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.tag {
  font-size: 10px;
  color: var(--text-muted);
  margin-left: auto;
  font-family: "JetBrains Mono", monospace;
}

.tag.upcoming {
  color: var(--warning);
  background: var(--warning-muted);
  padding: 1px 5px;
  border-radius: 4px;
}

/* PNG size chips */
.png-sizes {
  margin-left: 22px;
  padding: 4px 0 4px 0;
}

.png-sizes-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
}

.png-sizes-title {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.png-sizes-actions {
  display: flex;
  gap: 6px;
}

.link-btn {
  background: none;
  border: none;
  color: var(--accent);
  font-size: 10px;
  cursor: pointer;
  padding: 0;
  transition: color var(--transition-fast) ease;
}

.link-btn:hover {
  color: var(--accent-hover);
  text-decoration: underline;
}

.size-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.size-chip {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 42px;
  height: 26px;
  padding: 0 6px;
  font-size: 11px;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast) ease;
  user-select: none;
}

.size-chip:hover {
  border-color: var(--accent);
  color: var(--text-secondary);
}

.size-chip.active {
  background: var(--accent-muted);
  border-color: var(--accent);
  color: var(--accent);
}

.hidden-checkbox {
  display: none;
}

/* Output directory */
.output-dir-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 8px;
}

.output-path {
  flex: 1;
  font-size: 11px;
  color: var(--text-muted);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  padding: 5px 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.action-btn {
  flex-shrink: 0;
  height: 30px;
  padding: 0 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
  white-space: nowrap;
}

.action-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.action-btn:active {
  background: var(--accent);
  color: var(--bg-primary);
}

/* Presets */
.preset-save-row {
  display: flex;
  gap: 6px;
  padding: 0 8px;
}

.preset-input {
  flex: 1;
  height: 28px;
  padding: 0 8px;
  font-size: 11px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
  transition: border-color var(--transition-fast) ease;
}

.preset-input:focus {
  border-color: var(--accent);
}

.preset-input::placeholder {
  color: var(--text-muted);
}

.preset-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 4px 8px 0;
}

.preset-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 6px;
  border-radius: var(--radius-sm);
  transition: background var(--transition-fast) ease;
}

.preset-item:hover {
  background: var(--bg-hover);
}

.preset-name {
  background: none;
  border: none;
  color: var(--accent);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  padding: 0;
  transition: color var(--transition-fast) ease;
}

.preset-name:hover {
  color: var(--accent-hover);
  text-decoration: underline;
}

.preset-info {
  flex: 1;
  font-size: 10px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preset-delete {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 14px;
  cursor: pointer;
  padding: 0 2px;
  line-height: 1;
  transition: color var(--transition-fast) ease;
}

.preset-delete:hover {
  color: var(--danger);
}

/* App icon pack buttons */
.app-icon-buttons {
  display: flex;
  gap: 6px;
  padding: 0 8px;
}

.pack-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 34px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.pack-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.pack-btn:active:not(:disabled) {
  background: var(--accent);
  color: var(--bg-primary);
}

.pack-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pack-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  line-height: 0;
}

.pack-icon svg {
  width: 16px;
  height: 16px;
}

/* Export all button */
.export-all-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: calc(100% - 16px);
  margin: 0 8px;
  height: 38px;
  background: var(--accent);
  border: 1px solid var(--accent);
  border-radius: var(--radius-md);
  color: var(--bg-primary);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

/* Copy for Figma button */
.figma-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: calc(100% - 16px);
  margin: 0 8px;
  height: 32px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition-fast) ease;
}

.figma-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent);
  color: var(--text-primary);
}

.figma-btn:active {
  background: var(--accent);
  color: var(--bg-primary);
}

.export-all-btn:hover:not(:disabled) {
  background: var(--accent-hover);
  border-color: var(--accent-hover);
}

.export-all-btn:active:not(:disabled) {
  background: var(--accent-pressed);
  border-color: var(--accent-pressed);
}

.export-all-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.export-arrow {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 0;
}

.export-arrow svg {
  width: 14px;
  height: 14px;
}

/* Spinner */
.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--accent-muted);
  border-top-color: var(--bg-primary);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* Status message */
.status-msg {
  margin: 0 8px;
  padding: 6px 10px;
  font-size: 11px;
  border-radius: var(--radius-sm);
  line-height: 1.4;
}

.status-msg.success {
  color: var(--success);
  background: var(--success-muted);
  border: 1px solid var(--success-muted);
}

.status-msg.error {
  color: var(--danger);
  background: var(--danger-muted);
  border: 1px solid var(--danger-muted);
}

/* Animation export */
.anim-export-divider {
  height: 1px;
  background: var(--border-color);
  margin: 4px 12px;
}

.anim-export-row {
  padding: 0 8px;
}

.anim-settings {
  margin-left: 22px;
  display: flex;
  gap: 8px;
  padding: 2px 0;
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 4px;
}

.setting-label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.setting-select {
  height: 24px;
  padding: 0 4px;
  font-size: 11px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
  cursor: pointer;
}

.setting-select:focus {
  border-color: var(--accent);
}

/* Code export section */
.code-export-section {
  margin-left: 22px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 6px 0;
}

.code-field {
  display: flex;
  align-items: center;
  gap: 6px;
}

.code-label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.3px;
  min-width: 90px;
  flex-shrink: 0;
}

.code-input {
  flex: 1;
  height: 26px;
  padding: 0 8px;
  font-size: 11px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
  transition: border-color var(--transition-fast) ease;
}

.code-input:focus {
  border-color: var(--accent);
}

.code-select {
  flex: 1;
  height: 26px;
  padding: 0 4px;
  font-size: 11px;
  color: var(--text-primary);
  background: var(--input-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  outline: none;
  cursor: pointer;
}

.code-select:focus {
  border-color: var(--accent);
}

.code-option {
  padding: 2px 0;
}

.code-actions {
  display: flex;
  gap: 4px;
}

.font-format-chips {
  display: flex;
  gap: 4px;
}

.code-preview {
  margin-top: 4px;
  max-height: 240px;
  overflow: auto;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  padding: 8px;
}

.code-preview pre {
  margin: 0;
  font-size: 10px;
  line-height: 1.5;
  color: var(--text-secondary);
  font-family: "JetBrains Mono", monospace;
  white-space: pre-wrap;
  word-break: break-all;
}

/* Variant export */
.variant-export-list {
  margin-left: 22px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 4px 0;
}

.variant-export-row {
  display: flex;
}

.checkbox-row.compact {
  padding: 3px 4px;
}

/* Modal overlay */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  width: 340px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.modal-header {
  padding: 12px 16px;
  font-size: 13px;
  font-weight: 600;
  border-bottom: 1px solid var(--border-color);
}

.modal-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
}

/* Weight variants */
.weight-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.weight-results {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.weight-results-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.weight-result-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.weight-preview {
  width: 56px;
  height: 56px;
}
</style>
