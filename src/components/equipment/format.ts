// Mise en forme des valeurs affichées. Aucun calcul : on formate ce que Rust
// ou le JSON du catalogue ont fourni.

import type { Vec2 } from "@/lib/types";

export function fmt(v: number, d = 2): string {
  return v.toFixed(d);
}

export function fmtMm(v: number, d = 1): string {
  return `${fmt(v, d)} mm`;
}

export function fmtDeg(v: number, d = 1): string {
  return `${fmt(v, d)}°`;
}

export function fmtPoint(p: Vec2 | [number, number], d = 2): string {
  const [x, y] = Array.isArray(p) ? p : [p.x, p.y];
  return `${fmt(x, d)} ; ${fmt(y, d)}`;
}

export function yesNo(v: boolean): string {
  return v ? "oui" : "non";
}
