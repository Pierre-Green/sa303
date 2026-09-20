// Seules conversions tolérées côté front (brief §1) : repère modèle -> repère
// de dessin, et radians -> degrés horaires de Konva. Aucune statique, aucune
// reconstruction de géométrie.

import type { Vec2 } from "@/lib/types";

/** Repère modèle (x = arrière, y = haut) -> repère de dessin local, mm = px,
 * un seul axe retourné. L'échelle réelle vient du cadrage impératif appliqué
 * sur le groupe englobant, jamais calculée à la main. */
export function toLocal(p: Vec2): Vec2 {
  return { x: p.x, y: -p.y };
}

/** `phi` (radians, sens trigonométrique, calculé par Rust) -> degrés horaires,
 * convention de rotation de Konva. */
export function toLocalRotationDeg(phiRad: number): number {
  return (-phiRad * 180) / Math.PI;
}

/** Le même point en coordonnées locales, aplati pour un `points:` Konva. */
export function localPoints(...pts: Vec2[]): number[] {
  return pts.flatMap((p) => {
    const l = toLocal(p);
    return [l.x, l.y];
  });
}

export function magnitude(v: Vec2): number {
  return Math.hypot(v.x, v.y);
}

export function midpoint(a: Vec2, b: Vec2): Vec2 {
  return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
}

export function add(a: Vec2, b: Vec2): Vec2 {
  return { x: a.x + b.x, y: a.y + b.y };
}
