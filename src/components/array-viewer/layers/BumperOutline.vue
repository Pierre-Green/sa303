<script setup lang="ts">
// Le bumper est déjà en repère global côté Rust (attaché à l'enceinte de
// référence, haut en vol / bas en stack) : ni rotation ni choix de côté ici.
//
// Survolable dans les deux compartiments : il portait déjà les efforts de
// jonction en vol, il porte la charge reprise en stack — il n'est décoratif
// nulle part.

import { computed } from "vue";
import { useViewer } from "../context";
import { toLocal } from "../geometry";
import { BUMPER_IDX } from "../composables/useHoverPin";

const props = defineProps<{
  /** Liaisons pleines (aperçu du bumper seul) ou en simple contour (vue de
   * grappe, où elles passent devant l'enceinte et ne doivent pas la masquer). */
  filledBars?: boolean;
}>();

const { result, colors, px, hover } = useViewer();

/// Bielle avant et barre arrière, telles que placées par Rust.
const barConfigs = computed(() => {
  const bv = result.value.bumperView;
  if (!bv) return [];
  return [bv.frontBarOutlineGlobal, bv.rearBarOutlineGlobal].flatMap((outline) =>
    outline
      ? [
          {
            points: outline.map(toLocal).flatMap((p) => [p.x, p.y]),
            closed: true,
            fill: props.filledBars ? colors.value.muted : undefined,
            stroke: props.filledBars ? colors.value.border : colors.value.mutedForeground,
            strokeWidth: px(1),
            listening: false,
          },
        ]
      : [],
  );
});

const outlineConfig = computed(() => {
  const bv = result.value.bumperView;
  if (!bv) return null;
  return {
    points: bv.outlineGlobal.map(toLocal).flatMap((p) => [p.x, p.y]),
    closed: true,
    fill: colors.value.muted,
    stroke: colors.value.border,
    strokeWidth: px(1),
  };
});
</script>

<template>
  <v-line v-for="(bar, i) in barConfigs" :key="'bar-' + i" :config="bar" />
  <v-line
    v-if="outlineConfig"
    :config="outlineConfig"
    @click="hover.onClick(BUMPER_IDX)"
    @mouseenter="hover.onEnter(BUMPER_IDX)"
    @mouseleave="hover.onLeave()"
  />
</template>
