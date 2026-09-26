// Vignette d'un équipement : une grappe d'exemple minimale (une enceinte, un
// bumper) calculée par le solveur comme n'importe quelle grappe, puis dessinée
// par l'ArrayViewer. Le front ne dessine rien lui-même : il choisit seulement
// quel montage montrer, parmi les compatibilités déclarées dans les JSON.

import { ref, watch } from "vue";
import { api } from "@/lib/api";
import type {
  BumperBarModel,
  BumperModel,
  Cluster,
  ClusterResult,
  Compartment,
  RiggingSupport,
  SpeakerModel,
} from "@/lib/types";

/** Montage d'exemple, ou la raison pour laquelle il n'y en a pas. */
export type PreviewSpec = { cluster: Cluster } | { unavailable: string };

function exampleCluster(
  speakerModelId: string,
  bumperModelId: string,
  compartment: Compartment,
  support: RiggingSupport,
): Cluster {
  return {
    id: `preview-${speakerModelId}-${bumperModelId}-${support}`,
    name: "Aperçu",
    speakerModelIds: [speakerModelId],
    compartment,
    joints: [],
    imposedTilt: null,
    pullBackAngle: null,
    pullBackEnabled: false,
    manualPullBackTensionN: null,
    rigging: { support, points: 1, barMountIndex: null },
    bumperModelId,
    bumperHeight: 0,
  };
}

/** Enceinte seule sous le premier bumper qui l'accepte, en vol de préférence. */
export function speakerPreview(speaker: SpeakerModel, bumpers: BumperModel[]): PreviewSpec {
  for (const compartment of ["flown", "stacked"] as const) {
    const bumper = bumpers.find((b) =>
      b.compatibleSpeakers.some((c) => c.speakerModelId === speaker.id && c[compartment]),
    );
    if (bumper) return { cluster: exampleCluster(speaker.id, bumper.id, compartment, "bumper") };
  }
  return { unavailable: "Aucun bumper compatible avec cette enceinte." };
}

/** Bumper en vol sous la première enceinte qu'il accepte, accroche directe. */
export function bumperPreview(bumper: BumperModel): PreviewSpec {
  const flown = bumper.compatibleSpeakers.find((c) => c.flown);
  if (flown) return { cluster: exampleCluster(flown.speakerModelId, bumper.id, "flown", "bumper") };
  const stacked = bumper.compatibleSpeakers.find((c) => c.stacked);
  if (stacked) {
    return { cluster: exampleCluster(stacked.speakerModelId, bumper.id, "stacked", "bumper") };
  }
  return { unavailable: "Ce bumper ne déclare aucune enceinte compatible." };
}

/** Barre montée sur le premier bumper compatible, en vol. Le solveur retient
 * lui-même la barre compatible avec ce bumper. */
export function bumperBarPreview(bar: BumperBarModel, bumpers: BumperModel[]): PreviewSpec {
  for (const { bumperModelId } of bar.compatibleBumpers) {
    const bumper = bumpers.find((b) => b.id === bumperModelId);
    const flown = bumper?.compatibleSpeakers.find((c) => c.flown);
    if (bumper && flown) {
      return { cluster: exampleCluster(flown.speakerModelId, bumper.id, "flown", "bar") };
    }
  }
  return { unavailable: "Aucun bumper compatible ne se monte en vol." };
}

export function useEquipmentPreview(spec: () => PreviewSpec | null) {
  const result = ref<ClusterResult | null>(null);
  const cluster = ref<Cluster | null>(null);
  const message = ref<string | null>(null);
  // Un calcul plus ancien qui répond après le suivant ne doit pas l'écraser.
  let seq = 0;

  watch(
    spec,
    async (s) => {
      const mine = ++seq;
      result.value = null;
      cluster.value = null;
      message.value = null;
      if (!s) return;
      if ("unavailable" in s) {
        message.value = s.unavailable;
        return;
      }
      try {
        const r = await api.computeClusterResult(s.cluster);
        if (mine !== seq) return;
        cluster.value = s.cluster;
        result.value = r;
      } catch (e) {
        if (mine !== seq) return;
        message.value = `Aperçu impossible : ${String(e)}`;
      }
    },
    { immediate: true },
  );

  return { result, cluster, message };
}
