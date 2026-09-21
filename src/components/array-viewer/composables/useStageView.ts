// Cadrage, zoom molette, panoramique et lecture de la position du curseur.
//
// Le cadrage est basé sur la géométrie RENDUE (Konva `getClientRect`, qui
// applique déjà les rotations), pas sur une estimation faite à la main : ça
// évite tout calcul de coin de trapèze tourné en TypeScript et reste exact quel
// que soit l'angle.

import { ref, type Ref } from "vue";
import type Konva from "konva";
import type { Vec2 } from "@/lib/types";
import type { KonvaNodeRef } from "../types";
import type { Scale } from "./useScale";

const PADDING = 48;
const WHEEL_STEP = 1.05;

interface StageViewOptions {
  stageRef: Ref<KonvaNodeRef<Konva.Stage>>;
  /** Groupe porteur du cadrage (échelle + décalage). */
  fitGroupRef: Ref<KonvaNodeRef<Konva.Group>>;
  /** Contenu qui dicte le cadrage — hors pointillés CG/accroche. */
  measureGroupRef: Ref<KonvaNodeRef<Konva.Group>>;
  size: Ref<{ width: number; height: number }>;
  scale: Scale;
  referenceDepth: Ref<number> | { value: number };
  /** Ordonnée du sol (altitude 0) dans le repère de dessin local. */
  groundLocalY: () => number;
}

