// Les deux textes de référence dont sont tirées les formules du calculateur.

export interface WstDocument {
  label: string;
  detail: string;
  /** Chemin servi depuis `public/` : valable en développement comme dans
   * l'application empaquetée, où le dossier est copié à la racine du bundle. */
  path: string;
}

export const WST_DOCUMENTS: WstDocument[] = [
  {
    label: "Wavefront Sculpture Technology",
    detail:
      "Urban, Heil & Bauman — AES 5488. L'article d'origine : les cinq critères, leurs démonstrations et les tableaux que reproduit ce calculateur.",
    path: "/wst/Wavefront_Sculpture_Technology_convention.pdf",
  },
  {
    label: "La technologie WST : sculpture du front d'onde",
    detail:
      "Gramondo & Heil, L-Acoustics — Acoustique & Techniques n° 29. Reprise en français, plus courte et plus imagée.",
    path: "/wst/78_09943.pdf",
  },
];
