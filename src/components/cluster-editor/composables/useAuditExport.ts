// Export d'audit. Trois portées, un seul bouton : la grappe courante, les
// grappes cochées, ou tout le catalogue. Le mode se déduit de l'état plutôt
// que d'être choisi dans un menu — cocher des grappes est déjà l'expression de
// l'intention.

import { computed, ref, type Ref } from "vue";
import { api } from "@/lib/api";
import type { Cluster } from "@/lib/types";

export function useAuditExport(opts: {
  clusters: Ref<Cluster[]>;
  /** Grappe affichée, si elle est enregistrée. */
  currentId: () => string | null;
  error: Ref<string | null>;
}) {
  const selection = ref<Set<string>>(new Set());
  const exporting = ref(false);
  const message = ref<string | null>(null);

  function toggle(id: string) {
    const next = new Set(selection.value);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selection.value = next;
  }

  /** Ce que le bouton exportera si on clique maintenant. */
  const scope = computed(() => {
    if (selection.value.size > 0) {
      return { ids: [...selection.value], label: `${selection.value.size} cochée(s)` };
    }
    const current = opts.currentId();
    if (current) return { ids: [current], label: "la grappe courante" };
    return { ids: undefined, label: `tout (${opts.clusters.value.length})` };
  });

  function fileName(ids: string[] | undefined): string {
    const date = new Date().toISOString().slice(0, 10);
    if (!ids) return `sa303-audit-toutes-${date}.json`;
    if (ids.length === 1) {
      const one = opts.clusters.value.find((c) => c.id === ids[0]);
      // Le nom de la grappe se retrouve dans le nom de fichier : un dossier
      // d'audit contient vite une dizaine de ces exports.
      const slug = (one?.name ?? "grappe").replace(/[^\p{L}\p{N}]+/gu, "-").toLowerCase();
      return `sa303-audit-${slug}-${date}.json`;
    }
    return `sa303-audit-${ids.length}-grappes-${date}.json`;
  }

  async function run() {
    exporting.value = true;
    message.value = null;
    opts.error.value = null;
    try {
      const { ids } = scope.value;
      const path = await api.exportClustersForAudit(ids, fileName(ids));
      // Annuler la boîte de dialogue n'est pas une erreur : pas de bandeau
      // rouge pour un geste délibéré.
      message.value = path ? `Exporté : ${path}` : null;
    } catch (e) {
      opts.error.value = String(e);
    } finally {
      exporting.value = false;
    }
  }

  return { selection, toggle, scope, exporting, message, run };
}
