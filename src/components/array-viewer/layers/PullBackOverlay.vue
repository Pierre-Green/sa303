<script setup lang="ts">
// Le pull-back : son point d'accroche sur l'enceinte du bas, sa direction de
// traction et sa tension. Vol uniquement.

import { computed } from "vue";
import { useViewer } from "../context";
import { useKeyPoints } from "../composables/useKeyPoints";

const { result, compartment, colors, px } = useViewer();
const { pullBackPointLocal, pullBackEndLocal } = useKeyPoints();

const visible = computed(
  () => compartment.value === "flown" && !!pullBackPointLocal.value && !!pullBackEndLocal.value,
);

const arrowConfig = computed(() => {
  if (!visible.value) return null;
  const from = pullBackPointLocal.value!;
  const to = pullBackEndLocal.value!;
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
    x: pullBackPointLocal.value!.x,
    y: pullBackPointLocal.value!.y,
    radius: px(4),
    fill: colors.value.lift,
  };
});

const labelConfig = computed(() => {
  if (!visible.value) return null;
  return {
    x: pullBackEndLocal.value!.x,
    y: pullBackEndLocal.value!.y + px(6),
    text: `pull-back (compression) ${(result.value.pullBackTensionN / 1000).toFixed(2)} kN`,
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
