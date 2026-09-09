// Conventions d'affichage partagées entre l'éditeur et le viewer : purement de
// la présentation, aucun calcul physique (brief §1). Vivre ici plutôt que d'être
// recopié dans chaque composant garantit que l'éditeur et le canvas numérotent
// les enceintes exactement pareil.

import type { Compartment } from "./types";

/** Numéro affiché d'une enceinte (1-indexé) à partir de son index dans la
 * chaîne — les données vont toujours du haut vers le bas.
 *
 * En stack, on numérote depuis le sol : l'enceinte n°1 est celle qu'on pose en
 * premier, celle qui porte le bumper et l'angle de calage. C'est l'ordre de
 * montage réel, pas l'ordre interne de la chaîne. En vol, on numérote depuis
 * l'accroche : n°1 est celle du haut. */
export function speakerDisplayNumber(
  index: number,
  count: number,
  compartment: Compartment,
): number {
  return compartment === "stacked" ? count - index : index + 1;
}
