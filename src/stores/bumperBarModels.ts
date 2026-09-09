import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "@/lib/api";
import type { BumperBarModel } from "@/lib/types";

export const useBumperBarModelsStore = defineStore("bumperBarModels", () => {
  const items = ref<BumperBarModel[]>([]);
  const loading = ref(false);

  async function fetchAll() {
    loading.value = true;
    try {
      items.value = await api.listBumperBarModels();
    } finally {
      loading.value = false;
    }
  }

  async function save(bumperBarModel: BumperBarModel) {
    await api.saveBumperBarModel(bumperBarModel);
    await fetchAll();
  }

  async function remove(id: string) {
    await api.deleteBumperBarModel(id);
    await fetchAll();
  }

  return { items, loading, fetchAll, save, remove };
});
