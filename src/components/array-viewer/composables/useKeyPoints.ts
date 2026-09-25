// Les quelques points remarquables de la grappe, déjà passés en coordonnées
// locales : partagés entre les pointillés (dessinés hors du groupe mesuré) et
// les marqueurs (dessinés dedans), qui doivent tomber exactement au même
// endroit.

import { computed } from "vue";
import { useViewer } from "../context";
import { toLocal } from "../geometry";

export function useKeyPoints() {
  const { result, annotation, referenceDepth } = useViewer();

  const cgLocal = computed(() => toLocal(result.value.cg));

  const pickupLocal = computed(() =>
    result.value.pickupGlobal ? toLocal(result.value.pickupGlobal) : null,
  );

  const pullBackPointLocal = computed(() =>
    result.value.pullBackPointGlobal ? toLocal(result.value.pullBackPointGlobal) : null,
  );

  /** Bout de la flèche de pull-back : une longueur d'annotation dans la direction
   * de traction, pas une dimension du modèle. */
  const pullBackEndLocal = computed(() => {
    const from = result.value.pullBackPointGlobal;
    const dir = result.value.pullBackDirectionGlobal;
    if (!from || !dir) return null;
    const len = annotation(referenceDepth.value);
    return toLocal({ x: from.x + dir.x * len, y: from.y + dir.y * len });
  });

  return { cgLocal, pickupLocal, pullBackPointLocal, pullBackEndLocal };
}
