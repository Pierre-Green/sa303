<script setup lang="ts">
// Géométrie de l'arc (CCA).
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
        Géométrie de l'arc (CCA)
        <InfoTip text="Rayon et flèche ne dépendent que de l'angle et du pas : ils sont vrais quelle que soit la nature du guide. La flèche est l'écart entre une corde plate et l'arc sur une caisse." />
      </CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-2">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Angle</TableHead>
            <TableHead>Rayon</TableHead>
            <TableHead>Flèche</TableHead>
            <TableHead>Courbure critique au-dessus de</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="row in report.cca.rows" :key="row.splayDeg">
            <TableCell class="font-medium">{{ deg(row.splayDeg) }}</TableCell>
            <TableCell>{{ row.radiusM == null ? "plate" : metres(row.radiusM) }}</TableCell>
            <TableCell>{{ mm(row.sagittaMm) }}</TableCell>
            <TableCell>{{ khz(row.curvatureMattersAboveHz) }}</TableCell>
          </TableRow>
        </TableBody>
      </Table>

    </CardContent>
  </Card>
</template>
