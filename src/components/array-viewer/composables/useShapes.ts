// Fabriques de formes partagées : une flèche d'effort, un marqueur de trou, un
// entraxe, un anneau. Jonctions, paire de barre de bumper et pions dessinent
// tous la même quincaillerie — ces fabriques sont ce qui garantit qu'ils la
// dessinent pareil.

import type { Vec2 } from "@/lib/types";
import { useViewer } from "../context";
import { localPoints, magnitude, toLocal } from "../geometry";

export function useShapes() {
  const { colors, px, annotation, referenceDepth, maxForceN } = useViewer();

  /** Longueur d'une flèche à pleine échelle. */
  const forceRefLength = () => annotation(referenceDepth.value * 0.85);

  /** Flèche d'effort partant de son point d'application, longueur
   * proportionnelle à l'intensité sur l'échelle commune. */
  function arrow(from: Vec2, force: Vec2, color: string) {
    const mag = magnitude(force);
    const lenMm = (mag / maxForceN.value) * forceRefLength();
    const dir = mag > 0 ? { x: force.x / mag, y: force.y / mag } : { x: 0, y: 0 };
    const to = { x: from.x + dir.x * lenMm, y: from.y + dir.y * lenMm };
    return {
      points: localPoints(from, to),
      stroke: color,
      fill: color,
      strokeWidth: px(2.5),
      pointerLength: px(8),
      pointerWidth: px(8),
    };
  }

  function hole(p: Vec2, color: string, radiusPx = 3) {
    const s = toLocal(p);
    return {
      x: s.x,
      y: s.y,
      radius: px(radiusPx),
      fill: colors.value.card,
      stroke: color,
      strokeWidth: px(1.5),
    };
  }

  /** Trait fin entre les deux goupilles d'une paire : il matérialise l'entraxe,
   * bras du couple qui reprend le moment de barre. */
  function span(a: Vec2, b: Vec2, color: string) {
    return {
      points: localPoints(a, b),
      stroke: color,
      strokeWidth: px(1),
      opacity: 0.5,
    };
  }

  function ring(p: Vec2) {
    const s = toLocal(p);
    return { x: s.x, y: s.y, radius: px(8), stroke: colors.value.alarm, strokeWidth: px(1.5) };
  }

  /** Étiquette posée à côté d'un point du modèle. `dx`/`dy` sont des pixels
   * écran : un décalage de libellé se pense à l'écran, pas en millimètres. */
  function label(
    p: Vec2,
    text: string,
    color: string,
    {
      dx = 0,
      dy = 0,
      size = 12,
      weight = "600",
    }: { dx?: number; dy?: number; size?: number; weight?: string } = {},
  ) {
    const s = toLocal(p);
    return {
      x: s.x + px(dx),
      y: s.y + px(dy),
      text,
      fontSize: px(size),
      fill: color,
      fontStyle: weight,
    };
  }

  return { arrow, hole, span, ring, label, forceRefLength };
}
