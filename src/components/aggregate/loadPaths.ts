// Lecture des cinq chemins de charge d'un cas. Aucun calcul : le pire taux
// (`utilizationWorst`) vient déjà de Rust, on ne fait que nommer le chemin
// qui le porte.

import type { LoadCaseReport } from "@/lib/types";

export type LoadPath = "couronne" | "ancrage" | "verrou" | "bielle" | "flexion de barre";

/// L'ordre n'a pas d'importance : c'est le maximum qui gouverne.
function paths(c: LoadCaseReport): [LoadPath, number][] {
  return [
    ["couronne", c.utilizationOrientation],
    ["ancrage", c.utilizationAnchor],
    ["verrou", c.utilizationLatch],
    ["bielle", c.utilizationPivot],
    ["flexion de barre", c.utilizationBar],
  ];
}

export function governingPath(c: LoadCaseReport): LoadPath {
  return paths(c).reduce((a, b) => (b[1] > a[1] ? b : a))[0];
}

export function utilizationVariant(u: number): "default" | "secondary" | "destructive" {
  if (u > 1) return "destructive";
  if (u > 0.8) return "secondary";
  return "default";
}
