<script setup lang="ts">
// Le cadre de la fiche : positionnement au curseur, titre, fermeture. Le
// contenu dépend de ce qui est survolé et vit dans les deux corps de fiche.

import { computed } from "vue";
import type { Compartment } from "@/lib/types";
import type { PopupData } from "../composables/usePopupData";
import SpeakerPopupBody from "./SpeakerPopupBody.vue";
import BumperPopupBody from "./BumperPopupBody.vue";

const props = defineProps<{
  data: PopupData;
  /** Position du curseur au moment de l'ouverture, en pixels du conteneur. */
  at: { x: number; y: number };
  size: { width: number; height: number };
  compartment: Compartment;
}>();

const emit = defineEmits<{
  close: [];
}>();

// Bornée au conteneur : près d'un bord, la fiche sortirait du cadre et on
// perdrait la moitié des valeurs.
const style = computed(() => ({
  left: `${Math.min(Math.max(props.at.x + 12, 0), props.size.width - 230)}px`,
  top: `${Math.min(Math.max(props.at.y + 12, 0), props.size.height - 170)}px`,
}));

const title = computed(() =>
  props.data.kind === "bumper" ? "Bumper" : `Enceinte ${props.data.number}`,
);
</script>

<template>
  <!-- Tant qu'elle n'est qu'un aperçu de survol, la fiche est transparente au
       pointeur. Posée à 12 px du curseur, elle se trouve pile sur le trajet
       vers l'enceinte suivante : si elle captait la souris, Konva ne recevrait
       plus le `mouseenter` de cette enceinte-là et la fiche resterait figée sur
       les valeurs de la précédente — deux enceintes voisines semblaient alors
       porter exactement la même charge. Épinglée, elle reprend le pointeur :
       c'est tout l'intérêt d'épingler, aller y sélectionner du texte. -->
  <div
    class="absolute z-10 w-55 select-text rounded-md border border-border bg-popover p-3 text-xs text-popover-foreground shadow-lg"
    :class="data.pinned ? 'pointer-events-auto' : 'pointer-events-none'"
    :style="style"
  >
    <div class="mb-1.5 flex items-center justify-between">
      <span class="font-semibold">{{ title }}</span>
      <button class="text-muted-foreground hover:text-foreground" @click="emit('close')">✕</button>
    </div>

    <SpeakerPopupBody v-if="data.kind === 'speaker'" :data="data" />
    <BumperPopupBody v-else :data="data" :compartment="compartment" />

    <p v-if="!data.pinned" class="mt-2 text-[10px] text-muted-foreground">
      Clique pour épingler et sélectionner le texte.
    </p>
  </div>
</template>
