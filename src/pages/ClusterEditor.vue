<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { useSpeakerModelsStore } from "@/stores/speakerModels";
import { useBumperModelsStore } from "@/stores/bumperModels";
import { useClustersStore } from "@/stores/clusters";
import { useBuiltinsStore } from "@/stores/builtins";
import { api } from "@/lib/api";
import { speakerDisplayNumber } from "@/lib/display";
import type { BumperView, Cluster, ClusterResult, Compartment, RiggingSupport } from "@/lib/types";
import ArrayViewer from "@/components/array-viewer/ArrayViewer.vue";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const speakerModelsStore = useSpeakerModelsStore();
const bumperModelsStore = useBumperModelsStore();
const clustersStore = useClustersStore();
const builtinsStore = useBuiltinsStore();

const selectedClusterId = ref<string | null>(null);
const clusterResult = ref<ClusterResult | null>(null);
const error = ref<string | null>(null);
const saving = ref(false);

// Dernier bumperView valide connu : une saisie transitoire invalide (ex. un
// angle de pull-back en cours de frappe, momentanément hors plage) fait échouer
// le calcul le temps d'un caractère — sans ce cache, le bloc "accroche
// calculée" (et son champ d'angle) disparaîtrait à ce moment précis,
// empêchant de finir de taper la valeur voulue.
const lastBumperView = ref<BumperView | null>(null);
const effectiveBumperView = computed(() => clusterResult.value?.bumperView ?? lastBumperView.value);

interface FormState {
  id: string;
  name: string;
  schemaVersion: number;
  /** Une enceinte par position, du haut vers le bas : `splays.length + 1`
   * entrées. Chaque jonction doit être déclarée compatible côté modèle. */
  speakerModelIds: string[];
  compartment: Compartment;
  splays: number[];
  /** `null` tant qu'aucun bumper compatible n'est encore sélectionné. Un
   * bumper est obligatoire, en vol comme en stack : `buildCluster` ne doit
   * jamais partir en enregistrement avec `null`. */
  bumperModelId: string | null;
  imposedTiltEnabled: boolean;
  imposedTilt: number;
  /** `null` tant que l'utilisateur n'a pas choisi sa propre direction de
   * pull-back : le solveur suggère alors la verticale (180°, convention §2),
   * ou la borne la plus proche dans 180° ± tolérance. */
  pullBackAngle: number | null;
  /** Pull-back activé à la main, pour répartir la charge quand les points
   * d'accroche sont faibles. Ignoré quand il est de toute façon obligatoire. */
  pullBackEnabled: boolean;
  /** Pull-back manuel : tension voulue, kN (même base que les efforts
   * affichés, poids × k_dyn). `null` : la moitié de la charge au pull-back. */
  pullBackTensionKn: number | null;
  /** Vol : famille d'accroche (le trou est toujours choisi par le solveur). */
  riggingSupport: RiggingSupport;
  /** 1 ou 2 moteurs. */
  riggingPoints: number;
  /** Montage de barre imposé, `null` : au choix du solveur. */
  barMountIndex: number | null;
  /** Altitude du dessous du bumper, mm. Situe la grappe dans l'espace sans
   * rien changer aux efforts. */
  bumperHeight: number;
}

/** Angles disponibles pour le cadre de calage de l'enceinte la plus basse d'un
 * stack — trois positions de calage physiques, pas un réglage continu. */
const STACK_TILT_OPTIONS = [0, 5, 10, 15, 20, 30, 40];

function blankForm(): FormState {
  return {
    id: crypto.randomUUID(),
    name: "Nouvelle grappe",
    schemaVersion: 1,
    speakerModelIds: [
      speakerModelsStore.items[0]?.id ?? "",
      speakerModelsStore.items[0]?.id ?? "",
    ],
    compartment: "flown",
    splays: [0],
    bumperModelId: null,
    imposedTiltEnabled: false,
    imposedTilt: 0,
    pullBackAngle: null,
    pullBackEnabled: false,
    pullBackTensionKn: null,
    riggingSupport: "auto",
    riggingPoints: 1,
    barMountIndex: null,
    bumperHeight: 0,
  };
}

const form = reactive<FormState>(blankForm());

function loadIntoForm(c: Cluster) {
  Object.assign(form, {
    id: c.id,
    name: c.name,
    schemaVersion: c.schemaVersion,
    speakerModelIds: [...c.speakerModelIds],
    compartment: c.compartment,
    splays: c.joints.map((j) => j.splay),
    bumperModelId: c.bumperModelId,
    imposedTiltEnabled: c.imposedTilt != null,
    imposedTilt: c.imposedTilt ?? 0,
    pullBackAngle: c.pullBackAngle ?? null,
    pullBackEnabled: c.pullBackEnabled ?? false,
    pullBackTensionKn: c.manualPullBackTensionN != null ? c.manualPullBackTensionN / 1000 : null,
    riggingSupport: c.rigging?.support ?? "auto",
    riggingPoints: c.rigging?.points ?? 1,
    barMountIndex: c.rigging?.barMountIndex ?? null,
    bumperHeight: c.bumperHeight,
  });
}

onMounted(async () => {
  await Promise.all([
    speakerModelsStore.fetchAll(),
    bumperModelsStore.fetchAll(),
    clustersStore.fetchAll(),
    builtinsStore.fetchOnce(),
  ]);
  if (clustersStore.items.length > 0) {
    selectedClusterId.value = clustersStore.items[0].id;
  } else {
    Object.assign(form, blankForm());
  }
});

watch(selectedClusterId, (id) => {
  const cluster = clustersStore.items.find((c) => c.id === id);
  if (cluster) loadIntoForm(cluster);
  lastBumperView.value = null;
});

