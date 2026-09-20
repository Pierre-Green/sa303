<script setup lang="ts">
// La tirette : son point d'accroche sur l'enceinte du bas, sa direction de
// traction et sa tension. Vol uniquement.

import { computed } from "vue";
import { useViewer } from "../context";
import { useKeyPoints } from "../composables/useKeyPoints";

const { result, compartment, colors, px } = useViewer();
const { tiePointLocal, tieEndLocal } = useKeyPoints();

const visible = computed(
  () => compartment.value === "flown" && !!tiePointLocal.value && !!tieEndLocal.value,
);

const arrowConfig = computed(() => {
  if (!visible.value) return null;
  const from = tiePointLocal.value!;
  const to = tieEndLocal.value!;
  return {
    points: [from.x, from.y, to.x, to.y],
    stroke: colors.value.lift,
    fill: colors.value.lift,
    strokeWidth: px(2.5),
    dash: [px(7), px(4)],
    pointerLength: px(8),
    pointerWidth: px(8),
  };
});

const pointMarkerConfig = computed(() => {
  if (!visible.value) return null;
  return {
    x: tiePointLocal.value!.x,
    y: tiePointLocal.value!.y,
    radius: px(4),
    fill: colors.value.lift,
  };
});

const labelConfig = computed(() => {
  if (!visible.value) return null;
  return {
    x: tieEndLocal.value!.x,
    y: tieEndLocal.value!.y + px(6),
    text: `tirette ${(result.value.tieTensionN / 1000).toFixed(2)} kN`,
    fontSize: px(12),
    fill: colors.value.lift,
  };
});
</script>

<template>
  <template v-if="arrowConfig">
    <v-arrow :config="arrowConfig" />
    <v-circle :config="pointMarkerConfig!" />
    <v-text :config="labelConfig!" />
  </template>
</template>
