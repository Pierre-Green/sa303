// Ce que la fiche affiche, assemblé à partir du seul `ClusterResult` : on
// sélectionne et on renomme, on ne recalcule rien (brief §1).

import { computed } from "vue";
import type { JointResult } from "@/lib/types";
import { useViewer, type ViewerContext } from "../context";
import { useDisplayNumber } from "./useDisplayNumber";
import { BUMPER_IDX } from "./useHoverPin";

/** La paire ancrage/verrou chargée par la barre du bumper, pour l'enceinte qui
 * la porte. */
export interface BumperBarPairLoads {
  anchorN: number;
  anchorAngleDeg: number;
  latchN: number;
  latchAngleDeg: number;
  momentNm: number;
}

export interface PullBackLoads {
  tensionKn: number;
  /** Convention §2 : 180° = verticale vers le haut. */
  angleDeg: number | null;
}

export interface SpeakerPopupData {
  kind: "speaker";
  pinned: boolean;
  number: number;
  modelName: string | null;
  angleDeg: number;
  splay: number | null;
  /** Le joint dont cette enceinte est le flanc porteur. */
  joint: JointResult | null;
  /** Le joint dont elle porte la paire ancrage/verrou. */
  pairJoint: JointResult | null;
  bumperBarPair: BumperBarPairLoads | null;
  pullBack: PullBackLoads | null;
  bottomElevationMm: number;
  /** Distance, en mètres, entre la face avant et le croisement de son axe avec
   * la ligne d'écoute. `null` quand l'axe ne la croise pas. */
  listeningDistanceM: number | null;
}

export interface BumperPopupData {
  kind: "bumper";
  pinned: boolean;
  pickupOffsetMm: number | null;
  barDeportMm: number | null;
  bumperBarExceeded: boolean;
  supportForceN: number;
  supportAngleDeg: number;
  pinPairMomentNm: number;
  pinSpanMm: number;
  orientationForceN: number | null;
  orientationAngleDeg: number | null;
  pivotForceN: number | null;
  pivotAngleDeg: number | null;
  bottomElevationMm: number;
  pickupElevationMm: number | null;
}

export type PopupData = SpeakerPopupData | BumperPopupData;

export function usePopupData(ctx?: ViewerContext) {
  const viewer = useViewer(ctx);
  const { result, compartment, hover, listening } = viewer;
  const displayNumber = useDisplayNumber(viewer);

  /** Splay associé à une enceinte : celui qui la relie à l'enceinte du dessus
   * (repère enceinte supérieure), indépendant du compartiment — kinématique §4. */
  function splayDegOf(idx: number): number | null {
    return idx === 0 ? null : (result.value.joints[idx - 1]?.splayDeg ?? null);
  }

  /** Le joint dont cette enceinte est le flanc porteur, s'il y en a un. */
  function loadedJointOf(idx: number): JointResult | null {
    return result.value.joints.find((j) => j.loadedFlank === idx) ?? null;
  }

  /// La jonction dont CETTE enceinte est le caisson du **bas**, donc celle dont
  /// elle porte la paire ancrage/verrou. La paire de la jonction `j` est toujours
  /// sur l'enceinte `j + 1`, dans les deux compartiments.
  ///
  /// Sans ça, la dernière enceinte d'une grappe suspendue n'affichait aucune
  /// charge : le flanc « chargé » d'une jonction en vol est celui du **haut**,
  /// donc elle n'était jamais trouvée — alors qu'elle porte bien deux goupilles
  /// et que le viewer y dessine deux flèches.
  function pairJointOf(idx: number): JointResult | null {
    return result.value.joints.find((j) => j.jointIndex === idx - 1) ?? null;
  }

  /// La paire ancrage/verrou chargée par la barre du bumper, pour l'enceinte
  /// qui la porte. `null` partout ailleurs, et en stack où il n'y a pas de barre.
  function bumperBarPairLoads(idx: number): BumperBarPairLoads | null {
    const bv = result.value.bumperView;
    if (idx !== 0 || compartment.value !== "flown") return null;
    if (bv.fPairAnchorN === null || bv.fPairLatchN === null) return null;
    return {
      anchorN: bv.fPairAnchorN,
      anchorAngleDeg: bv.fPairAnchorAngleDeg ?? 0,
      latchN: bv.fPairLatchN,
      latchAngleDeg: bv.fPairLatchAngleDeg ?? 0,
      momentNm: bv.pairMomentNm ?? 0,
    };
  }

  /// Le pull-back s'accroche sur l'enceinte du **bas** de la grappe suspendue.
  /// C'est donc dans sa fiche qu'on attend sa tension et sa direction : c'est
  /// elle qui les encaisse.
  function pullBackLoads(idx: number): PullBackLoads | null {
    const last = result.value.speakers.length - 1;
    if (compartment.value !== "flown" || idx !== last) return null;
    if (!result.value.pullBackPointGlobal || result.value.pullBackTensionN <= 0) return null;
    return {
      tensionKn: result.value.pullBackTensionN / 1000,
      angleDeg: result.value.pullBackDirectionAngleDeg,
    };
  }

  const popupData = computed<PopupData | null>(() => {
    const idx = hover.anchor.value?.idx;
    if (idx === undefined) return null;

    if (idx === BUMPER_IDX) {
      const bv = result.value.bumperView;
      if (!bv) return null;
      return {
        kind: "bumper",
        pinned: hover.pinnedIdx.value === BUMPER_IDX,
        pickupOffsetMm: bv.pickupOffsetMm,
        barDeportMm: bv.barDeportMm,
        bumperBarExceeded: bv.bumperBarExceeded,
        supportForceN: bv.supportForceN,
        supportAngleDeg: bv.supportAngleDeg,
        pinPairMomentNm: bv.pinPairMomentNm,
        pinSpanMm: bv.pinSpanMm,
        orientationForceN: bv.orientationForceN,
        orientationAngleDeg: bv.orientationAngleDeg,
        pivotForceN: bv.pivotForceN,
        pivotAngleDeg: bv.pivotAngleDeg,
        bottomElevationMm: result.value.elevation.bumperBottomMm,
        pickupElevationMm: result.value.elevation.pickupMm,
      };
    }

    const speaker = result.value.speakers[idx];
    if (!speaker) return null;
    return {
      kind: "speaker",
      pinned: hover.pinnedIdx.value === idx,
      number: displayNumber(idx),
      modelName: result.value.speakerNames[idx] ?? null,
      angleDeg: (speaker.phi * 180) / Math.PI,
      splay: splayDegOf(idx),
      joint: loadedJointOf(idx),
      pairJoint: pairJointOf(idx),
      // La barre du bumper est boulonnée sur la paire de CETTE enceinte : ses
      // deux goupilles sont les siennes, au même titre que celles d'une
      // jonction. (Les efforts de pion, eux, restent au bumper : ils sont à lui.)
      bumperBarPair: bumperBarPairLoads(idx),
      pullBack: pullBackLoads(idx),
      bottomElevationMm: result.value.elevation.speakerBottomMm[idx],
      listeningDistanceM: listening.rays.value[idx]?.distanceM ?? null,
    };
  });

  return { popupData };
}
