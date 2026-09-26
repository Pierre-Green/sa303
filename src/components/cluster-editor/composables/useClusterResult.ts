// Prévisualisation live : le résultat est recalculé à partir du formulaire tel
// quel, jamais depuis la version enregistrée sur disque — sinon rien ne bouge
// dans le canvas tant qu'on n'a pas cliqué "Enregistrer" (source de confusion
// vécue : choisir un bumper ne se voyait pas avant la sauvegarde).
//
// L'état est créé d'abord (`useClusterResult`), le recalcul branché en dernier
// (`startLiveRecompute`) : la construction du `Cluster` dépend du pull-back,
// qui lui-même lit ce résultat.

import { computed, ref, watch } from "vue";
import { api } from "@/lib/api";
import type { BumperView, Cluster, ClusterResult } from "@/lib/types";
import type { FormState } from "../types";

export function useClusterResult() {
  const result = ref<ClusterResult | null>(null);
  const error = ref<string | null>(null);

  // Dernier bumperView valide connu : une saisie transitoire invalide (ex. un
  // angle de pull-back en cours de frappe, momentanément hors plage) fait
  // échouer le calcul le temps d'un caractère — sans ce cache, le bloc
  // "accroche calculée" (et son champ d'angle) disparaîtrait à ce moment
  // précis, empêchant de finir de taper la valeur voulue.
  const lastBumperView = ref<BumperView | null>(null);
  const bumperView = computed(() => result.value?.bumperView ?? lastBumperView.value);

  function forgetLastBumperView() {
    lastBumperView.value = null;
  }

  function startLiveRecompute(form: FormState, build: () => Cluster) {
    let token = 0;
    async function recompute() {
      const mine = ++token;
      error.value = null;
      if (form.speakerModelIds.some((id) => !id) || form.splays.length === 0) {
        result.value = null;
        return;
      }
      try {
        const r = await api.computeClusterResult(build());
        if (mine !== token) return; // une saisie plus récente a déjà pris le relais
        result.value = r;
        lastBumperView.value = r.bumperView;
      } catch (e) {
        if (mine === token) {
          result.value = null;
          error.value = String(e);
        }
      }
    }
    watch(form, recompute, { deep: true, immediate: true });
  }

  return { result, error, bumperView, forgetLastBumperView, startLiveRecompute };
}

export type ClusterResultState = ReturnType<typeof useClusterResult>;
