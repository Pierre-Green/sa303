<script setup lang="ts">
// Un compartiment (vol ou stack) : bloc A par chemin de charge, bloc B en
// enveloppe par splay.
import type { CompartmentReport } from "@/lib/types";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import LoadCaseTable from "./LoadCaseTable.vue";

defineProps<{
  title: string;
  compartment: CompartmentReport;
}>();
</script>

<template>
  <section>
    <h2 class="mb-2 text-sm font-semibold">{{ title }}</h2>

    <Card class="mb-4">
      <CardHeader>
        <CardTitle class="text-sm">Bloc A — par chemin de charge</CardTitle>
      </CardHeader>
      <CardContent>
        <LoadCaseTable :cases="compartment.blockA" block="a" />
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle class="text-sm">
          Bloc B — enveloppe par splay ({{ compartment.blockB.length }} angles couverts sur
          {{ compartment.blockB.length + compartment.uncoveredSplaysDeg.length }} percés)
        </CardTitle>
      </CardHeader>
      <CardContent>
        <LoadCaseTable :cases="compartment.blockB" block="b" />

        <!-- L'enveloppe ne dimensionne que les angles réellement montés. Sans
             cette mention, un tableau de 7 lignes pour 17 trous percés aurait
             l'air complet. -->
        <p
          v-if="compartment.uncoveredSplaysDeg.length > 0"
          class="mt-3 rounded-md border border-status-warn/40 bg-status-warn/10 p-2 text-xs"
        >
          <span class="font-medium">Trous percés sans cas de charge :</span>
          {{ compartment.uncoveredSplaysDeg.map((s) => `${s}°`).join(", ") }}. Aucune grappe de ce
          compartiment ne les utilise, l'enveloppe ne dit donc rien de ces angles.
        </p>
      </CardContent>
    </Card>
  </section>
</template>
