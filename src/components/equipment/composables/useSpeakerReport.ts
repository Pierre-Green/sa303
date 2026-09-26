// Valeurs dérivées d'une enceinte (charnières, bielle, trous de couronne),
// calculées par sa303-core à chaque changement de sélection.

import { ref, watch, type Ref } from "vue";
import { api } from "@/lib/api";
import type { SpeakerGeometryReport } from "@/lib/types";

export function useSpeakerReport(speakerId: Ref<string | null>) {
  const report = ref<SpeakerGeometryReport | null>(null);
  const error = ref<string | null>(null);

  watch(
    speakerId,
    async (id) => {
      report.value = null;
      error.value = null;
      if (!id) return;
      try {
        const r = await api.getSpeakerGeometryReport(id);
        if (speakerId.value === id) report.value = r;
      } catch (e) {
        if (speakerId.value === id) error.value = String(e);
      }
    },
    { immediate: true },
  );

  return { report, error };
}
