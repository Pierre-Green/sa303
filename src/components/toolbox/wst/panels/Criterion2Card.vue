<script setup lang="ts">
// Critère 2 : pas < λ/2.
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { WstReport } from "@/lib/types";
import { deg, khz } from "../format";

defineProps<{ report: WstReport }>();
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center text-sm">
        Critère 2 — pas &lt; λ/2
        <InfoTip text="Sous cette fréquence, aucun lobe de réseau ne peut exister quel que soit l'ARF : le critère 1 n'a alors pas lieu de s'appliquer." />
      </CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-2">
      <p class="text-xs text-muted-foreground">
        Limite : <strong>{{ khz(report.criterion2.frequencyLimitHz) }}</strong>
      </p>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Fréquence</TableHead>
            <TableHead>Lobe réseau</TableHead>
            <TableHead>Premier creux</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="sample in report.criterion2.samples" :key="sample.frequencyHz">
            <TableCell :class="sample.satisfied ? 'text-status-ok' : ''">
              {{ khz(sample.frequencyHz) }}
            </TableCell>
            <TableCell>{{ sample.gratingLobeDeg == null ? "aucun" : deg(sample.gratingLobeDeg) }}</TableCell>
            <TableCell>{{ deg(sample.firstDipDeg) }}</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </CardContent>
  </Card>
</template>
