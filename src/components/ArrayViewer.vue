<script setup lang="ts">
// Consomme un ClusterResult déjà calculé par sa303-core : aucun calcul de
// statique ou de géométrie ici. La seule exception tolérée (brief §1) est la
// mise à l'échelle pixels <-> modèle (zoom/pan/offset, ajustement au cadre),
// et le passage de `phi` (radians, calculé par Rust) à la convention
// degrés/sens horaire de Konva.
//
// Le cadrage (centrage, ligne de sol tangente au caisson le plus bas) est basé
// sur la géométrie RENDUE (Konva `getClientRect`, qui applique déjà les
// rotations), pas sur une estimation faite à la main : ça évite tout calcul de
// coin de trapèze tourné en TypeScript et reste exact quel que soit l'angle.

import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type Konva from "konva";
import type { ClusterResult, Compartment, Vec2 } from "@/lib/types";
import { speakerDisplayNumber } from "@/lib/display";

// Tout vient de `result` : silhouette par enceinte (une grappe est
// hétérogène) et masse totale déjà sommée côté Rust — le front ne recompose
// rien, il dessine (brief §1).
const props = defineProps<{
  result: ClusterResult;
  compartment: Compartment;
  /** Identifiant de la grappe affichée. C'est lui, et non `result`, qui dit
   * qu'on regarde autre chose : `result` est un objet neuf à chaque recalcul,
   * donc le surveiller ferait sauter le cadrage à chaque splay modifié. */
  viewKey?: string | null;
}>();

/** Longueur de référence pour les traits et flèches (mm) : l'étendue
 * avant-arrière de l'enceinte du haut. Pure échelle de rendu, pas de la
 * physique. */
const referenceDepth = computed(() => {
  const outline = props.result.speakers[0]?.outline;
  if (!outline) return 700;
  const xs = outline.map((p) => p.x);
  return Math.max(...xs) - Math.min(...xs);
});

type NodeRef<T extends Konva.Node> = { getNode: () => T } | null;

const containerEl = ref<HTMLDivElement | null>(null);
const stageRef = ref<NodeRef<Konva.Stage>>(null);
const fitGroupRef = ref<NodeRef<Konva.Group>>(null);
const measureGroupRef = ref<NodeRef<Konva.Group>>(null);
const speakersGroupRef = ref<NodeRef<Konva.Group>>(null);
const size = ref({ width: 800, height: 560 });
const groundLineY = ref<number | null>(null);
/** Sommet des pointillés CG/accroche, en coordonnées locales (avant mise à
 * l'échelle) — juste au-dessus du contenu réellement mesuré, jamais un
 * multiple arbitraire qui fausserait le cadrage. */
const dashTopY = ref<number | null>(null);
/** Échelle appliquée par le fit (mm -> px). Les traits, rayons et textes sont
 * définis à une taille écran constante en la divisant : sinon un grand stack
 * qui doit beaucoup rétrécir pour tenir dans le cadre rendrait tout illisible. */
const fitScale = ref(1);
/// Zoom molette du stage, tenu à part de `fitScale` : le premier est choisi par
/// l'utilisateur, le second par le cadrage automatique.
const stageScale = ref(1);
/// Convertit une taille voulue **en pixels écran** vers les unités du modèle.
///
/// Divise par les deux échelles qui séparent le modèle de l'écran : le cadrage
/// automatique (`fitScale`) et le zoom molette (`stageScale`). Sans le second,
/// flèches et étiquettes grossissaient avec le zoom — en entrant dans un détail,
/// les libellés finissaient par couvrir ce qu'ils annotaient. Avec, elles gardent
/// une taille constante à l'écran, donc elles rapetissent par rapport au modèle
/// à mesure qu'on zoome, ce qui est le comportement attendu d'une annotation.
function px(sizeAtScale1: number): number {
  const scale = Math.max(fitScale.value, 1e-6) * Math.max(stageScale.value, 1e-6);
  return sizeAtScale1 / scale;
}

/// Longueur d'annotation exprimée en unités modèle **à l'échelle de cadrage**,
/// ramenée à une taille d'écran constante sous le zoom.
///
/// Les flèches d'effort sont des annotations, pas des objets : leur longueur
/// code une intensité, pas une dimension. Les laisser suivre le modèle ferait
/// qu'en zoomant sur une jonction, sa flèche traverserait tout le cadre.
function annotation(lengthAtFit: number): number {
  return lengthAtFit / Math.max(stageScale.value, 1e-6);
}

let resizeObserver: ResizeObserver | null = null;
onMounted(() => {
  if (!containerEl.value) return;
  resizeObserver = new ResizeObserver((entries) => {
    const entry = entries[0];
    if (!entry) return;
    size.value = { width: entry.contentRect.width, height: entry.contentRect.height };
  });
  resizeObserver.observe(containerEl.value);
  void nextTick(fitContent);
});
onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  darkModeQuery.removeEventListener("change", onDarkModeChange);
});

// Konva peint sur un canvas, pas du DOM : il ne suit pas les classes Tailwind
// tout seul. On relit les variables CSS du thème (déjà basculées par
// syncThemeWithSystem sur <html>) à chaque changement système, pour que le
// dessin change de palette sans jamais coder une couleur en dur ici.
const darkModeQuery = window.matchMedia("(prefers-color-scheme: dark)");
const themeTick = ref(0);
function onDarkModeChange() {
  // S'exécute après le handler de syncThemeWithSystem (enregistré avant, au
  // niveau main.ts) : la classe .dark est déjà à jour quand on relit les vars.
  themeTick.value++;
}
darkModeQuery.addEventListener("change", onDarkModeChange);

function cssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

