<script setup lang="ts">
// Un trou de couronne par cran de la grille, tel que calculé par sa303-core.
import type { CrownHoleReport } from "@/lib/types";
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
import { fmt } from "../format";

defineProps<{ holes: CrownHoleReport[] }>();
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center text-sm">
        Trous de couronne ({{ holes.length }})
        <InfoTip text="Un trou par angle de la grille percée. Les splays impairs sont sur la couronne intérieure (retrait), donc avec un bras de levier différent des splays pairs — jamais une valeur unique." />
      </CardTitle>
    </CardHeader>
    <CardContent>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Splay</TableHead>
            <TableHead>Couronne</TableHead>
            <TableHead>Rayon</TableHead>
            <TableHead>
              <span class="flex items-center">
                Position
                <InfoTip text="Trou de couronne dans le repère de l'enceinte. Les trous ne sont pas sur un arc centré sur un point fixe : leur centre est la goupille basse de bielle, qui se déplace avec le splay." />
              </span>
            </TableHead>
            <TableHead>
              <span class="flex items-center">
                PV
                <InfoTip text="Goupille basse de bielle à ce cran : le centre depuis lequel ce trou est percé." />
              </span>
            </TableHead>
            <TableHead>
              <span class="flex items-center">
                Écartement
                <InfoTip text="Écartement vertical des coins avant à ce cran : c'est l'espacement entre caissons. Le décalage avant qui l'accompagne reste sous 0,05 mm sur toute la plage line source, donc invisible." />
              </span>
            </TableHead>
            <TableHead>Bras</TableHead>
            <TableHead>
              <span class="flex items-center">
                Recoupement
                <InfoTip text="Écart entre le bras reconstruit géométriquement et la formule trigonométrique de contrôle. Au-delà de 0,5 mm, la géométrie serait incohérente." />
              </span>
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody class="font-mono">
          <TableRow v-for="h in holes" :key="h.splayDeg">
            <TableCell>{{ h.splayDeg }}°</TableCell>
            <TableCell>{{ h.row === "int" ? "intérieure" : "extérieure" }}</TableCell>
            <TableCell>{{ fmt(h.radius, 0) }} mm</TableCell>
            <TableCell>{{ fmt(h.position.x) }} ; {{ fmt(h.position.y) }}</TableCell>
            <TableCell>{{ fmt(h.pv.x) }} ; {{ fmt(h.pv.y) }}</TableCell>
            <TableCell>{{ fmt(-h.offset.verticalMm) }} mm</TableCell>
            <TableCell>{{ fmt(h.leverMm, 1) }} mm</TableCell>
            <TableCell>{{ fmt(h.discrepancyMm, 3) }} mm</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </CardContent>
  </Card>
</template>
