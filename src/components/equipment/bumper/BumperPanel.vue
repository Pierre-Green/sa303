<script setup lang="ts">
// Fiche d'un bumper, en lecture seule : cotes, perçages d'accroche, barres
// arrière et compatibilités déclarées dans son JSON.
import { computed } from "vue";
import type { BumperBarModel, BumperModel } from "@/lib/types";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import SpecCard from "../shared/SpecCard.vue";
import HoleTable from "../shared/HoleTable.vue";
import EquipmentPreview from "../shared/EquipmentPreview.vue";
import { bumperPreview } from "../composables/useEquipmentPreview";
import type { SpecRow } from "../types";
import { fmt, fmtMm, yesNo } from "../format";

const props = defineProps<{
  bumper: BumperModel;
  bumperBars: BumperBarModel[];
  speakerName: (id: string) => string;
}>();

const RIG_TIP =
  "Repère bumper : origine au centre du dessous, X vers l'arrière, Y vers le haut, en mm. Les seuls points où le solveur peut accrocher en vol : il choisit le trou, jamais une position libre.";

const general = computed<SpecRow[]>(() => {
  const b = props.bumper;
  return [
    { label: "Profondeur", value: fmtMm(b.depth), tip: "Étendue avant-arrière du bumper." },
    { label: "Hauteur", value: fmtMm(b.height), tip: "Épaisseur du bumper (dimension verticale du tube)." },
    {
      label: "Pion avant ← face avant",
      value: fmtMm(b.pins.frontFromFrontMm, 3),
      tip: "Les deux pions qui goupillent le bumper à l'enceinte de référence, cotés depuis les bords du bumper comme sur le plan. Perçage déclaré, pas déduit de la quincaillerie de l'enceinte.",
    },
    { label: "Pion arrière ← face arrière", value: fmtMm(b.pins.rearFromRearMm, 3) },
    { label: "Hauteur des pions", value: fmtMm(b.pins.heightFromBottomMm) },
    {
      label: "Bielle de pivot (utile)",
      value: fmtMm(b.pivotBarUsableLengthMm, 2),
      tip: "Entraxe entre la charnière haute du caisson et la goupille haute dans le bumper. Avec le trou haut de la barre, c'est elle qui place le bumper en vol.",
    },
    {
      label: "CMU par point",
      value: `${fmt(b.rigging.wllKg, 0)} kg`,
      tip: "Charge maximale d'utilisation de chaque point, comparée à la charge dynamique (poids × k_dyn).",
    },
  ];
});

const compatibleBars = computed(() =>
  props.bumperBars.filter((bar) =>
    bar.compatibleBumpers.some((c) => c.bumperModelId === props.bumper.id),
  ),
);

const preview = computed(() => bumperPreview(props.bumper));
</script>

<template>
  <div class="grid grid-cols-1 gap-4 xl:grid-cols-2">
    <EquipmentPreview part="bumper" :spec="preview" />
    <SpecCard title="Général" :rows="general" />

    <HoleTable title="Trous de manille" :tip="RIG_TIP" :holes="bumper.rigging.shackleHoles" />
    <HoleTable
      title="Trous de liaison barre"
      tip="Trous où se goupillent les pattes d'une barre de déport, même repère que les manilles."
      :holes="bumper.rigging.barLinkHoles"
    />

    <Card>
      <CardHeader>
        <CardTitle class="flex items-center text-sm">
          Barres arrière ({{ bumper.rearBars.length }})
          <InfoTip text="Une barre par inclinaison : c'est ainsi qu'on penche la première tête en stack. Le vol monte toujours celle à 0°. Trou haut coté depuis l'ancrage, le long de l'axe de la paire, déport positif vers l'avant." />
        </CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Inclinaison</TableHead>
              <TableHead>Trou haut (le long)</TableHead>
              <TableHead>Trou haut (déport)</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody class="font-mono">
            <TableRow v-for="bar in bumper.rearBars" :key="bar.tiltDeg">
              <TableCell>{{ bar.tiltDeg }}°</TableCell>
              <TableCell>{{ fmtMm(bar.topHoleAlongMm, 3) }}</TableCell>
              <TableCell>{{ fmtMm(bar.topHoleLateralMm, 3) }}</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle class="flex items-center text-sm">
          Compatibilités
          <InfoTip text="Un bumper peut être posable en vol, en stack, ou les deux, selon l'enceinte. Non listé : le bumper n'apparaît pas comme option pour cette enceinte." />
        </CardTitle>
      </CardHeader>
      <CardContent class="flex flex-col gap-4">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Enceinte</TableHead>
              <TableHead>Vol</TableHead>
              <TableHead>Stack</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="c in bumper.compatibleSpeakers" :key="c.speakerModelId">
              <TableCell>{{ speakerName(c.speakerModelId) }}</TableCell>
              <TableCell>{{ yesNo(c.flown) }}</TableCell>
              <TableCell>{{ yesNo(c.stacked) }}</TableCell>
            </TableRow>
          </TableBody>
        </Table>
        <p class="text-xs text-muted-foreground">
          Barres de déport :
          {{ compatibleBars.length ? compatibleBars.map((b) => b.name).join(", ") : "aucune" }}
        </p>
      </CardContent>
    </Card>
  </div>
</template>
