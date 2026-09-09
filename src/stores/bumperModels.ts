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

  async function save(bumperModel: BumperModel) {
    await api.saveBumperModel(bumperModel);
    await fetchAll();
  }

  async function remove(id: string) {
    await api.deleteBumperModel(id);
    await fetchAll();
  }

  return { items, loading, fetchAll, save, remove };
});
