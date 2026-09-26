<script setup lang="ts">
// Champ proche / lointain et critère 3.
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { WstReport } from "@/lib/types";
import { deg, khz, metres, mm } from "../format";

defineProps<{ report: WstReport }>();
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center text-sm">
        Champ proche &amp; critère 3
        <InfoTip text="La transition champ proche → lointain est progressive : la forme complète et la forme de Fresnel en donnent les deux bornes. Le critère 3 borne la déviation admissible d'un front plan à λ/4." />
      </CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-2">
      <p class="text-xs text-muted-foreground">
        Aucun champ proche sous
        <strong>{{ khz(report.nearField.noNearFieldBelowHz) }}</strong> · déviation
        admissible à {{ khz(report.criterion3.fMaxHz) }} :
        <strong>{{ mm(report.criterion3.maxDeviationMm) }}</strong>
      </p>
      <p
        v-if="report.criterion3.wavefrontDeviationMm != null"
        class="text-xs text-muted-foreground"
      >
        Front relevé à {{ metres(report.criterion3.wavefrontRadiusM) }} de rayon : écart
        au plan
        <strong>{{ mm(report.criterion3.wavefrontDeviationMm) }}</strong> sur la bouche
        (s = (D/2)² / 2R), soit isophase jusqu'à
        <strong>{{ khz(report.criterion3.isophaseFrequencyLimitHz) }}</strong
        >. C'est la vraie mesure de « à quel point le front est plan ».
      </p>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Fréquence</TableHead>
            <TableHead>Frontière</TableHead>
            <TableHead>Fresnel</TableHead>
            <TableHead>1er creux</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="sample in report.nearField.samples" :key="sample.frequencyHz">
            <TableCell>{{ khz(sample.frequencyHz) }}</TableCell>
            <TableCell>{{ metres(sample.boundaryM) }}</TableCell>
            <TableCell class="text-muted-foreground">{{ metres(sample.boundaryFresnelM) }}</TableCell>
            <TableCell>{{ deg(sample.firstDipDeg) }}</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </CardContent>
  </Card>
</template>
