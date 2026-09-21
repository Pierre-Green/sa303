<script setup lang="ts">
// Consomme un ClusterResult déjà calculé par sa303-core : aucun calcul de
// statique ou de géométrie ici. Les seules exceptions tolérées (brief §1) sont
// la mise à l'échelle pixels <-> modèle (zoom/pan/cadrage) et le passage de
// `phi` (radians, calculé par Rust) à la convention degrés/horaire de Konva.
//
// Ce fichier ne fait qu'assembler : il monte le stage, fournit le contexte
// commun (résultat, palette, échelles, survol) et empile les couches. Chaque
// couche sait dessiner sa part et va chercher ce qu'il lui faut dans le
// contexte — voir `context.ts`.

import { computed, nextTick, ref, watch } from "vue";
import { useElementSize } from "@vueuse/core";
import type Konva from "konva";
import type { ClusterResult, Compartment } from "@/lib/types";

import { provideViewer, type ViewerContext } from "./context";
import type { KonvaNodeRef } from "./types";
import { magnitude } from "./geometry";
import { useScale } from "./composables/useScale";
import { useThemeColors } from "./composables/useThemeColors";
import { useStageView } from "./composables/useStageView";
import { useHoverPin } from "./composables/useHoverPin";
import { usePopupData } from "./composables/usePopupData";

import GroundLine from "./layers/GroundLine.vue";
import GuideDashes from "./layers/GuideDashes.vue";
import SpeakersLayer from "./layers/SpeakersLayer.vue";
import BumperOutline from "./layers/BumperOutline.vue";
import BumperAnnotations from "./layers/BumperAnnotations.vue";
import JointsLayer from "./layers/JointsLayer.vue";
import TieOverlay from "./layers/TieOverlay.vue";
import CgMarker from "./layers/CgMarker.vue";
import ViewerPopup from "./popup/ViewerPopup.vue";
import ElevationReadout from "./overlays/ElevationReadout.vue";
import CursorReadout from "./overlays/CursorReadout.vue";

// Tout vient de `result` : silhouette par enceinte (une grappe est hétérogène)
// et masse totale déjà sommée côté Rust — le front ne recompose rien, il
// dessine (brief §1).
const props = defineProps<{
  result: ClusterResult;
  compartment: Compartment;
  /** Identifiant de la grappe affichée. C'est lui, et non `result`, qui dit
   * qu'on regarde autre chose : `result` est un objet neuf à chaque recalcul,
   * donc le surveiller ferait sauter le cadrage à chaque splay modifié. */
  viewKey?: string | null;
}>();

const containerEl = ref<HTMLDivElement | null>(null);
const stageRef = ref<KonvaNodeRef<Konva.Stage>>(null);
const fitGroupRef = ref<KonvaNodeRef<Konva.Group>>(null);
const measureGroupRef = ref<KonvaNodeRef<Konva.Group>>(null);

const { width, height } = useElementSize(containerEl);
// Repli le temps que le conteneur soit mesuré : un stage de taille nulle ne
// dessine rien, et le cadrage n'aurait pas de place où centrer.
const size = computed(() => ({
  width: width.value || 800,
  height: height.value || 560,
}));

/** Longueur de référence pour les traits et flèches (mm) : l'étendue
 * avant-arrière de l'enceinte du haut. Pure échelle de rendu, pas de la
 * physique. */
const referenceDepth = computed(() => {
  const outline = props.result.speakers[0]?.outline;
  if (!outline) return 700;
  const xs = outline.map((p) => p.x);
  return Math.max(...xs) - Math.min(...xs);
});

// Échelle commune à toutes les flèches. Les efforts de paire y entrent : ils
// dépassent souvent celui de couronne, et les laisser hors de l'échelle les
// ferait sortir du cadre — ou, pire, ferait paraître la couronne dominante.
const maxForceN = computed(() =>
  Math.max(
    1,
    ...props.result.joints.flatMap((j) => [
      magnitude(j.fOrientationGlobal),
      magnitude(j.fPivotGlobal),
      magnitude(j.fAnchorGlobal),
      magnitude(j.fLatchGlobal),
    ]),
  ),
);

const scale = useScale();
const colors = useThemeColors();

