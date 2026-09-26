// Contexte partagé par les panneaux de l'éditeur : fourni une fois par
// `ClusterEditor.vue`, injecté par chaque panneau — plutôt qu'une douzaine de
// props et d'événements redescendus à l'identique. L'état reste local à
// l'éditeur : il n'a rien de global, d'où un provide/inject plutôt qu'un store.

import { inject, provide, type InjectionKey } from "vue";
import type { ClusterEditor } from "./composables/useClusterEditor";

const EDITOR_KEY: InjectionKey<ClusterEditor> = Symbol("cluster-editor");

export function provideEditor(ctx: ClusterEditor): void {
  provide(EDITOR_KEY, ctx);
}

export function useEditor(): ClusterEditor {
  const ctx = inject(EDITOR_KEY, null);
  if (!ctx) throw new Error("useEditor() hors d'un ClusterEditor");
  return ctx;
}
