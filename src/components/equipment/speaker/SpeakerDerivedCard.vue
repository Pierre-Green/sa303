<script setup lang="ts">
// Valeurs dérivées par sa303-core à partir de la géométrie déclarée.
import { computed } from "vue";
import type { SpeakerGeometryReport } from "@/lib/types";
import SpecCard from "../shared/SpecCard.vue";
import type { SpecRow } from "../types";
import { fmt, fmtPoint } from "../format";

const props = defineProps<{ report: SpeakerGeometryReport }>();

const rows = computed<SpecRow[]>(() => {
  const r = props.report;
  return [
    {
      label: "demi-angle (ha)",
      value: `${fmt(r.ha, 1)}°`,
      tip: "Moitié du dièdre de l'enceinte. Sert de référence pour toutes les positions angulaires de la zone orientation.",
    },
    {
      label: "HT (charnière av.-haut)",
      value: fmtPoint(r.ht),
      tip: "Trou avant-haut de l'enceinte. Goupille ronde dans trou rond : effort de direction quelconque, moment nul autour de l'axe.",
    },
    {
      label: "HB (charnière av.-bas)",
      value: fmtPoint(r.hb),
      tip: "Symétrique de HT. C'est la goupille haute de la bielle avant ; sa goupille basse est le HT de l'enceinte du dessous.",
    },
    {
      label: "PV0 (goupille basse de bielle, 0°)",
      value: fmtPoint(r.pv0),
      tip: "Trou avant-haut de l'enceinte inférieure, exprimé dans le repère de l'enceinte supérieure, au splay 0. Ce n'est pas un pivot fixe : la bielle s'inclinant de la moitié du splay, ce point décrit un arc. Le tableau des trous donne sa position à chaque cran.",
    },
    {
      label: "Verrou local",
      value: fmtPoint(r.latchLocal),
      tip: "Second point de fixation de la barre arrière dans l'enceinte inférieure. Avec l'ancrage, il encastre la barre sur elle : la barre lui transmet donc un moment en plus d'une force.",
    },
    {
      label: "Coin avant de jonction",
      value: fmtPoint(r.frontEdge),
      tip: "Arête avant-bas de la jonction, celle qui porte sur l'enceinte du dessous au splay 0. C'est depuis elle que se mesure l'écartement entre caissons.",
    },
    {
      label: "Ancrage local",
      value: fmtPoint(r.anchorLocal),
      tip: "Ancrage de la barre orientation, sous la face supérieure. Un signe inversé ici renverserait complètement la direction de l'effort pivot.",
    },
    {
      label: "Entraxe bielle",
      value: `${fmt(r.bielleEntraxe)} mm`,
      tip: "Longueur de la bielle avant, entre ses deux goupilles. Elle est fixe : c'est son inclinaison, la moitié du splay, qui change d'un cran à l'autre.",
    },
  ];
});
</script>

<template>
  <SpecCard title="Valeurs dérivées" :rows="rows" />
</template>
