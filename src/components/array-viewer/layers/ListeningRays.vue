<script setup lang="ts">
// L'axe acoustique de chaque enceinte, prolongé jusqu'à la ligne d'écoute, avec
// un point au croisement — ou à l'infini quand il ne la croise jamais. C'est la
// longueur du trait jusqu'au croisement que la fiche de l'enceinte chiffre en
// mètres.
//
// Dans le groupe cadré (les traits partent d'un point du modèle et doivent le
// suivre), mais hors du groupe mesuré : leur longueur dépend d'une hauteur
// choisie, pas du matériel, et n'a donc pas à peser sur le cadrage.

import { computed } from "vue";
import { useViewer } from "../context";

const { colors, px, hover, listening } = useViewer();

const raysConfig = computed(() =>
  listening.rays.value.map((ray, i) => {
    const active = hover.isActive(i);
    return {
      key: i,
      line: {
        points: [ray.fromLocal.x, ray.fromLocal.y, ray.toLocal.x, ray.toLocal.y],
        stroke: colors.value.lift,
        strokeWidth: px(active ? 1.8 : 1),
        opacity: active ? 0.95 : 0.45,
        listening: false,
      },
      dot: ray.hitLocal && {
        x: ray.hitLocal.x,
        y: ray.hitLocal.y,
        radius: px(active ? 4 : 2.5),
        fill: colors.value.lift,
        opacity: active ? 1 : 0.6,
        listening: false,
      },
    };
  }),
);
</script>

<template>
  <template v-for="ray in raysConfig" :key="'listen-' + ray.key">
    <v-line :config="ray.line" />
    <v-circle v-if="ray.dot" :config="ray.dot" />
  </template>
</template>
