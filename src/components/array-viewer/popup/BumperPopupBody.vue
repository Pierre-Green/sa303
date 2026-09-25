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
    <PopupRow label="Accroche">
      {{ data.pickupOffsetMm !== null ? `${Math.abs(data.pickupOffsetMm).toFixed(0)} mm` : "—" }}
    </PopupRow>
    <PopupRow v-if="(data.barDeportMm ?? 0) !== 0" label="Dont barre">
      {{ Math.abs(data.barDeportMm!).toFixed(0) }} mm
    </PopupRow>

    <div v-if="data.bumperBarExceeded" class="text-status-alarm">
      Portée barre dépassée : pull-back actif.
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
