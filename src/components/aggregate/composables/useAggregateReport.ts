// Rapport agrégé, calculé par sa303-core sur toutes les grappes enregistrées,
// et son export pour l'optimisation de forme FEM.

import { onMounted, ref } from "vue";
import { api } from "@/lib/api";
import type { AggregateReport } from "@/lib/types";

export function useAggregateReport() {
  const report = ref<AggregateReport | null>(null);
  const error = ref<string | null>(null);

  onMounted(async () => {
    try {
      report.value = await api.computeAggregateReport();
    } catch (e) {
      error.value = String(e);
    }
  });

  async function exportForShapeOptimizationFem() {
    try {
      await api.exportAggregateReportForShapeOptimizationFem();
    } catch (e) {
      error.value = String(e);
    }
  }

  return { report, error, exportForShapeOptimizationFem };
}
