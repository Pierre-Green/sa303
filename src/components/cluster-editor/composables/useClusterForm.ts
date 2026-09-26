// Le formulaire de la grappe éditée : son état, et les deux conversions
// `Cluster` <-> formulaire. Aucune règle de compatibilité ici — voir
// `useSpeakerChain` et `useBumperChoice`.

import { reactive } from "vue";
import type { Cluster } from "@/lib/types";
import type { FormState } from "../types";

/** Angles disponibles pour le cadre de calage de l'enceinte la plus basse d'un
 * stack — positions de calage physiques, pas un réglage continu. */
export const STACK_TILT_OPTIONS = [0, 5, 10, 15, 20, 30, 40];

export function blankForm(defaultSpeakerId: string): FormState {
  return {
    id: crypto.randomUUID(),
    name: "Nouvelle grappe",
    speakerModelIds: [defaultSpeakerId, defaultSpeakerId],
    compartment: "flown",
    splays: [0],
    bumperModelId: null,
    imposedTiltEnabled: false,
    imposedTilt: 0,
    pullBackAngle: null,
    pullBackEnabled: false,
    pullBackTensionKn: null,
    rigging: { support: "auto", points: 1, barMountIndex: null },
    bumperHeight: 0,
  };
}

export function formFromCluster(c: Cluster): FormState {
  return {
    id: c.id,
    name: c.name,
    speakerModelIds: [...c.speakerModelIds],
    compartment: c.compartment,
    splays: c.joints.map((j) => j.splay),
    bumperModelId: c.bumperModelId,
    imposedTiltEnabled: c.imposedTilt != null,
    imposedTilt: c.imposedTilt ?? 0,
    pullBackAngle: c.pullBackAngle,
    pullBackEnabled: c.pullBackEnabled,
    pullBackTensionKn: c.manualPullBackTensionN != null ? c.manualPullBackTensionN / 1000 : null,
    rigging: { ...c.rigging },
    bumperHeight: c.bumperHeight,
  };
}

/** `pullBackForced` vient du dernier résultat : un pull-back obligatoire garde
 * sa tension réglable même sans avoir été coché. */
export function clusterFromForm(form: FormState, pullBackForced: boolean): Cluster {
  return {
    id: form.id,
    name: form.name,
    speakerModelIds: form.speakerModelIds,
    compartment: form.compartment,
    joints: form.splays.map((splay) => ({ splay })),
    // Bumper obligatoire (brief) : `form.bumperModelId` ne devrait jamais
    // être `null` ici (auto-sélectionné dès qu'un bumper compatible existe),
    // mais s'il l'est encore (aucun bumper compatible), on laisse le
    // backend le signaler explicitement plutôt que d'inventer une valeur.
    bumperModelId: form.bumperModelId ?? "",
    imposedTilt:
      form.compartment === "stacked"
        ? form.imposedTilt
        : form.imposedTiltEnabled
          ? form.imposedTilt
          : null,
    pullBackAngle: form.pullBackAngle,
    pullBackEnabled: form.compartment === "flown" && form.pullBackEnabled,
    manualPullBackTensionN:
      (form.pullBackEnabled || pullBackForced) && form.pullBackTensionKn !== null
        ? form.pullBackTensionKn * 1000
        : null,
    rigging: {
      ...form.rigging,
      // Un montage n'a de sens que sur la barre.
      barMountIndex: form.rigging.support === "bar" ? form.rigging.barMountIndex : null,
    },
    bumperHeight: form.bumperHeight,
  };
}

export function useClusterForm(defaultSpeakerId: () => string) {
  const form = reactive<FormState>(blankForm(defaultSpeakerId()));

  function reset() {
    Object.assign(form, blankForm(defaultSpeakerId()));
  }

  function load(c: Cluster) {
    Object.assign(form, formFromCluster(c));
  }

  return { form, reset, load };
}
