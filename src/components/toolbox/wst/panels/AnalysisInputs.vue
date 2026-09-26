<script setup lang="ts">
// Paramètres de déploiement et d'écoute : toujours libres.
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { useWst } from "../context";

const { form } = useWst();
</script>

<template>
  <Card class="h-fit">
    <CardHeader>
      <CardTitle class="flex items-center text-sm">
        Analyse
        <InfoTip text="Ce qui décrit le déploiement et l'écoute, pas la caisse : ces paramètres restent libres même quand une enceinte est sélectionnée." />
      </CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-3">
      <div class="flex flex-col gap-1.5">
        <Label class="flex items-center text-xs">
          Nombre de caisses
          <InfoTip text="Longueur de la ligne : H = N × pas. Entre dans l'ARF minimal et dans la frontière de champ proche." />
        </Label>
        <Input v-model.number="form.speakerCount" type="number" min="1" />
      </div>

      <div class="flex flex-col gap-1.5">
        <Label class="flex items-center text-xs">
          Angles évalués (°)
          <InfoTip text="Liste d'angles entre caisses adjacentes. Le premier sert de référence pour les critères qui ne dépendent pas de l'angle." />
        </Label>
        <Input v-model="form.splaysText" />
      </div>

      <div class="flex flex-col gap-1.5">
        <Label class="flex items-center text-xs">
          Distances auditeur (m)
          <InfoTip text="Le pire cas est le premier rang : c'est la distance la plus courte qui contraint l'angle." />
        </Label>
        <Input v-model="form.distancesText" />
      </div>

      <div class="flex flex-col gap-1.5">
        <Label class="flex items-center text-xs">
          f max (Hz)
          <InfoTip text="Haut de la bande utile visée : sert au critère 3 (planéité) et au pas maximal." />
        </Label>
        <Input v-model.number="form.fMaxHz" type="number" />
      </div>

      <Separator />

      <label class="flex items-center gap-2 text-xs">
        <input v-model="form.usePaperConvention" type="checkbox" class="size-3.5" />
        <span class="flex items-center">
          Convention papier (c = 333,3 m/s)
          <InfoTip text="Le papier approxime λ = 1/(3F). Cocher pour retrouver ses chiffres au dixième ; décocher pour la célérité réelle (343 m/s), qui donne des fréquences ~3 % plus hautes." />
        </span>
      </label>
    </CardContent>
  </Card>
</template>
