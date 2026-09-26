<script setup lang="ts">
// Tout ce que le bumper porte : ses trous et sa barre de déport, les manilles
// retenues et leur charge, la réaction du sol en stack, la paire qui le boulonne sur l'enceinte de référence et
// ses deux pions.

import { computed } from "vue";
import type { Vec2 } from "@/lib/types";
import { useViewer } from "../context";
import { add, localPoints, magnitude, midpoint } from "../geometry";
import { useShapes } from "../composables/useShapes";
import { BUMPER_IDX } from "../composables/useHoverPin";

const { result, colors, px, annotation, referenceDepth, hover } = useViewer();
const { arrow, hole, span, label } = useShapes();

const bumperView = computed(() => result.value.bumperView);

const rigging = computed(() => bumperView.value?.rigging ?? null);

// Réaction du sol en stack. Hors de l'échelle commune des flèches,
// volontairement : elle porte toute la pile, donc elle vaut plusieurs fois le
// plus gros effort de jonction. Elle a sa propre longueur, fixe, et son
// intensité se lit sur son étiquette — pas sur sa taille. En vol, chaque
// manille a sa flèche, dessinée avec les trous.
const supportArrow = computed(() => {
  const bv = bumperView.value;
  const mag = magnitude(bv.supportForceGlobal);
  if (mag <= 0 || bv.rigging) return null;
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

/// Trous réellement percés : ceux du bumper toujours, ceux de la barre quand
/// elle est montée, et la barre elle-même, de patte en trous. Les points retenus
/// portent chacun leur flèche de levage, de longueur fixe comme la manille
/// unique, et leur charge en kN — en alarme au-delà de la CMU.
const riggingShapes = computed(() => {
  const r = rigging.value;
  if (!r) return null;
  const muted = colors.value.mutedForeground;
  const len = annotation(referenceDepth.value * 0.6);
  const linkLen = annotation(referenceDepth.value * 0.35);
  return {
    bumperHoles: r.bumperHolesGlobal.map((p) => hole(p, muted, 2.5)),
    linkHoles: r.bumperLinkHolesGlobal.map((p) => hole(p, muted, 2.5)),
    bar:
      r.barOutlineGlobal.length > 0
        ? {
            points: localPoints(...r.barOutlineGlobal),
            closed: true,
            fill: muted,
            opacity: 0.25,
            stroke: muted,
            strokeWidth: px(1),
          }
        : null,
    // Ce que chaque patte tire sur son trou de liaison du bumper. Longueur fixe,
    // comme la manille : ces efforts portent toute la grappe, l'échelle des
    // jonctions les écraserait. L'intensité se lit sur l'étiquette.
    links: r.barLinkForces.map((f) => {
      const mag = magnitude(f.forceGlobal);
      const to =
        mag > 0
          ? { x: f.pointGlobal.x + (f.forceGlobal.x / mag) * linkLen, y: f.pointGlobal.y + (f.forceGlobal.y / mag) * linkLen }
          : f.pointGlobal;
      return {
        arrow: {
          points: localPoints(f.pointGlobal, to),
          stroke: colors.value.orientation,
          fill: colors.value.orientation,
          strokeWidth: px(2.5),
          pointerLength: px(8),
          pointerWidth: px(8),
        },
        label: label(to, `${(f.forceN / 1000).toFixed(2)} kN @ ${f.angleDeg.toFixed(1)}°`, colors.value.orientation, {
          dx: 6,
          dy: 4,
          size: 11,
        }),
      };
    }),
    barPins: r.barPinsGlobal.map((p) => hole(p, muted, 3)),
    barHoles: r.barHolesGlobal.map((p) => hole(p, muted, 2)),
    points: r.points.map((pt) => {
      const color = pt.overloaded ? colors.value.alarm : colors.value.lift;
      const to = { x: pt.pointGlobal.x, y: pt.pointGlobal.y + len };
      return {
        marker: { ...hole(pt.pointGlobal, color, 6), strokeWidth: px(2.5) },
        arrow: {
          points: localPoints(pt.pointGlobal, to),
          stroke: color,
          fill: color,
          strokeWidth: px(3),
          pointerLength: px(10),
          pointerWidth: px(10),
        },
        label: label(to, `${pt.label} · ${(pt.tensionN / 1000).toFixed(2)} kN`, color, { dx: 6, dy: -6 }),
      };
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

  <!-- Trous déclarés du bumper et de sa barre, et les points retenus. -->
  <template v-if="riggingShapes">
    <!-- La barre fait partie du bumper : même fiche au survol et au clic. -->
    <v-line
      v-if="riggingShapes.bar"
      :config="riggingShapes.bar"
      @click="hover.onClick(BUMPER_IDX)"
      @mouseenter="hover.onEnter(BUMPER_IDX)"
      @mouseleave="hover.onLeave()"
    />
    <v-circle v-for="(h, i) in riggingShapes.linkHoles" :key="'lh-' + i" :config="h" />
    <v-circle v-for="(h, i) in riggingShapes.barPins" :key="'bpin-' + i" :config="h" />
    <v-circle v-for="(h, i) in riggingShapes.barHoles" :key="'bh-' + i" :config="h" />
    <v-circle v-for="(h, i) in riggingShapes.bumperHoles" :key="'mh-' + i" :config="h" />
    <template v-for="(l, i) in riggingShapes.links" :key="'link-' + i">
      <v-arrow :config="l.arrow" />
      <v-text :config="l.label" />
    </template>
    <template v-for="(p, i) in riggingShapes.points" :key="'rp-' + i">
      <v-arrow :config="p.arrow" />
      <v-circle :config="p.marker" />
      <v-text :config="p.label" />
    </template>
  </template>

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
