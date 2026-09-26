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

  return { items, loading, fetchAll };
});