// Suggestion de départ pour la direction du pull-back, posée une seule fois au
// moment où elle devient nécessaire (jamais si l'utilisateur a déjà choisi,
// jamais recopiée ensuite) — même principe que pour l'assiette imposée :
// une valeur de départ pratique, pas un couplage permanent.
watch(
  () => effectiveBumperView.value?.pullBackAngleRangeDeg != null,
  (needed, wasNeeded) => {
    if (needed && !wasNeeded && form.pullBackAngle === null && clusterResult.value?.pullBackDirectionAngleDeg != null) {
      form.pullBackAngle = Number(clusterResult.value.pullBackDirectionAngleDeg.toFixed(1));
    }
  },
);

/** Champ numérique borné à une plage que le solveur renvoie : les flèches
 * (souris et clavier) s'arrêtent aux bornes grâce à `min`/`max` ; pendant la frappe, seule une valeur dans la plage
 * part au solveur (un « 1 » ou un « 17 » intermédiaire n'est pas ramené de
 * force à la borne) ; en sortie de champ ou sur Entrée, la valeur est ramenée
 * dans la plage. Quand la plage bouge (assiette modifiée), la valeur déjà
 * saisie y est ramenée, pour que le champ affiche ce que le solveur calcule. */
function useBoundedNumber(
  range: () => [number, number] | null,
  get: () => number | null,
  set: (v: number) => void,
) {
  const clamp = (v: number) => {
    const r = range();
    return r ? Math.min(r[1], Math.max(r[0], v)) : v;
  };
  function onInput(v: string | number) {
    if (v === "") return;
    const n = Number(v);
    if (Number.isFinite(n) && clamp(n) === n) set(n);
  }
  /** Le champ n'est jamais recréé (une `key` changeante lui faisait perdre
   * le focus à chaque cran de flèche, souris ou clavier) : la valeur bornée
   * est réécrite directement dans l'élément, et seulement si elle diffère. */
  function onCommit(e: Event) {
    const el = e.target as HTMLInputElement;
    const n = Number(el.value);
    if (el.value === "" || !Number.isFinite(n)) return;
    const c = clamp(n);
    if (c !== n) el.value = String(c);
    if (c !== get()) set(c);
  }
  watch(range, (r) => {
    const v = get();
    if (r && v !== null && clamp(v) !== v) set(clamp(v));
  });
  return { onInput, onCommit };
}

/** Arrondit une plage vers l'intérieur, pour que les bornes affichées dans le
 * champ restent valables. */
function inwardRange(r: [number, number] | null | undefined, step: number): [number, number] | null {
  if (!r) return null;
  return [Math.ceil(r[0] / step) * step, Math.floor(r[1] / step) * step];
}

/** Plage de direction du pull-back, au dixième de degré. */
const pullBackAngleRange = computed(() => inwardRange(effectiveBumperView.value?.pullBackAngleRangeDeg, 0.1));
const pullBackAngleField = useBoundedNumber(
  () => pullBackAngleRange.value,
  () => form.pullBackAngle,
  (v) => (form.pullBackAngle = v),
);

/** Pull-back manuel : tensions saisissables, en kN au centième. */
const pullBackTensionRange = computed(() => {
  const r = effectiveBumperView.value?.pullBackTensionRangeN;
  return r ? inwardRange([r[0] / 1000, r[1] / 1000], 0.01) : null;
});
const pullBackTensionField = useBoundedNumber(
  () => pullBackTensionRange.value,
  () => form.pullBackTensionKn,
  (v) => (form.pullBackTensionKn = v),
);

/** Le bumper et sa barre ne suffisent pas : le pull-back est obligatoire, la
 * case est cochée et grisée. */
const pullBackForced = computed(() => !!effectiveBumperView.value?.bumperBarExceeded);
const pullBackActive = computed(() => effectiveBumperView.value?.pullBackAngleRangeDeg != null);

// Sans assiette imposée, un pull-back manuel n'a rien pour fixer la
// répartition : décocher l'assiette le désactive aussi.
watch(
  () => form.imposedTiltEnabled,
  (on) => {
    if (!on && form.pullBackEnabled) {
      form.pullBackEnabled = false;
      form.pullBackTensionKn = null;
    }
  },
);

/** Accroche sur trous déclarés : présente dès que le bumper cote ses trous. */
const rigging = computed(() => effectiveBumperView.value?.rigging ?? null);

/** Les `Select` ne portent que des chaînes. */
const riggingPointsModel = computed({
  get: () => String(form.riggingPoints),
  set: (v: string) => (form.riggingPoints = Number(v)),
});
const barMountModel = computed({
  get: () => (form.barMountIndex === null ? "auto" : String(form.barMountIndex)),
  set: (v: string) => (form.barMountIndex = v === "auto" ? null : Number(v)),
});

// À 2 points, la charge se répartit déjà entre les deux moteurs : le
// pull-back manuel n'a plus de sens, le solveur le refuse.
watch(
  () => form.riggingPoints,
  (n) => {
    if (n === 2 && form.pullBackEnabled) {
      form.pullBackEnabled = false;
      form.pullBackTensionKn = null;
    }
  },
);

/** Activer le pull-back à la main demande une assiette imposée : c'est elle
 * qui fixe la répartition. On part de l'assiette actuelle, pour ne rien
 * changer à la grappe au moment où on coche. */
function onPullBackToggle(v: boolean | "indeterminate") {
  const on = v === true;
  form.pullBackEnabled = on;
  if (on && !form.imposedTiltEnabled) {
    const phi = clusterResult.value?.phiFreeHang;
    if (phi != null) form.imposedTilt = Number(((phi * 180) / Math.PI).toFixed(1));
    form.imposedTiltEnabled = true;
  }
  if (!on) form.pullBackTensionKn = null;
}

