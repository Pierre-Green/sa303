// Fiche technique d'une enceinte, telle que déclarée dans son JSON. Mise en
// lignes uniquement : aucune valeur n'est recalculée ici.

import { computed, type Ref } from "vue";
import type { SpeakerModel } from "@/lib/types";
import type { SpecRow } from "../types";
import { fmt, fmtDeg, fmtMm, fmtPoint } from "../format";

export function useSpeakerSpecs(speaker: Ref<SpeakerModel | null>) {
  const general = computed<SpecRow[]>(() => {
    const s = speaker.value;
    if (!s) return [];
    return [
      { label: "Masse", value: `${fmt(s.massKg, 1)} kg` },
      { label: "Profondeur", value: fmtMm(s.depth) },
      { label: "Hauteur", value: fmtMm(s.height) },
      {
        label: "Dièdre",
        value: fmtDeg(s.totalVerticalAngle),
        tip: "Angle total entre faces haute et basse de l'enceinte.",
      },
      { label: "CG (x ; y)", value: `${fmtPoint(s.cg)} mm`, tip: "Centre de gravité, repère enceinte." },
      {
        label: "Grille de splay",
        value: s.splayGrid.map((a) => `${a}°`).join(" · "),
        tip: "Angles percés dans la couronne : les seuls splays possibles.",
      },
      { label: "Splay trou de châssis", value: fmtDeg(s.frameHoleSplay) },
      {
        label: "Face arrière (x)",
        value: fmtMm(s.rearFaceX),
        tip: "Abscisse de la face arrière du caisson : le bord arrière de la barre ne doit pas la franchir.",
      },
    ];
  });

  const hinge = computed<SpecRow[]>(() => {
    const h = speaker.value?.hinge;
    if (!h) return [];
    return [
      { label: "Position (x ; y)", value: `${fmtPoint(h)} mm` },
      {
        label: "Séparation au splay 0",
        value: fmtMm(h.jointSeparation, 2),
        tip: "Distance verticale centre à centre au splay 0 : 2·y + entraxe de bielle.",
      },
      {
        label: "Pince transversale",
        value: fmtMm(h.edgePerp, 2),
        tip: "Recul du trou de charnière depuis la face avant (e⊥, EN 1993-1-8 t.3.9).",
      },
    ];
  });

  const pair = computed<SpecRow[]>(() => {
    const s = speaker.value;
    if (!s) return [];
    return [
      {
        label: "Ancrage (r ; θ)",
        value: `${fmtMm(s.anchor.radius, 3)} ; ${fmtDeg(s.anchor.angleDeg, 4)}`,
      },
      {
        label: "Verrou (r ; θ)",
        value: `${fmtMm(s.latch.radius, 3)} ; ${fmtDeg(s.latch.angleDeg, 4)}`,
      },
      {
        label: "Entraxe ancrage-verrou",
        value: fmtMm(s.latchOffset, 2),
        tip: "Bras du couple qui reprend le moment de la barre arrière.",
      },
    ];
  });

  const crown = computed<SpecRow[]>(() => {
    const c = speaker.value?.crown;
    if (!c) return [];
    return [
      { label: "Rayon", value: fmtMm(c.radius, 2) },
      { label: "Retrait couronne int.", value: fmtMm(c.delta, 2) },
      { label: "Angle splay 0", value: fmtDeg(c.splay0Angle, 3) },
      {
        label: "Splays couronne int.",
        value: c.innerSplays.length ? c.innerSplays.map((a) => `${a}°`).join(" · ") : "—",
        tip: "Splays percés sur la rangée intérieure. Déclarés dans le plan, jamais déduits.",
      },
    ];
  });

  const rearBar = computed<SpecRow[]>(() => {
    const b = speaker.value?.rearBar;
    if (!b) return [];
    return [
      { label: "Épaisseur", value: fmtMm(b.thickness) },
      { label: "Longueur", value: fmtMm(b.length) },
      {
        label: "Profil de largeur",
        value: b.widthProfile.map(([x, w]) => `${fmt(x, 0)}→${fmt(w, 0)}`).join(" · "),
        tip: "[abscisse depuis le petit bout → largeur], interpolé linéairement. L'élargissement est entièrement vers l'avant.",
      },
      { label: "Axe → bord arrière", value: fmtMm(b.rearEdgeOffset) },
      { label: "Ø trous", value: fmtMm(b.holeDiameter, 2) },
      { label: "Trou verrou", value: `${fmtPoint(b.holes.latch, 3)} mm` },
      { label: "Trou ancrage", value: `${fmtPoint(b.holes.anchor, 3)} mm` },
      {
        label: "Trou couronne int. (660)",
        value: `${fmtPoint(b.holes.up660, 3)} mm`,
        tip: "Splays impairs. Déporté latéralement : l'effort de couronne n'y est pas sur l'axe.",
      },
      { label: "Trou couronne ext. (680)", value: `${fmtPoint(b.holes.up680, 3)} mm` },
      { label: "Masse", value: `${fmt(b.massKg, 2)} kg` },
      { label: "fy", value: `${fmt(b.yieldStrength, 0)} MPa` },
      { label: "fu", value: `${fmt(b.ultimateStrength, 0)} MPa` },
    ];
  });

  const acoustics = computed<SpecRow[]>(() => {
    const a = speaker.value?.acoustics;
    if (!a) return [];
    const rows: SpecRow[] = [
      { label: "Fs", value: `${fmt(a.fs, 0)} Hz` },
      { label: "Directivité horizontale", value: fmtDeg(a.directivityHorizontal, 0) },
      {
        label: "Directivité verticale",
        value: fmtDeg(a.directivityVertical, 1),
        tip: "Ouverture d'une caisse seule. Guide à courbure constante : secteur rayonné, donc splay maximal exploitable. 0 pour un front isophase.",
      },
      {
        label: "Front du guide",
        value: a.wgFront === "isophase" ? "isophase (plan)" : "courbure constante",
      },
      {
        label: "Hauteur de bouche",
        value: a.wgOutputHeight > 0 ? fmtMm(a.wgOutputHeight) : "non renseignée",
        tip: "Le D des critères WST.",
      },
      { label: "Niveau à mi-secteur", value: `${fmt(a.wgLevelAtHalfCoverageDb, 1)} dB` },
      { label: "Secteur isophase", value: fmtDeg(a.wgIsophaseSectorDeg, 1) },
    ];
    const m = a.guideMeasurement;
    if (m) {
      rows.push(
        { label: "Mesure : D identifié", value: fmtMm(m.acousticMouthHeightMm) },
        {
          label: "Mesure : rayon du front",
          value: m.wavefrontRadiusM === null ? "plan" : `${fmt(m.wavefrontRadiusM, 3)} m`,
        },
        {
          label: "Mesure : niveau au bord",
          value: m.edgeLevelDb.map(([f, db]) => `${fmt(f, 0)} Hz ${fmt(db, 1)} dB`).join(" · ") || "—",
        },
      );
    }
    return rows;
  });

  return { general, hinge, pair, crown, rearBar, acoustics };
}
