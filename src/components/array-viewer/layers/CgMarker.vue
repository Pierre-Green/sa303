<script setup lang="ts">
// Le centre de gravité et la masse totale, tous deux sommés côté Rust : le
// front ne recompose rien, il place le point.

import { computed } from "vue";
import { useViewer } from "../context";
import { useKeyPoints } from "../composables/useKeyPoints";

const { result, colors, px } = useViewer();
const { cgLocal } = useKeyPoints();

const markerConfig = computed(() => ({
  x: cgLocal.value.x,
  y: cgLocal.value.y,
  radius: px(5),
  fill: colors.value.alarm,
}));

const labelConfig = computed(() => ({
  x: cgLocal.value.x + px(10),
  y: cgLocal.value.y - px(6),
  text: `CG ${result.value.totalMassKg.toFixed(0)} kg`,
  fontSize: px(13),
  fill: colors.value.alarm,
  fontStyle: "600",
}));
</script>

<template>
  <v-circle :config="markerConfig" />
  <v-text :config="labelConfig" />
</template>
