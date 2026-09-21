<script setup lang="ts">
// La ligne d'écoute : un trait horizontal continu à l'altitude choisie, tracé
// comme le sol — hors du groupe cadré, donc en unités de stage — pour qu'il
// traverse tout le cadre quoi qu'on déplace, et garde son épaisseur à l'écran.
//
// Tracé aussi hors du groupe mesuré : à 1700 mm sous une grappe en vol, il
// dicterait sinon le cadrage à lui tout seul et l'écraserait.

import { computed } from "vue";
import { useViewer } from "../context";

const props = defineProps<{
  y: number | null;
  viewport: { x: number; y: number; width: number; height: number };
  stageScale: number;
  heightMm: number;
}>();

const { colors } = useViewer();

const LINE_WIDTH = 1.5;
const LABEL_SIZE = 11;
const LABEL_MARGIN = 6;

const k = computed(() => Math.max(props.stageScale, 1e-6));

const lineConfig = computed(() => ({
  points: [
    props.viewport.x,
    props.y ?? 0,
    props.viewport.x + props.viewport.width,
    props.y ?? 0,
  ],
  stroke: colors.value.lift,
  strokeWidth: LINE_WIDTH / k.value,
  dash: [10 / k.value, 6 / k.value],
  opacity: 0.85,
  listening: false,
}));

/** Collée au bord gauche du cadre : la ligne est infinie à l'écran, sa cote
 * doit rester lisible sans avoir à chercher où elle commence. */
const labelConfig = computed(() => ({
  x: props.viewport.x + LABEL_MARGIN / k.value,
  y: (props.y ?? 0) - (LABEL_SIZE + LABEL_MARGIN) / k.value,
  text: `écoute ${props.heightMm.toFixed(0)} mm`,
  fontSize: LABEL_SIZE / k.value,
  fill: colors.value.lift,
  listening: false,
}));
</script>

<template>
  <v-line :config="lineConfig" />
  <v-text :config="labelConfig" />
</template>
