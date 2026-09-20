<script setup lang="ts">
// Un effort tel qu'il se lit partout dans la fiche : intensité en newtons et
// direction en degrés (convention §2), toutes deux pré-calculées côté Rust.

import PopupRow from "./PopupRow.vue";

defineProps<{ label: string; tone?: string; newtons: number; angleDeg: number }>();
</script>

<template>
  <PopupRow :label="label" :tone="tone">
    {{ newtons.toFixed(0) }} N @ {{ angleDeg.toFixed(1) }}°
  </PopupRow>
</template>
