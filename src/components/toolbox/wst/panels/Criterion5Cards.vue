<script setup lang="ts">
// Critère 5, fronts plans anglés uniquement : fréquence tenable par angle, puis
// la lecture inverse, angle maximal par fréquence.
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { WstReport } from "@/lib/types";
import { deg, khz, metres, mm, ratio } from "../format";

defineProps<{ report: WstReport; criterion5: NonNullable<WstReport["criterion5"]>; distances: number[] }>();
</script>

<template>
  <div class="flex flex-col gap-4">
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center text-sm">
          Critère 5 — fréquence tenable par angle
          <InfoTip text="α_max = 2λ/D − pas/d, où D est la bouche ACOUSTIQUE — dans la dérivation du §6.2, le produit ARF·STEP ne vaut jamais autre chose. Au-delà de cette fréquence, la zone sans énergie entre deux caisses se referme après l'auditeur : le trou devient audible. L'ARF de la colonne est dérivé du pas pour l'affichage, il ne pilote pas le calcul." />
        </CardTitle>
      </CardHeader>
      <CardContent class="flex flex-col gap-3">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Angle</TableHead>
              <TableHead>Pas</TableHead>
              <TableHead>Jour façade</TableHead>
              <TableHead>ARF</TableHead>
              <TableHead v-for="d in distances" :key="d">à {{ d }} m</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="row in criterion5.rows" :key="row.splayDeg">
              <TableCell class="font-medium">{{ deg(row.splayDeg) }}</TableCell>
              <TableCell class="text-muted-foreground">{{ mm(row.stepMm) }}</TableCell>
              <TableCell class="text-muted-foreground">{{ mm(row.gapMm) }}</TableCell>
              <TableCell class="text-muted-foreground">{{ ratio(row.arf) }}</TableCell>
              <TableCell
                v-for="(f, i) in row.fMaxByDistanceHz"
                :key="i"
                :class="f != null && f >= report.criterion3.fMaxHz ? 'text-status-ok' : ''"
              >
                {{ khz(f) }}
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
        <p class="text-xs text-muted-foreground">
          En vert : la bande utile ({{ khz(report.criterion3.fMaxHz) }}) passe entièrement à cet
          angle pour cette distance.
          <template v-if="criterion5.maxStepMm != null">
            Pas maximal laissant un angle possible à {{ khz(report.criterion3.fMaxHz) }} pour le
            premier rang ({{ metres(criterion5.closestDistanceM) }}) :
            <strong>{{ mm(criterion5.maxStepMm) }}</strong>.
          </template>
        </p>
      </CardContent>
    </Card>

    <!-- Critère 5, lecture inverse -->
    <Card>
      <CardHeader>
        <CardTitle class="text-sm">Critère 5 — angle maximal par fréquence</CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Fréquence</TableHead>
              <TableHead v-for="d in distances" :key="d">à {{ d }} m</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="limit in criterion5.angleLimits" :key="limit.frequencyHz">
              <TableCell class="font-medium">{{ khz(limit.frequencyHz) }}</TableCell>
              <TableCell
                v-for="(a, i) in limit.maxSplayDegByDistance"
                :key="i"
                :class="a == null ? 'text-status-alarm' : ''"
              >
                {{ a == null ? "impossible" : deg(a) }}
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  </div>
</template>
