// Contexte partagé par toutes les couches du viewer : le résultat à dessiner,
// la palette, les deux conversions d'échelle et l'état de survol. Fourni une
// fois par `ArrayViewer.vue`, injecté par chaque couche — plutôt qu'une dizaine
// de props redescendues à l'identique dans chaque composant.

import { inject, provide, type ComputedRef, type InjectionKey, type Ref } from "vue";
import type { ClusterResult, Compartment } from "@/lib/types";
import type { ThemeColors } from "./types";
import type { Scale } from "./composables/useScale";
import type { HoverPin } from "./composables/useHoverPin";

export interface ViewerContext {
  result: ComputedRef<ClusterResult>;
  compartment: ComputedRef<Compartment>;
  colors: ComputedRef<ThemeColors>;
  /** Étendue avant-arrière de l'enceinte du haut (mm) : longueur de référence
   * des traits et flèches. Pure échelle de rendu, pas de la physique. */
  referenceDepth: ComputedRef<number>;
  /** Échelle commune à toutes les flèches d'effort (N). */
  maxForceN: ComputedRef<number>;
  /** Sommet des pointillés CG/accroche, en coordonnées locales. */
  dashTopY: Ref<number | null>;
  hover: HoverPin;
  px: Scale["px"];
  annotation: Scale["annotation"];
}

const VIEWER_KEY: InjectionKey<ViewerContext> = Symbol("array-viewer");

export function provideViewer(ctx: ViewerContext): void {
  provide(VIEWER_KEY, ctx);
}

/// Contexte d'une couche du viewer.
///
/// `inject()` ne remonte que les ancêtres : `ArrayViewer`, qui *fournit* le
/// contexte, ne peut pas se l'injecter à lui-même. D'où le paramètre — les
/// couches filles appellent sans rien, `ArrayViewer` passe le contexte qu'il
/// vient de construire.
export function useViewer(ctx?: ViewerContext): ViewerContext {
  if (ctx) return ctx;
  const injected = inject(VIEWER_KEY, null);
  if (!injected) throw new Error("useViewer() hors d'un ArrayViewer");
  return injected;
}
