<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "@/lib/api";
import type { AggregateReport, CompartmentReport, LoadCaseReport } from "@/lib/types";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { FileIcon } from "@lucide/vue";

const report = ref<AggregateReport | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    report.value = await api.computeAggregateReport();
  } catch (e) {
    error.value = String(e);
  }
});

async function exportAggregateReportForShapeOptimizationFem() {
  try {
    await api.exportAggregateReportForShapeOptimizationFem();
  } catch (e) {
    error.value = String(e);
  }
};

function utilizationVariant(u: number): "default" | "secondary" | "destructive" {
  if (u > 1) return "destructive";
  if (u > 0.8) return "secondary";
  return "default";
}

function rowLabel(c: LoadCaseReport, block: "a" | "b"): string {
  if (block === "a") return c.labels.join(" · ");
  return `splay ${c.result.splayDeg}°`;
}
</script>

<template>
  <!-- La coquille de l'application est en `overflow-hidden` et donne une hauteur
       fixe à `main` : c'est donc à chaque page de défiler. Sans `h-full` ni zone
       défilante, tout ce qui dépasse était simplement coupé. -->
  <div class="flex h-full flex-col gap-6 overflow-y-auto p-4">
    <p v-if="error" class="text-sm text-destructive">{{ error }}</p>

    <Button
      variant="outline"
      class="w-fit gap-2"
      size="sm"
      @click="exportAggregateReportForShapeOptimizationFem"
    >
      <FileIcon />
      Exporter le rapport pour l'optimization fem
    </Button>

    <template v-if="report">
      <Card v-if="report.impossibleClusters.length > 0" class="border-destructive/50">
        <CardHeader>
          <CardTitle class="text-sm text-destructive">
            Grappes exclues — configuration physiquement impossible ({{ report.impossibleClusters.length }})
          </CardTitle>
        </CardHeader>
        <CardContent class="flex flex-col gap-2">
          <div
            v-for="(c, idx) in report.impossibleClusters"
            :key="idx"
            class="rounded-md border border-destructive/50 bg-destructive/10 p-2 text-xs text-destructive"
          >
            <span class="font-medium">{{ c.clusterName }}</span> — {{ c.reason }}
          </div>
        </CardContent>
      </Card>

      <section v-for="(compartment, key) in { suspendu: report.flown, stack: report.stacked }" :key="key">
        <h2 class="mb-2 text-sm font-semibold capitalize">Compartiment {{ key }}</h2>

        <Card class="mb-4">
          <CardHeader>
            <CardTitle class="text-sm">Bloc A — par chemin de charge</CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Cas</TableHead>
                  <TableHead>Grappe</TableHead>
                  <TableHead>Joint</TableHead>
                  <TableHead>F orientation</TableHead>
                  <TableHead>A orientation</TableHead>
                  <TableHead>F pivot</TableHead>
                  <TableHead>A pivot</TableHead>
                  <TableHead>Angle absolut</TableHead>
                  <TableHead>Taux</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow v-for="(c, idx) in (compartment as CompartmentReport).blockA" :key="idx">
                  <TableCell>{{ rowLabel(c, "a") }}</TableCell>
                  <TableCell>{{ c.clusterName }}</TableCell>
                  <TableCell>J{{ c.jointNumber }}</TableCell>
                  <TableCell>
                    {{ c.result.fOrientationN.toFixed(0) }} N
                  </TableCell>
                  <TableCell>
                    {{ c.result.fOrientationAngleDeg.toFixed(0) }} °
                  </TableCell>
                  <TableCell>
                    {{ c.result.fPivotN.toFixed(0) }} N
                  </TableCell>
                  <TableCell>
                    {{ c.result.fPivotAngleDeg.toFixed(0) }} °
                  </TableCell>
                  <TableCell>
                    {{ c.result.inclinationDeg.toFixed(0) }} °
                  </TableCell>
                  <TableCell>
                    <Badge :variant="utilizationVariant(Math.max(c.utilizationOrientation, c.utilizationPivot))">
                      {{ (Math.max(c.utilizationOrientation, c.utilizationPivot) * 100).toFixed(0) }}%
                    </Badge>
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle class="text-sm">
              Bloc B — enveloppe par splay
              ({{ (compartment as CompartmentReport).blockB.length }} angles couverts sur
              {{
                (compartment as CompartmentReport).blockB.length +
                (compartment as CompartmentReport).uncoveredSplaysDeg.length
              }}
              percés)
            </CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Splay</TableHead>
                  <TableHead>Grappe</TableHead>
                  <TableHead>Joint</TableHead>
                  <TableHead>F orientation</TableHead>
                  <TableHead>A orientation</TableHead>
                  <TableHead>F pivot</TableHead>
                  <TableHead>A pivot</TableHead>
                  <TableHead>Angle absolut</TableHead>
                  <TableHead>Taux</TableHead>
                  <TableHead>Doublon</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow
                  v-for="(c, idx) in (compartment as CompartmentReport).blockB"
                  :key="idx"
                  :class="c.duplicate ? 'opacity-40 line-through' : ''"
                >
                  <TableCell>{{ c.result.splayDeg }}°</TableCell>
                  <TableCell>{{ c.clusterName }}</TableCell>
                  <TableCell>J{{ c.jointNumber }}</TableCell>
                  <TableCell>
                    {{ c.result.fOrientationN.toFixed(0) }} N
                  </TableCell>
                  <TableCell>
                    {{ c.result.fOrientationAngleDeg.toFixed(0) }} °
                  </TableCell>
                  <TableCell>
                    {{ c.result.fPivotN.toFixed(0) }} N
                  </TableCell>
                  <TableCell>
                    {{ c.result.fPivotAngleDeg.toFixed(0) }} °
                  </TableCell>
                  <TableCell>
                    {{ c.result.inclinationDeg.toFixed(0) }} °
                  </TableCell>
                  <TableCell>
                    <Badge :variant="utilizationVariant(Math.max(c.utilizationOrientation, c.utilizationPivot))">
                      {{ (Math.max(c.utilizationOrientation, c.utilizationPivot) * 100).toFixed(0) }}%
                    </Badge>
                  </TableCell>
                  <TableCell>{{ c.duplicate ? "bloc A" : "" }}</TableCell>
                </TableRow>
              </TableBody>
            </Table>

            <!-- L'enveloppe ne dimensionne que les angles réellement montés.
                 Sans cette mention, un tableau de 7 lignes pour 17 trous percés
                 aurait l'air complet. -->
            <p
              v-if="(compartment as CompartmentReport).uncoveredSplaysDeg.length > 0"
              class="mt-3 rounded-md border border-status-warn/40 bg-status-warn/10 p-2 text-xs"
            >
              <span class="font-medium">Trous percés sans cas de charge :</span>
              {{ (compartment as CompartmentReport).uncoveredSplaysDeg.map((s) => `${s}°`).join(", ") }}.
              Aucune grappe de ce compartiment ne les utilise, l'enveloppe ne dit
              donc rien de ces angles.
            </p>
          </CardContent>
        </Card>
      </section>
    </template>
  </div>
</template>
