<script setup lang="ts">
// Position du curseur en millimètres modèle (x = arrière, y = haut), plus son
// altitude au-dessus du sol. Le décalage sol est calculé côté Rust à partir de
// la hauteur de bumper déclarée : ici on ne fait que l'ajouter.

import type { Vec2 } from "@/lib/types";

const props = defineProps<{ pos: Vec2; offsetMm: number }>();

const elevation = () => props.pos.y + props.offsetMm;
</script>

<template>
  <div
    class="pointer-events-none absolute bottom-2 right-2 rounded-md border border-border bg-card/90 px-2 py-1 font-mono text-[11px] text-muted-foreground"
  >
    x {{ pos.x.toFixed(0) }} · y {{ pos.y.toFixed(0) }} · sol {{ elevation().toFixed(0) }} mm
  </div>
</template>
