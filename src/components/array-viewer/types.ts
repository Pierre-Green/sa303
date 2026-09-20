// Types internes au viewer : rien de métier ici, uniquement de quoi typer les
// références Konva et les couleurs relues dans le thème.

import type Konva from "konva";

/** `vue-konva` expose le nœud Konva derrière `getNode()`, jamais directement. */
export type KonvaNodeRef<T extends Konva.Node> = { getNode: () => T } | null;

/** Palette du dessin, relue dans les variables CSS du thème. */
export interface ThemeColors {
  /** Les trois perçages de la zone orientation ont chacun leur teinte : ils
   * sont sur des cercles différents et ne voient pas la même charge, les
   * confondre à l'écran revenait à les confondre tout court. */
  orientation: string;
  anchor: string;
  latch: string;
  pivot: string;
  lift: string;
  alarm: string;
  border: string;
  muted: string;
  mutedForeground: string;
  card: string;
}
