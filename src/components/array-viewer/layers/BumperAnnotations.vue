<script setup lang="ts">
// Tout ce que le bumper porte : sa barre de déport, le point d'accroche, la
// charge qu'il reprend, la paire qui le boulonne sur l'enceinte de référence et
// ses deux pions.

import { computed } from "vue";
import type { Vec2 } from "@/lib/types";
import { useViewer } from "../context";
import { add, localPoints, magnitude, midpoint, toLocal } from "../geometry";
import { useShapes } from "../composables/useShapes";
import { BUMPER_IDX } from "../composables/useHoverPin";

const { result, compartment, colors, px, annotation, referenceDepth, hover } = useViewer();
const { arrow, hole, span, label } = useShapes();

const bumperView = computed(() => result.value.bumperView);

const barConfig = computed(() => {
  const bv = bumperView.value;
  if (!bv?.bumperBarStartGlobal || !bv.pickupGlobal) return null;
  return {
    points: localPoints(bv.bumperBarStartGlobal, bv.pickupGlobal),
    stroke: colors.value.lift,
    strokeWidth: px(4),
    lineCap: "round" as const,
  };
});

const pickupMarkerConfig = computed(() => {
  if (compartment.value !== "flown" || !result.value.pickupGlobal) return null;
  const p = toLocal(result.value.pickupGlobal);
  return {
    x: p.x,
    y: p.y,
    radius: px(6),
    stroke: colors.value.lift,
    strokeWidth: px(2.5),
    fill: colors.value.card,
  };
});

// La charge de manille est hors de l'échelle commune des flèches,
// volontairement : elle porte toute la grappe, donc elle vaut plusieurs fois le
// plus gros effort de jonction. La mettre à la même échelle écraserait toutes
// les autres à quelques pixels. Elle a donc sa propre longueur, fixe, et son
// intensité se lit sur son étiquette — pas sur sa taille.
const supportArrow = computed(() => {
  const bv = bumperView.value;
  const mag = magnitude(bv.supportForceGlobal);
  if (mag <= 0) return null;
  const from = bv.supportPointGlobal;
  const len = annotation(referenceDepth.value * 0.9);
  const to = {
    x: from.x + (bv.supportForceGlobal.x / mag) * len,
    y: from.y + (bv.supportForceGlobal.y / mag) * len,
  };
  return {
    arrow: {
      points: localPoints(from, to),
      stroke: colors.value.lift,
      fill: colors.value.lift,
      strokeWidth: px(3),
      pointerLength: px(10),
      pointerWidth: px(10),
    },
    label: label(to, `${(bv.supportForceN / 1000).toFixed(2)} kN`, colors.value.lift, {
      dx: 6,
      dy: -6,
    }),
  };
});

/// La paire ancrage/verrou par laquelle la barre du bumper est boulonnée sur
/// l'enceinte de référence. Dessinée exactement comme la paire d'une jonction —
/// deux flèches à leurs goupilles, l'entraxe qui les relie — parce que c'est la
/// même liaison : cette enceinte est tenue par la même quincaillerie que les
/// autres, ce n'est pas un cas particulier.
const barPair = computed(() => {
  const bv = bumperView.value;
  const an = bv.pairAnchorHoleGlobal;
  const lt = bv.pairLatchHoleGlobal;
  const fa = bv.fPairAnchorGlobal;
  const fl = bv.fPairLatchGlobal;
  if (!an || !lt || !fa || !fl) return null;
  const mid = midpoint(an, lt);
  return {
    // Détaillée quand l'enceinte qui la porte, ou le bumper d'où vient la
    // barre, est sous le curseur ou épinglé : dans les deux cas c'est cette
    // liaison-là qu'on est en train de lire.
    detailed: hover.isActive(0, BUMPER_IDX),
    span: span(an, lt, colors.value.anchor),
    anchor: arrow(an, fa, colors.value.anchor),
    latch: arrow(lt, fl, colors.value.latch),
    anchorHole: hole(an, colors.value.anchor),
    latchHole: hole(lt, colors.value.latch),
    // Au repos, la résultante au milieu de la paire : le couple s'y annule, il
    // ne reste que ce que la barre déverse dans le caisson. Même lecture que
    // pour les paires de jonction, et même raison — trois flèches à l'arrière
    // d'un même caisson se chevauchent.
    resultant: arrow(mid, add(fa, fl), colors.value.orientation),
    midHole: hole(mid, colors.value.orientation),
  };
});

