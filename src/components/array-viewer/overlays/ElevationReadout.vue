<script setup lang="ts">
// Les trois cotes d'ensemble, toujours visibles : ce qu'un rigger vérifie en
// premier sans avoir à survoler quoi que ce soit.

import type { Elevation } from "@/lib/types";

defineProps<{ elevation: Elevation }>();
</script>

<template>
  <div
    class="pointer-events-none absolute left-2 top-2 rounded-md border border-border bg-card/90 px-2 py-1 font-mono text-[11px] text-muted-foreground"
  >
    bumper {{ elevation.bumperBottomMm.toFixed(0) }} mm ·
    bas {{ elevation.lowestPointMm.toFixed(0) }} ·
    haut {{ elevation.highestPointMm.toFixed(0) }}
  </div>
</template>
