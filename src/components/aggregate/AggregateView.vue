<script setup lang="ts">
// Agrégat de toutes les grappes enregistrées, compartiment par compartiment.
// Ce fichier ne fait qu'assembler : le rapport vient de `useAggregateReport`.
import { Button } from "@/components/ui/button";
import { FileIcon } from "@lucide/vue";
import { useAggregateReport } from "./composables/useAggregateReport";
import ImpossibleClusters from "./panels/ImpossibleClusters.vue";
import CompartmentSection from "./panels/CompartmentSection.vue";

const { report, error, exportForShapeOptimizationFem } = useAggregateReport();
</script>

<template>
  <!-- La coquille de l'application est en `overflow-hidden` et donne une
       hauteur fixe à `main` : c'est donc à chaque page de défiler. -->
  <div class="flex h-full flex-col gap-6 overflow-y-auto p-4">
    <p v-if="error" class="text-sm text-destructive">{{ error }}</p>

    <Button variant="outline" class="w-fit gap-2" size="sm" @click="exportForShapeOptimizationFem">
      <FileIcon />
      Exporter le rapport pour l'optimisation FEM
    </Button>

    <template v-if="report">
      <ImpossibleClusters v-if="report.impossibleClusters.length > 0" :clusters="report.impossibleClusters" />
      <CompartmentSection title="Compartiment suspendu" :compartment="report.flown" />
      <CompartmentSection title="Compartiment stack" :compartment="report.stacked" />
    </template>
  </div>
</template>
