// La chaîne d'enceintes : compatibilités déclarées entre positions voisines,
// ajout/retrait, et les lignes affichées dans l'ordre de montage.

import { computed, watch } from "vue";
import { useSpeakerModelsStore } from "@/stores/speakerModels";
import { speakerDisplayNumber } from "@/lib/display";
import type { FormState, SpeakerRow } from "../types";
import type { ClusterResultState } from "./useClusterResult";

export function useSpeakerChain(form: FormState, res: ClusterResultState) {
  const speakers = useSpeakerModelsStore();

  function modelById(id: string) {
    return speakers.items.find((s) => s.id === id) ?? null;
  }

  /** L'enceinte qui porte le bumper : celle du haut en vol, celle du bas en
   * stack — c'est sa compatibilité bumper qui compte, comme côté solveur. */
  const referenceModel = computed(() => {
    const ids = form.speakerModelIds;
    const id = form.compartment === "flown" ? ids[0] : ids[ids.length - 1];
    return id ? modelById(id) : null;
  });

  /** Enceintes accrochables sous celle de la position précédente, pour le
   * compartiment courant. La première position n'a pas de contrainte :
   * n'importe quel modèle peut ouvrir la chaîne. */
  function compatibleAt(index: number) {
    if (index === 0) return speakers.items;
    const above = modelById(form.speakerModelIds[index - 1]);
    if (!above) return [];
    const flown = form.compartment === "flown";
    const allowedIds = new Set(
      above.compatibleBelow.filter((c) => (flown ? c.flown : c.stacked)).map((c) => c.speakerModelId),
    );
    return speakers.items.filter((s) => allowedIds.has(s.id));
  }

  /** Modèles qui déclarent `lowerId` accrochable sous eux, pour le
   * compartiment courant — l'autre sens de `compatibleAt`, utile quand on
   * ajoute une enceinte AU-DESSUS d'une chaîne existante (empilage d'un stack). */
  function acceptingBelow(lowerId: string) {
    const flown = form.compartment === "flown";
    return speakers.items.filter((s) =>
      s.compatibleBelow.some((c) => c.speakerModelId === lowerId && (flown ? c.flown : c.stacked)),
    );
  }

  /** Repose la chaîne sur des jonctions déclarées : dès qu'une position n'est
   * plus accrochable sous celle du dessus (changement de modèle ou de
   * compartiment), on retombe sur la première compatible plutôt que de laisser
   * une grappe que le solveur refusera. */
  function realign() {
    for (let i = 1; i < form.speakerModelIds.length; i += 1) {
      const allowed = compatibleAt(i);
      if (!allowed.some((s) => s.id === form.speakerModelIds[i])) {
        form.speakerModelIds[i] = allowed[0]?.id ?? form.speakerModelIds[i - 1];
      }
    }
  }

  watch(
    () => [...form.speakerModelIds, form.compartment],
    () => realign(),
  );

  function add() {
    if (form.compartment === "stacked") {
      // On empile par le haut : la nouvelle enceinte entre en tête de chaîne,
      // et c'est elle qui doit accepter l'ancienne tête en dessous.
      const below = form.speakerModelIds[0] ?? "";
      form.splays.unshift(form.splays[0] ?? 0);
      const accepting = acceptingBelow(below);
      form.speakerModelIds.unshift(
        accepting.some((s) => s.id === below) ? below : (accepting[0]?.id ?? below),
      );
    } else {
      // On descend la grappe : la nouvelle enceinte va en bas de chaîne, il
      // faut que celle du dessus l'accepte.
      form.splays.push(form.splays[form.splays.length - 1] ?? 0);
      const index = form.speakerModelIds.length;
      const above = form.speakerModelIds[index - 1] ?? "";
      form.speakerModelIds.push(above);
      const allowed = compatibleAt(index);
      if (!allowed.some((s) => s.id === above)) {
        form.speakerModelIds[index] = allowed[0]?.id ?? above;
      }
    }
    realign();
  }

  function remove(dataIndex: number, splayIndex: number) {
    if (form.splays.length <= 1) return;
    form.splays.splice(splayIndex, 1);
    form.speakerModelIds.splice(dataIndex, 1);
    realign();
  }

  /** Listées dans l'ordre de montage : du haut vers le bas en vol (on part de
   * l'accroche), du bas vers le haut en stack (on part du sol). `dataIndex`
   * reste l'index dans la chaîne, toujours ordonnée du haut vers le bas —
   * l'inversion est purement d'affichage. */
  const rows = computed<SpeakerRow[]>(() => {
    const n = form.splays.length + 1;
    const list = Array.from({ length: n }, (_, dataIndex): SpeakerRow => {
      const label = `Enceinte ${speakerDisplayNumber(dataIndex, n, form.compartment)}`;
      if (form.compartment === "flown" && dataIndex === 0) {
        return { label, kind: "flown-reference", dataIndex, splayIndex: null };
      }
      if (form.compartment === "stacked" && dataIndex === n - 1) {
        return { label, kind: "stack-reference", dataIndex, splayIndex: null };
      }
      const splayIndex = form.compartment === "flown" ? dataIndex - 1 : dataIndex;
      return { label, kind: "splay", dataIndex, splayIndex };
    });
    return form.compartment === "stacked" ? list.reverse() : list;
  });

  /** Message d'avertissement acoustique d'une jonction, ou `null` si elle est
   * dans la plage recommandée (ou qu'aucune n'est déclarée). Simple lecture du
   * résultat calculé côté Rust : aucun jugement acoustique côté front. */
  function acousticWarning(jointIndex: number): string | null {
    const joint = res.result.value?.joints[jointIndex];
    if (!joint || joint.acousticallyOptimal) return null;
    const range = joint.recommendedSplayRangeDeg;
    if (!range) return null;
    const target = range[0] === range[1] ? `${range[0]}°` : `${range[0]}° à ${range[1]}°`;
    return `Jonction non optimale acoustiquement : ${target} recommandé(s) entre ces deux enceintes.`;
  }

  /** Grille de splay percée dans l'enceinte au-dessus de la jonction. */
  function splayGridAt(splayIndex: number) {
    return modelById(form.speakerModelIds[splayIndex])?.splayGrid ?? [];
  }

  return { referenceModel, compatibleAt, rows, add, remove, acousticWarning, splayGridAt };
}
