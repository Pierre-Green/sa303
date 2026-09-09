import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "@/lib/api";
import type { Settings } from "@/lib/types";

export const useSettingsStore = defineStore("settings", () => {
  const current = ref<Settings | null>(null);
  const loading = ref(false);

  async function fetch() {
    loading.value = true;
    try {
      current.value = await api.getSettings();
    } finally {
      loading.value = false;
    }
  }

  async function save(settings: Settings) {
    await api.saveSettings(settings);
    current.value = settings;
  }

  return { current, loading, fetch, save };
});
