// Konva peint sur un canvas, pas du DOM : il ne suit pas les classes Tailwind
// tout seul. On relit les variables CSS du thème (déjà basculées par
// syncThemeWithSystem sur <html>) à chaque changement système, pour que le
// dessin change de palette sans jamais coder une couleur en dur.

import { computed, onScopeDispose, ref, type ComputedRef } from "vue";
import type { ThemeColors } from "../types";

function cssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

export function useThemeColors(): ComputedRef<ThemeColors> {
  const darkModeQuery = window.matchMedia("(prefers-color-scheme: dark)");
  const themeTick = ref(0);

  function onDarkModeChange() {
    // S'exécute après le handler de syncThemeWithSystem (enregistré avant, au
    // niveau main.ts) : la classe .dark est déjà à jour quand on relit les vars.
    themeTick.value++;
  }

  darkModeQuery.addEventListener("change", onDarkModeChange);
  onScopeDispose(() => darkModeQuery.removeEventListener("change", onDarkModeChange));

  return computed<ThemeColors>(() => {
    void themeTick.value; // dépendance réactive volontaire, cf. commentaire ci-dessus
    return {
      orientation: cssVar("--zone-orientation"),
      anchor: cssVar("--zone-anchor"),
      latch: cssVar("--zone-latch"),
      pivot: cssVar("--zone-pivot"),
      lift: cssVar("--zone-lift"),
      alarm: cssVar("--status-alarm"),
      border: cssVar("--border"),
      muted: cssVar("--muted"),
      mutedForeground: cssVar("--muted-foreground"),
      card: cssVar("--card"),
    };
  });
}
