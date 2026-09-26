// Champ numérique borné à une plage que le solveur renvoie.

import { watch } from "vue";

/** Les flèches (souris et clavier) s'arrêtent aux bornes grâce à `min`/`max` ;
 * pendant la frappe, seule une valeur dans la plage part au solveur (un « 1 »
 * ou un « 17 » intermédiaire n'est pas ramené de force à la borne) ; en sortie
 * de champ ou sur Entrée, la valeur est ramenée dans la plage. Quand la plage
 * bouge (assiette modifiée), la valeur déjà saisie y est ramenée, pour que le
 * champ affiche ce que le solveur calcule. */
export function useBoundedNumber(
  range: () => [number, number] | null,
  get: () => number | null,
  set: (v: number) => void,
) {
  const clamp = (v: number) => {
    const r = range();
    return r ? Math.min(r[1], Math.max(r[0], v)) : v;
  };
  function onInput(v: string | number) {
    if (v === "") return;
    const n = Number(v);
    if (Number.isFinite(n) && clamp(n) === n) set(n);
  }
  /** Le champ n'est jamais recréé (une `key` changeante lui faisait perdre
   * le focus à chaque cran de flèche, souris ou clavier) : la valeur bornée
   * est réécrite directement dans l'élément, et seulement si elle diffère. */
  function onCommit(e: Event) {
    const el = e.target as HTMLInputElement;
    const n = Number(el.value);
    if (el.value === "" || !Number.isFinite(n)) return;
    const c = clamp(n);
    if (c !== n) el.value = String(c);
    if (c !== get()) set(c);
  }
  watch(range, (r) => {
    const v = get();
    if (r && v !== null && clamp(v) !== v) set(clamp(v));
  });
  return { onInput, onCommit };
}

export type BoundedNumberField = ReturnType<typeof useBoundedNumber>;

/** Arrondit une plage vers l'intérieur, pour que les bornes affichées dans le
 * champ restent valables. */
export function inwardRange(
  r: [number, number] | null | undefined,
  step: number,
): [number, number] | null {
  if (!r) return null;
  return [Math.ceil(r[0] / step) * step, Math.floor(r[1] / step) * step];
}
