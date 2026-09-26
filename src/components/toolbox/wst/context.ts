// Contexte partagé par les panneaux de saisie du calculateur WST. Les cartes de
// résultats, elles, reçoivent le rapport par props : elles ne font que le lire.

import { inject, provide, type InjectionKey } from "vue";
import type { WstCalculator } from "./composables/useWstCalculator";

const WST_KEY: InjectionKey<WstCalculator> = Symbol("wst-calculator");

export function provideWst(ctx: WstCalculator): void {
  provide(WST_KEY, ctx);
}

export function useWst(): WstCalculator {
  const ctx = inject(WST_KEY, null);
  if (!ctx) throw new Error("useWst() hors d'un WstCalculator");
  return ctx;
}
