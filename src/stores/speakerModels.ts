import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "@/lib/api";
import type { SpeakerModel } from "@/lib/types";

export const useSpeakerModelsStore = defineStore("speakerModels", () => {
  const items = ref<SpeakerModel[]>([]);
  const loading = ref(false);

  async function fetchAll() {
    loading.value = true;
    try {
      items.value = await api.listSpeakerModels();
    } finally {
      loading.value = false;
    }
  }

  return { items, loading, fetchAll };
});