export function useStageView(opts: StageViewOptions) {
  const { stageRef, fitGroupRef, measureGroupRef, size, scale } = opts;

  /** Le sol en coordonnées de stage. Il n'est pas mesuré sur le rendu : le sol
   * est une altitude, pas le bas d'un caisson. On applique donc simplement au
   * `y` local de l'altitude 0 la transform que le cadrage vient de poser. */
  const groundLineY = ref<number | null>(null);
  /** Décalage vertical posé par le cadrage, en coordonnées de stage. De quoi
   * situer n'importe quelle autre altitude dessinée hors du groupe cadré — la
   * ligne d'écoute, elle, bouge sans qu'on recadre. */
  const fitOffsetY = ref<number | null>(null);
  /** Sommet des pointillés CG/accroche, en coordonnées locales (avant mise à
   * l'échelle) — juste au-dessus du contenu réellement mesuré, jamais un
   * multiple arbitraire qui fausserait le cadrage. */
  const dashTopY = ref<number | null>(null);
  const cursorPos = ref<Vec2 | null>(null);
  /** Portion du repère de stage réellement visible dans le cadre. Le sol est
   * dessiné hors du groupe cadré : sans ça, ses hachures s'arrêteraient là où
   * le cadre était au montage et laisseraient un trou dès qu'on déplace la vue. */
  const viewport = ref({ x: 0, y: 0, width: 0, height: 0 });

  function updateViewport() {
    const stage = stageRef.value?.getNode();
    if (!stage) return;
    const k = Math.max(stage.scaleX(), 1e-6);
    viewport.value = {
      x: -stage.x() / k,
      y: -stage.y() / k,
      width: size.value.width / k,
      height: size.value.height / k,
    };
  }

  /** Centre et met à l'échelle le contenu pour qu'il tienne dans le cadre, à
   * partir de sa boîte englobante réellement rendue (post-rotation). Positionne
   * aussi la ligne de sol tangente au bas des enceintes, mesurée pareil. */
  function fitContent() {
    const fitGroup = fitGroupRef.value?.getNode();
    const measureGroup = measureGroupRef.value?.getNode();
    const stage = stageRef.value?.getNode();
    if (!fitGroup || !measureGroup || !stage) return;

    fitGroup.scale({ x: 1, y: 1 });
    fitGroup.position({ x: 0, y: 0 });

    // Mesuré hors pointillés CG/accroche : eux seuls ne doivent pas dicter le cadrage.
    const rect = measureGroup.getClientRect();
    if (rect.width <= 0 || rect.height <= 0) return;
    dashTopY.value = rect.y - opts.referenceDepth.value * 0.4;

    const availW = Math.max(size.value.width - PADDING * 2, 10);
    const availH = Math.max(size.value.height - PADDING * 2, 10);
    const fitted = Math.min(availW / rect.width, availH / rect.height);

    const offsetY = (size.value.height - rect.height * fitted) / 2 - rect.y * fitted;
    fitGroup.scale({ x: fitted, y: fitted });
    fitGroup.position({
      x: (size.value.width - rect.width * fitted) / 2 - rect.x * fitted,
      y: offsetY,
    });
    scale.fitScale.value = fitted;

    fitOffsetY.value = offsetY;
    groundLineY.value = offsetY + opts.groundLocalY() * fitted;
    updateViewport();
    stage.batchDraw();
  }

  /** Remet le zoom molette et le panoramique à zéro. Le cadrage automatique
   * travaille sur le groupe interne : sans ce reset, la vue gardait le zoom et
   * le déplacement choisis pour la grappe précédente, et on atterrissait hors
   * champ en changeant de grappe. */
  function resetView() {
    const stage = stageRef.value?.getNode();
    if (!stage) return;
    stage.scale({ x: 1, y: 1 });
    stage.position({ x: 0, y: 0 });
    scale.stageScale.value = 1;
    updateViewport();
  }

  // Position du curseur en mm modèle (x = arrière, y = haut) : on repasse du
  // repère écran au repère local du groupe cadré via l'inverse de sa transform
  // absolue — elle compose déjà le pan/zoom du stage et la mise à l'échelle du
  // cadrage, donc aucun calcul de zoom à refaire à la main.
  function updateCursorPos() {
    const stage = stageRef.value?.getNode();
    const fitGroup = fitGroupRef.value?.getNode();
    const pointer = stage?.getPointerPosition();
    if (!stage || !fitGroup || !pointer) {
      cursorPos.value = null;
      return;
    }
    const local = fitGroup.getAbsoluteTransform().copy().invert().point(pointer);
    cursorPos.value = { x: local.x, y: -local.y };
  }

  function clearCursorPos() {
    cursorPos.value = null;
  }

  function handleWheel(e: { evt: WheelEvent }) {
    e.evt.preventDefault();
    const stage = stageRef.value?.getNode();
    if (!stage) return;
    const pointer = stage.getPointerPosition();
    if (!pointer) return;

    const oldScale = stage.scaleX();
    const mousePointTo = {
      x: (pointer.x - stage.x()) / oldScale,
      y: (pointer.y - stage.y()) / oldScale,
    };
    const newScale = e.evt.deltaY > 0 ? oldScale / WHEEL_STEP : oldScale * WHEEL_STEP;
    stage.scale({ x: newScale, y: newScale });
    stage.position({
      x: pointer.x - mousePointTo.x * newScale,
      y: pointer.y - mousePointTo.y * newScale,
    });
    // Réactif, pas seulement posé sur le nœud Konva : c'est lui qui pilote
    // `px`, donc les tailles d'annotation doivent se recalculer au même instant.
    scale.stageScale.value = newScale;
    stage.batchDraw();
    updateViewport();
    updateCursorPos();
  }

  /** Suivi du panoramique : le stage est `draggable`, donc sa position change
   * sans passer par aucune de nos fonctions. */
  function onDragMove() {
    updateViewport();
    updateCursorPos();
  }

  function setPointerCursor(pointer: boolean) {
    const container = stageRef.value?.getNode()?.container();
    if (container) container.style.cursor = pointer ? "pointer" : "default";
  }

  return {
    groundLineY,
    fitOffsetY,
    dashTopY,
    cursorPos,
    viewport,
    fitContent,
    resetView,
    updateCursorPos,
    clearCursorPos,
    handleWheel,
    onDragMove,
    setPointerCursor,
  };
}