function speakerModelById(id: string) {
  return speakerModelsStore.items.find((s) => s.id === id) ?? null;
}

/** L'enceinte qui porte le bumper : celle du haut en vol, celle du bas en
 * stack — c'est sa compatibilité bumper qui compte, comme côté solveur. */
const referenceSpeakerModel = computed(() => {
  const ids = form.speakerModelIds;
  const id = form.compartment === "flown" ? ids[0] : ids[ids.length - 1];
  return id ? speakerModelById(id) : null;
});

/** Enceintes accrochables sous celle de la position précédente, pour le
 * compartiment courant. La première position n'a pas de contrainte : n'importe
 * quel modèle peut ouvrir la chaîne. */
function compatibleSpeakersAt(index: number) {
  if (index === 0) return speakerModelsStore.items;
  const above = speakerModelById(form.speakerModelIds[index - 1]);
  if (!above) return [];
  const flown = form.compartment === "flown";
  const allowedIds = new Set(
    above.compatibleBelow.filter((c) => (flown ? c.flown : c.stacked)).map((c) => c.speakerModelId),
  );
  return speakerModelsStore.items.filter((s) => allowedIds.has(s.id));
}

/** Repose la chaîne sur des jonctions déclarées : dès qu'une position n'est
 * plus accrochable sous celle du dessus (changement de modèle ou de
 * compartiment), on retombe sur la première compatible plutôt que de laisser
 * une grappe que le solveur refusera. */
function realignChainCompatibility() {
  for (let i = 1; i < form.speakerModelIds.length; i += 1) {
    const allowed = compatibleSpeakersAt(i);
    if (!allowed.some((s) => s.id === form.speakerModelIds[i])) {
      form.speakerModelIds[i] = allowed[0]?.id ?? form.speakerModelIds[i - 1];
    }
  }
}

watch(
  () => [...form.speakerModelIds, form.compartment],
  () => realignChainCompatibility(),
);
const isEditingExisting = computed(() =>
  clustersStore.items.some((c) => c.id === form.id),
);
/** Grappe livrée avec le logiciel : elle suit les mises à jour, donc elle ne
 * s'enregistre ni ne se supprime ici. `duplicate` est la porte de sortie. */
const isBuiltin = computed(() => builtinsStore.isCluster(form.id));

// Bumpers compatibles avec l'enceinte et le compartiment choisis.
const compatibleBumpers = computed(() =>
  bumperModelsStore.items.filter((b) =>
    b.compatibleSpeakers.some(
      (c) =>
        c.speakerModelId === referenceSpeakerModel.value?.id &&
        (form.compartment === "flown" ? c.flown : c.stacked),
    ),
  ),
);

// Si le bumper sélectionné n'est plus compatible (changement d'enceinte ou de
// compartiment), on ne le garde pas silencieusement sélectionné dans le vide.
// Un bumper est obligatoire, en vol comme en stack : l'accroche manuelle
// n'existe pas, donc on retombe automatiquement sur le premier compatible.
watch(
  [referenceSpeakerModel, () => form.compartment, compatibleBumpers],
  () => {
    if (form.bumperModelId && !compatibleBumpers.value.some((b) => b.id === form.bumperModelId)) {
      form.bumperModelId = null;
    }
    if (!form.bumperModelId && compatibleBumpers.value.length > 0) {
      form.bumperModelId = compatibleBumpers.value[0].id;
    }
  },
  { immediate: true },
);

// Grappes suspendues d'abord, stacks ensuite — plus lisible qu'un ordre de
// fichier arbitraire (tri stable : l'ordre relatif au sein d'un groupe est conservé).
/* --- Export d'audit -------------------------------------------------------
 *
 * Trois portées, un seul bouton : la grappe courante, les grappes cochées, ou
 * tout le catalogue. Le mode se déduit de l'état plutôt que d'être choisi dans
 * un menu — cocher des grappes est déjà l'expression de l'intention.
 */
const exportSelection = ref<Set<string>>(new Set());
const exporting = ref(false);
const exportMessage = ref<string | null>(null);

