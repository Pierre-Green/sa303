<script setup lang="ts">
// Fiche d'une barre de déport, en lecture seule.
import { computed } from "vue";
import type { BumperBarModel, BumperModel } from "@/lib/types";
import SpecCard from "../shared/SpecCard.vue";
import HoleTable from "../shared/HoleTable.vue";
import EquipmentPreview from "../shared/EquipmentPreview.vue";
import { bumperBarPreview } from "../composables/useEquipmentPreview";
import type { SpecRow } from "../types";
import { fmt, fmtPoint } from "../format";

const props = defineProps<{
  bar: BumperBarModel;
  bumpers: BumperModel[];
  bumperName: (id: string) => string;
}>();

const general = computed<SpecRow[]>(() => {
  const g = props.bar.geometry;
  return [
    {
      label: "Patte 1",
      value: `${fmtPoint(g.linkPins[0], 1)} mm`,
      tip: "Pattes qui se goupillent dans les trous de liaison du bumper. Les montages possibles s'en déduisent : rien à saisir de plus.",
    },
    { label: "Patte 2", value: `${fmtPoint(g.linkPins[1], 1)} mm` },
    { label: "CMU par point", value: `${fmt(g.wllKg, 0)} kg` },
    {
      label: "Bumpers compatibles",
      value: props.bar.compatibleBumpers.map((c) => props.bumperName(c.bumperModelId)).join(", ") || "aucun",
    },
  ];
});

const preview = computed(() => bumperBarPreview(props.bar, props.bumpers));
</script>

<template>
  <div class="grid grid-cols-1 gap-4 xl:grid-cols-2">
    <EquipmentPreview part="bumper" :spec="preview" />
    <SpecCard title="Général" :rows="general" />
    <HoleTable
      title="Trous de levage"
      tip="Repère barre : origine au centre, X le long de la barre, Y vers le haut, barre dans son sens normal, en mm."
      :holes="bar.geometry.pickupHoles"
    />
  </div>
</template>
