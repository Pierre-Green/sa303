// Types internes au navigateur d'équipement : de quoi décrire une ligne de
// fiche technique, rien de métier.

export type Category = "enceintes" | "bumpers";

/** Une ligne de fiche : libellé, valeur déjà formatée, bulle d'aide éventuelle. */
export interface SpecRow {
  label: string;
  value: string;
  tip?: string;
}
