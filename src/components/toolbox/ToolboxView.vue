<script setup lang="ts">
// Boîte à outils : un menu d'outils à gauche, l'outil actif à droite. Un seul
// outil pour l'instant, les critères WST.
import WstCalculator from "./wst/WstCalculator.vue";
</script>

<template>
  <div class="flex h-full">
    <aside class="flex w-55 shrink-0 flex-col gap-1 border-r border-border bg-card p-3">
      <button class="rounded-md bg-accent px-3 py-2 text-left text-sm font-medium text-accent-foreground">
        Critères WST
      </button>
    </aside>

    <div class="flex-1 overflow-y-auto p-4">
      <WstCalculator />
    </div>
  </div>
</template>
