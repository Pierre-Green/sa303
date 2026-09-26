import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "@/lib/api";
import type { BumperModel } from "@/lib/types";

export const useBumperModelsStore = defineStore("bumperModels", () => {
  const items = ref<BumperModel[]>([]);
  const loading = ref(false);

  async function fetchAll() {
    loading.value = true;
    try {
      items.value = await api.listBumperModels();
    } finally {
      loading.value = false;
    }
  }

  return { items, loading, fetchAll };
});
