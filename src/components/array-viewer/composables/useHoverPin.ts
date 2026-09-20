// Survol = aperçu (pas besoin de cliquer) ; clic = épinglé, pour pouvoir
// déplacer la souris jusque dans la popup et sélectionner le texte sans qu'elle
// se referme. Un petit délai à la sortie du survol laisse le temps d'atteindre
// la popup (élément DOM, distinct des formes Konva) ; épinglé, on n'auto-referme
// plus du tout.

import { computed, onScopeDispose, ref } from "vue";

/** Sentinelle réutilisant tout le mécanisme survol/épinglage des enceintes
 * (mêmes délais, même logique) pour le bumper, sans dupliquer cet état. */
export const BUMPER_IDX = -1;

const HIDE_DELAY_MS = 200;

export interface HoverAnchor {
  idx: number;
  x: number;
  y: number;
}

interface HoverPinOptions {
  /** Position du pointeur dans le stage, ou `null` hors canvas. */
  pointerPosition: () => { x: number; y: number } | null;
  /** Repli quand le pointeur est inconnu (clavier, épinglage programmatique). */
  fallbackPosition: () => { x: number; y: number };
  setPointerCursor: (pointer: boolean) => void;
}

export function useHoverPin(opts: HoverPinOptions) {
  const anchor = ref<HoverAnchor | null>(null);
  const pinnedIdx = ref<number | null>(null);
  let hideTimer: ReturnType<typeof setTimeout> | null = null;

  function clearHideTimer() {
    if (hideTimer !== null) {
      clearTimeout(hideTimer);
      hideTimer = null;
    }
  }
  onScopeDispose(clearHideTimer);

  function show(idx: number) {
    clearHideTimer();
    const pointer = opts.pointerPosition() ?? opts.fallbackPosition();
    anchor.value = { idx, x: pointer.x, y: pointer.y };
  }

  function scheduleHide() {
    if (pinnedIdx.value !== null) return;
    clearHideTimer();
    hideTimer = setTimeout(() => {
      anchor.value = null;
    }, HIDE_DELAY_MS);
  }

  function onEnter(idx: number) {
    opts.setPointerCursor(true);
    show(idx);
  }

  function onLeave() {
    opts.setPointerCursor(false);
    scheduleHide();
  }

  function onClick(idx: number) {
    if (pinnedIdx.value === idx) {
      pinnedIdx.value = null;
      scheduleHide();
    } else {
      pinnedIdx.value = idx;
      show(idx);
    }
  }

  function close() {
    pinnedIdx.value = null;
    clearHideTimer();
    anchor.value = null;
  }

  /** Vrai dès que l'un des index donnés est survolé ou épinglé : c'est à ce
   * moment-là qu'on détaille ce qu'il porte. */
  function isActive(...indices: number[]): boolean {
    return (
      (anchor.value !== null && indices.includes(anchor.value.idx)) ||
      (pinnedIdx.value !== null && indices.includes(pinnedIdx.value))
    );
  }

  return {
    anchor: computed(() => anchor.value),
    pinnedIdx: computed(() => pinnedIdx.value),
    isActive,
    onEnter,
    onLeave,
    onClick,
    close,
    clearHideTimer,
    scheduleHide,
  };
}

export type HoverPin = ReturnType<typeof useHoverPin>;
