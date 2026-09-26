// La ligne d'écoute : une altitude de référence (1700 mm par défaut, hauteur
// d'oreille d'un public debout) et, pour chaque enceinte, le point où son axe
// acoustique la croise.
//
// Exception assumée au brief §1 : ce n'est pas de la statique ni de la
// géométrie de matériel — c'est une annotation de lecture, au même titre que
// l'altitude affichée sous le curseur. L'axe et la position de chaque enceinte
// viennent, eux, entièrement de Rust ; on ne fait que prolonger une demi-droite
// jusqu'à une hauteur choisie à l'écran.

import { computed, ref, type ComputedRef, type Ref } from "vue";
import type { ClusterResult, SpeakerInstance, Vec2 } from "@/lib/types";
import { toLocal } from "../geometry";

export const DEFAULT_LISTENING_HEIGHT_MM = 1700;

/** Longueur du trait quand l'axe ne croise jamais la ligne, mm : assez pour
 * sortir de n'importe quel cadrage, donc vu comme une demi-droite infinie. */
const UNBOUNDED_RAY_MM = 1e7;

export interface ListeningRay {
  /** Départ : le centre de la face avant de l'enceinte, en repère de dessin. */
  fromLocal: Vec2;
  /** Bout du trait, en repère de dessin : le croisement avec la ligne d'écoute
   * s'il existe, sinon un point très lointain sur l'axe. */
  toLocal: Vec2;
  /** Croisement de l'axe avec la ligne d'écoute ; `null` quand il ne la croise
   * jamais (il s'en éloigne, ou il est parfaitement horizontal). */
  hitLocal: Vec2 | null;
  /** Longueur réelle du trajet jusqu'à la ligne, en mètres — la cote lue dans
   * la fiche. `null` sans croisement. */
  distanceM: number | null;
}

export interface ListeningLine {
  /** Altitude de la ligne au-dessus du sol (mm), réglable. */
  heightMm: Ref<number>;
  /** La même altitude en ordonnée du repère de dessin local. */
  localY: ComputedRef<number>;
  /** L'axe de chaque enceinte, toujours tracé : jusqu'à la ligne quand il la
   * croise, à l'infini sinon. */
  rays: ComputedRef<ListeningRay[]>;
}

/** Centre de la face avant : milieu des deux coins avant de la silhouette
 * (indices 0 et 3, à `x = -profondeur/2`, cf. `speaker_outline`), tourné et
 * posé à la position résolue. */
function frontCenter(speaker: SpeakerInstance): Vec2 {
  const [topFront, , , bottomFront] = speaker.outline;
  const local = { x: (topFront.x + bottomFront.x) / 2, y: (topFront.y + bottomFront.y) / 2 };
  const c = Math.cos(speaker.phi);
  const s = Math.sin(speaker.phi);
  return {
    x: speaker.o.x + local.x * c - local.y * s,
    y: speaker.o.y + local.x * s + local.y * c,
  };
}

/** Axe acoustique, repère global : l'enceinte regarde vers `-x` (x = arrière),
 * tourné de son `phi`. Unitaire, donc le paramètre de la demi-droite est
 * directement une distance en mm. */
function axisDirection(speaker: SpeakerInstance): Vec2 {
  return { x: -Math.cos(speaker.phi), y: -Math.sin(speaker.phi) };
}

export function useListeningLine(result: ComputedRef<ClusterResult>): ListeningLine {
  const heightMm = ref(DEFAULT_LISTENING_HEIGHT_MM);

  // `elevationOf(y) = y + offsetMm` : la ligne est donc à `heightMm - offsetMm`
  // en repère modèle, et l'axe vertical est retourné pour le dessin.
  const modelY = computed(() => heightMm.value - result.value.elevation.offsetMm);
  const localY = computed(() => -modelY.value);

  const rays = computed<ListeningRay[]>(() =>
    result.value.speakers.map((speaker) => {
      const from = frontCenter(speaker);
      const dir = axisDirection(speaker);
      const at = (t: number) => ({ x: from.x + dir.x * t, y: from.y + dir.y * t });
      // Axe horizontal, ou qui s'éloigne de la ligne : pas de croisement, mais
      // l'axe reste tracé — il dit toujours où l'enceinte regarde.
      const t = Math.abs(dir.y) < 1e-9 ? -1 : (modelY.value - from.y) / dir.y;
      if (t <= 0) {
        return {
          fromLocal: toLocal(from),
          toLocal: toLocal(at(UNBOUNDED_RAY_MM)),
          hitLocal: null,
          distanceM: null,
        };
      }
      const hit = toLocal(at(t));
      return { fromLocal: toLocal(from), toLocal: hit, hitLocal: hit, distanceM: t / 1000 };
    }),
  );

  return { heightMm, localY, rays };
}
