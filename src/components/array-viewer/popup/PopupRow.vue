<script setup lang="ts">
// Une ligne de fiche : libellé à gauche, valeur à droite. Sans teinte, le
// libellé est estompé ; avec, toute la ligne prend la couleur de la zone
// concernée — la même que celle de sa flèche sur le dessin.

defineProps<{ label: string; tone?: string; valueClass?: string }>();
</script>

<template>
  <div class="flex justify-between" :class="tone">
    <dt :class="tone ? undefined : 'text-muted-foreground'">{{ label }}</dt>
    <dd :class="valueClass"><slot /></dd>
  </div>
</template>
