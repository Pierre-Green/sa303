<script setup lang="ts">
// Tableau de cas de charge, commun aux deux blocs : le bloc A les étiquette
// par cas, le bloc B par splay et signale les doublons du bloc A.
import type { LoadCaseReport } from "@/lib/types";
import { Badge } from "@/components/ui/badge";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { governingPath, utilizationVariant, type LoadPath } from "../loadPaths";

const props = defineProps<{
  cases: LoadCaseReport[];
  block: "a" | "b";
}>();

function rowLabel(c: LoadCaseReport): string {
  return props.block === "a" ? c.labels.join(" · ") : `${c.result.splayDeg}°`;
}

/// Met en évidence la colonne du chemin qui gouverne : sur une ligne de cinq
/// nombres, savoir lequel dimensionne est ce qui se lit le moins bien.
function pathClass(c: LoadCaseReport, path: LoadPath): string {
  return governingPath(c) === path ? "font-semibold" : "";
}
</script>

<template>
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>{{ block === "a" ? "Cas" : "Splay" }}</TableHead>
        <TableHead>Grappe</TableHead>
        <TableHead>Joint</TableHead>
        <TableHead>F couronne</TableHead>
        <TableHead>F ancrage</TableHead>
        <TableHead>F verrou</TableHead>
        <TableHead>F pivot</TableHead>
        <TableHead>M barre</TableHead>
        <TableHead>Angle absolu</TableHead>
        <TableHead>
          <span class="flex items-center">
            Chemin
            <InfoTip
              text="Le chemin de charge qui gouverne : couronne, ancrage, verrou, bielle ou flexion de barre. C'est lui qui dit quoi renforcer."
            />
          </span>
        </TableHead>
        <TableHead>
          <span class="flex items-center">
            Taux
            <InfoTip
              text="Pire des cinq chemins. Regarder la seule couronne sous-estime la paire d'un facteur 3 et ignore complètement la flexion de barre."
            />
          </span>
        </TableHead>
        <TableHead v-if="block === 'b'">Doublon</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow
        v-for="(c, idx) in cases"
        :key="idx"
        :class="block === 'b' && c.duplicate ? 'opacity-40 line-through' : ''"
      >
        <TableCell>{{ rowLabel(c) }}</TableCell>
        <TableCell>{{ c.clusterName }}</TableCell>
        <TableCell>J{{ c.jointNumber }}</TableCell>
        <TableCell :class="pathClass(c, 'couronne')">{{ c.result.fOrientationN.toFixed(0) }} N</TableCell>
        <TableCell :class="pathClass(c, 'ancrage')">{{ c.result.fAnchorN.toFixed(0) }} N</TableCell>
        <TableCell :class="pathClass(c, 'verrou')">{{ c.result.fLatchN.toFixed(0) }} N</TableCell>
        <TableCell :class="pathClass(c, 'bielle')">{{ c.result.fPivotN.toFixed(0) }} N</TableCell>
        <TableCell :class="pathClass(c, 'flexion de barre')">{{ c.result.barMomentMaxNm.toFixed(0) }} N·m</TableCell>
        <TableCell>{{ c.result.inclinationDeg.toFixed(0) }} °</TableCell>
        <TableCell class="text-muted-foreground">{{ governingPath(c) }}</TableCell>
        <TableCell>
          <Badge :variant="utilizationVariant(c.utilizationWorst)">
            {{ (c.utilizationWorst * 100).toFixed(0) }}%
          </Badge>
        </TableCell>
        <TableCell v-if="block === 'b'">{{ c.duplicate ? "bloc A" : "" }}</TableCell>
      </TableRow>
    </TableBody>
  </Table>
</template>
