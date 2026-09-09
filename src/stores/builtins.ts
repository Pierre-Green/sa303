import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "@/lib/api";
import type { BuiltinIds } from "@/lib/types";

/**
 * Catalogue livré avec le logiciel. Ces composants sont immuables dans
 * l'application : la persistance refuse de les enregistrer ou de les supprimer,
 * seule une mise à jour les fait bouger. Ce store sert à ne pas proposer des
 * boutons qui ne pourraient qu'échouer.
 *
 * La liste ne change pas en cours d'exécution — elle est figée dans le binaire —
 * donc un seul chargement suffit.
 */
export const useBuiltinsStore = defineStore("builtins", () => {
  const ids = ref<BuiltinIds>({
    speakers: [],
    bumpers: [],
    bumperBars: [],
    clusters: [],
  });
  const loaded = ref(false);

  async function fetchOnce() {
    if (loaded.value) return;
    ids.value = await api.getBuiltinIds();
    loaded.value = true;
  }

  const isSpeaker = (id: string) => ids.value.speakers.includes(id);
  const isBumper = (id: string) => ids.value.bumpers.includes(id);
  const isBumperBar = (id: string) => ids.value.bumperBars.includes(id);
  const isCluster = (id: string) => ids.value.clusters.includes(id);

  return { ids, loaded, fetchOnce, isSpeaker, isBumper, isBumperBar, isCluster };
});
