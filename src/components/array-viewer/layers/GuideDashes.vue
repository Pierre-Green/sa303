<script setup lang="ts">
// Pointillés verticaux du CG et de l'accroche : dessinés hors du groupe mesuré,
// sinon leur longueur (qui dépend du cadrage) fausserait le cadrage lui-même.

import { computed } from "vue";
import { useViewer } from "../context";
import { useKeyPoints } from "../composables/useKeyPoints";

const { compartment, colors, px, dashTopY } = useViewer();
const { cgLocal, pickupLocal } = useKeyPoints();

const cgDashConfig = computed(() => ({
  points: [cgLocal.value.x, cgLocal.value.y, cgLocal.value.x, dashTopY.value ?? cgLocal.value.y],
  stroke: colors.value.alarm,
  strokeWidth: px(1),
  dash: [px(3), px(4)],
  opacity: 0.7,
}));

const pickupDashConfig = computed(() => {
  const p = pickupLocal.value;
  if (compartment.value !== "flown" || !p) return null;
  return {
    points: [p.x, p.y, p.x, dashTopY.value ?? p.y],
    stroke: colors.value.lift,
    strokeWidth: px(1.5),
    dash: [px(6), px(4)],
  };
});
</script>

<template>
  <v-line v-if="pickupDashConfig" :config="pickupDashConfig" />
  <v-line :config="cgDashConfig" />
</template>
