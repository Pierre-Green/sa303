import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "@/lib/api";
import type { Cluster } from "@/lib/types";

export const useClustersStore = defineStore("clusters", () => {
  const items = ref<Cluster[]>([]);
  const loading = ref(false);

  async function fetchAll() {
    loading.value = true;
    try {
      items.value = await api.listClusters();
    } finally {
      loading.value = false;
    }
  }

  async function save(cluster: Cluster) {
    await api.saveCluster(cluster);
    await fetchAll();
  }

  async function remove(id: string) {
    await api.deleteCluster(id);
    await fetchAll();
  }

  return { items, loading, fetchAll, save, remove };
});
