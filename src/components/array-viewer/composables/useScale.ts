// Les deux échelles qui séparent le modèle de l'écran, et les deux conversions
// qui permettent de dessiner à taille d'écran constante par-dessus.

import { ref, type Ref } from "vue";

export interface Scale {
  /** Échelle appliquée par le cadrage automatique (mm -> px). */
  fitScale: Ref<number>;
  /** Zoom molette du stage, tenu à part de `fitScale` : le premier est choisi
   * par l'utilisateur, le second par le cadrage automatique. */
  stageScale: Ref<number>;
  /** Convertit une taille voulue **en pixels écran** vers les unités du modèle.
   *
   * Divise par les deux échelles. Sans le zoom molette, flèches et étiquettes
   * grossissaient avec le zoom — en entrant dans un détail, les libellés
   * finissaient par couvrir ce qu'ils annotaient. Avec, elles gardent une taille
   * constante à l'écran, comportement attendu d'une annotation. */
  px: (sizeAtScale1: number) => number;
  /** Longueur d'annotation exprimée en unités modèle **à l'échelle de
   * cadrage**, ramenée à une taille d'écran constante sous le zoom.
   *
   * Les flèches d'effort sont des annotations, pas des objets : leur longueur
   * code une intensité, pas une dimension. Les laisser suivre le modèle ferait
   * qu'en zoomant sur une jonction, sa flèche traverserait tout le cadre. */
  annotation: (lengthAtFit: number) => number;
}

const EPS = 1e-6;

export function useScale(): Scale {
  const fitScale = ref(1);
  const stageScale = ref(1);

  return {
    fitScale,
    stageScale,
    px: (sizeAtScale1) =>
      sizeAtScale1 / (Math.max(fitScale.value, EPS) * Math.max(stageScale.value, EPS)),
    annotation: (lengthAtFit) => lengthAtFit / Math.max(stageScale.value, EPS),
  };
}
