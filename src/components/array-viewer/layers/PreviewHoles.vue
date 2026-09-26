<script setup lang="ts">
// Mode aperçu : les perçages seuls, sans aucun effort. Une vignette de
// catalogue n'est pas un montage chargé — une flèche y serait une valeur
// inventée. Mêmes positions globales que les autres couches, venues de Rust.
//
// Les trous de l'enceinte sont en repère enceinte : ils sont posés dans un
// groupe placé et tourné comme la silhouette (`SpeakersLayer`), c'est Konva qui
// fait la rotation, pas le front.

import { computed } from "vue";
import type { Vec2 } from "@/lib/types";
import { useViewer } from "../context";
import { toLocal, toLocalRotationDeg } from "../geometry";
import { useShapes } from "../composables/useShapes";

const props = defineProps<{
  /** Équipement regardé : seuls ses perçages sont dessinés. */
  part: "speaker" | "bumper";
  /** Perçages de l'enceinte du haut, repère enceinte. */
  speakerHoles?: Vec2[];
}>();

const { result, colors } = useViewer();
const { hole } = useShapes();

const speakerGroup = computed(() => {
  if (props.part !== "speaker") return null;
  const s = result.value.speakers[0];
  if (!s) return null;
  const p = toLocal(s.o);
  return { x: p.x, y: p.y, rotation: toLocalRotationDeg(s.phi), listening: false };
});

const globalHoles = computed(() => {
  if (props.part !== "bumper") return [];
  const bv = result.value.bumperView;
  const muted = colors.value.mutedForeground;
  const r = bv.rigging;
  const pts: { p: Vec2; r: number }[] = [
    ...(r?.bumperHolesGlobal ?? []).map((p) => ({ p, r: 2.5 })),
    ...(r?.bumperLinkHolesGlobal ?? []).map((p) => ({ p, r: 2.5 })),
    ...(r?.barPinsGlobal ?? []).map((p) => ({ p, r: 3 })),
    ...(r?.barHolesGlobal ?? []).map((p) => ({ p, r: 2 })),
    ...[bv.pivotPointGlobal, bv.orientationPointGlobal, bv.pairAnchorHoleGlobal, bv.pairLatchHoleGlobal]
      .filter((p): p is Vec2 => p !== null)
      .map((p) => ({ p, r: 3 })),
  ];
  return pts.map(({ p, r }) => ({ ...hole(p, muted, r), listening: false }));
});

const barOutline = computed(() => {
  if (props.part !== "bumper") return null;
  const outline = result.value.bumperView.rigging?.barOutlineGlobal ?? [];
  if (outline.length === 0) return null;
  const muted = colors.value.mutedForeground;
  return {
    points: outline.map(toLocal).flatMap((p) => [p.x, p.y]),
    closed: true,
    fill: muted,
    opacity: 0.25,
    stroke: muted,
    listening: false,
  };
});

// Repère enceinte -> repère de dessin du groupe : le même retournement d'axe
// que le contour dans `SpeakersLayer`.
const speakerHoleConfigs = computed(() =>
  (props.speakerHoles ?? []).map((p) => ({
    ...hole({ x: p.x, y: p.y }, colors.value.mutedForeground, 2.5),
    listening: false,
  })),
);
</script>

<template>
  <v-line v-if="barOutline" :config="barOutline" />
  <v-circle v-for="(h, i) in globalHoles" :key="'g-' + i" :config="h" />
  <v-group v-if="speakerGroup" :config="speakerGroup">
    <v-circle v-for="(h, i) in speakerHoleConfigs" :key="'s-' + i" :config="h" />
  </v-group>
</template>
