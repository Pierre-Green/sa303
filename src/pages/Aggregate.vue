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
import InfoTip from "@/components/InfoTip.vue";
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

/// Les cinq chemins de charge d'un cas, nommés. L'ordre n'a pas d'importance :
/// c'est le maximum qui gouverne, et `utilizationWorst` le donne déjà côté
/// Rust — on ne le recalcule pas ici, on ne fait que le nommer.
function paths(c: LoadCaseReport): [string, number][] {
  return [
    ["couronne", c.utilizationOrientation],
    ["ancrage", c.utilizationAnchor],
    ["verrou", c.utilizationLatch],
    ["bielle", c.utilizationPivot],
    ["flexion de barre", c.utilizationBar],
  ];
}

function governingPath(c: LoadCaseReport): string {
  return paths(c).reduce((a, b) => (b[1] > a[1] ? b : a))[0];
}

/// Met en évidence la colonne du chemin qui gouverne : sur une ligne de cinq
/// nombres, savoir lequel dimensionne est ce qui se lit le moins bien.
function pathClass(c: LoadCaseReport, path: string): string {
  return governingPath(c) === path ? "font-semibold" : "";
}

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
                  <TableHead>F couronne</TableHead>
                  <TableHead>F ancrage</TableHead>
                  <TableHead>F verrou</TableHead>
                  <TableHead>F pivot</TableHead>
                  <TableHead>M barre</TableHead>
                  <TableHead>Angle absolut</TableHead>
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
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow v-for="(c, idx) in (compartment as CompartmentReport).blockA" :key="idx">
                  <TableCell>{{ rowLabel(c, "a") }}</TableCell>
                  <TableCell>{{ c.clusterName }}</TableCell>
                  <TableCell>J{{ c.jointNumber }}</TableCell>
                  <TableCell :class="pathClass(c, 'couronne')">
                    {{ c.result.fOrientationN.toFixed(0) }} N
                  </TableCell>
                  <TableCell :class="pathClass(c, 'ancrage')">
                    {{ c.result.fAnchorN.toFixed(0) }} N
                  </TableCell>
                  <TableCell :class="pathClass(c, 'verrou')">
                    {{ c.result.fLatchN.toFixed(0) }} N
                  </TableCell>
                  <TableCell :class="pathClass(c, 'bielle')">
                    {{ c.result.fPivotN.toFixed(0) }} N
                  </TableCell>
                  <TableCell :class="pathClass(c, 'flexion de barre')">
                    {{ c.result.barMomentMaxNm.toFixed(0) }} N·m
                  </TableCell>
                  <TableCell>
                    {{ c.result.inclinationDeg.toFixed(0) }} °
                  </TableCell>
                  <TableCell class="text-muted-foreground">{{ governingPath(c) }}</TableCell>
                  <TableCell>
                    <Badge :variant="utilizationVariant(c.utilizationWorst)">
                      {{ (c.utilizationWorst * 100).toFixed(0) }}%
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
                  <TableHead>F couronne</TableHead>
                  <TableHead>F ancrage</TableHead>
                  <TableHead>F verrou</TableHead>
                  <TableHead>F pivot</TableHead>
                  <TableHead>M barre</TableHead>
                  <TableHead>Angle absolut</TableHead>
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
                  <TableCell :class="pathClass(c, 'couronne')">
                    {{ c.result.fOrientationN.toFixed(0) }} N
                  </TableCell>
                  <TableCell :class="pathClass(c, 'ancrage')">
                    {{ c.result.fAnchorN.toFixed(0) }} N
                  </TableCell>
                  <TableCell :class="pathClass(c, 'verrou')">
                    {{ c.result.fLatchN.toFixed(0) }} N
                  </TableCell>
                  <TableCell :class="pathClass(c, 'bielle')">
                    {{ c.result.fPivotN.toFixed(0) }} N
                  </TableCell>
                  <TableCell :class="pathClass(c, 'flexion de barre')">
                    {{ c.result.barMomentMaxNm.toFixed(0) }} N·m
                  </TableCell>
                  <TableCell>
                    {{ c.result.inclinationDeg.toFixed(0) }} °
                  </TableCell>
                  <TableCell class="text-muted-foreground">{{ governingPath(c) }}</TableCell>
                  <TableCell>
                    <Badge :variant="utilizationVariant(c.utilizationWorst)">
                      {{ (c.utilizationWorst * 100).toFixed(0) }}%
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
