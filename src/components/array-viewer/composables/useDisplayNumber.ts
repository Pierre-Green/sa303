// Numérotation affichée des enceintes : partagée entre la silhouette dessinée
// et la fiche, pour qu'elles ne puissent pas diverger.

import { speakerDisplayNumber } from "@/lib/display";
import { useViewer, type ViewerContext } from "../context";

export function useDisplayNumber(ctx?: ViewerContext): (idx: number) => number {
  const { result, compartment } = useViewer(ctx);
  /** Même numérotation que l'éditeur : en stack on compte depuis le sol. */
  return (idx) => speakerDisplayNumber(idx, result.value.speakers.length, compartment.value);
}
