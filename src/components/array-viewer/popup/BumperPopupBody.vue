<script setup lang="ts">
// Fiche du bumper : sa position, ce qu'il reprend en entier (manille ou sol) et
// ce qu'il redescend dans l'enceinte de référence par ses deux pions.

import type { Compartment } from "@/lib/types";
import type { BumperPopupData } from "../composables/usePopupData";
import PopupRow from "./PopupRow.vue";
import ForceRow from "./ForceRow.vue";

defineProps<{ data: BumperPopupData; compartment: Compartment }>();
</script>

<template>
  <dl class="flex flex-col gap-1">
    <PopupRow label="Dessous">{{ data.bottomElevationMm.toFixed(0) }} mm</PopupRow>
    <PopupRow v-if="data.pickupElevationMm !== null" label="Levage">
      {{ data.pickupElevationMm.toFixed(0) }} mm
    </PopupRow>
    <template v-if="data.rigging">
      <PopupRow v-if="data.rigging.barMountIndex !== null" label="Barre">
        {{ data.rigging.barMounts[data.rigging.barMountIndex].label }}
      </PopupRow>
      <PopupRow
        v-for="(p, i) in data.rigging.points"
        :key="i"
        :label="p.label"
        :tone="p.overloaded ? 'text-status-alarm' : 'text-zone-lift'"
      >
        {{ (p.tensionN / 1000).toFixed(2) }} kN
        <span class="text-xs">/ CMU {{ p.wllKg.toFixed(0) }} kg</span>
      </PopupRow>
      <ForceRow
        v-for="(f, i) in data.rigging.barLinkForces"
        :key="'link-' + i"
        :label="`Liaison barre ${i + 1}`"
        tone="text-zone-orientation"
        :newtons="f.forceN"
        :angle-deg="f.angleDeg"
      />
    </template>

    <div v-if="data.pullBackForced" class="text-status-alarm">
      Aucun trou n'approche l'assiette : pull-back obligatoire.
    </div>

    <PopupRow
      :label="compartment === 'flown' ? 'Charge manille' : 'Réaction sol'"
      tone="text-zone-lift"
      class="mt-1"
    >
      {{ (data.supportForceN / 1000).toFixed(2) }} kN @ {{ data.supportAngleDeg.toFixed(1) }}°
    </PopupRow>
    <p class="text-xs text-muted-foreground">
      Charge entière, pas par flanc : une manille n'est pas doublée.
    </p>

    <ForceRow
      v-if="data.pivotForceN !== null"
      label="Pion avant"
      tone="text-zone-pivot"
      :newtons="data.pivotForceN"
      :angle-deg="data.pivotAngleDeg!"
    />
    <ForceRow
      v-if="data.orientationForceN !== null"
      label="Pion arrière"
      tone="text-zone-orientation"
      :newtons="data.orientationForceN"
      :angle-deg="data.orientationAngleDeg!"
    />
    <PopupRow label="M entre pions">
      {{ data.pinPairMomentNm.toFixed(1) }} N·m
      <span class="text-xs">sur {{ data.pinSpanMm.toFixed(0) }} mm</span>
    </PopupRow>
    <p v-if="data.pivotForceN !== null" class="text-xs text-muted-foreground">
      Charges de pion par flanc : elles traversent les flancs, elles sont doublées.
    </p>
  </dl>
</template>