const colors = computed(() => {
  void themeTick.value; // dépendance réactive volontaire, cf. commentaire ci-dessus
  return {
    // Les trois perçages de la zone orientation ont chacun leur teinte : ils
    // sont sur des cercles différents et ne voient pas la même charge, les
    // confondre à l'écran revenait à les confondre tout court.
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

const PADDING = 48;

// Repère modèle (x = arrière, y = haut) -> repère de dessin local, mm = px,
// un seul axe retourné. L'échelle réelle vient du fit imperatif ci-dessous,
// appliqué sur le groupe englobant, jamais calculée à la main ici.
function toLocal(p: Vec2): Vec2 {
  return { x: p.x, y: -p.y };
}
function toLocalRotationDeg(phiRad: number): number {
  return (-phiRad * 180) / Math.PI;
}

/** Centre et met à l'échelle le contenu pour qu'il tienne dans le cadre, à
 * partir de sa boîte englobante réellement rendue (post-rotation). Positionne
 * aussi la ligne de sol tangente au bas des enceintes, mesurée de la même
 * façon. */
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
  dashTopY.value = rect.y - referenceDepth.value * 0.4;

  const availW = Math.max(size.value.width - PADDING * 2, 10);
  const availH = Math.max(size.value.height - PADDING * 2, 10);
  const scale = Math.min(availW / rect.width, availH / rect.height);

  const offsetX = (size.value.width - rect.width * scale) / 2 - rect.x * scale;
  const offsetY = (size.value.height - rect.height * scale) / 2 - rect.y * scale;
  fitGroup.scale({ x: scale, y: scale });
  fitGroup.position({ x: offsetX, y: offsetY });
  fitScale.value = scale;

  const speakersGroup = speakersGroupRef.value?.getNode();
  if (speakersGroup) {
    const speakersRect = speakersGroup.getClientRect();
    groundLineY.value = speakersRect.y + speakersRect.height + 14;
  }
  stage.batchDraw();
}

/** Remet le zoom molette et le panoramique à zéro. Le cadrage automatique
 * (`fitContent`) travaille sur le groupe interne : sans ce reset, la vue
 * gardait le zoom et le déplacement choisis pour la grappe précédente, et on
 * atterrissait hors champ en changeant de grappe. */
function resetView() {
  const stage = stageRef.value?.getNode();
  if (!stage) return;
  stage.scale({ x: 1, y: 1 });
  stage.position({ x: 0, y: 0 });
  stageScale.value = 1;
}

watch(
  () => [props.result, size.value.width, size.value.height],
  () => void nextTick(fitContent),
  { deep: true },
);

// Changer de grappe rend la vue précédente sans objet : on repart du cadrage
// initial, popup comprise — celle d'avant désignait une enceinte qui n'est plus
// celle-là. Réglé sur `viewKey` et non sur `result` : régler une grappe la
// recalcule en continu, et le zoom qu'on vient d'ajuster pour regarder un
// détail ne doit pas se défaire à chaque degré saisi.
watch(
  () => props.viewKey,
  () => {
    closePopup();
    resetView();
    void nextTick(fitContent);
  },
);

const stageConfig = computed(() => ({
  width: size.value.width,
  height: size.value.height,
  draggable: true,
}));

// Tracée hors du groupe cadré, donc en unités de stage : seul le zoom molette
// l'affecte, pas le cadrage. D'où la division par `stageScale` seule et non par
// `px`, qui compenserait aussi un cadrage qui ne s'applique pas ici.
const groundLineConfig = computed(() => {
  const k = Math.max(stageScale.value, 1e-6);
  return {
    points: [0, groundLineY.value ?? 0, size.value.width, groundLineY.value ?? 0],
    stroke: colors.value.border,
    strokeWidth: 2 / k,
    dash: [2 / k, 6 / k],
  };
});

function speakerGroupConfig(speaker: ClusterResult["speakers"][number]) {
  const p = toLocal(speaker.o);
  return { x: p.x, y: p.y, rotation: toLocalRotationDeg(speaker.phi) };
}

function outlinePointsOf(speaker: ClusterResult["speakers"][number]): number[] {
  return speaker.outline.flatMap((p) => [p.x, -p.y]);
}

function outlineConfigOf(speaker: ClusterResult["speakers"][number]) {
  return {
    points: outlinePointsOf(speaker),
    closed: true,
    fill: colors.value.muted,
    stroke: colors.value.border,
    strokeWidth: px(1),
  };
}

// Boîte englobante des points déjà dessinés (min/max, pas de trigonométrie) :
// sert à centrer le numéro d'enceinte dans SA silhouette quel que soit le splay.
/** Même numérotation que l'éditeur : en stack on compte depuis le sol. */
function displayNumber(idx: number): number {
  return speakerDisplayNumber(idx, props.result.speakers.length, props.compartment);
}

function speakerNumberLabelConfig(idx: number) {
  const pts = outlinePointsOf(props.result.speakers[idx]);
  const xs: number[] = [];
  const ys: number[] = [];
  for (let i = 0; i < pts.length; i += 2) {
    xs.push(pts[i]);
    ys.push(pts[i + 1]);
  }
  const b = {
    minX: Math.min(...xs),
    maxX: Math.max(...xs),
    minY: Math.min(...ys),
    maxY: Math.max(...ys),
  };
  return {
    x: b.minX,
    y: b.minY,
    width: b.maxX - b.minX,
    height: b.maxY - b.minY,
    text: String(displayNumber(idx)),
    fontSize: px(18),
    fill: colors.value.mutedForeground,
    fontStyle: "700",
    align: "center",
    verticalAlign: "middle",
    listening: false,
  };
}

/** Splay associé à une enceinte : celle qui la relie à l'enceinte du dessus
 * (repère enceinte supérieure), indépendant du compartiment — kinématique §4. */
function speakerSplayDeg(idx: number): number | null {
  return idx === 0 ? null : (props.result.joints[idx - 1]?.splayDeg ?? null);
}

/** Le joint dont cette enceinte est le flanc porteur, s'il y en a un. */
function speakerLoadedJoint(idx: number) {
  return props.result.joints.find((j) => j.loadedFlank === idx) ?? null;
}

/// La jonction dont CETTE enceinte est le caisson du **bas**, donc celle dont
/// elle porte la paire ancrage/verrou. La paire de la jonction `j` est toujours
/// sur l'enceinte `j + 1`, dans les deux compartiments.
///
/// Sans ça, la dernière enceinte d'une grappe suspendue n'affichait aucune
/// charge : le flanc « chargé » d'une jonction en vol est celui du **haut**,
/// donc elle n'était jamais trouvée — alors qu'elle porte bien deux goupilles
/// et que le viewer y dessine deux flèches.
function speakerPairJoint(idx: number) {
  return props.result.joints.find((j) => j.jointIndex === idx - 1) ?? null;
}

// Survol = aperçu (pas besoin de cliquer) ; clic = épinglé, pour pouvoir
// déplacer la souris jusque dans la popup et sélectionner le texte sans
// qu'elle se referme. Un petit délai à la sortie du survol laisse le temps
// d'atteindre la popup (élément DOM, distinct des formes Konva) ; épinglé,
// on n'auto-referme plus du tout.
interface SpeakerAnchor {
  idx: number;
  x: number;
  y: number;
}
const anchor = ref<SpeakerAnchor | null>(null);
const pinnedIdx = ref<number | null>(null);
let hideTimer: ReturnType<typeof setTimeout> | null = null;

function clearHideTimer() {
  if (hideTimer !== null) {
    clearTimeout(hideTimer);
    hideTimer = null;
  }
}

function showSpeaker(idx: number) {
  clearHideTimer();
  const stage = stageRef.value?.getNode();
  const pointer = stage?.getPointerPosition();
  anchor.value = { idx, x: pointer?.x ?? size.value.width / 2, y: pointer?.y ?? size.value.height / 2 };
}

function scheduleHide() {
  if (pinnedIdx.value !== null) return;
  clearHideTimer();
  hideTimer = setTimeout(() => {
    anchor.value = null;
  }, 200);
}

function onSpeakerMouseEnter(idx: number) {
  setPointerCursor(true);
  showSpeaker(idx);
}

function onSpeakerMouseLeave() {
  setPointerCursor(false);
  scheduleHide();
}

function onSpeakerClick(idx: number) {
  if (pinnedIdx.value === idx) {
    pinnedIdx.value = null;
    scheduleHide();
  } else {
    pinnedIdx.value = idx;
    showSpeaker(idx);
  }
}

function closePopup() {
  pinnedIdx.value = null;
  clearHideTimer();
  anchor.value = null;
}

function onStageClick(e: Konva.KonvaEventObject<MouseEvent>) {
  const stage = stageRef.value?.getNode();
  if (stage && e.target === stage) closePopup();
}

function setPointerCursor(pointer: boolean) {
  const stage = stageRef.value?.getNode();
  const container = stage?.container();
  if (container) container.style.cursor = pointer ? "pointer" : "default";
}

// Sentinelle réutilisant tout le mécanisme survol/épinglage des enceintes
// (mêmes délais, même logique) pour le bumper, sans dupliquer cet état.
const BUMPER_ANCHOR_IDX = -1;

function onBumperMouseEnter() {
  setPointerCursor(true);
  showSpeaker(BUMPER_ANCHOR_IDX);
}
function onBumperMouseLeave() {
  setPointerCursor(false);
  scheduleHide();
}
function onBumperClick() {
  onSpeakerClick(BUMPER_ANCHOR_IDX);
}

/** Altitude au-dessus du sol d'une ordonnée du repère global. Le décalage est
 * calculé côté Rust à partir de la hauteur de bumper déclarée : ici on ne fait
 * que l'ajouter. */
function elevationOf(y: number): number {
  return y + props.result.elevation.offsetMm;
}

const speakerPopupData = computed(() => {
  if (!anchor.value) return null;
  const idx = anchor.value.idx;

  if (idx === BUMPER_ANCHOR_IDX) {
    const bv = props.result.bumperView;
    if (!bv) return null;
    return {
      kind: "bumper" as const,
      pinned: pinnedIdx.value === BUMPER_ANCHOR_IDX,
      pickupOffsetMm: bv.pickupOffsetMm,
      barDeportMm: bv.barDeportMm,
      bumperBarExceeded: bv.bumperBarExceeded,
      supportForceN: bv.supportForceN,
      supportAngleDeg: bv.supportAngleDeg,
      pinPairMomentNm: bv.pinPairMomentNm,
      pinSpanMm: bv.pinSpanMm,
      orientationForceN: bv.orientationForceN,
      orientationAngleDeg: bv.orientationAngleDeg,
      pivotForceN: bv.pivotForceN,
      pivotAngleDeg: bv.pivotAngleDeg,
      bottomElevationMm: props.result.elevation.bumperBottomMm,
      pickupElevationMm: props.result.elevation.pickupMm,
    };
  }

  const speaker = props.result.speakers[idx];
  return {
    kind: "speaker" as const,
    number: displayNumber(idx),
    modelName: props.result.speakerNames[idx] ?? null,
    angleDeg: (speaker.phi * 180) / Math.PI,
    splay: speakerSplayDeg(idx),
    joint: speakerLoadedJoint(idx),
    pairJoint: speakerPairJoint(idx),
    // La barre du bumper est boulonnée sur la paire de CETTE enceinte : ses
    // deux goupilles sont les siennes, au même titre que celles d'une jonction.
    // (Les efforts de pion, eux, restent au bumper : ils sont à lui.)
    bumperBarPair: bumperBarPairLoads(idx),
    tie: tiePopupData(idx),
    bottomElevationMm: props.result.elevation.speakerBottomMm[idx],
    pinned: pinnedIdx.value === idx,
  };
});



/// La paire ancrage/verrou chargée par la barre du bumper, pour l'enceinte qui
/// la porte. `null` partout ailleurs, et en stack où il n'y a pas de barre.
function bumperBarPairLoads(idx: number) {
  const bv = props.result.bumperView;
  if (idx !== 0 || props.compartment !== "flown") return null;
  if (bv.fPairAnchorN === null || bv.fPairLatchN === null) return null;
  return {
    anchorN: bv.fPairAnchorN,
    anchorAngleDeg: bv.fPairAnchorAngleDeg ?? 0,
    latchN: bv.fPairLatchN,
    latchAngleDeg: bv.fPairLatchAngleDeg ?? 0,
    momentNm: bv.pairMomentNm ?? 0,
  };
}

/// La tirette s'accroche sur l'enceinte du **bas** de la grappe suspendue.
/// C'est donc dans sa fiche qu'on attend sa tension et sa direction : c'est
/// elle qui les encaisse.
function tiePopupData(idx: number) {
  const last = props.result.speakers.length - 1;
  if (props.compartment !== "flown" || idx !== last) return null;
  if (!props.result.tiePointGlobal || props.result.tieTensionN <= 0) return null;
  return {
    tensionKn: props.result.tieTensionN / 1000,
    angleDeg: props.result.tieDirectionAngleDeg,
  };
}

const speakerPopupStyle = computed(() => {
  if (!anchor.value) return {};
  const left = Math.min(Math.max(anchor.value.x + 12, 0), size.value.width - 230);
  const top = Math.min(Math.max(anchor.value.y + 12, 0), size.value.height - 170);
  return { left: `${left}px`, top: `${top}px` };
});

// Le bumper est déjà en repère global côté Rust (attaché à l'enceinte de
// référence, haut en vol / bas en stack) : ni rotation ni choix de côté ici.
const bumperOutlineConfig = computed(() => {
  const bv = props.result.bumperView;
  if (!bv) return null;
  const pts = bv.outlineGlobal.map(toLocal);
  return {
    points: pts.flatMap((p) => [p.x, p.y]),
    closed: true,
    fill: colors.value.muted,
    stroke: colors.value.border,
    strokeWidth: px(1),
  };
});

const bumperBarConfig = computed(() => {
  const bv = props.result.bumperView;
  if (!bv?.bumperBarStartGlobal || !bv.pickupGlobal) return null;
  const p0 = toLocal(bv.bumperBarStartGlobal);
  const p1 = toLocal(bv.pickupGlobal);
  return {
    points: [p0.x, p0.y, p1.x, p1.y],
    stroke: colors.value.lift,
    strokeWidth: px(4),
    lineCap: "round" as const,
  };
});

const cgLocal = computed(() => toLocal(props.result.cg));
const pickupLocal = computed(() =>
  props.result.pickupGlobal ? toLocal(props.result.pickupGlobal) : null,
);
const tiePointLocal = computed(() =>
  props.result.tiePointGlobal ? toLocal(props.result.tiePointGlobal) : null,
);
const tieEndLocal = computed(() => {
  if (!props.result.tiePointGlobal || !props.result.tieDirectionGlobal) return null;
  const end = {
    x: props.result.tiePointGlobal.x + props.result.tieDirectionGlobal.x * annotation(referenceDepth.value),
    y: props.result.tiePointGlobal.y + props.result.tieDirectionGlobal.y * annotation(referenceDepth.value),
  };
  return toLocal(end);
});

const cgDashConfig = computed(() => ({
  points: [cgLocal.value.x, cgLocal.value.y, cgLocal.value.x, dashTopY.value ?? cgLocal.value.y],
  stroke: colors.value.alarm,
  strokeWidth: px(1),
  dash: [px(3), px(4)],
  opacity: 0.7,
}));

const cgMarkerConfig = computed(() => ({
  x: cgLocal.value.x,
  y: cgLocal.value.y,
  radius: px(5),
  fill: colors.value.alarm,
}));

const cgLabelConfig = computed(() => ({
  x: cgLocal.value.x + px(10),
  y: cgLocal.value.y - px(6),
  text: `CG ${props.result.totalMassKg.toFixed(0)} kg`,
  fontSize: px(13),
  fill: colors.value.alarm,
  fontStyle: "600",
}));

const pickupDashConfig = computed(() => {
  if (!pickupLocal.value) return null;
  return {
    points: [
      pickupLocal.value.x,
      pickupLocal.value.y,
      pickupLocal.value.x,
      dashTopY.value ?? pickupLocal.value.y,
    ],
    stroke: colors.value.lift,
    strokeWidth: px(1.5),
    dash: [px(6), px(4)],
  };
});

const pickupMarkerConfig = computed(() => {
  if (!pickupLocal.value) return null;
  return {
    x: pickupLocal.value.x,
    y: pickupLocal.value.y,
    radius: px(6),
    stroke: colors.value.lift,
    strokeWidth: px(2.5),
    fill: colors.value.card,
  };
});

const tieArrowConfig = computed(() => {
  if (!tiePointLocal.value || !tieEndLocal.value) return null;
  return {
    points: [tiePointLocal.value.x, tiePointLocal.value.y, tieEndLocal.value.x, tieEndLocal.value.y],
    stroke: colors.value.lift,
    fill: colors.value.lift,
    strokeWidth: px(2.5),
    dash: [px(7), px(4)],
    pointerLength: px(8),
    pointerWidth: px(8),
  };
});

const tieLabelConfig = computed(() => {
  if (!tieEndLocal.value) return null;
  return {
    x: tieEndLocal.value.x,
    y: tieEndLocal.value.y + px(6),
    text: `tirette ${(props.result.tieTensionN / 1000).toFixed(2)} kN`,
    fontSize: px(12),
    fill: colors.value.lift,
  };
});

const tiePointMarkerConfig = computed(() => {
  if (!tiePointLocal.value) return null;
  return {
    x: tiePointLocal.value.x,
    y: tiePointLocal.value.y,
    radius: px(4),
    fill: colors.value.lift,
  };
});

const FORCE_REF_LENGTH_MM = computed(() => annotation(referenceDepth.value * 0.85));
// Échelle commune à toutes les flèches. Les efforts de paire y entrent : ils
// dépassent souvent celui de couronne, et les laisser hors de l'échelle les
// ferait sortir du cadre — ou, pire, ferait paraître la couronne dominante.
const maxForceN = computed(() =>
  Math.max(
    1,
    ...props.result.joints.flatMap((j) => [
      Math.hypot(j.fOrientationGlobal.x, j.fOrientationGlobal.y),
      Math.hypot(j.fPivotGlobal.x, j.fPivotGlobal.y),
      Math.hypot(j.fAnchorGlobal.x, j.fAnchorGlobal.y),
      Math.hypot(j.fLatchGlobal.x, j.fLatchGlobal.y),
    ]),
  ),
);

// La charge de manille est hors de cette échelle, volontairement : elle porte
// toute la grappe, donc elle vaut plusieurs fois le plus gros effort de
// jonction. La mettre à la même échelle écraserait toutes les autres flèches à
// quelques pixels. Elle a donc sa propre longueur, fixe, et son intensité se lit
// sur son étiquette — pas sur sa taille.
const SUPPORT_ARROW_LENGTH_MM = computed(() => annotation(referenceDepth.value * 0.9));

const bumperSupportArrow = computed(() => {
  const bv = props.result.bumperView;
  const mag = Math.hypot(bv.supportForceGlobal.x, bv.supportForceGlobal.y);
  if (mag <= 0) return null;
  const from = bv.supportPointGlobal;
  const len = SUPPORT_ARROW_LENGTH_MM.value;
  const to = {
    x: from.x + (bv.supportForceGlobal.x / mag) * len,
    y: from.y + (bv.supportForceGlobal.y / mag) * len,
  };
  const p0 = toLocal(from);
  const p1 = toLocal(to);
  return {
    arrow: {
      points: [p0.x, p0.y, p1.x, p1.y],
      stroke: colors.value.lift,
      fill: colors.value.lift,
      strokeWidth: px(3),
      pointerLength: px(10),
      pointerWidth: px(10),
    },
    label: {
      x: p1.x + px(6),
      y: p1.y - px(6),
      text: `${(bv.supportForceN / 1000).toFixed(2)} kN`,
      fontSize: px(12),
      fontStyle: "600",
      fill: colors.value.lift,
    },
  };
});

// Les deux efforts que le bumper transmet à l'enceinte de référence, à leur
// point d'application réel. Même schéma qu'une jonction : un bras à deux forces
// au trou de splay 0, un pivot à la charnière haute.
/// Les deux pions du bumper — avant et arrière — avec leurs efforts.
///
/// Leurs points d'application ne sont **pas** sur le rectangle du bumper : le
/// bumper se goupille dans la charnière avant et le trou de splay 0 de
/// l'enceinte de référence, tous deux à l'intérieur du caisson. Le rectangle
/// dessiné n'est qu'un schéma de son encombrement, pas son emprise réelle.
///
/// D'où le trait de rattachement : sans lui les deux flèches se confondent avec
/// celles des jonctions, et le bumper a l'air de ne rien porter.


/// La paire ancrage/verrou par laquelle la barre du bumper est boulonnée sur
/// l'enceinte de référence. Dessinée exactement comme la paire d'une jonction —
/// deux flèches à leurs goupilles, l'entraxe qui les relie — parce que c'est la
/// même liaison : cette enceinte est tenue par la même quincaillerie que les
/// autres, ce n'est pas un cas particulier.
const bumperBarPair = computed(() => {
  const bv = props.result.bumperView;
  const an = bv.pairAnchorHoleGlobal;
  const lt = bv.pairLatchHoleGlobal;
  const fa = bv.fPairAnchorGlobal;
  const fl = bv.fPairLatchGlobal;
  if (!an || !lt || !fa || !fl) return null;
  const a = toLocal(an);
  const b = toLocal(lt);
  const mid = { x: (an.x + lt.x) / 2, y: (an.y + lt.y) / 2 };
  return {
    // Détaillée quand l'enceinte qui la porte, ou le bumper d'où vient la
    // barre, est sous le curseur ou épinglé : dans les deux cas c'est cette
    // liaison-là qu'on est en train de lire.
    detailed:
      [0, BUMPER_ANCHOR_IDX].includes(anchor.value?.idx ?? -99) ||
      [0, BUMPER_ANCHOR_IDX].includes(pinnedIdx.value ?? -99),
    span: {
      points: [a.x, a.y, b.x, b.y],
      stroke: colors.value.anchor,
      strokeWidth: px(1),
      opacity: 0.5,
    },
    anchor: arrowConfig(an, fa, colors.value.anchor),
    latch: arrowConfig(lt, fl, colors.value.latch),
    anchorHole: holeMarkerConfig(an, colors.value.anchor),
    latchHole: holeMarkerConfig(lt, colors.value.latch),
    // Au repos, la résultante au milieu de la paire : le couple s'y annule, il
    // ne reste que ce que la barre déverse dans le caisson. Même lecture que
    // pour les paires de jonction, et même raison — trois flèches à l'arrière
    // d'un même caisson se chevauchent.
    resultant: arrowConfig(mid, { x: fa.x + fl.x, y: fa.y + fl.y }, colors.value.orientation),
    midHole: holeMarkerConfig(mid, colors.value.orientation),
  };
});

const bumperPins = computed(() => {
  const bv = props.result.bumperView;
  if (!bv.orientationForceGlobal || !bv.pivotForceGlobal) return null;
  // Perçage **déclaré** du bumper (`BumperModel.pins`), placé côté Rust comme
  // tout le reste. C'est aussi là que la statique se résout : ces deux points
  // sont la seule définition des pions, le viewer n'en reconstruit aucune.
  const frontAt = bv.pivotPointGlobal;
  const rearAt = bv.orientationPointGlobal;
  if (!frontAt || !rearAt) return null;
  return {
    rear: arrowConfig(rearAt, bv.orientationForceGlobal, colors.value.lift),
    front: arrowConfig(frontAt, bv.pivotForceGlobal, colors.value.lift),
    // Plus gros que les marqueurs de jonction : ces deux pions reprennent toute
    // la grappe, pas la charge d'une seule liaison.
    rearHole: holeMarkerConfig(rearAt, colors.value.lift, 5),
    frontHole: holeMarkerConfig(frontAt, colors.value.lift, 5),
    // Étiquettes seulement quand le bumper est sous le curseur : nommer les
    // pions en permanence rajouterait du texte là où on vient justement d'en
    // enlever.
    labels: pinLabelsVisible.value
      ? [pinLabel(frontAt, "pion avant"), pinLabel(rearAt, "pion arrière")]
      : [],
  };
});

/// Les pions se nomment quand on regarde le bumper : ils sont à lui, et c'est
/// sur lui que leurs valeurs sont affichées.
const pinLabelsVisible = computed(
  () => anchor.value?.idx === BUMPER_ANCHOR_IDX || pinnedIdx.value === BUMPER_ANCHOR_IDX,
);

function pinLabel(p: Vec2, text: string) {
  const s = toLocal(p);
  return {
    x: s.x + px(7),
    y: s.y - px(14),
    text,
    fontSize: px(11),
    fontStyle: "600",
    fill: colors.value.lift,
  };
}

function arrowConfig(from: Vec2, force: Vec2, color: string) {
  const mag = Math.hypot(force.x, force.y);
  const lenMm = (mag / maxForceN.value) * FORCE_REF_LENGTH_MM.value;
  const dir = mag > 0 ? { x: force.x / mag, y: force.y / mag } : { x: 0, y: 0 };
  const to = { x: from.x + dir.x * lenMm, y: from.y + dir.y * lenMm };
  const p0 = toLocal(from);
  const p1 = toLocal(to);
  return {
    points: [p0.x, p0.y, p1.x, p1.y],
    stroke: color,
    fill: color,
    strokeWidth: px(2.5),
    pointerLength: px(8),
    pointerWidth: px(8),
  };
}

// Trait fin entre les deux goupilles de la paire : il matérialise l'entraxe,
// bras du couple qui reprend le moment de barre.
/// La paire de la jonction `j` est portée par l'enceinte `j + 1` : c'est elle
/// qu'il faut survoler pour en voir le détail.
function pairDetailed(j: ClusterResult["joints"][number]): boolean {
  const owner = j.jointIndex + 1;
  return anchor.value?.idx === owner || pinnedIdx.value === owner;
}

function pairMidpoint(j: ClusterResult["joints"][number]): Vec2 {
  return {
    x: (j.anchorHoleGlobal.x + j.latchHoleGlobal.x) / 2,
    y: (j.anchorHoleGlobal.y + j.latchHoleGlobal.y) / 2,
  };
}

/// Résultante de la paire, au milieu des deux goupilles. La somme des deux
/// efforts : le couple s'y annule, il ne reste que ce que la barre déverse.
function pairResultantConfig(j: ClusterResult["joints"][number]) {
  const sum = {
    x: j.fAnchorGlobal.x + j.fLatchGlobal.x,
    y: j.fAnchorGlobal.y + j.fLatchGlobal.y,
  };
  return arrowConfig(pairMidpoint(j), sum, colors.value.orientation);
}

function pairSpanConfig(j: ClusterResult["joints"][number]) {
  const a = toLocal(j.anchorHoleGlobal);
  const b = toLocal(j.latchHoleGlobal);
  return {
    points: [a.x, a.y, b.x, b.y],
    stroke: colors.value.orientation,
    strokeWidth: px(1),
    opacity: 0.5,
  };
}

function holeMarkerConfig(p: Vec2, color: string, radiusPx = 3) {
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

function reversedRingConfig(p: Vec2) {
  const s = toLocal(p);
  return { x: s.x, y: s.y, radius: px(8), stroke: colors.value.alarm, strokeWidth: px(1.5) };
}

function jointLabelConfig(j: ClusterResult["joints"][number], idx: number) {
  const s = toLocal(j.loadedPivotHoleGlobal);
  return {
    x: s.x + px(9),
    y: s.y + px(9) + (idx % 2) * px(13),
    text: `J${idx + 1}`,
    fontSize: px(12),
    fill: colors.value.mutedForeground,
    fontStyle: "600",
  };
}

// Position du curseur affichée en bas à droite, en mm modèle (x = arrière,
// y = haut) : on repasse du repère écran au repère local du groupe cadré
// (`fitGroup`) via l'inverse de sa transform absolue — elle compose déjà le
// pan/zoom du stage et la mise à l'échelle du fit, donc aucun calcul de zoom
// à refaire à la main ici.
const cursorPos = ref<Vec2 | null>(null);

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
  const oldScale = stage.scaleX();
  const pointer = stage.getPointerPosition();
  if (!pointer) return;
  const mousePointTo = {
    x: (pointer.x - stage.x()) / oldScale,
    y: (pointer.y - stage.y()) / oldScale,
  };
  const direction = e.evt.deltaY > 0 ? -1 : 1;
  const scaleBy = 1.05;
  const newScale = direction > 0 ? oldScale * scaleBy : oldScale / scaleBy;
  stage.scale({ x: newScale, y: newScale });
  stage.position({
    x: pointer.x - mousePointTo.x * newScale,
    y: pointer.y - mousePointTo.y * newScale,
  });
  // Réactif, pas seulement posé sur le nœud Konva : c'est lui qui pilote `px`,
  // donc les tailles d'annotation doivent se recalculer au même instant.
  stageScale.value = newScale;
  stage.batchDraw();
  updateCursorPos();
}
</script>

<template>
  <div class="relative h-full w-full">
    <div ref="containerEl" class="h-full w-full overflow-hidden rounded-md border border-border bg-card">
      <v-stage
        ref="stageRef"
        :config="stageConfig"
        @wheel="handleWheel"
        @click="onStageClick"
        @mousemove="updateCursorPos"
        @dragmove="updateCursorPos"
        @mouseleave="clearCursorPos"
      >
        <v-layer>
          <v-line v-if="compartment === 'stacked'" :config="groundLineConfig" />

          <v-group ref="fitGroupRef" :config="{}">
            <!-- Pointillés CG/accroche : hors mesure de cadrage, sinon leur
                 longueur (dépendante du cadrage) fausserait le cadrage lui-même. -->
            <v-line v-if="compartment === 'flown' && pickupDashConfig" :config="pickupDashConfig!" />
            <v-line :config="cgDashConfig" />

            <v-group ref="measureGroupRef" :config="{}">
              <v-group ref="speakersGroupRef" :config="{}">
                <v-group
                  v-for="(speaker, i) in result.speakers"
                  :key="i"
                  :config="speakerGroupConfig(speaker)"
                  @click="onSpeakerClick(i)"
                  @mouseenter="onSpeakerMouseEnter(i)"
                  @mouseleave="onSpeakerMouseLeave"
                >
                  <v-line :config="outlineConfigOf(speaker)" />
                  <v-text :config="speakerNumberLabelConfig(i)" />
                </v-group>
                <!-- Repère global déjà résolu côté Rust : posé ici (et non
                     dans un groupe d'enceinte) pour ne subir aucune rotation
                     supplémentaire. Inclus dans le groupe mesuré pour la ligne
                     de sol : en stack, c'est le bumper qui touche le sol.
                     Survolable dans les deux compartiments : il portait déjà
                     les efforts de jonction en vol, il porte maintenant la
                     charge reprise en stack aussi — il n'est plus décoratif
                     nulle part. -->
                <v-line
                  v-if="bumperOutlineConfig"
                  :config="bumperOutlineConfig!"
                  @click="onBumperClick()"
                  @mouseenter="onBumperMouseEnter()"
                  @mouseleave="onBumperMouseLeave()"
                />
              </v-group>

            <v-line v-if="bumperBarConfig" :config="bumperBarConfig!" />
            <v-circle v-if="compartment === 'flown' && pickupMarkerConfig" :config="pickupMarkerConfig!" />

            <!-- Charge reprise par le bumper : la manille en vol, la réaction
                 du sol en stack. Toute la grappe pend dessus, c'est donc le
                 chiffre qui dit le calibre du point d'accroche. -->
            <template v-if="bumperSupportArrow">
              <v-arrow :config="bumperSupportArrow!.arrow" />
              <v-text :config="bumperSupportArrow!.label" />
            </template>

            <!-- La paire qui boulonne la barre du bumper sur l'enceinte de
                 référence : ses deux goupilles à elle, comme sur toute autre
                 enceinte de la grappe — donc lue de la même façon, une seule
                 résultante au repos et le détail au survol. -->
            <template v-if="bumperBarPair">
              <template v-if="bumperBarPair!.detailed">
                <v-arrow :config="bumperBarPair!.anchor" />
                <v-arrow :config="bumperBarPair!.latch" />
                <v-line :config="bumperBarPair!.span" />
                <v-circle :config="bumperBarPair!.anchorHole" />
                <v-circle :config="bumperBarPair!.latchHole" />
              </template>
              <template v-else>
                <v-arrow :config="bumperBarPair!.resultant" />
                <v-circle :config="bumperBarPair!.midHole" />
              </template>
            </template>

            <!-- Les deux pions du bumper, avant et arrière, portés par la face
                 du bumper qui les tient : c'est le bumper qui charge, pas
                 l'enceinte de référence. -->
            <template v-if="bumperPins">
              <v-arrow :config="bumperPins!.front" />
              <v-arrow :config="bumperPins!.rear" />
              <v-circle :config="bumperPins!.frontHole" />
              <v-circle :config="bumperPins!.rearHole" />
              <v-text v-for="(l, i) in bumperPins!.labels" :key="'pin-' + i" :config="l" />
            </template>

            <template v-if="compartment === 'flown' && tieArrowConfig">
              <v-arrow :config="tieArrowConfig!" />
              <v-circle :config="tiePointMarkerConfig!" />
              <v-text :config="tieLabelConfig!" />
            </template>

            <template v-for="(j, idx) in result.joints" :key="'joint-' + idx">
              <!-- La paire ancrage/verrou : une seule flèche résultante par
                   défaut, les deux détaillées quand l'enceinte qui la porte est
                   survolée ou épinglée.

                   Trois flèches à l'arrière d'un même caisson — ancrage, verrou
                   et trou de splay — se chevauchent dès qu'une grappe dépasse
                   quelques enceintes. La résultante dit ce que la barre déverse
                   dans le caisson ; le détail du couple ne se lit de toute façon
                   qu'en regardant une jonction en particulier. -->
              <template v-if="pairDetailed(j)">
                <v-arrow :config="arrowConfig(j.anchorHoleGlobal, j.fAnchorGlobal, colors.anchor)" />
                <v-arrow :config="arrowConfig(j.latchHoleGlobal, j.fLatchGlobal, colors.latch)" />
                <!-- L'entraxe, bras du couple : c'est lui qui explique l'écart
                     entre les deux flèches. -->
                <v-line :config="pairSpanConfig(j)" />
                <v-circle :config="holeMarkerConfig(j.anchorHoleGlobal, colors.anchor)" />
                <v-circle :config="holeMarkerConfig(j.latchHoleGlobal, colors.latch)" />
              </template>
              <template v-else>
                <v-arrow :config="pairResultantConfig(j)" />
                <v-circle :config="holeMarkerConfig(pairMidpoint(j), colors.orientation)" />
              </template>

              <v-arrow :config="arrowConfig(j.loadedOrientationHoleGlobal, j.fOrientationGlobal, colors.orientation)" />
              <v-arrow :config="arrowConfig(j.loadedPivotHoleGlobal, j.fPivotGlobal, colors.pivot)" />
              <v-circle :config="holeMarkerConfig(j.loadedOrientationHoleGlobal, colors.orientation)" />
              <v-circle :config="holeMarkerConfig(j.loadedPivotHoleGlobal, colors.pivot)" />
              <v-circle v-if="j.hingeReversed" :config="reversedRingConfig(j.loadedPivotHoleGlobal)" />
              <v-text :config="jointLabelConfig(j, idx)" />
            </template>

            <v-circle :config="cgMarkerConfig" />
            <v-text :config="cgLabelConfig" />
          </v-group>
        </v-group>
        </v-layer>
      </v-stage>
    </div>

    <div
      v-if="speakerPopupData"
      class="absolute z-10 w-[220px] select-text rounded-md border border-border bg-popover p-3 text-xs text-popover-foreground shadow-lg"
      :style="speakerPopupStyle"
      @mouseenter="clearHideTimer"
      @mouseleave="scheduleHide"
    >
      <div class="mb-1.5 flex items-center justify-between">
        <span class="font-semibold">{{ speakerPopupData.kind === "bumper" ? "Bumper" : `Enceinte ${speakerPopupData.number}` }}</span>
        <button class="text-muted-foreground hover:text-foreground" @click="closePopup">✕</button>
      </div>

      <dl v-if="speakerPopupData.kind === 'speaker'" class="flex flex-col gap-1">
        <div v-if="speakerPopupData.modelName" class="flex justify-between">
          <dt class="text-muted-foreground">Modèle</dt>
          <dd class="font-medium">{{ speakerPopupData.modelName }}</dd>
        </div>
        <div class="flex justify-between">
          <dt class="text-muted-foreground">Angle absolu</dt>
          <dd>{{ speakerPopupData.angleDeg.toFixed(1) }}°</dd>
        </div>
        <div class="flex justify-between">
          <dt class="text-muted-foreground">Splay</dt>
          <dd>{{ speakerPopupData.splay !== null ? `${speakerPopupData.splay}°` : "— (référence)" }}</dd>
        </div>
        <div class="flex justify-between">
          <dt class="text-muted-foreground">Bas de caisse</dt>
          <dd>{{ speakerPopupData.bottomElevationMm.toFixed(0) }} mm</dd>
        </div>
        <!-- Uniquement ce que CETTE enceinte subit. Les deux jonctions qui
             l'encadrent y contribuent : celle du dessous lui prend son trou de
             splay et sa bielle, celle du dessus sa paire ancrage/verrou. Les
             quatre trous sont donc bien les siens — c'est le nom du trou qui
             compte, pas celui de la jonction. -->
        <template
          v-if="
            speakerPopupData.joint || speakerPopupData.pairJoint || speakerPopupData.bumperBarPair
          "
        >
          <div class="mt-1 text-xs text-muted-foreground">Efforts sur ses perçages</div>
        </template>

        <template v-if="speakerPopupData.joint">
          <div class="flex justify-between text-zone-orientation">
            <dt>Trou de splay</dt>
            <dd>
              {{ speakerPopupData.joint.fOrientationN.toFixed(0) }} N @
              {{ speakerPopupData.joint.fOrientationAngleDeg.toFixed(1) }}°
            </dd>
          </div>
          <div class="flex justify-between text-zone-pivot">
            <dt>Bielle</dt>
            <dd>
              {{ speakerPopupData.joint.fPivotN.toFixed(0) }} N @
              {{ speakerPopupData.joint.fPivotAngleDeg.toFixed(1) }}°
            </dd>
          </div>
        </template>

        <!-- Tenue par la barre du bumper, sur sa propre paire : mêmes trous,
             mêmes noms que partout ailleurs dans la grappe. -->
        <template v-if="speakerPopupData.bumperBarPair">
          <div class="flex justify-between text-zone-anchor">
            <dt>Ancrage</dt>
            <dd>
              {{ speakerPopupData.bumperBarPair.anchorN.toFixed(0) }} N @
              {{ speakerPopupData.bumperBarPair.anchorAngleDeg.toFixed(1) }}°
            </dd>
          </div>
          <div class="flex justify-between text-zone-latch">
            <dt>Verrou</dt>
            <dd>
              {{ speakerPopupData.bumperBarPair.latchN.toFixed(0) }} N @
              {{ speakerPopupData.bumperBarPair.latchAngleDeg.toFixed(1) }}°
            </dd>
          </div>
          <div class="flex justify-between">
            <dt class="text-muted-foreground">M barre bumper</dt>
            <dd>{{ speakerPopupData.bumperBarPair.momentNm.toFixed(1) }} N·m</dd>
          </div>
        </template>

        <template v-if="speakerPopupData.pairJoint">
          <div class="flex justify-between text-zone-anchor">
            <dt>Ancrage</dt>
            <dd>
              {{ speakerPopupData.pairJoint.fAnchorN.toFixed(0) }} N @
              {{ speakerPopupData.pairJoint.fAnchorAngleDeg.toFixed(1) }}°
            </dd>
          </div>
          <div class="flex justify-between text-zone-latch">
            <dt>Verrou</dt>
            <dd>
              {{ speakerPopupData.pairJoint.fLatchN.toFixed(0) }} N @
              {{ speakerPopupData.pairJoint.fLatchAngleDeg.toFixed(1) }}°
            </dd>
          </div>
          <!-- La barre est encastrée sur la paire de CETTE enceinte : c'est ici
               que son moment atterrit, pas sur celle d'au-dessus. -->
          <div class="flex justify-between">
            <dt class="text-muted-foreground">M barre (ancrage)</dt>
            <dd>{{ speakerPopupData.pairJoint.barMomentMaxNm.toFixed(1) }} N·m</dd>
          </div>
        </template>

        <!-- La tirette tire sur cette enceinte-ci : sa tension est une charge
             qu'elle subit, au même titre que ses goupilles. -->
        <template v-if="speakerPopupData.tie">
          <div class="mt-1 text-xs text-muted-foreground">Tirette</div>
          <div class="flex justify-between text-zone-lift">
            <dt>Tension</dt>
            <dd>{{ speakerPopupData.tie.tensionKn.toFixed(2) }} kN</dd>
          </div>
          <div class="flex justify-between">
            <dt class="text-muted-foreground">Direction</dt>
            <dd>
              {{
                speakerPopupData.tie.angleDeg !== null
                  ? `${speakerPopupData.tie.angleDeg.toFixed(1)}°`
                  : "—"
              }}
            </dd>
          </div>
        </template>

        <p
          v-if="
            !speakerPopupData.joint &&
            !speakerPopupData.pairJoint &&
            !speakerPopupData.bumperBarPair &&
            !speakerPopupData.tie
          "
          class="mt-1 text-muted-foreground"
        >
          Aucune charge directe (flanc non porteur).
        </p>
      </dl>

      <dl v-else class="flex flex-col gap-1">
        <div class="flex justify-between">
          <dt class="text-muted-foreground">Dessous</dt>
          <dd>{{ speakerPopupData.bottomElevationMm.toFixed(0) }} mm</dd>
        </div>
        <div v-if="speakerPopupData.pickupElevationMm !== null" class="flex justify-between">
          <dt class="text-muted-foreground">Levage</dt>
          <dd>{{ speakerPopupData.pickupElevationMm.toFixed(0) }} mm</dd>
        </div>
        <div class="flex justify-between">
          <dt class="text-muted-foreground">Accroche</dt>
          <dd>
            {{ speakerPopupData.pickupOffsetMm !== null ? `${Math.abs(speakerPopupData.pickupOffsetMm).toFixed(0)} mm` : "—" }}
          </dd>
        </div>
        <div v-if="(speakerPopupData.barDeportMm ?? 0) !== 0" class="flex justify-between">
          <dt class="text-muted-foreground">Dont barre</dt>
          <dd>{{ Math.abs(speakerPopupData.barDeportMm!).toFixed(0) }} mm</dd>
        </div>
        <div v-if="speakerPopupData.bumperBarExceeded" class="text-status-alarm">Portée barre dépassée : tirette active.</div>
        <div class="mt-1 flex justify-between text-zone-lift">
          <dt>{{ compartment === "flown" ? "Charge manille" : "Réaction sol" }}</dt>
          <dd>
            {{ (speakerPopupData.supportForceN / 1000).toFixed(2) }} kN @
            {{ speakerPopupData.supportAngleDeg.toFixed(1) }}°
          </dd>
        </div>
        <p class="text-xs text-muted-foreground">
          Charge entière, pas par flanc : une manille n'est pas doublée.
        </p>
        <div v-if="speakerPopupData.pivotForceN !== null" class="flex justify-between text-zone-pivot">
          <dt>Pion avant</dt>
          <dd>{{ speakerPopupData.pivotForceN.toFixed(0) }} N @ {{ speakerPopupData.pivotAngleDeg!.toFixed(1) }}°</dd>
        </div>
        <div v-if="speakerPopupData.orientationForceN !== null" class="flex justify-between text-zone-orientation">
          <dt>Pion arrière</dt>
          <dd>{{ speakerPopupData.orientationForceN.toFixed(0) }} N @ {{ speakerPopupData.orientationAngleDeg!.toFixed(1) }}°</dd>
        </div>
        <div class="flex justify-between">
          <dt class="text-muted-foreground">M entre pions</dt>
          <dd>
            {{ speakerPopupData.pinPairMomentNm.toFixed(1) }} N·m
            <span class="text-xs">sur {{ speakerPopupData.pinSpanMm.toFixed(0) }} mm</span>
          </dd>
        </div>
        <p v-if="speakerPopupData.pivotForceN !== null" class="text-xs text-muted-foreground">
          Charges de pion par flanc : elles traversent les flancs, elles sont doublées.
        </p>
      </dl>

      <p v-if="!speakerPopupData.pinned" class="mt-2 text-[10px] text-muted-foreground">
        Clique pour épingler et sélectionner le texte.
      </p>
    </div>

    <div
      class="pointer-events-none absolute left-2 top-2 rounded-md border border-border bg-card/90 px-2 py-1 font-mono text-[11px] text-muted-foreground"
    >
      bumper {{ result.elevation.bumperBottomMm.toFixed(0) }} mm ·
      bas {{ result.elevation.lowestPointMm.toFixed(0) }} ·
      haut {{ result.elevation.highestPointMm.toFixed(0) }}
    </div>

    <div
      v-if="cursorPos"
      class="pointer-events-none absolute bottom-2 right-2 rounded-md border border-border bg-card/90 px-2 py-1 font-mono text-[11px] text-muted-foreground"
    >
      x {{ cursorPos.x.toFixed(0) }} · y {{ cursorPos.y.toFixed(0) }} ·
      sol {{ elevationOf(cursorPos.y).toFixed(0) }} mm
    </div>
  </div>
</template>