/// Les deux pions du bumper — avant et arrière — avec leurs efforts.
///
/// Leurs points d'application ne sont **pas** sur le rectangle du bumper : le
/// bumper se goupille dans la charnière avant et le trou de splay 0 de
/// l'enceinte de référence, tous deux à l'intérieur du caisson. Le rectangle
/// dessiné n'est qu'un schéma de son encombrement, pas son emprise réelle.
const pins = computed(() => {
  const bv = bumperView.value;
  if (!bv.orientationForceGlobal || !bv.pivotForceGlobal) return null;
  // Perçage **déclaré** du bumper (`BumperModel.pins`), placé côté Rust comme
  // tout le reste. C'est aussi là que la statique se résout : ces deux points
  // sont la seule définition des pions, le viewer n'en reconstruit aucune.
  const frontAt = bv.pivotPointGlobal;
  const rearAt = bv.orientationPointGlobal;
  if (!frontAt || !rearAt) return null;
  return {
    rear: arrow(rearAt, bv.orientationForceGlobal, colors.value.lift),
    front: arrow(frontAt, bv.pivotForceGlobal, colors.value.lift),
    // Plus gros que les marqueurs de jonction : ces deux pions reprennent toute
    // la grappe, pas la charge d'une seule liaison.
    rearHole: hole(rearAt, colors.value.lift, 5),
    frontHole: hole(frontAt, colors.value.lift, 5),
    // Étiquettes seulement quand le bumper est sous le curseur : nommer les
    // pions en permanence rajouterait du texte là où on vient d'en enlever.
    // Ils sont à lui, et c'est sur lui que leurs valeurs sont affichées.
    labels: hover.isActive(BUMPER_IDX)
      ? [pinLabel(frontAt, "pion avant"), pinLabel(rearAt, "pion arrière")]
      : [],
  };
});

function pinLabel(p: Vec2, text: string) {
  return label(p, text, colors.value.lift, { dx: 7, dy: -14, size: 11 });
}
</script>

<template>
  <v-line v-if="barConfig" :config="barConfig" />
  <v-circle v-if="pickupMarkerConfig" :config="pickupMarkerConfig" />

  <!-- Charge reprise par le bumper : la manille en vol, la réaction du sol en
       stack. Toute la grappe pend dessus, c'est donc le chiffre qui dit le
       calibre du point d'accroche. -->
  <template v-if="supportArrow">
    <v-arrow :config="supportArrow.arrow" />
    <v-text :config="supportArrow.label" />
  </template>

  <!-- La paire qui boulonne la barre sur l'enceinte de référence : ses deux
       goupilles à elle, comme sur toute autre enceinte de la grappe — donc lue
       de la même façon, une seule résultante au repos et le détail au survol. -->
  <template v-if="barPair">
    <template v-if="barPair.detailed">
      <v-arrow :config="barPair.anchor" />
      <v-arrow :config="barPair.latch" />
      <v-line :config="barPair.span" />
      <v-circle :config="barPair.anchorHole" />
      <v-circle :config="barPair.latchHole" />
    </template>
    <template v-else>
      <v-arrow :config="barPair.resultant" />
      <v-circle :config="barPair.midHole" />
    </template>
  </template>

  <!-- Les deux pions, portés par la face du bumper qui les tient : c'est le
       bumper qui charge, pas l'enceinte de référence. -->
  <template v-if="pins">
    <v-arrow :config="pins.front" />
    <v-arrow :config="pins.rear" />
    <v-circle :config="pins.frontHole" />
    <v-circle :config="pins.rearHole" />
    <v-text v-for="(l, i) in pins.labels" :key="'pin-' + i" :config="l" />
  </template>
</template>
