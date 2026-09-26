<script setup lang="ts">
// Fiche complète d'une enceinte, en lecture seule : ce que déclare son JSON,
// ce qu'en dérive sa303-core, et un aperçu monté sous son bumper.
import { computed, toRef } from "vue";
import type { BumperModel, SpeakerModel } from "@/lib/types";
import SpecCard from "../shared/SpecCard.vue";
import EquipmentPreview from "../shared/EquipmentPreview.vue";
import SpeakerDerivedCard from "./SpeakerDerivedCard.vue";
import CrownHolesTable from "./CrownHolesTable.vue";
import SpeakerCompatibility from "./SpeakerCompatibility.vue";
import { useSpeakerSpecs } from "../composables/useSpeakerSpecs";
import { useSpeakerReport } from "../composables/useSpeakerReport";
import { speakerPreview } from "../composables/useEquipmentPreview";

const props = defineProps<{
  speaker: SpeakerModel;
  bumpers: BumperModel[];
  speakerName: (id: string) => string;
}>();

const speaker = toRef(props, "speaker");
const specs = useSpeakerSpecs(speaker);
const { report, error } = useSpeakerReport(computed(() => props.speaker.id));

const preview = computed(() => speakerPreview(props.speaker, props.bumpers));
// Charnières, paire ancrage/verrou et trous de couronne, tels que dérivés par
// sa303-core : ce sont les perçages qu'on veut voir sur la silhouette.
const speakerHoles = computed(() => {
  const r = report.value;
  if (!r) return undefined;
  return [r.ht, r.hb, r.anchorLocal, r.latchLocal, ...r.holes.map((h) => h.position)];
});

</script>

<template>
  <div class="flex flex-col gap-4">
    <p v-if="error" class="text-sm text-destructive">Incohérence géométrique : {{ error }}</p>

    <div class="grid grid-cols-1 gap-4 xl:grid-cols-2">
      <EquipmentPreview part="speaker" :spec="preview" :speaker-holes="speakerHoles" />
      <SpecCard title="Général" :rows="specs.general.value" />
      <SpecCard title="Acoustique" :rows="specs.acoustics.value" />
      <SpeakerCompatibility :speaker="speaker" :bumpers="bumpers" :speaker-name="speakerName" />
      <SpecCard
        title="Charnière avant"
        tip="Trous de charnière déclarés, repère enceinte."
        :rows="specs.hinge.value"
      />
      <SpecCard title="Couronne" :rows="specs.crown.value" />
      <SpecCard
        title="Paire ancrage / verrou"
        tip="Cotés en polaire depuis HT, comme sur le plan."
        :rows="specs.pair.value"
      />
      <SpecCard
        title="Barre arrière"
        tip="Repère barre : abscisse depuis le petit bout (verrou), déport positif vers l'avant."
        :rows="specs.rearBar.value"
      />
      <SpeakerDerivedCard v-if="report" :report="report" />
    </div>

    <CrownHolesTable v-if="report" :holes="report.holes" />
  </div>
</template>
