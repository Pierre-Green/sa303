<script setup lang="ts">
// Calculateur WST : saisies à gauche, verdicts à droite. Ce fichier ne fait
// qu'assembler.
//
// La nature du guide décide des critères applicables : le critère 5 ne juge
// que des fronts *plans* anglés. Un guide qui rayonne déjà un secteur juxtapose
// le sien à celui de son voisin, donc le backend ne renvoie alors pas de
// critère 5 du tout et on affiche à la place les verdicts de directivité.
import { useWstCalculator } from "./composables/useWstCalculator";
import { provideWst } from "./context";
import WstDocuments from "./panels/WstDocuments.vue";
import SpeakerInputs from "./panels/SpeakerInputs.vue";
import AnalysisInputs from "./panels/AnalysisInputs.vue";
import Criterion1Card from "./panels/Criterion1Card.vue";
import Criterion5Cards from "./panels/Criterion5Cards.vue";
import CurvedGuideCard from "./panels/CurvedGuideCard.vue";
import Criterion2Card from "./panels/Criterion2Card.vue";
import NearFieldCard from "./panels/NearFieldCard.vue";
import CurvatureCard from "./panels/CurvatureCard.vue";
import CcaCard from "./panels/CcaCard.vue";

const wst = useWstCalculator();
provideWst(wst);
const { report, error, criterion5, curvedGuide, distances, referenceSplayDeg } = wst;
</script>

<template>
  <div class="mb-4">
    <h2 class="text-sm font-semibold">Critères WST — angle entre sources</h2>
    <p class="mt-1 text-xs text-muted-foreground">
      Urban, Heil &amp; Bauman, <em>Wavefront Sculpture Technology</em> (AES 5488). Le papier ne traite que des
      sources à front plan : c'est le choix « front rayonné » qui décide des critères réellement applicables.
    </p>
    <WstDocuments />
  </div>

  <div class="grid grid-cols-1 gap-4 xl:grid-cols-[320px_1fr]">
    <!-- Deux blocs distincts : ce qui décrit l'enceinte, et ce qui décrit
         l'analyse. Les premiers viennent du modèle acoustique dès qu'une
         enceinte est sélectionnée ; les seconds restent toujours libres. -->
    <div class="flex flex-col gap-4">
      <SpeakerInputs />
      <AnalysisInputs />
    </div>

    <div v-if="error" class="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
      {{ error }}
    </div>

    <div v-else-if="report" class="flex flex-col gap-4">
      <Criterion1Card :report="report" />
      <Criterion5Cards v-if="criterion5" :report="report" :criterion5="criterion5" :distances="distances" />
      <CurvedGuideCard v-if="curvedGuide" :curved-guide="curvedGuide" :reference-splay-deg="referenceSplayDeg" />
      <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <Criterion2Card :report="report" />
        <NearFieldCard :report="report" />
        <CurvatureCard :report="report" />
        <CcaCard :report="report" />
      </div>
    </div>
  </div>
</template>
