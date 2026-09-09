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
function px(sizeAtScale1: number): number {
  return sizeAtScale1 / Math.max(fitScale.value, 1e-6);
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
    orientation: cssVar("--zone-orientation"),
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

watch(
  () => [props.result, size.value.width, size.value.height],
  () => void nextTick(fitContent),
  { deep: true },
);

const stageConfig = computed(() => ({
  width: size.value.width,
  height: size.value.height,
  draggable: true,
}));

const groundLineConfig = computed(() => ({
  points: [0, groundLineY.value ?? 0, size.value.width, groundLineY.value ?? 0],
  stroke: colors.value.border,
  strokeWidth: 2,
  dash: [2, 6],
}));

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
      deportMm: bv.deportMm,
      bumperBarExceeded: bv.bumperBarExceeded,
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
    bottomElevationMm: props.result.elevation.speakerBottomMm[idx],
    pinned: pinnedIdx.value === idx,
  };
});

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
    x: props.result.tiePointGlobal.x + props.result.tieDirectionGlobal.x * referenceDepth.value,
    y: props.result.tiePointGlobal.y + props.result.tieDirectionGlobal.y * referenceDepth.value,
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
  return { x: tiePointLocal.value.x, y: tiePointLocal.value.y, radius: 4, fill: colors.value.lift };
});

const FORCE_REF_LENGTH_MM = computed(() => referenceDepth.value * 0.85);
const maxForceN = computed(() =>
  Math.max(
    1,
    ...props.result.joints.flatMap((j) => [
      Math.hypot(j.fOrientationGlobal.x, j.fOrientationGlobal.y),
      Math.hypot(j.fPivotGlobal.x, j.fPivotGlobal.y),
    ]),
  ),
);

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

function holeMarkerConfig(p: Vec2, color: string) {
  const s = toLocal(p);
  return { x: s.x, y: s.y, radius: px(3), fill: colors.value.card, stroke: color, strokeWidth: px(1.5) };
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
                     Survolable/cliquable en vol uniquement (charges calculées
                     là ; en stack le bumper est purement décoratif). -->
                <v-line
                  v-if="bumperOutlineConfig"
                  :config="bumperOutlineConfig!"
                  @click="compartment === 'flown' ? onBumperClick() : undefined"
                  @mouseenter="compartment === 'flown' ? onBumperMouseEnter() : undefined"
                  @mouseleave="compartment === 'flown' ? onBumperMouseLeave() : undefined"
                />
              </v-group>

            <v-line v-if="bumperBarConfig" :config="bumperBarConfig!" />
            <v-circle v-if="compartment === 'flown' && pickupMarkerConfig" :config="pickupMarkerConfig!" />

            <template v-if="compartment === 'flown' && tieArrowConfig">
              <v-arrow :config="tieArrowConfig!" />
              <v-circle :config="tiePointMarkerConfig!" />
              <v-text :config="tieLabelConfig!" />
            </template>

            <template v-for="(j, idx) in result.joints" :key="'joint-' + idx">
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
        <template v-if="speakerPopupData.joint">
          <div class="mt-1 flex justify-between text-zone-orientation">
            <dt>F orientation</dt>
            <dd>
              {{ speakerPopupData.joint.fOrientationN.toFixed(0) }} N @
              {{ speakerPopupData.joint.fOrientationAngleDeg.toFixed(1) }}°
            </dd>
          </div>
          <div class="flex justify-between text-zone-pivot">
            <dt>F pivot</dt>
            <dd>
              {{ speakerPopupData.joint.fPivotN.toFixed(0) }} N @
              {{ speakerPopupData.joint.fPivotAngleDeg.toFixed(1) }}°
            </dd>
          </div>
        </template>
        <p v-else class="mt-1 text-muted-foreground">Aucune charge directe (flanc non porteur).</p>
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
          <dt class="text-muted-foreground">Déport</dt>
          <dd>{{ speakerPopupData.deportMm !== null ? `${Math.abs(speakerPopupData.deportMm).toFixed(0)} mm` : "—" }}</dd>
        </div>
        <div v-if="speakerPopupData.bumperBarExceeded" class="text-status-alarm">Portée barre dépassée : tirette active.</div>
        <div v-if="speakerPopupData.orientationForceN !== null" class="mt-1 flex justify-between text-zone-orientation">
          <dt>F orientation</dt>
          <dd>{{ speakerPopupData.orientationForceN.toFixed(0) }} N @ {{ speakerPopupData.orientationAngleDeg!.toFixed(1) }}°</dd>
        </div>
        <div v-if="speakerPopupData.pivotForceN !== null" class="flex justify-between text-zone-pivot">
          <dt>F pivot</dt>
          <dd>{{ speakerPopupData.pivotForceN.toFixed(0) }} N @ {{ speakerPopupData.pivotAngleDeg!.toFixed(1) }}°</dd>
        </div>
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
