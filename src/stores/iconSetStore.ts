import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface SetEntry {
  id: string;
  name: string;
  tags: string[];
  project_path: string;
  thumbnail: string;
}

export interface IconSet {
  id: string;
  name: string;
  description: string;
  entries: SetEntry[];
  created_at: string;
}

export interface IconSetInfo {
  id: string;
  name: string;
  description: string;
  entry_count: number;
  created_at: string;
}

export interface ConsistencyIssue {
  property: string;
  expected: string;
  actual: string;
  element_id: string;
  project_path: string;
}

export interface ConsistencyReport {
  consistent: boolean;
  issues: ConsistencyIssue[];
  summary: string;
}

export const useIconSetStore = defineStore("iconSet", () => {
  const sets = ref<IconSetInfo[]>([]);
  const activeSetId = ref<string | null>(null);
  const activeSet = ref<IconSet | null>(null);
  const searchResults = ref<SetEntry[]>([]);
  const consistencyReport = ref<ConsistencyReport | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const allTags = computed(() => {
    if (!activeSet.value) return [];
    const tagSet = new Set<string>();
    for (const entry of activeSet.value.entries) {
      for (const tag of entry.tags) {
        tagSet.add(tag);
      }
    }
    return Array.from(tagSet).sort();
  });

  async function loadSets() {
    loading.value = true;
    error.value = null;
    try {
      sets.value = await invoke<IconSetInfo[]>("list_icon_sets");
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function createSet(name: string, description?: string) {
    try {
      const info = await invoke<IconSetInfo>("create_icon_set", {
        name,
        description: description ?? "",
      });
      sets.value.unshift(info);
      return info;
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function selectSet(setId: string) {
    activeSetId.value = setId;
    consistencyReport.value = null;
    try {
      activeSet.value = await invoke<IconSet>("get_icon_set", { setId });
    } catch (e) {
      error.value = String(e);
      activeSet.value = null;
    }
  }

  async function addCurrentToSet(name?: string, tags?: string[]) {
    if (!activeSetId.value) return null;
    try {
      const entry = await invoke<SetEntry>("add_to_icon_set", {
        setId: activeSetId.value,
        name: name ?? "",
        tags: tags ?? [],
      });
      // Refresh the active set
      await selectSet(activeSetId.value);
      await loadSets();
      return entry;
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function removeFromSet(entryId: string) {
    if (!activeSetId.value) return;
    try {
      await invoke("remove_from_icon_set", {
        setId: activeSetId.value,
        entryId,
      });
      await selectSet(activeSetId.value);
      await loadSets();
    } catch (e) {
      error.value = String(e);
    }
  }

  async function exportSet(
    format: string = "png",
    sizes: number[] = [16, 32, 64, 128, 256, 512],
    outputDir: string
  ) {
    if (!activeSetId.value) return [];
    try {
      return await invoke<string[]>("export_icon_set", {
        setId: activeSetId.value,
        format,
        sizes,
        outputDir,
      });
    } catch (e) {
      error.value = String(e);
      return [];
    }
  }

  async function checkConsistency() {
    if (!activeSetId.value) return;
    try {
      consistencyReport.value = await invoke<ConsistencyReport>(
        "check_icon_set_consistency",
        { setId: activeSetId.value }
      );
    } catch (e) {
      error.value = String(e);
    }
  }

  async function tagEntry(entryId: string, tags: string[]) {
    if (!activeSetId.value) return;
    try {
      await invoke("tag_icon_entry", {
        setId: activeSetId.value,
        entryId,
        tags,
      });
      await selectSet(activeSetId.value);
    } catch (e) {
      error.value = String(e);
    }
  }

  async function search(query: string, tags?: string[]) {
    try {
      searchResults.value = await invoke<SetEntry[]>("search_icons", {
        query,
        setId: activeSetId.value ?? undefined,
        tags,
      });
    } catch (e) {
      error.value = String(e);
      searchResults.value = [];
    }
  }

  function clearSelection() {
    activeSetId.value = null;
    activeSet.value = null;
    consistencyReport.value = null;
  }

  return {
    sets,
    activeSetId,
    activeSet,
    searchResults,
    consistencyReport,
    loading,
    error,
    allTags,
    loadSets,
    createSet,
    selectSet,
    addCurrentToSet,
    removeFromSet,
    exportSet,
    checkConsistency,
    tagEntry,
    search,
    clearSelection,
  };
});
