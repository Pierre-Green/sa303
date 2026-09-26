<script setup lang="ts">
// Critère 4 : courbure.
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { WstReport } from "@/lib/types";
import { db, deg, metres } from "../format";

defineProps<{ report: WstReport }>();
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center text-sm">
        Critère 4 — courbure
        <InfoTip text="Une ligne bien courbée garde α·d constant. Le niveau chute de 3 dB quand α·d atteint le pas. En dessous de l'angle minimal indiqué, la ligne se comporte comme plate." />
      </CardTitle>
    </CardHeader>
    <CardContent>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Angle</TableHead>
            <TableHead>Rayon</TableHead>
            <TableHead>α·d</TableHead>
            <TableHead>Niveau</TableHead>
            <TableHead>α mini</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="row in report.curvature" :key="row.splayDeg">
            <TableCell class="font-medium">{{ deg(row.splayDeg) }}</TableCell>
            <TableCell>{{ row.radiusM == null ? "plate" : metres(row.radiusM) }}</TableCell>
            <TableCell>{{ row.angleDistanceProduct == null ? "—" : row.angleDistanceProduct.toFixed(2) }}</TableCell>
            <TableCell>{{ db(row.relativeLevelDb) }}</TableCell>
            <TableCell class="text-muted-foreground">{{ deg(row.curvedModelMinSplayDeg) }}</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </CardContent>
  </Card>
</template>
