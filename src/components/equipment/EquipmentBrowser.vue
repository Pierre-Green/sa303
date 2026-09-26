<script setup lang="ts">
// Catalogue d'équipement en lecture seule : enceintes, bumpers et barres de
// déport, tels que livrés dans les JSON. Aucune création ni modification ici —
// un modèle se change dans son fichier.
//
// Le bumper et sa barre de déport partagent une catégorie : la barre n'existe
// jamais indépendamment d'un bumper qui la déclare compatible.

import { ref } from "vue";
import { Separator } from "@/components/ui/separator";
import type { Category } from "./types";
import { useEquipmentCatalog } from "./composables/useEquipmentCatalog";
import ItemList from "./shared/ItemList.vue";
import SpeakerPanel from "./speaker/SpeakerPanel.vue";
import BumperPanel from "./bumper/BumperPanel.vue";
import BumperBarPanel from "./bar/BumperBarPanel.vue";

const category = ref<Category>("enceintes");
const categories: { id: Category; label: string }[] = [
  { id: "enceintes", label: "Enceintes" },
  { id: "bumpers", label: "Bumper & Barres de déport" },
];

const {
  speakers,
  bumpers,
  bumperBars,
  selectedSpeakerId,
  selectedBumperId,
  selectedBumperBarId,
  selectedSpeaker,
  selectedBumper,
  selectedBumperBar,
  speakerName,
  bumperName,
} = useEquipmentCatalog();
</script>

<template>
  <div class="flex h-full">
    <aside class="flex w-55 shrink-0 flex-col gap-1 border-r border-border bg-card p-3">
      <button
        v-for="c in categories"
        :key="c.id"
        class="rounded-md px-3 py-2 text-left text-sm hover:bg-accent"
        :class="category === c.id ? 'bg-accent font-medium text-accent-foreground' : ''"
        @click="category = c.id"
      >
        {{ c.label }}
      </button>
      <p class="mt-auto text-[11px] text-muted-foreground">
        Lecture seule : les modèles se modifient dans leurs fichiers JSON.
      </p>
    </aside>

    <div class="flex-1 overflow-y-auto p-4">
      <div v-if="category === 'enceintes'" class="grid grid-cols-1 gap-4 lg:grid-cols-[240px_1fr]">
        <ItemList v-model="selectedSpeakerId" title="Enceintes" :items="speakers.items" />
        <SpeakerPanel
          v-if="selectedSpeaker"
          :key="selectedSpeaker.id"
          :speaker="selectedSpeaker"
          :bumpers="bumpers.items"
          :speaker-name="speakerName"
        />
      </div>

      <div v-else class="flex flex-col gap-6">
        <section>
          <h3 class="mb-2 text-sm font-semibold text-muted-foreground">Bumpers</h3>
          <div class="grid grid-cols-1 gap-4 lg:grid-cols-[240px_1fr]">
            <ItemList v-model="selectedBumperId" title="Bumpers" :items="bumpers.items" />
            <BumperPanel
              v-if="selectedBumper"
              :bumper="selectedBumper"
              :bumper-bars="bumperBars.items"
              :speaker-name="speakerName"
            />
          </div>
        </section>

        <Separator />

        <section>
          <h3 class="mb-2 text-sm font-semibold text-muted-foreground">Barres de déport</h3>
          <div class="grid grid-cols-1 gap-4 lg:grid-cols-[240px_1fr]">
            <ItemList v-model="selectedBumperBarId" title="Barres" :items="bumperBars.items" />
            <BumperBarPanel
              v-if="selectedBumperBar"
              :bar="selectedBumperBar"
              :bumpers="bumpers.items"
              :bumper-name="bumperName"
            />
          </div>
        </section>
      </div>
    </div>
  </div>
</template>
