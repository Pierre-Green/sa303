<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "@/lib/api";
import type { AggregateReport, CompartmentReport, LoadCaseReport } from "@/lib/types";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";

const report = ref<AggregateReport | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    report.value = await api.computeAggregateReport();
  } catch (e) {
    error.value = String(e);
  }
});

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
  <div class="flex flex-col gap-6 p-4">
    <p v-if="error" class="text-sm text-destructive">{{ error }}</p>

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
                  <TableHead>F pivot</TableHead>
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
                    {{ c.result.fPivotN.toFixed(0) }} N
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
              Bloc B — enveloppe par splay ({{ (compartment as CompartmentReport).blockB.length }} angles)
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
                  <TableHead>F pivot</TableHead>
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
                    {{ c.result.fPivotN.toFixed(0) }} N
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
          </CardContent>
        </Card>
      </section>
    </template>
  </div>
</template>
