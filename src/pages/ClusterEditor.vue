<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { useSpeakerModelsStore } from "@/stores/speakerModels";
import { useBumperModelsStore } from "@/stores/bumperModels";
import { useClustersStore } from "@/stores/clusters";
import { api } from "@/lib/api";
import { speakerDisplayNumber } from "@/lib/display";
import type { BumperView, Cluster, ClusterResult, Compartment } from "@/lib/types";
import ArrayViewer from "@/components/ArrayViewer.vue";
import InfoTip from "@/components/InfoTip.vue";
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

const selectedClusterId = ref<string | null>(null);
const clusterResult = ref<ClusterResult | null>(null);
const error = ref<string | null>(null);
const saving = ref(false);

// Dernier bumperView valide connu : une saisie transitoire invalide (ex. un
// angle de tirette en cours de frappe, momentanément hors plage) fait échouer
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
   * tirette : le solveur suggère alors celle qui minimise la tension. */
  tieAngle: number | null;
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
    tieAngle: null,
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
    tieAngle: c.tieAngle ?? null,
    bumperHeight: c.bumperHeight,
  });
}

onMounted(async () => {
  await Promise.all([speakerModelsStore.fetchAll(), bumperModelsStore.fetchAll(), clustersStore.fetchAll()]);
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

// Suggestion de départ pour la direction de tirette, posée une seule fois au
// moment où elle devient nécessaire (jamais si l'utilisateur a déjà choisi,
// jamais recopiée ensuite) — même principe que pour l'assiette imposée :
// une valeur de départ pratique, pas un couplage permanent.
watch(
  () => effectiveBumperView.value?.bumperBarExceeded,
  (needed, wasNeeded) => {
    if (needed && !wasNeeded && form.tieAngle === null && clusterResult.value?.tieDirectionAngleDeg != null) {
      form.tieAngle = Number(clusterResult.value.tieDirectionAngleDeg.toFixed(1));
    }
  },
);

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
    tieAngle: form.tieAngle,
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
    <aside class="flex w-[380px] shrink-0 flex-col gap-4 overflow-y-auto border-r border-border bg-card p-4">
      <div>
        <div class="mb-2 flex items-center justify-between">
          <h2 class="text-sm font-semibold">Grappes &amp; stacks</h2>
          <Button size="sm" variant="outline" @click="newCluster">+ Ajouter</Button>
        </div>
        <div class="flex flex-col gap-1">
          <button
            v-for="c in sortedClusters"
            :key="c.id"
            class="flex items-center justify-between rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent"
            :class="c.id === selectedClusterId ? 'bg-accent text-accent-foreground' : ''"
            @click="selectedClusterId = c.id"
          >
            <span>{{ c.name }}</span>
            <Badge :class="c.compartment === 'flown' ? 'bg-zone-orientation' : 'bg-zone-lift'" class="text-white">
              {{ c.compartment === "flown" ? "vol" : "stack" }}
            </Badge>
          </button>
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
            Accroche calculée
            <InfoTip
              text="Toujours calculée, jamais saisie : centrée sur le bumper par défaut, décalée le long de la barre si l'assiette imposée l'exige. Si même la barre ne suffit plus, le solveur active lui-même une tirette (accrochée au point 0° arrière-bas de l'enceinte du bas) — choisis sa direction dans la plage indiquée, selon où se trouve un point d'ancrage réel : ce n'est pas à l'algorithme de le deviner."
            />
          </div>
          <div>
            Déport : {{ Math.abs(effectiveBumperView.deportMm ?? 0).toFixed(0) }} mm
            ({{
              (effectiveBumperView.deportMm ?? 0) === 0
                ? "centré"
                : effectiveBumperView.deportMm! > 0
                  ? "vers l'arrière"
                  : "vers l'avant"
            }})
          </div>
          <div v-if="effectiveBumperView.bumperBarExceeded" class="flex flex-col gap-1.5 text-status-alarm">
            <div v-if="clusterResult">Tirette automatique : {{ (clusterResult.tieTensionN / 1000).toFixed(2) }} kN</div>
            <div class="flex items-center gap-2">
              <Label class="shrink-0 text-[11px]">
                Direction (entre {{ effectiveBumperView.tieAngleRangeDeg![0].toFixed(0) }}° et
                {{ effectiveBumperView.tieAngleRangeDeg![1].toFixed(0) }}°)
              </Label>
              <Input
                class="h-7 w-20"
                type="number"
                :model-value="form.tieAngle ?? undefined"
                @update:model-value="(v) => (form.tieAngle = v === '' ? null : Number(v))"
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
          <Button class="flex-1" :disabled="saving" @click="save">Enregistrer</Button>
          <Button v-if="isEditingExisting" variant="destructive" @click="remove">Supprimer</Button>
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
        class="h-full"
      />
      <div v-else class="flex h-full items-center justify-center text-sm text-muted-foreground">
        Enregistre la grappe pour voir la visualisation.
      </div>
    </div>
  </div>
</template>
