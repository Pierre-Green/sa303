// Types internes à l'éditeur de grappe : l'état du formulaire et la forme des
// lignes de la chaîne d'enceintes. Rien de métier au-delà de ce que le
// formulaire manipule avant d'être converti en `Cluster`.

import type { Compartment, RiggingRequest } from "@/lib/types";

export interface FormState {
  id: string;
  name: string;
  /** Une enceinte par position, du haut vers le bas : `splays.length + 1`
   * entrées. Chaque jonction doit être déclarée compatible côté modèle. */
  speakerModelIds: string[];
  compartment: Compartment;
  splays: number[];
  /** `null` tant qu'aucun bumper compatible n'est encore sélectionné. Un
   * bumper est obligatoire, en vol comme en stack : `buildCluster` ne doit
   * jamais partir en enregistrement avec `null`. */
  bumperModelId: string | null;
  imposedTiltEnabled: boolean;
  imposedTilt: number;
  /** `null` tant que l'utilisateur n'a pas choisi sa propre direction de
   * pull-back : le solveur suggère alors la verticale (180°, convention §2),
   * ou la borne la plus proche dans 180° ± tolérance. */
  pullBackAngle: number | null;
  /** Pull-back activé à la main, pour répartir la charge quand les points
   * d'accroche sont faibles. Ignoré quand il est de toute façon obligatoire. */
  pullBackEnabled: boolean;
  /** Tension voulue dans le pull-back, kN (même base que les efforts
   * affichés, poids × k_dyn). `null` : celle qui tient exactement l'assiette. */
  pullBackTensionKn: number | null;
  /** Vol : famille d'accroche et nombre de points — jamais le trou, que le
   * solveur choisit. */
  rigging: RiggingRequest;
  /** Altitude du dessous du bumper, mm. Situe la grappe dans l'espace sans
   * rien changer aux efforts. */
  bumperHeight: number;
}

/** Une ligne par enceinte (pas par joint) : l'enceinte de référence — celle du
 * haut pour une grappe, du bas pour un stack — n'a pas de splay propre, elle a
 * son inclinaison propre à la place (brief §4 : φ_initial est ancré sur cette
 * enceinte-là). */
export type SpeakerRowKind = "flown-reference" | "stack-reference" | "splay";

export interface SpeakerRow {
  label: string;
  kind: SpeakerRowKind;
  /** Index dans `form.speakerModelIds` (chaîne haut → bas). */
  dataIndex: number;
  splayIndex: number | null;
}
