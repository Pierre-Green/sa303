// Bumpers compatibles avec l'enceinte de référence et le compartiment choisis.

import { computed, watch, type ComputedRef } from "vue";
import { useBumperModelsStore } from "@/stores/bumperModels";
import type { SpeakerModel } from "@/lib/types";
import type { FormState } from "../types";

export function useBumperChoice(form: FormState, referenceModel: ComputedRef<SpeakerModel | null>) {
  const bumpers = useBumperModelsStore();

  const compatible = computed(() =>
    bumpers.items.filter((b) =>
      b.compatibleSpeakers.some(
        (c) =>
          c.speakerModelId === referenceModel.value?.id &&
          (form.compartment === "flown" ? c.flown : c.stacked),
      ),
    ),
  );

  // Si le bumper sélectionné n'est plus compatible (changement d'enceinte ou
  // de compartiment), on ne le garde pas silencieusement sélectionné dans le
  // vide. Un bumper est obligatoire, en vol comme en stack : l'accroche
  // manuelle n'existe pas, donc on retombe automatiquement sur le premier
  // compatible.
  watch(
    [referenceModel, () => form.compartment, compatible],
    () => {
      if (form.bumperModelId && !compatible.value.some((b) => b.id === form.bumperModelId)) {
        form.bumperModelId = null;
      }
      if (!form.bumperModelId && compatible.value.length > 0) {
        form.bumperModelId = compatible.value[0].id;
      }
    },
    { immediate: true },
  );

  return { compatible };
}