const view = useStageView({
  stageRef,
  fitGroupRef,
  measureGroupRef,
  size,
  scale,
  referenceDepth,
  // Altitude 0 en repère de dessin : `elevationOf(y) = y + offsetMm`, donc le
  // sol est à `y = -offsetMm` en repère modèle, soit `+offsetMm` une fois l'axe
  // vertical retourné. Le décalage vient de Rust, on ne fait que l'appliquer.
  groundLocalY: () => props.result.elevation.offsetMm,
});

const hover = useHoverPin({
  pointerPosition: () => stageRef.value?.getNode()?.getPointerPosition() ?? null,
  fallbackPosition: () => ({ x: size.value.width / 2, y: size.value.height / 2 }),
  setPointerCursor: view.setPointerCursor,
});

const viewer: ViewerContext = {
  result: computed(() => props.result),
  compartment: computed(() => props.compartment),
  colors,
  referenceDepth,
  maxForceN,
  dashTopY: view.dashTopY,
  hover,
  px: scale.px,
  annotation: scale.annotation,
};
provideViewer(viewer);

// Passé explicitement : `inject()` ne remonte que les ancêtres, ce composant ne
// peut pas s'injecter le contexte qu'il vient de fournir.
const { popupData } = usePopupData(viewer);

// Sortis du composable pour le template : seules les refs de premier niveau y
// sont déballées automatiquement.
const { groundLineY, cursorPos, viewport, handleWheel, updateCursorPos, clearCursorPos, onDragMove } =
  view;
const { stageScale } = scale;
const popupAnchor = hover.anchor;

const stageConfig = computed(() => ({
  width: size.value.width,
  height: size.value.height,
  draggable: true,
}));

watch(
  () => [props.result, size.value.width, size.value.height],
  () => void nextTick(view.fitContent),
  { deep: true, immediate: true },
);

// Changer de grappe rend la vue précédente sans objet : on repart du cadrage
// initial, fiche comprise — celle d'avant désignait une enceinte qui n'est plus
// celle-là. Réglé sur `viewKey` et non sur `result` : régler une grappe la
// recalcule en continu, et le zoom qu'on vient d'ajuster pour regarder un
// détail ne doit pas se défaire à chaque degré saisi.
watch(
  () => props.viewKey,
  () => {
    hover.close();
    view.resetView();
    void nextTick(view.fitContent);
  },
);

function onStageClick(e: Konva.KonvaEventObject<MouseEvent>) {
  const stage = stageRef.value?.getNode();
  if (stage && e.target === stage) hover.close();
}
</script>

<template>
  <div class="relative h-full w-full">
    <div
      ref="containerEl"
      class="h-full w-full overflow-hidden rounded-md border border-border bg-card"
    >
      <v-stage
        ref="stageRef"
        :config="stageConfig"
        @wheel="handleWheel"
        @click="onStageClick"
        @mousemove="updateCursorPos"
        @dragmove="onDragMove"
        @mouseleave="clearCursorPos"
      >
        <v-layer>
          <!-- Le sol, à l'altitude 0 dans les deux compartiments : en vol il
               est simplement plus bas que la grappe, et c'est justement ce
               qu'on veut voir. -->
          <GroundLine
            :y="groundLineY"
            :viewport="viewport"
            :stage-scale="stageScale"
          />

          <v-group ref="fitGroupRef" :config="{}">
            <GuideDashes />

            <!-- Ce qui dicte le cadrage : le matériel et ses annotations, pas
                 les pointillés ci-dessus. -->
            <v-group ref="measureGroupRef" :config="{}">
              <SpeakersLayer />
              <BumperOutline />
              <BumperAnnotations />
              <TieOverlay />
              <JointsLayer />
              <CgMarker />
            </v-group>
          </v-group>
        </v-layer>
      </v-stage>
    </div>

    <ViewerPopup
      v-if="popupData && popupAnchor"
      :key="popupAnchor.idx"
      :data="popupData"
      :at="popupAnchor"
      :size="size"
      :compartment="compartment"
      @close="hover.close"
    />

    <ElevationReadout :elevation="result.elevation" />
    <CursorReadout
      v-if="cursorPos"
      :pos="cursorPos"
      :offset-mm="result.elevation.offsetMm"
    />
  </div>
</template>