function toggleExportSelection(id: string) {
  const next = new Set(exportSelection.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  exportSelection.value = next;
}

/** Ce que le bouton exportera si on clique maintenant. */
const exportScope = computed(() => {
  if (exportSelection.value.size > 0) {
    return { ids: [...exportSelection.value], label: `${exportSelection.value.size} cochée(s)` };
  }
  if (selectedClusterId.value && isEditingExisting.value) {
    return { ids: [selectedClusterId.value], label: "la grappe courante" };
  }
  return { ids: undefined, label: `tout (${clustersStore.items.length})` };
});

function exportFileName(ids: string[] | undefined): string {
  const date = new Date().toISOString().slice(0, 10);
  if (!ids) return `sa303-audit-toutes-${date}.json`;
  if (ids.length === 1) {
    const one = clustersStore.items.find((c) => c.id === ids[0]);
    // Le nom de la grappe se retrouve dans le nom de fichier : un dossier
    // d'audit contient vite une dizaine de ces exports.
    const slug = (one?.name ?? "grappe").replace(/[^\p{L}\p{N}]+/gu, "-").toLowerCase();
    return `sa303-audit-${slug}-${date}.json`;
  }
  return `sa303-audit-${ids.length}-grappes-${date}.json`;
}

async function exportForAudit() {
  exporting.value = true;
  exportMessage.value = null;
  error.value = null;
  try {
    const { ids } = exportScope.value;
    const path = await api.exportClustersForAudit(ids, exportFileName(ids));
    // Annuler la boîte de dialogue n'est pas une erreur : pas de bandeau rouge
    // pour un geste délibéré.
    exportMessage.value = path ? `Exporté : ${path}` : null;
  } catch (e) {
    error.value = String(e);
  } finally {
    exporting.value = false;
  }
}

const sortedClusters = computed(() =>
  [...clustersStore.items].sort((a, b) => {
    if (a.compartment === b.compartment) return 0;
    return a.compartment === "flown" ? -1 : 1;
  }),
);

/** Message d'avertissement acoustique d'une jonction, ou `null` si elle est
 * dans la plage recommandée (ou qu'aucune n'est déclarée). Simple lecture du
 * résultat calculé côté Rust : aucun jugement acoustique côté front. */
function jointAcousticWarning(jointIndex: number): string | null {
  const joint = clusterResult.value?.joints[jointIndex];
  if (!joint || joint.acousticallyOptimal) return null;
  const range = joint.recommendedSplayRangeDeg;
  if (!range) return null;
  const target = range[0] === range[1] ? `${range[0]}°` : `${range[0]}° à ${range[1]}°`;
  return `Jonction non optimale acoustiquement : ${target} recommandé(s) entre ces deux enceintes.`;
}

/** Repart de la configuration affichée sous une nouvelle identité. C'est ce qui
 * rend les grappes livrées utilisables sans les rendre modifiables : on les lit,
 * on en dérive la sienne. Rien n'est enregistré tant que l'utilisateur ne le
 * demande pas. */
function duplicate() {
  const source = form.name;
  selectedClusterId.value = null;
  form.id = crypto.randomUUID();
  form.name = `${source} (copie)`;
}

function newCluster() {
  selectedClusterId.value = null;
  Object.assign(form, blankForm());
  lastBumperView.value = null;
}

/** Modèles qui déclarent `lowerId` accrochable sous eux, pour le compartiment
 * courant — l'autre sens de `compatibleSpeakersAt`, utile quand on ajoute une
 * enceinte AU-DESSUS d'une chaîne existante (empilage d'un stack). */
function speakersAcceptingBelow(lowerId: string) {
  const flown = form.compartment === "flown";
  return speakerModelsStore.items.filter((s) =>
    s.compatibleBelow.some((c) => c.speakerModelId === lowerId && (flown ? c.flown : c.stacked)),
  );
}

function addSpeaker() {
  if (form.compartment === "stacked") {
    // On empile par le haut : la nouvelle enceinte entre en tête de chaîne, et
    // c'est elle qui doit accepter l'ancienne tête en dessous.
    const below = form.speakerModelIds[0] ?? "";
    form.splays.unshift(form.splays[0] ?? 0);
    const accepting = speakersAcceptingBelow(below);
    form.speakerModelIds.unshift(
      accepting.some((s) => s.id === below) ? below : (accepting[0]?.id ?? below),
    );
  } else {
    // On descend la grappe : la nouvelle enceinte va en bas de chaîne, il faut
    // que celle du dessus l'accepte.
    form.splays.push(form.splays[form.splays.length - 1] ?? 0);
    const index = form.speakerModelIds.length;
    const above = form.speakerModelIds[index - 1] ?? "";
    form.speakerModelIds.push(above);
    const allowed = compatibleSpeakersAt(index);
    if (!allowed.some((s) => s.id === above)) {
      form.speakerModelIds[index] = allowed[0]?.id ?? above;
    }
  }
  realignChainCompatibility();
}

function removeSpeaker(dataIndex: number, splayIndex: number) {
  if (form.splays.length <= 1) return;
  form.splays.splice(splayIndex, 1);
  form.speakerModelIds.splice(dataIndex, 1);
  realignChainCompatibility();
}

/** Une ligne par enceinte (pas par joint) : l'enceinte de référence — celle du
 * haut pour une grappe, du bas pour un stack — n'a pas de splay propre, elle a
 * son inclinaison propre à la place (brief §4 : φ_initial est ancré sur cette
 * enceinte-là). Les autres lignes portent la valeur de `form.splays`.
 *
 * Les lignes sont listées dans l'ordre de montage : du haut vers le bas en
 * vol (on part de l'accroche), du bas vers le haut en stack (on part du sol).
 * `dataIndex` reste l'index dans la chaîne, elle toujours ordonnée du haut
 * vers le bas — l'inversion est purement d'affichage. */
type SpeakerRowKind = "flown-reference" | "stack-reference" | "splay";
interface SpeakerRow {
  label: string;
  kind: SpeakerRowKind;
  /** Index dans `form.speakerModelIds` (chaîne haut → bas). */
  dataIndex: number;
  splayIndex: number | null;
}
const speakerRows = computed<SpeakerRow[]>(() => {
  const n = form.splays.length + 1;
  const rows = Array.from({ length: n }, (_, dataIndex): SpeakerRow => {
    const label = `Enceinte ${speakerDisplayNumber(dataIndex, n, form.compartment)}`;
    if (form.compartment === "flown" && dataIndex === 0) {
      return { label, kind: "flown-reference", dataIndex, splayIndex: null };
    }
    if (form.compartment === "stacked" && dataIndex === n - 1) {
      return { label, kind: "stack-reference", dataIndex, splayIndex: null };
    }
    const splayIndex = form.compartment === "flown" ? dataIndex - 1 : dataIndex;
    return { label, kind: "splay", dataIndex, splayIndex };
  });
  return form.compartment === "stacked" ? rows.reverse() : rows;
});

function buildCluster(): Cluster {
  return {
    id: form.id,
    name: form.name,
    schemaVersion: form.schemaVersion,
    speakerModelIds: form.speakerModelIds,
    compartment: form.compartment,
    joints: form.splays.map((splay) => ({ splay })),
    // Bumper obligatoire (brief) : `form.bumperModelId` ne devrait jamais
    // être `null` ici (auto-sélectionné dès qu'un bumper compatible existe),
    // mais s'il l'est encore (aucun bumper compatible), on laisse le
    // backend le signaler explicitement plutôt que d'inventer une valeur.
    bumperModelId: form.bumperModelId ?? "",
    imposedTilt:
      form.compartment === "stacked"
        ? form.imposedTilt
        : form.imposedTiltEnabled
          ? form.imposedTilt
          : null,
    pullBackAngle: form.pullBackAngle,
    pullBackEnabled: form.compartment === "flown" && form.pullBackEnabled,
    manualPullBackTensionN:
      form.pullBackEnabled && form.pullBackTensionKn !== null ? form.pullBackTensionKn * 1000 : null,
    rigging: {
      support: form.riggingSupport,
      points: form.riggingPoints,
      // Un montage n'a de sens que sur la barre.
      barMountIndex: form.riggingSupport === "bar" ? form.barMountIndex : null,
    },
    bumperHeight: form.bumperHeight,
  };
}

async function save() {
  saving.value = true;
  error.value = null;
  try {
    const cluster = buildCluster();
    await clustersStore.save(cluster);
    selectedClusterId.value = cluster.id;
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

async function remove() {
  if (!isEditingExisting.value) return;
  await clustersStore.remove(form.id);
  if (clustersStore.items.length > 0) {
    selectedClusterId.value = clustersStore.items[0].id;
  } else {
    newCluster();
  }
}

// Prévisualisation live : recalculée à partir du formulaire tel quel, jamais
// depuis la version enregistrée sur disque — sinon rien ne bouge dans le
// canvas tant qu'on n'a pas cliqué "Enregistrer" (source de confusion vécue :
// choisir un bumper ne se voyait pas avant la sauvegarde).
let recomputeToken = 0;
async function recomputeViewer() {
  const token = ++recomputeToken;
  error.value = null;
  if (form.speakerModelIds.some((id) => !id) || form.splays.length === 0) {
    clusterResult.value = null;
    return;
  }
  try {
    const result = await api.computeClusterResult(buildCluster());
    if (token !== recomputeToken) return; // une saisie plus récente a déjà pris le relais
    clusterResult.value = result;
    lastBumperView.value = result.bumperView;
  } catch (e) {
    if (token === recomputeToken) {
      clusterResult.value = null;
      error.value = String(e);
    }
  }
}

watch(form, recomputeViewer, { deep: true, immediate: true });
</script>

<template>
  <div class="flex h-full">
    <aside class="flex w-95 shrink-0 flex-col gap-4 overflow-y-auto border-r border-border bg-card p-4">
      <div>
        <div class="mb-2 flex items-center justify-between">
          <h2 class="text-sm font-semibold">Grappes &amp; stacks</h2>
          <Button size="sm" variant="outline" @click="newCluster">+ Ajouter</Button>
        </div>
        <div class="flex flex-col gap-1">
          <div
            v-for="c in sortedClusters"
            :key="c.id"
            class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm hover:bg-accent"
            :class="c.id === selectedClusterId ? 'bg-accent text-accent-foreground' : ''"
          >
            <Checkbox
              :model-value="exportSelection.has(c.id)"
              :aria-label="`Cocher ${c.name} pour l'export d'audit`"
              @update:model-value="toggleExportSelection(c.id)"
            />
            <button class="flex flex-1 items-center justify-between text-left" @click="selectedClusterId = c.id">
              <span>{{ c.name }}</span>
              <Badge :class="c.compartment === 'flown' ? 'bg-zone-orientation' : 'bg-zone-lift'" class="text-white">
                {{ c.compartment === "flown" ? "vol" : "stack" }}
              </Badge>
            </button>
          </div>
        </div>

        <div class="mt-3 flex flex-col gap-1.5">
          <Button
            size="sm"
            variant="outline"
            :disabled="exporting || clustersStore.items.length === 0"
            @click="exportForAudit"
          >
            {{ exporting ? "Export en cours…" : "Exporter pour audit" }}
          </Button>
          <p class="flex items-center text-xs text-muted-foreground">
            Portée : {{ exportScope.label }}
            <InfoTip
              text="JSON autoportant : d'abord les définitions des enceintes, bumpers et barres utilisés, puis chaque grappe avec le détail de ses jonctions — positions, efforts, moments de barre, efforts sur chaque goupille et taux de travail des cinq chemins. Cocher des grappes exporte la sélection ; sans coche, la grappe affichée ; sans grappe enregistrée affichée, tout le catalogue."
            />
          </p>
          <p v-if="exportMessage" class="break-all text-xs text-status-ok">{{ exportMessage }}</p>
        </div>
      </div>

      <Separator />

      <div class="flex flex-col gap-3">
        <h3 class="text-sm font-semibold">{{ isEditingExisting ? "Éditer" : "Créer" }}</h3>

        <div class="flex flex-col gap-1.5">
          <Label class="flex items-center text-xs">Nom<InfoTip text="Nom affiché dans les listes et l'agrégat." /></Label>
          <Input v-model="form.name" />
        </div>

        <div class="flex flex-col gap-1.5">
          <Label class="flex items-center text-xs">
            Type
            <InfoTip
              text="Grappe suspendue : le corps libre d'un joint est ce qui pend en dessous. Stack : le corps libre est ce qui repose au-dessus du joint, posé au sol par l'enceinte du bas."
            />
          </Label>
          <Select v-model="form.compartment">
            <SelectTrigger class="w-full"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="flown">Suspendue (vol)</SelectItem>
              <SelectItem value="stacked">Stack (sol)</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div class="flex flex-col gap-1.5">
          <Label class="flex items-center text-xs">
            Bumper
            <InfoTip
              text="Bumper obligatoire, en vol comme en stack. En vol, il dérive le point d'accroche : centré par défaut, décalé le long d'une barre de déport si l'assiette imposée l'exige — l'accroche manuelle n'est pas une option. En stack, il sert de support visuel sous l'enceinte du bas."
            />
          </Label>
          <Select v-model="form.bumperModelId" :disabled="compatibleBumpers.length === 0">
            <SelectTrigger class="w-full"><SelectValue placeholder="Aucun bumper compatible" /></SelectTrigger>
            <SelectContent>
              <SelectItem v-for="b in compatibleBumpers" :key="b.id" :value="b.id">{{ b.name }}</SelectItem>
            </SelectContent>
          </Select>
          <p v-if="compatibleBumpers.length === 0" class="text-xs text-destructive">
            Aucun bumper compatible avec cette enceinte {{ form.compartment === "flown" ? "en vol" : "en stack" }}.
            Configure-en un dans Équipement et enceinte.
          </p>
        </div>

        <div class="flex flex-col gap-1.5">
          <Label class="flex items-center text-xs">
            {{ form.compartment === "flown" ? "Hauteur d'accroche" : "Hauteur du bumper" }} (mm)
            <InfoTip
              text="Altitude du dessous du bumper au-dessus du sol. En stack, 0 = posé au sol et une valeur positive décrit un praticable ; en vol, c'est la hauteur d'accroche. N'entre dans aucun calcul d'effort — une grappe pèse le même poids à 2 m qu'à 12 m — mais situe l'ensemble dans l'espace : les altitudes affichées dans le viewer en découlent."
            />
          </Label>
          <Input v-model.number="form.bumperHeight" type="number" step="100" min="0" />
          <p v-if="clusterResult" class="text-xs text-muted-foreground">
            Bas de l'ensemble : {{ clusterResult.elevation.lowestPointMm.toFixed(0) }} mm ·
            haut : {{ clusterResult.elevation.highestPointMm.toFixed(0) }} mm
          </p>
        </div>

        <div v-if="form.compartment === 'flown'" class="flex items-center justify-between gap-2">
          <div class="flex items-center gap-2">
            <Checkbox id="imposed" v-model="form.imposedTiltEnabled" />
            <Label for="imposed" class="flex items-center text-xs">
              Assiette imposée
              <InfoTip
                text="Sans assiette imposée, la grappe pend librement : l'inclinaison initiale est calculée pour que le CG passe sous le point d'accroche. φ libre reste affiché même assiette imposée cochée, pour comparaison."
              />
            </Label>
          </div>
          <span v-if="clusterResult?.phiFreeHang != null" class="text-xs text-muted-foreground">
            φ libre : {{ ((clusterResult.phiFreeHang * 180) / Math.PI).toFixed(1) }}°
          </span>
        </div>

        <div v-if="form.compartment === 'flown' && form.imposedTiltEnabled" class="flex flex-col gap-1.5">
          <Label class="flex items-center text-xs">
            Assiette imposée
            <InfoTip text="Inclinaison nez vers le bas positive, en degrés, de l'enceinte du haut." />
          </Label>
          <Input v-model.number="form.imposedTilt" type="number" step="0.5" />
        </div>

        <div
          v-if="form.compartment === 'flown' && form.bumperModelId && effectiveBumperView"
          class="flex flex-col gap-1 rounded-md border border-border bg-muted/40 p-2 text-xs"
        >
          <div class="flex items-center text-muted-foreground">
            Accroche
            <InfoTip
              :text="rigging
                ? 'Le solveur choisit le trou parmi ceux réellement percés. À 1 point, l\'assiette découle du trou : il prend celui qui approche le mieux l\'assiette imposée et affiche l\'écart. En Auto, le bumper seul passe avant la barre, qui n\'est montée que si elle fait nettement mieux. À 2 points, l\'assiette est tenue exactement par les longueurs de chaîne, et les deux trous sont choisis pour équilibrer les charges. Chaque point est comparé à sa charge maximale d\'utilisation (poids × k_dyn).'
                : 'Calculée par défaut : centrée sur le bumper, décalée le long de la barre si l\'assiette imposée l\'exige. Si même la barre ne suffit plus, le pull-back devient obligatoire. Il peut aussi être activé à la main pour répartir la charge : on règle alors sa direction et sa tension, et l\'accroche en découle.'"
            />
          </div>
          <!-- Bumper aux trous déclarés : on choisit la famille et le nombre de
               points, jamais le trou — c'est le solveur qui le choisit. -->
          <template v-if="rigging">
            <div class="grid grid-cols-2 gap-1.5">
              <Select v-model="form.riggingSupport">
                <SelectTrigger class="h-7 text-xs"><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="auto">Auto</SelectItem>
                  <SelectItem value="bumper">Bumper seul</SelectItem>
                  <SelectItem value="bar">Barre de déport</SelectItem>
                </SelectContent>
              </Select>
              <Select v-model="riggingPointsModel">
                <SelectTrigger class="h-7 text-xs"><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="1">1 point</SelectItem>
                  <SelectItem value="2">2 points</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <Select v-if="form.riggingSupport === 'bar'" v-model="barMountModel">
              <SelectTrigger class="h-7 text-xs"><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="auto">Montage auto</SelectItem>
                <SelectItem v-for="(m, i) in rigging.barMounts" :key="i" :value="String(i)">
                  Barre {{ m.label }}{{ m.flipped ? " (retournée)" : "" }}
                </SelectItem>
              </SelectContent>
            </Select>
            <div v-if="rigging.barMountIndex !== null" class="text-muted-foreground">
              Barre montée : {{ rigging.barMounts[rigging.barMountIndex].label }}
            </div>
            <div v-else class="text-muted-foreground">Sur le bumper, sans barre.</div>
            <div
              v-for="(p, i) in rigging.points"
              :key="i"
              class="flex justify-between"
              :class="p.overloaded ? 'text-status-alarm' : ''"
            >
              <span>{{ p.label }}</span>
              <span>{{ (p.tensionN / 1000).toFixed(2) }} kN (CMU {{ p.wllKg.toFixed(0) }} kg)</span>
            </div>
            <div
              v-for="(f, i) in rigging.barLinkForces"
              :key="'link-' + i"
              class="flex justify-between text-muted-foreground"
            >
              <span>Liaison barre {{ i + 1 }}</span>
              <span>{{ (f.forceN / 1000).toFixed(2) }} kN @ {{ f.angleDeg.toFixed(1) }}°</span>
            </div>
            <div v-if="rigging.points.some((p) => p.overloaded)" class="text-status-alarm">
              Charge maximale d'utilisation dépassée.
            </div>
            <div v-if="rigging.tiltErrorDeg !== null && rigging.points.length === 1">
              Assiette obtenue : {{ rigging.achievedTiltDeg.toFixed(1) }}°
              <span :class="Math.abs(rigging.tiltErrorDeg) > 0.5 ? 'text-status-warn' : 'text-muted-foreground'">
                (écart {{ rigging.tiltErrorDeg >= 0 ? "+" : "" }}{{ rigging.tiltErrorDeg.toFixed(1) }}°)
              </span>
            </div>
            <div v-else-if="rigging.targetTiltDeg === null && rigging.points.length === 1" class="text-muted-foreground">
              Assiette obtenue : {{ rigging.achievedTiltDeg.toFixed(1) }}°
            </div>
          </template>
          <template v-else>
            <!-- Deux cotes distinctes : où se trouve l'accroche sur le bumper, et
                 ce que la barre porte au-delà. La seconde est nulle tant que
                 l'accroche reste dans l'aplomb du bumper — l'afficher seule
                 donnait « centré » pour une accroche pourtant décalée. -->
            <div>
              Position : {{ Math.abs(effectiveBumperView.pickupOffsetMm ?? 0).toFixed(0) }} mm
              ({{
                (effectiveBumperView.pickupOffsetMm ?? 0) === 0
                  ? "centrée sur le bumper"
                  : effectiveBumperView.pickupOffsetMm! > 0
                    ? "vers l'arrière"
                    : "vers l'avant"
              }})
            </div>
            <div v-if="(effectiveBumperView.barDeportMm ?? 0) !== 0">
              Dont barre de déport : {{ Math.abs(effectiveBumperView.barDeportMm!).toFixed(0) }} mm
            </div>
            <div v-else class="text-muted-foreground">Dans l'aplomb du bumper : pas de barre.</div>
          </template>
          <!-- Pull-back : coché et grisé quand il est obligatoire (la barre ne
               suffit plus), sinon activable à la main pour répartir la charge
               quand les points d'accroche sont faibles. -->
          <div class="mt-1 flex items-center gap-2">
            <Checkbox
              id="pull-back"
              :model-value="pullBackForced || form.pullBackEnabled"
              :disabled="pullBackForced || (!!rigging && form.riggingPoints === 2)"
              @update:model-value="onPullBackToggle"
            />
            <Label for="pull-back" class="flex items-center text-xs">
              Pull-back (compression)
              <InfoTip
                :text="pullBackForced
                  ? 'Obligatoire : même la barre de déport ne suffit plus à tenir l\'assiette imposée. Le pull-back est un second moteur accroché au trou de couronne 0° de l\'enceinte du bas, qui tire verticalement vers le haut.'
                  : 'Second moteur accroché au trou de couronne 0° de l\'enceinte du bas, qui tire verticalement vers le haut. Il porte le bas de la grappe, met une partie de la chaîne en compression et décharge la manille principale. À activer quand les points d\'accroche sont faibles, pour répartir la charge. Demande une assiette imposée. On règle sa direction et sa tension ; l\'accroche du moteur principal en découle.'"
              />
            </Label>
          </div>
          <div
            v-if="pullBackActive"
            class="flex flex-col gap-1.5"
            :class="pullBackForced ? 'text-status-alarm' : ''"
          >
            <div v-if="clusterResult">
              Tension : {{ (clusterResult.pullBackTensionN / 1000).toFixed(2) }} kN, soit
              {{ (effectiveBumperView.pullBackLoadShare * 100).toFixed(0) }} % de la charge
              (manille : {{ (effectiveBumperView.supportForceN / 1000).toFixed(2) }} kN)
            </div>
            <div v-if="!pullBackForced && pullBackTensionRange" class="flex items-center gap-2">
              <Label class="flex shrink-0 items-center text-[11px]">
                Tension (entre {{ pullBackTensionRange[0].toFixed(2) }} et {{ pullBackTensionRange[1].toFixed(2) }} kN)
                <InfoTip
                  text="Tension voulue dans le pull-back, sur la même base que les efforts affichés (poids × k_dyn). Avec la direction et l'assiette imposée, elle fixe l'accroche du moteur principal, affichée plus haut. La plage correspond aux accroches atteignables sur le bumper et sa barre. Sans saisie, le pull-back reprend la moitié de la charge."
                />
              </Label>
              <Input
                class="h-7 w-20"
                type="number"
                step="0.1"
                :min="pullBackTensionRange[0]"
                :max="pullBackTensionRange[1]"
                :model-value="form.pullBackTensionKn ?? (clusterResult ? (clusterResult.pullBackTensionN / 1000).toFixed(2) : undefined)"
                @update:model-value="pullBackTensionField.onInput"
                @change="pullBackTensionField.onCommit"
              />
            </div>
            <div class="flex items-center gap-2">
              <Label class="flex shrink-0 items-center text-[11px]">
                Direction (entre {{ pullBackAngleRange?.[0].toFixed(1) }}° et
                {{ pullBackAngleRange?.[1].toFixed(1) }}°)
                <InfoTip
                  text="180° = verticale vers le haut. Le pull-back reste dans 180° ± la tolérance des réglages (10° par défaut, comme Meyer Sound). La suggestion par défaut est 180°."
                />
              </Label>
              <Input
                class="h-7 w-20"
                type="number"
                step="0.5"
                :min="pullBackAngleRange?.[0]"
                :max="pullBackAngleRange?.[1]"
                :model-value="form.pullBackAngle ?? undefined"
                @update:model-value="pullBackAngleField.onInput"
                @change="pullBackAngleField.onCommit"
              />
            </div>
          </div>
        </div>

        <Separator />

        <div>
          <div class="mb-1.5 flex items-center justify-between">
            <Label class="flex items-center text-xs">
              Enceintes — {{ form.compartment === "stacked" ? "du bas vers le haut" : "du haut vers le bas" }}
              <InfoTip
                text="Listées dans l'ordre de montage : en stack on part du sol (l'enceinte n°1 est celle qui porte le bumper et l'angle de calage), en grappe on part de l'accroche. L'enceinte de référence n'a pas de splay propre : son inclinaison ancre toute la chaîne."
              />
            </Label>
            <Button size="sm" variant="outline" @click="addSpeaker">+ enceinte</Button>
          </div>
          <div class="flex flex-col gap-1.5">
            <div v-for="(row, idx) in speakerRows" :key="idx" class="flex items-center gap-2">
              <span class="w-20 shrink-0 text-xs text-muted-foreground">{{ row.label }}</span>

              <div class="flex-1">
                <Select v-if="row.kind === 'splay'" v-model="form.splays[row.splayIndex!]">
                  <SelectTrigger class="w-full"><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem
                      v-for="s in speakerModelById(form.speakerModelIds[row.splayIndex!])?.splayGrid ?? []"
                      :key="s"
                      :value="s"
                    >
                      {{ s }}°
                    </SelectItem>
                  </SelectContent>
                </Select>

                <Select v-else-if="row.kind === 'stack-reference'" v-model="form.imposedTilt">
                  <SelectTrigger class="w-full"><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem v-for="t in STACK_TILT_OPTIONS" :key="t" :value="t">{{ t }}°</SelectItem>
                  </SelectContent>
                </Select>

                <Select v-else disabled :model-value="0">
                  <SelectTrigger class="w-full text-muted-foreground"><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem :value="0">0°</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div class="w-40 shrink-0">
                <Select v-model="form.speakerModelIds[row.dataIndex]">
                  <SelectTrigger class="w-full"><SelectValue placeholder="Enceinte" /></SelectTrigger>
                  <SelectContent>
                    <SelectItem v-for="s in compatibleSpeakersAt(row.dataIndex)" :key="s.id" :value="s.id">
                      {{ s.name }}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <!-- Conseil de calage, jamais une erreur : la jonction reste
                   mécaniquement valable même hors de la plage recommandée. -->
              <span
                v-if="row.splayIndex !== null && jointAcousticWarning(row.splayIndex)"
                class="w-4 shrink-0 cursor-help text-center text-status-alarm"
                :title="jointAcousticWarning(row.splayIndex)!"
              >
                ⚠
              </span>
              <span v-else class="w-4 shrink-0" />

              <Button
                v-if="row.kind === 'splay'"
                size="sm"
                variant="ghost"
                :disabled="form.splays.length <= 1"
                @click="removeSpeaker(row.dataIndex, row.splayIndex!)"
              >
                ✕
              </Button>
              <span v-else class="w-9 shrink-0" />
            </div>
          </div>
        </div>

        <div class="flex gap-2">
          <p
            v-if="isBuiltin"
            class="w-full rounded-md border border-border bg-muted/40 p-2 text-xs text-muted-foreground"
          >
            Grappe livrée avec le logiciel : en lecture seule, elle suit les mises
            à jour. Duplique-la pour en dériver la tienne.
          </p>
          <template v-if="isBuiltin">
            <Button class="flex-1" @click="duplicate">Dupliquer pour modifier</Button>
          </template>
          <template v-else>
            <Button class="flex-1" :disabled="saving" @click="save">Enregistrer</Button>
            <Button v-if="isEditingExisting" variant="destructive" @click="remove">Supprimer</Button>
          </template>
        </div>

        <div v-if="error" class="rounded-md border border-destructive/50 bg-destructive/10 p-2 text-xs text-destructive">
          {{ error }}
        </div>
      </div>
    </aside>

    <div class="min-h-0 flex-1 p-3">
      <ArrayViewer
        v-if="clusterResult"
        :result="clusterResult"
        :compartment="form.compartment"
        :view-key="form.id"
        class="h-full"
      />
      <div v-else class="flex h-full items-center justify-center text-sm text-muted-foreground">
        Enregistre la grappe pour voir la visualisation.
      </div>
    </div>
  </div>
</template>
