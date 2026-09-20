<script setup lang="ts">
// Fiche d'une enceinte : ce qu'elle est, et uniquement ce qu'ELLE subit.
//
// Les deux jonctions qui l'encadrent y contribuent : celle du dessous lui prend
// son trou de splay et sa bielle, celle du dessus sa paire ancrage/verrou. Les
// quatre trous sont donc bien les siens — c'est le nom du trou qui compte, pas
// celui de la jonction.

import { computed } from "vue";
import type { SpeakerPopupData } from "../composables/usePopupData";
import PopupRow from "./PopupRow.vue";
import ForceRow from "./ForceRow.vue";

const props = defineProps<{ data: SpeakerPopupData }>();

const hasLoads = computed(
  () => !!(props.data.joint || props.data.pairJoint || props.data.bumperBarPair),
);
</script>

<template>
  <dl class="flex flex-col gap-1">
    <PopupRow v-if="data.modelName" label="Modèle" value-class="font-medium">
      {{ data.modelName }}
    </PopupRow>
    <PopupRow label="Angle absolu">{{ data.angleDeg.toFixed(1) }}°</PopupRow>
    <PopupRow label="Splay">
      {{ data.splay !== null ? `${data.splay}°` : "— (référence)" }}
    </PopupRow>
    <PopupRow label="Bas de caisse">{{ data.bottomElevationMm.toFixed(0) }} mm</PopupRow>

    <div v-if="hasLoads" class="mt-1 text-xs text-muted-foreground">Efforts sur ses perçages</div>

    <template v-if="data.joint">
      <ForceRow
        label="Trou de splay"
        tone="text-zone-orientation"
        :newtons="data.joint.fOrientationN"
        :angle-deg="data.joint.fOrientationAngleDeg"
      />
      <ForceRow
        label="Bielle"
        tone="text-zone-pivot"
        :newtons="data.joint.fPivotN"
        :angle-deg="data.joint.fPivotAngleDeg"
      />
    </template>

    <!-- Tenue par la barre du bumper, sur sa propre paire : mêmes trous, mêmes
         noms que partout ailleurs dans la grappe. -->
    <template v-if="data.bumperBarPair">
      <ForceRow
        label="Ancrage"
        tone="text-zone-anchor"
        :newtons="data.bumperBarPair.anchorN"
        :angle-deg="data.bumperBarPair.anchorAngleDeg"
      />
      <ForceRow
        label="Verrou"
        tone="text-zone-latch"
        :newtons="data.bumperBarPair.latchN"
        :angle-deg="data.bumperBarPair.latchAngleDeg"
      />
      <PopupRow label="M barre bumper">{{ data.bumperBarPair.momentNm.toFixed(1) }} N·m</PopupRow>
    </template>

    <template v-if="data.pairJoint">
      <ForceRow
        label="Ancrage"
        tone="text-zone-anchor"
        :newtons="data.pairJoint.fAnchorN"
        :angle-deg="data.pairJoint.fAnchorAngleDeg"
      />
      <ForceRow
        label="Verrou"
        tone="text-zone-latch"
        :newtons="data.pairJoint.fLatchN"
        :angle-deg="data.pairJoint.fLatchAngleDeg"
      />
      <!-- La barre est encastrée sur la paire de CETTE enceinte : c'est ici que
           son moment atterrit, pas sur celle d'au-dessus. -->
      <PopupRow label="M barre (ancrage)">
        {{ data.pairJoint.barMomentMaxNm.toFixed(1) }} N·m
      </PopupRow>
    </template>

    <!-- La tirette tire sur cette enceinte-ci : sa tension est une charge
         qu'elle subit, au même titre que ses goupilles. -->
    <template v-if="data.tie">
      <div class="mt-1 text-xs text-muted-foreground">Tirette</div>
      <PopupRow label="Tension" tone="text-zone-lift">
        {{ data.tie.tensionKn.toFixed(2) }} kN
      </PopupRow>
      <PopupRow label="Direction">
        {{ data.tie.angleDeg !== null ? `${data.tie.angleDeg.toFixed(1)}°` : "—" }}
      </PopupRow>
    </template>

    <p v-if="!hasLoads && !data.tie" class="mt-1 text-muted-foreground">
      Aucune charge directe (flanc non porteur).
    </p>
  </dl>
</template>
