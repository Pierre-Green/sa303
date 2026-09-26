// Fine wrapper autour des commandes Tauri : aucun calcul ici, uniquement du
// transport JSON vers `sa303-core` (brief §1).

import { invoke } from "@tauri-apps/api/core";
import type {
  AggregateReport,
  BuiltinIds,
  BumperBarModel,
  BumperModel,
  AuditExport,
  Cluster,
  ClusterResult,
  Settings,
  SpeakerGeometryReport,
  SpeakerModel,
  WstInputs,
  WstReport,
} from "./types";

export const api = {
  // Catalogue livré avec le logiciel : ces éléments ne sont ni modifiables ni
  // supprimables depuis l'application, seule une mise à jour les fait bouger.
  getBuiltinIds: () => invoke<BuiltinIds>("get_builtin_ids"),

  // Catalogue d'équipement en lecture seule : il se modifie dans les JSON.
  listSpeakerModels: () => invoke<SpeakerModel[]>("list_speaker_models"),

  listBumperModels: () => invoke<BumperModel[]>("list_bumper_models"),

  listBumperBarModels: () => invoke<BumperBarModel[]>("list_bumper_bar_models"),

  listClusters: () => invoke<Cluster[]>("list_clusters"),
  saveCluster: (cluster: Cluster) => invoke<void>("save_cluster", { cluster }),
  deleteCluster: (id: string) => invoke<void>("delete_cluster", { id }),

  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<void>("save_settings", { settings }),

  // Prend la grappe elle-même (pas seulement son id) : sert de prévisualisation
  // live pendant l'édition, avant tout enregistrement.
  computeClusterResult: (cluster: Cluster) =>
    invoke<ClusterResult>("compute_cluster_result", { cluster }),

  getSpeakerGeometryReport: (speakerModelId: string) =>
    invoke<SpeakerGeometryReport>("get_speaker_geometry_report", { speakerModelId }),

  computeAggregateReport: () => invoke<AggregateReport>("compute_aggregate_report"),

  // Export d'audit. `clusterIds` omis ou vide = toutes les grappes
  // enregistrées ; sinon la sélection, dans l'ordre donné. L'horodatage vient
  // d'ici parce que le cœur de calcul n'a volontairement pas d'horloge.
  buildClusterAuditExport: (clusterIds?: string[]) =>
    invoke<AuditExport>("build_cluster_audit_export", {
      clusterIds: clusterIds ?? null,
      generatedAt: new Date().toISOString(),
    }),

  // Même export, écrit via la boîte de dialogue système. Rend le chemin
  // retenu, ou `null` si l'utilisateur a annulé — annuler n'est pas une erreur.
  exportClustersForAudit: (clusterIds: string[] | undefined, suggestedFileName: string) =>
    invoke<string | null>("export_clusters_for_audit", {
      clusterIds: clusterIds ?? null,
      generatedAt: new Date().toISOString(),
      suggestedFileName,
    }),

  exportAggregateReportForShapeOptimizationFem: () =>
    invoke<string>("export_aggregate_report_for_shape_optimization_fem"),

  // Critères WST : `speakerModelId` renseigné → le pas entre centres
  // acoustiques est dérivé de la géométrie réelle, angle par angle.
  computeWstReport: (inputs: WstInputs, speakerModelId: string | null) =>
    invoke<WstReport>("compute_wst_report", { inputs, speakerModelId }),
};
