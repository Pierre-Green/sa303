// Assemble l'éditeur : formulaire, résultat live, accroche, chaîne, bumper,
// bibliothèque et export. L'ordre des appels compte — c'est l'ordre dans
// lequel les `watch` sont posés, et le recalcul live doit venir en dernier,
// une fois que les corrections automatiques (chaîne, bumper) sont en place.

import { onMounted } from "vue";
import { useSpeakerModelsStore } from "@/stores/speakerModels";
import { useBumperModelsStore } from "@/stores/bumperModels";
import { useBumperBarModelsStore } from "@/stores/bumperBarModels";
import { useClustersStore } from "@/stores/clusters";
import { useBuiltinsStore } from "@/stores/builtins";
import { clusterFromForm, useClusterForm } from "./useClusterForm";
import { useClusterResult } from "./useClusterResult";
import { useRigging } from "./useRigging";
import { useSpeakerChain } from "./useSpeakerChain";
import { useBumperChoice } from "./useBumperChoice";
import { useClusterLibrary } from "./useClusterLibrary";
import { useAuditExport } from "./useAuditExport";

export function useClusterEditor() {
  const speakers = useSpeakerModelsStore();

  const formApi = useClusterForm(() => speakers.items[0]?.id ?? "");
  const { form } = formApi;
  const result = useClusterResult();
  const library = useClusterLibrary(form, formApi, result, () => rigging.pullBackForced.value);
  const rigging = useRigging(form, result);
  const chain = useSpeakerChain(form, result);
  const bumper = useBumperChoice(form, chain.referenceModel);
  const audit = useAuditExport({
    clusters: library.items,
    currentId: () => (library.isEditingExisting.value ? library.selectedId.value : null),
    error: result.error,
  });

  result.startLiveRecompute(form, () => clusterFromForm(form, rigging.pullBackForced.value));

  onMounted(async () => {
    await Promise.all([
      speakers.fetchAll(),
      useBumperModelsStore().fetchAll(),
      useBumperBarModelsStore().fetchAll(),
      useClustersStore().fetchAll(),
      useBuiltinsStore().fetchOnce(),
    ]);
    library.selectFirstOrNew();
  });

  return { form, result, library, rigging, chain, bumper, audit };
}

export type ClusterEditor = ReturnType<typeof useClusterEditor>;
