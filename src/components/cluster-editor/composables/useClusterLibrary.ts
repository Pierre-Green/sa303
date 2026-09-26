// Les grappes enregistrées : sélection, création, duplication, enregistrement
// et suppression. La grappe sélectionnée est chargée dans le formulaire.

import { computed, ref, watch } from "vue";
import { useClustersStore } from "@/stores/clusters";
import { useBuiltinsStore } from "@/stores/builtins";
import type { Cluster } from "@/lib/types";
import { clusterFromForm } from "./useClusterForm";
import type { FormState } from "../types";
import type { ClusterResultState } from "./useClusterResult";

export function useClusterLibrary(
  form: FormState,
  formApi: { reset: () => void; load: (c: Cluster) => void },
  res: ClusterResultState,
  pullBackForced: () => boolean,
) {
  const clusters = useClustersStore();
  const builtins = useBuiltinsStore();

  const selectedId = ref<string | null>(null);
  const saving = ref(false);

  watch(selectedId, (id) => {
    const cluster = clusters.items.find((c) => c.id === id);
    if (cluster) formApi.load(cluster);
    res.forgetLastBumperView();
  });

  // Grappes suspendues d'abord, stacks ensuite — plus lisible qu'un ordre de
  // fichier arbitraire (tri stable : l'ordre relatif au sein d'un groupe est
  // conservé).
  const sorted = computed(() =>
    [...clusters.items].sort((a, b) => {
      if (a.compartment === b.compartment) return 0;
      return a.compartment === "flown" ? -1 : 1;
    }),
  );

  const isEditingExisting = computed(() => clusters.items.some((c) => c.id === form.id));
  /** Grappe livrée avec le logiciel : elle suit les mises à jour, donc elle ne
   * s'enregistre ni ne se supprime ici. `duplicate` est la porte de sortie. */
  const isBuiltin = computed(() => builtins.isCluster(form.id));

  function selectFirstOrNew() {
    if (clusters.items.length > 0) selectedId.value = clusters.items[0].id;
    else create();
  }

  function create() {
    selectedId.value = null;
    formApi.reset();
    res.forgetLastBumperView();
  }

  /** Repart de la configuration affichée sous une nouvelle identité. C'est ce
   * qui rend les grappes livrées utilisables sans les rendre modifiables : on
   * les lit, on en dérive la sienne. Rien n'est enregistré tant que
   * l'utilisateur ne le demande pas. */
  function duplicate() {
    const source = form.name;
    selectedId.value = null;
    form.id = crypto.randomUUID();
    form.name = `${source} (copie)`;
  }

  async function save() {
    saving.value = true;
    res.error.value = null;
    try {
      const cluster = clusterFromForm(form, pullBackForced());
      await clusters.save(cluster);
      selectedId.value = cluster.id;
    } catch (e) {
      res.error.value = String(e);
    } finally {
      saving.value = false;
    }
  }

  async function remove() {
    if (!isEditingExisting.value) return;
    await clusters.remove(form.id);
    selectFirstOrNew();
  }

  return {
    items: computed(() => clusters.items),
    sorted,
    selectedId,
    saving,
    isEditingExisting,
    isBuiltin,
    selectFirstOrNew,
    create,
    duplicate,
    save,
    remove,
  };
}
