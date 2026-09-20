<script setup lang="ts">
// Les jonctions : quatre perçages chargés par jonction, dessinés à leurs
// positions globales déjà résolues côté Rust.

import type { JointResult } from "@/lib/types";
import { useViewer } from "../context";
import { add, midpoint } from "../geometry";
import { useShapes } from "../composables/useShapes";

const { result, colors, hover } = useViewer();
const { arrow, hole, span, ring, label } = useShapes();

/// La paire de la jonction `j` est portée par l'enceinte `j + 1` : c'est elle
/// qu'il faut survoler pour en voir le détail.
function pairDetailed(j: JointResult): boolean {
  return hover.isActive(j.jointIndex + 1);
}

/// Résultante de la paire, au milieu des deux goupilles. La somme des deux
/// efforts : le couple s'y annule, il ne reste que ce que la barre déverse.
function pairResultant(j: JointResult) {
  return arrow(
    midpoint(j.anchorHoleGlobal, j.latchHoleGlobal),
    add(j.fAnchorGlobal, j.fLatchGlobal),
    colors.value.orientation,
  );
}

function pairMidHole(j: JointResult) {
  return hole(midpoint(j.anchorHoleGlobal, j.latchHoleGlobal), colors.value.orientation);
}

function jointLabel(j: JointResult, idx: number) {
  // Une ligne sur deux décalée : deux jonctions voisines rapprochent leurs
  // étiquettes au point de les superposer sur un splay serré.
  return label(j.loadedPivotHoleGlobal, `J${idx + 1}`, colors.value.mutedForeground, {
    dx: 9,
    dy: 9 + (idx % 2) * 13,
  });
}
</script>

<template>
  <template v-for="(j, idx) in result.joints" :key="'joint-' + idx">
    <!-- La paire ancrage/verrou : une seule flèche résultante par défaut, les
         deux détaillées quand l'enceinte qui la porte est survolée ou épinglée.

         Trois flèches à l'arrière d'un même caisson — ancrage, verrou et trou
         de splay — se chevauchent dès qu'une grappe dépasse quelques enceintes.
         La résultante dit ce que la barre déverse dans le caisson ; le détail
         du couple ne se lit de toute façon qu'en regardant une jonction en
         particulier. -->
    <template v-if="pairDetailed(j)">
      <v-arrow :config="arrow(j.anchorHoleGlobal, j.fAnchorGlobal, colors.anchor)" />
      <v-arrow :config="arrow(j.latchHoleGlobal, j.fLatchGlobal, colors.latch)" />
      <!-- L'entraxe, bras du couple : c'est lui qui explique l'écart entre les
           deux flèches. -->
      <v-line :config="span(j.anchorHoleGlobal, j.latchHoleGlobal, colors.orientation)" />
      <v-circle :config="hole(j.anchorHoleGlobal, colors.anchor)" />
      <v-circle :config="hole(j.latchHoleGlobal, colors.latch)" />
    </template>
    <template v-else>
      <v-arrow :config="pairResultant(j)" />
      <v-circle :config="pairMidHole(j)" />
    </template>

    <v-arrow :config="arrow(j.loadedOrientationHoleGlobal, j.fOrientationGlobal, colors.orientation)" />
    <v-arrow :config="arrow(j.loadedPivotHoleGlobal, j.fPivotGlobal, colors.pivot)" />
    <v-circle :config="hole(j.loadedOrientationHoleGlobal, colors.orientation)" />
    <v-circle :config="hole(j.loadedPivotHoleGlobal, colors.pivot)" />
    <v-circle v-if="j.hingeReversed" :config="ring(j.loadedPivotHoleGlobal)" />
    <v-text :config="jointLabel(j, idx)" />
  </template>
</template>
