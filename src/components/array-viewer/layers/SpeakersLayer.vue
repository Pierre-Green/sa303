<script setup lang="ts">
// Les silhouettes d'enceintes, chacune à sa position et son angle résolus par
// Rust : on pose le groupe, on trace le contour tel quel.

import type { SpeakerInstance } from "@/lib/types";
import { useViewer } from "../context";
import { toLocal, toLocalRotationDeg } from "../geometry";
import { useDisplayNumber } from "../composables/useDisplayNumber";

const { result, colors, px, hover } = useViewer();
const displayNumber = useDisplayNumber();

function groupConfig(speaker: SpeakerInstance) {
  const p = toLocal(speaker.o);
  return { x: p.x, y: p.y, rotation: toLocalRotationDeg(speaker.phi) };
}

function outlinePointsOf(speaker: SpeakerInstance): number[] {
  return speaker.outline.flatMap((p) => [p.x, -p.y]);
}

function outlineConfig(speaker: SpeakerInstance) {
  return {
    points: outlinePointsOf(speaker),
    closed: true,
    fill: colors.value.muted,
    stroke: colors.value.border,
    strokeWidth: px(1),
  };
}

// Boîte englobante des points déjà dessinés (min/max, pas de trigonométrie) :
// sert à centrer le numéro dans SA silhouette quel que soit le splay.
function numberLabelConfig(idx: number) {
  const pts = outlinePointsOf(result.value.speakers[idx]);
  const xs = pts.filter((_, i) => i % 2 === 0);
  const ys = pts.filter((_, i) => i % 2 === 1);
  const minX = Math.min(...xs);
  const minY = Math.min(...ys);
  return {
    x: minX,
    y: minY,
    width: Math.max(...xs) - minX,
    height: Math.max(...ys) - minY,
    text: String(displayNumber(idx)),
    fontSize: px(18),
    fill: colors.value.mutedForeground,
    fontStyle: "700",
    align: "center",
    verticalAlign: "middle",
    listening: false,
  };
}
</script>

<template>
  <v-group
    v-for="(speaker, i) in result.speakers"
    :key="i"
    :config="groupConfig(speaker)"
    @click="hover.onClick(i)"
    @mouseenter="hover.onEnter(i)"
    @mouseleave="hover.onLeave()"
  >
    <v-line :config="outlineConfig(speaker)" />
    <v-text :config="numberLabelConfig(i)" />
  </v-group>
</template>
