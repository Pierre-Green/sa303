<script setup lang="ts">
// Le sol, à l'altitude 0 : un trait plein épais et, sous lui, une bande
// hachurée — la convention de dessin technique pour « au-delà, c'est de la
// matière ». Sans elle, un trait seul se lit comme une cote de plus au milieu
// des pointillés CG et accroche.
//
// Toujours à l'altitude 0, jamais au bas du matériel : en stack la grappe y
// pose, en vol elle pend au-dessus, et c'est cet écart qu'on vient lire.
//
// Tracé hors du groupe cadré, donc en unités de stage : seul le zoom molette
// l'affecte, pas le cadrage. D'où la division par `stageScale` et non `px`, qui
// compenserait aussi un cadrage qui ne s'applique pas ici.

import { computed } from "vue";
import { useViewer } from "../context";

const props = defineProps<{
  y: number | null;
  /** Portion visible du repère de stage : les hachures doivent la couvrir en
   * entier, sinon elles s'arrêtent net dès qu'on déplace la vue. */
  viewport: { x: number; y: number; width: number; height: number };
  stageScale: number;
}>();

const { colors } = useViewer();

/** Tailles voulues à l'écran, en pixels. */
const LINE_WIDTH = 3;
const BAND_HEIGHT = 16;
const HATCH_SPACING = 9;
const HATCH_WIDTH = 1.5;
/** Garde-fou : très dézoomé, la bande couvre une largeur de modèle énorme et
 * les hachures deviennent un aplat coûteux à dessiner pour rien. */
const MAX_HATCHES = 400;

const k = computed(() => Math.max(props.stageScale, 1e-6));
const bandHeight = computed(() => BAND_HEIGHT / k.value);

/** Étendue à couvrir : le cadre visible, débordé d'une bande de chaque côté
 * pour que les diagonales entrent et sortent proprement du clip. */
const extent = computed(() => ({
  left: props.viewport.x - bandHeight.value,
  right: props.viewport.x + props.viewport.width + bandHeight.value,
}));

const lineConfig = computed(() => ({
  points: [extent.value.left, props.y ?? 0, extent.value.right, props.y ?? 0],
  stroke: colors.value.mutedForeground,
  strokeWidth: LINE_WIDTH / k.value,
  lineCap: "round" as const,
}));

/** Rectangle qui borne les hachures : elles sont tracées en diagonale et
 * dépasseraient de la bande sans lui. */
const clipConfig = computed(() => ({
  // `clip*` et non `x`/`y`/`width`/`height` : on découpe le groupe, on ne le
  // déplace pas — les hachures sont déjà posées en coordonnées de stage.
  clipX: extent.value.left,
  clipY: props.y ?? 0,
  clipWidth: extent.value.right - extent.value.left,
  clipHeight: bandHeight.value,
  listening: false,
}));

const hatchesConfig = computed(() => {
  const y = props.y ?? 0;
  const band = bandHeight.value;
  const width = extent.value.right - extent.value.left;
  const spacing = Math.max(HATCH_SPACING / k.value, width / MAX_HATCHES);

  const lines = [];
  for (let x = extent.value.left; x < extent.value.right + band; x += spacing) {
    // Diagonale à 45° : elle part du bas de la bande et remonte vers le trait.
    lines.push({
      points: [x, y + band, x + band, y],
      stroke: colors.value.mutedForeground,
      strokeWidth: HATCH_WIDTH / k.value,
      opacity: 0.55,
    });
  }
  return lines;
});
</script>

<template>
  <v-group :config="clipConfig">
    <v-line v-for="(h, i) in hatchesConfig" :key="'hatch-' + i" :config="h" />
  </v-group>
  <v-line :config="lineConfig" />
</template>
