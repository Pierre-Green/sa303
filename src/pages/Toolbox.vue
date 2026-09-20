<script setup lang="ts">
// Boîte à outils — premier outil : les critères WST (Urban, Heil & Bauman,
// AES 5488). Aucune formule ici : tout est calculé par `sa303-core::wst` et
// cette page met en page (brief §1). Sélectionner une enceinte fait dériver le
// pas entre centres acoustiques de sa géométrie réelle — le jour en façade
// s'ouvrant avec l'angle, chaque splay a son propre pas, donc son propre ARF.
//
// La nature du guide décide des critères applicables : le critère 5 ne juge que
// des fronts *plans* anglés. Un guide qui rayonne déjà un secteur juxtapose le
// sien à celui de son voisin, donc le backend ne renvoie alors pas de critère 5
// du tout et cette page affiche à la place les verdicts de directivité.

import { computed, onMounted, reactive, ref, watch } from "vue";
import { FileTextIcon } from "@lucide/vue";
import { useSpeakerModelsStore } from "@/stores/speakerModels";
import { api } from "@/lib/api";
import type { WstInputs, WstReport } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

const SPEED_OF_SOUND_REAL = 343;
/** Célérité implicite du papier (λ = 1/(3F), F en kHz) : à choisir pour
 * recouper ses tableaux au chiffre près. */
const SPEED_OF_SOUND_PAPER = 1000 / 3;
const NO_SPEAKER = "__manual__";

interface WstDocument {
  label: string;
  detail: string;
  /** Chemin servi depuis `public/` : valable en développement comme dans
   * l'application empaquetée, où le dossier est copié à la racine du bundle. */
  path: string;
}

/** Les deux textes de référence dont sont tirées les formules de cet outil. */
const WST_DOCUMENTS: WstDocument[] = [
  {
    label: "Wavefront Sculpture Technology",
    detail:
      "Urban, Heil & Bauman — AES 5488. L'article d'origine : les cinq critères, leurs démonstrations et les tableaux que reproduit ce calculateur.",
    path: "/wst/Wavefront_Sculpture_Technology_convention.pdf",
  },
  {
    label: "La technologie WST : sculpture du front d'onde",
    detail:
      "Gramondo & Heil, L-Acoustics — Acoustique & Techniques n° 29. Reprise en français, plus courte et plus imagée.",
    path: "/wst/78_09943.pdf",
  },
];

// Une seule référence ouverte à la fois : la lecture se fait par-dessus le
// calculateur, qui reste en place derrière.
const activeDocument = ref<WstDocument | null>(null);
const documentOpen = computed({
  get: () => activeDocument.value !== null,
  set: (open: boolean) => {
    if (!open) activeDocument.value = null;
  },
});

const speakerModelsStore = useSpeakerModelsStore();

const report = ref<WstReport | null>(null);
const error = ref<string | null>(null);

interface FormState {
  speakerModelId: string;
  usePaperConvention: boolean;
  boxHeightMm: number;
  gapMm: number;
  radiatingHeightMm: number;
  acousticMouthHeightMm: number;
  isophaseSectorDeg: number;
  speakerCount: number;
  splaysText: string;
  distancesText: string;
  fMaxHz: number;
  guideKind: "isophase" | "curved";
  guideCoverageDeg: number;
  guideLevelAtHalfSplayDb: number;
}

const form = reactive<FormState>({
  speakerModelId: NO_SPEAKER,
  usePaperConvention: false,
  boxHeightMm: 550,
  gapMm: 8,
  radiatingHeightMm: 464,
  acousticMouthHeightMm: 485,
  isophaseSectorDeg: 1.3,
  speakerCount: 6,
  splaysText: "0, 1, 2, 3, 4, 5, 10.5, 20",
  distancesText: "5, 10, 20, 25, 30, 35, 40, 45, 50",
  fMaxHz: 16000,
  guideKind: "isophase",
  guideCoverageDeg: 20,
  guideLevelAtHalfSplayDb: -6,
});

const selectedSpeaker = computed(() =>
  form.speakerModelId === NO_SPEAKER
    ? null
    : (speakerModelsStore.items.find((s) => s.id === form.speakerModelId) ?? null),
);

/** Une liste saisie « 0, 5, 10 » → nombres exploitables, silencieusement
 * tolérante aux séparateurs et aux entrées vides en cours de frappe. */
function parseList(text: string): number[] {
  return text
    .split(/[,;\s]+/)
    .map((part) => Number(part.replace(",", ".")))
    .filter((value) => Number.isFinite(value));
}

const splays = computed(() => parseList(form.splaysText));
const distances = computed(() => parseList(form.distancesText).filter((d) => d > 0));

// Sélectionner une enceinte reprend ses caractéristiques. C'est un simple
// report à l'écran : quand une enceinte est sélectionnée, c'est le backend qui
// fait foi et ignore ces champs — d'où leur désactivation dans le formulaire.
watch(selectedSpeaker, (speaker) => {
  if (!speaker) return;
  form.boxHeightMm = speaker.height;
  form.guideKind = speaker.acoustics.wgFront === "constantCurvature" ? "curved" : "isophase";
  form.guideCoverageDeg = speaker.acoustics.directivityVertical;
  form.guideLevelAtHalfSplayDb = speaker.acoustics.wgLevelAtHalfCoverageDb;
  if (speaker.acoustics.wgOutputHeight > 0) {
    form.radiatingHeightMm = speaker.acoustics.wgOutputHeight;
  }
  form.isophaseSectorDeg = speaker.acoustics.wgIsophaseSectorDeg ?? form.isophaseSectorDeg;
  // Bouche acoustique : elle n'a de sens qu'en front plan. Pour un guide
  // courbé, le `D` du relevé est corrélé au rayon et ne décrit aucune bouche —
  // le backend l'ignore, la case le montre en le laissant sur la physique.
  const measured = speaker.acoustics.guideMeasurement;
  if (speaker.acoustics.wgFront === "constantCurvature") {
    form.acousticMouthHeightMm = form.radiatingHeightMm;
  } else if (measured && measured.acousticMouthHeightMm > 0) {
    form.acousticMouthHeightMm = measured.acousticMouthHeightMm;
  }
});

/** La bouche acoustique n'est ni saisissable ni utilisée en front courbé. */
const acousticMouthApplies = computed(() => form.guideKind !== "curved");

// Jour affiché dans le formulaire. Une enceinte sélectionnée ouvre un jour
// différent à chaque angle : la case en montre celui de l'angle de référence,
// pris du rapport plutôt que recopié dans `form` — sinon la saisie de
// l'utilisateur serait écrasée dès qu'il déselectionne, et le rapport
// s'alimenterait de sa propre sortie.
const displayedGapMm = computed(() =>
  selectedSpeaker.value && report.value ? report.value.derived.gapMm : form.gapMm,
);

function buildInputs(): WstInputs {
  return {
    speedOfSound: form.usePaperConvention ? SPEED_OF_SOUND_PAPER : SPEED_OF_SOUND_REAL,
    boxHeightMm: form.boxHeightMm,
    gapMm: form.gapMm,
    radiatingHeightMm: form.radiatingHeightMm,
    acousticMouthHeightMm: acousticMouthApplies.value ? form.acousticMouthHeightMm : null,
    isophaseSectorDeg: form.isophaseSectorDeg,
    speakerCount: Math.max(1, Math.round(form.speakerCount)),
    splaysDeg: splays.value.length > 0 ? splays.value : [0],
    distancesM: distances.value.length > 0 ? distances.value : [25],
    fMaxHz: form.fMaxHz,
    guide:
      form.guideKind === "curved"
        ? {
            kind: "curved",
            coverageDeg: form.guideCoverageDeg,
            levelAtHalfSplayDb: form.guideLevelAtHalfSplayDb,
          }
        : { kind: "isophase" },
  };
}

// Le backend ne renseigne qu'un des deux : le critère 5 en front plan, les
// verdicts de directivité en front courbé.
const criterion5 = computed(() => report.value?.criterion5 ?? null);
const curvedGuide = computed(() => report.value?.curvedGuide ?? null);
/** Angle de référence : le premier de la grille, celui qui décrit la ligne. */
const referenceSplayDeg = computed(() => splays.value[0] ?? 0);

let recomputeToken = 0;
async function recompute() {
  const token = ++recomputeToken;
  error.value = null;
  try {
    const result = await api.computeWstReport(
      buildInputs(),
      selectedSpeaker.value?.id ?? null,
    );
    if (token !== recomputeToken) return;
    report.value = result;
  } catch (e) {
    if (token === recomputeToken) {
      report.value = null;
      error.value = String(e);
    }
  }
}

onMounted(async () => {
  await speakerModelsStore.fetchAll();
  await recompute();
});
watch(form, recompute, { deep: true });

// --- Formatage (conventions d'affichage de la spec) ---
const khz = (hz: number | null | undefined) =>
  hz == null ? "—" : `${(hz / 1000).toFixed(1)} kHz`;
const deg = (value: number | null | undefined) =>
  value == null ? "—" : `${value.toFixed(1)}°`;
const mm = (value: number | null | undefined) =>
  value == null ? "—" : `${value.toFixed(1)} mm`;
const metres = (value: number | null | undefined) =>
  value == null ? "—" : `${value.toFixed(1)} m`;
const db = (value: number | null | undefined) =>
  value == null ? "—" : `${value.toFixed(1)} dB`;
const ratio = (value: number | null | undefined) =>
  value == null ? "—" : value.toFixed(3);
</script>

<template>
  <div class="flex h-full">
    <aside class="flex w-55 shrink-0 flex-col gap-1 border-r border-border bg-card p-3">
      <button class="rounded-md bg-accent px-3 py-2 text-left text-sm font-medium text-accent-foreground">
        Critères WST
      </button>
    </aside>

    <div class="flex-1 overflow-y-auto p-4">
      <div class="mb-4">
        <h2 class="text-sm font-semibold">Critères WST — angle entre sources</h2>
        <p class="mt-1 text-xs text-muted-foreground">
          Urban, Heil &amp; Bauman, <em>Wavefront Sculpture Technology</em> (AES 5488).
          Le papier ne traite que des sources à front plan : c'est le choix
          « front rayonné » qui décide des critères réellement applicables.
        </p>
        <div class="mt-2 flex flex-wrap gap-2">
          <Button
            v-for="document in WST_DOCUMENTS"
            :key="document.path"
            variant="outline"
            size="sm"
            @click="activeDocument = document"
          >
            <FileTextIcon />
            {{ document.label }}
          </Button>
        </div>
      </div>

      <Dialog v-model:open="documentOpen">
        <DialogContent class="grid-rows-[auto_1fr] gap-3 sm:max-w-5xl h-[88vh]">
          <DialogHeader>
            <DialogTitle>{{ activeDocument?.label }}</DialogTitle>
            <DialogDescription>{{ activeDocument?.detail }}</DialogDescription>
          </DialogHeader>
          <iframe
            v-if="activeDocument"
            :src="activeDocument.path"
            :title="activeDocument.label"
            class="size-full rounded-md border border-border bg-white"
          />
        </DialogContent>
      </Dialog>

      <div class="grid grid-cols-1 gap-4 xl:grid-cols-[320px_1fr]">
        <!-- ===================== ENTRÉES ===================== -->
        <!-- Deux blocs distincts : ce qui décrit l'enceinte, et ce qui décrit
             l'analyse. Les premiers viennent du modèle acoustique dès qu'une
             enceinte est sélectionnée ; les seconds restent toujours libres. -->
        <div class="flex flex-col gap-4">
          <Card class="h-fit">
            <CardHeader>
              <CardTitle class="flex items-center text-sm">
                Enceinte
                <InfoTip text="Ces caractéristiques décrivent la caisse, pas l'analyse. Sélectionner une enceinte les reprend de son modèle acoustique et c'est lui qui fait foi : le pas entre centres acoustiques est alors dérivé de la géométrie réelle, angle par angle." />
              </CardTitle>
            </CardHeader>
            <CardContent class="flex flex-col gap-3">
              <div class="flex flex-col gap-1.5">
                <Label class="text-xs">Modèle</Label>
                <Select v-model="form.speakerModelId">
                  <SelectTrigger class="w-full"><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem :value="NO_SPEAKER">— saisie manuelle —</SelectItem>
                    <SelectItem v-for="s in speakerModelsStore.items" :key="s.id" :value="s.id">
                      {{ s.name }}
                    </SelectItem>
                  </SelectContent>
                </Select>
                <p v-if="selectedSpeaker" class="text-xs text-muted-foreground">
                  Champs repris du modèle : les modifier se fait dans la fiche de
                  l'enceinte.
                </p>
              </div>

              <div class="grid grid-cols-2 gap-2">
                <div class="flex flex-col gap-1.5">
                  <Label class="flex items-center text-xs">
                    Hauteur caisse
                    <InfoTip text="Hauteur de la face avant, mm." />
                  </Label>
                  <Input v-model.number="form.boxHeightMm" type="number" :disabled="!!selectedSpeaker" />
                </div>
                <div class="flex flex-col gap-1.5">
                  <Label class="flex items-center text-xs">
                    Jour façade
                    <InfoTip text="Jour entre deux caisses en façade, mm. Dérivé de la géométrie, et variable avec l'angle, dès qu'une enceinte est sélectionnée : la charnière étant en retrait de la face, incliner fait bâiller la façade. La valeur montrée est alors celle de l'angle de référence ; le tableau du critère 5 donne le jour de chaque angle." />
                  </Label>
                  <Input
                    :model-value="displayedGapMm"
                    type="number"
                    :disabled="!!selectedSpeaker"
                    @update:model-value="form.gapMm = Number($event)"
                  />
                </div>
              </div>

              <div class="flex flex-col gap-1.5">
                <Label class="flex items-center text-xs">
                  Bouche physique D (mm)
                  <InfoTip text="Hauteur réellement occupée par la source en sortie de guide, mm. C'est elle qui donne l'ARF du verdict (ARF = D / pas), le profil de retard du guide et toute la géométrie. Distincte de la hauteur de caisse : c'est le chiffre conservateur, celui qu'on peut aller mesurer." />
                </Label>
                <Input v-model.number="form.radiatingHeightMm" type="number" :disabled="!!selectedSpeaker" />
              </div>

              <div v-if="acousticMouthApplies" class="flex flex-col gap-1.5">
                <Label class="flex items-center text-xs">
                  Bouche acoustique D (mm)
                  <InfoTip text="Hauteur de bouche équivalente que voit le rayonnement, diffraction de bride comprise. Elle sert au critère 5 et à lui seul : dans la dérivation du §6.2, le produit ARF·STEP vaut exactement D et la demi-ouverture d'un élément est λ/D — le critère ne connaît que l'ouverture équivalente, jamais l'ARF géométrique. Laisser sur la bouche physique si elle n'a pas été identifiée." />
                </Label>
                <Input
                  v-model.number="form.acousticMouthHeightMm"
                  type="number"
                  :disabled="!!selectedSpeaker"
                />
              </div>

              <div class="flex flex-col gap-1.5">
                <Label class="flex items-center text-xs">
                  Secteur du guide isophase (°)
                  <InfoTip text="Secteur encore rayonné par le guide isophase, degrés. 0 = front parfaitement plan. Il entre dans l'angle de raccord vers une caisse à guide courbé, qui est une tangence des deux fronts : (θ_iso + θ_courbe) / 2, et non θ_courbe / 2." />
                </Label>
                <Input
                  v-model.number="form.isophaseSectorDeg"
                  type="number"
                  step="0.1"
                  :disabled="!!selectedSpeaker && form.guideKind !== 'curved'"
                />
              </div>

              <div class="flex flex-col gap-1.5">
                <Label class="flex items-center text-xs">
                  Front rayonné
                  <InfoTip text="Le critère 5 ne juge que des fronts plans anglés : la zone sans énergie qu'il décrit n'existe qu'entre deux plans. Un guide à courbure constante rayonne déjà un secteur qui se juxtapose à celui du voisin, donc ce critère ne s'y applique pas — d'autres verdicts le remplacent." />
                </Label>
                <Select v-model="form.guideKind" :disabled="!!selectedSpeaker">
                  <SelectTrigger class="w-full"><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="isophase">Plan (isophase, type DOSC)</SelectItem>
                    <SelectItem value="curved">Courbure constante</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div v-if="form.guideKind === 'curved'" class="grid grid-cols-2 gap-2">
                <div class="flex flex-col gap-1.5">
                  <Label class="flex items-center text-xs">
                    Guide θ (°)
                    <InfoTip text="Secteur vertical rayonné par le guide. C'est lui qui borne le splay : au-delà, les secteurs voisins ne se touchent plus et il reste un trou. Splay recommandé entre θ − 3° et θ, jamais au-dessus. Quand le guide a été identifié, ce secteur vaut D/R — le seul des deux paramètres du relevé qui soit bien déterminé." />
                  </Label>
                  <Input v-model.number="form.guideCoverageDeg" type="number" :disabled="!!selectedSpeaker" />
                </div>
                <div class="flex flex-col gap-1.5">
                  <Label class="flex items-center text-xs">
                    Niveau à θ/2 (dB)
                    <InfoTip text="Niveau du guide à la moitié de son secteur, relevé en simulation ou en mesure. Deux sources en phase s'y somment de façon cohérente sur la bissectrice, soit +6 dB : −6 dB donne donc un raccord plat, −8 dB un creux de 2 dB. Quand l'enceinte porte un relevé en fréquence, c'est sa médiane de bande qui sert — pas son minimum, qui ne dirait que le pire accident." />
                  </Label>
                  <Input
                    v-model.number="form.guideLevelAtHalfSplayDb"
                    type="number"
                    step="0.5"
                    :disabled="!!selectedSpeaker"
                  />
                </div>
              </div>
            </CardContent>
          </Card>

          <Card class="h-fit">
            <CardHeader>
              <CardTitle class="flex items-center text-sm">
                Analyse
                <InfoTip text="Ce qui décrit le déploiement et l'écoute, pas la caisse : ces paramètres restent libres même quand une enceinte est sélectionnée." />
              </CardTitle>
            </CardHeader>
            <CardContent class="flex flex-col gap-3">
              <div class="flex flex-col gap-1.5">
                <Label class="flex items-center text-xs">
                  Nombre de caisses
                  <InfoTip text="Longueur de la ligne : H = N × pas. Entre dans l'ARF minimal et dans la frontière de champ proche." />
                </Label>
                <Input v-model.number="form.speakerCount" type="number" min="1" />
              </div>

              <div class="flex flex-col gap-1.5">
                <Label class="flex items-center text-xs">
                  Angles évalués (°)
                  <InfoTip text="Liste d'angles entre caisses adjacentes. Le premier sert de référence pour les critères qui ne dépendent pas de l'angle." />
                </Label>
                <Input v-model="form.splaysText" />
              </div>

              <div class="flex flex-col gap-1.5">
                <Label class="flex items-center text-xs">
                  Distances auditeur (m)
                  <InfoTip text="Le pire cas est le premier rang : c'est la distance la plus courte qui contraint l'angle." />
                </Label>
                <Input v-model="form.distancesText" />
              </div>

              <div class="flex flex-col gap-1.5">
                <Label class="flex items-center text-xs">
                  f max (Hz)
                  <InfoTip text="Haut de la bande utile visée : sert au critère 3 (planéité) et au pas maximal." />
                </Label>
                <Input v-model.number="form.fMaxHz" type="number" />
              </div>

              <Separator />

              <label class="flex items-center gap-2 text-xs">
                <input v-model="form.usePaperConvention" type="checkbox" class="size-3.5" />
                <span class="flex items-center">
                  Convention papier (c = 333,3 m/s)
                  <InfoTip text="Le papier approxime λ = 1/(3F). Cocher pour retrouver ses chiffres au dixième ; décocher pour la célérité réelle (343 m/s), qui donne des fréquences ~3 % plus hautes." />
                </span>
              </label>
            </CardContent>
          </Card>
        </div>

        <!-- ===================== RÉSULTATS ===================== -->
        <div v-if="error" class="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
          {{ error }}
        </div>

        <div v-else-if="report" class="flex flex-col gap-4">
          <!-- Ligne dérivée + critère 1 -->
          <Card>
            <CardHeader>
              <CardTitle class="flex items-center text-sm">
                Ligne &amp; critère 1 — ARF
                <InfoTip text="ARF = part du pas réellement occupée par la source. En dessous du seuil, le premier lobe secondaire remonte au-dessus de −13,5 dB." />
              </CardTitle>
            </CardHeader>
            <CardContent>
              <dl class="grid grid-cols-2 gap-x-6 gap-y-2 font-mono text-sm sm:grid-cols-4">
                <dt class="text-muted-foreground">Pas</dt>
                <dd>
                  {{ mm(report.derived.stepMm) }}
                  <span class="text-xs text-muted-foreground">
                    (jour {{ mm(report.derived.gapMm) }}{{ report.derived.stepFromSpeaker ? ", dérivé" : "" }})
                  </span>
                </dd>
                <dt class="text-muted-foreground">Hauteur ligne</dt>
                <dd>{{ metres(report.derived.lineHeightM) }}</dd>

                <dt class="text-muted-foreground">Bouche D</dt>
                <dd>
                  {{ mm(report.derived.radiatingHeightMm) }}
                  <span
                    v-if="report.derived.acousticMouthIsMeasured"
                    class="text-xs text-muted-foreground"
                  >
                    (acoustique {{ mm(report.derived.acousticMouthHeightMm) }})
                  </span>
                </dd>
                <dt class="text-muted-foreground">Front</dt>
                <dd>
                  {{ report.derived.guide.kind === "isophase" ? "plan" : "courbure constante" }}
                  <span
                    v-if="report.derived.guide.kind === 'curved'"
                    class="text-xs text-muted-foreground"
                  >
                    (θ {{ deg(report.derived.guide.coverageDeg) }})
                  </span>
                </dd>

                <dt class="text-muted-foreground">ARF géométrique</dt>
                <dd :class="report.criterion1.satisfied ? 'text-status-ok' : 'text-status-alarm'">
                  {{ ratio(report.criterion1.arfGeometric) }}
                  <span class="text-xs">(min {{ ratio(report.criterion1.arfMin) }})</span>
                </dd>
                <dt class="text-muted-foreground">Lobe secondaire</dt>
                <dd>{{ db(report.criterion1.sideLobeAttenuationDb) }}</dd>

                <template v-if="report.criterion1.arfsDiffer">
                  <dt class="text-muted-foreground">ARF acoustique</dt>
                  <dd class="text-muted-foreground">
                    {{ ratio(report.criterion1.arfAcoustic) }}
                  </dd>
                  <dt class="text-muted-foreground">Lobe secondaire</dt>
                  <dd class="text-muted-foreground">
                    {{ db(report.criterion1.sideLobeAttenuationAcousticDb) }}
                  </dd>
                </template>

                <dt class="text-muted-foreground">Perte axiale</dt>
                <dd>{{ db(report.criterion1.axialLossDb) }}</dd>
                <dt class="text-muted-foreground">Verdict</dt>
                <dd :class="report.criterion1.satisfied ? 'text-status-ok' : 'text-status-alarm'">
                  {{ report.criterion1.satisfied ? "conforme" : "sous le seuil" }}
                </dd>
              </dl>
              <p v-if="report.criterion1.arfsDiffer" class="mt-3 text-xs text-muted-foreground">
                L'écart entre les deux ARF est la contribution de la diffraction de bride : la
                bouche acoustique est plus grande que la bouche physique. La vérité est entre les
                deux — le verdict se prend sur le géométrique, qui est le plus prudent.
              </p>
            </CardContent>
          </Card>

          <!-- Critère 5 : uniquement pour des fronts plans anglés -->
          <Card v-if="criterion5">
            <CardHeader>
              <CardTitle class="flex items-center text-sm">
                Critère 5 — fréquence tenable par angle
                <InfoTip text="α_max = 2λ/D − pas/d, où D est la bouche ACOUSTIQUE — dans la dérivation du §6.2, le produit ARF·STEP ne vaut jamais autre chose. Au-delà de cette fréquence, la zone sans énergie entre deux caisses se referme après l'auditeur : le trou devient audible. L'ARF de la colonne est dérivé du pas pour l'affichage, il ne pilote pas le calcul." />
              </CardTitle>
            </CardHeader>
            <CardContent class="flex flex-col gap-3">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Angle</TableHead>
                    <TableHead>Pas</TableHead>
                    <TableHead>Jour façade</TableHead>
                    <TableHead>ARF</TableHead>
                    <TableHead v-for="d in distances" :key="d">à {{ d }} m</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow v-for="row in criterion5.rows" :key="row.splayDeg">
                    <TableCell class="font-medium">{{ deg(row.splayDeg) }}</TableCell>
                    <TableCell class="text-muted-foreground">{{ mm(row.stepMm) }}</TableCell>
                    <TableCell class="text-muted-foreground">{{ mm(row.gapMm) }}</TableCell>
                    <TableCell class="text-muted-foreground">{{ ratio(row.arf) }}</TableCell>
                    <TableCell
                      v-for="(f, i) in row.fMaxByDistanceHz"
                      :key="i"
                      :class="f != null && f >= report.criterion3.fMaxHz ? 'text-status-ok' : ''"
                    >
                      {{ khz(f) }}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
              <p class="text-xs text-muted-foreground">
                En vert : la bande utile ({{ khz(report.criterion3.fMaxHz) }}) passe entièrement à cet
                angle pour cette distance.
                <template v-if="criterion5.maxStepMm != null">
                  Pas maximal laissant un angle possible à {{ khz(report.criterion3.fMaxHz) }} pour le
                  premier rang ({{ metres(criterion5.closestDistanceM) }}) :
                  <strong>{{ mm(criterion5.maxStepMm) }}</strong>.
                </template>
              </p>
            </CardContent>
          </Card>

          <!-- Critère 5, lecture inverse -->
          <Card v-if="criterion5">
            <CardHeader>
              <CardTitle class="text-sm">Critère 5 — angle maximal par fréquence</CardTitle>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Fréquence</TableHead>
                    <TableHead v-for="d in distances" :key="d">à {{ d }} m</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow v-for="limit in criterion5.angleLimits" :key="limit.frequencyHz">
                    <TableCell class="font-medium">{{ khz(limit.frequencyHz) }}</TableCell>
                    <TableCell
                      v-for="(a, i) in limit.maxSplayDegByDistance"
                      :key="i"
                      :class="a == null ? 'text-status-alarm' : ''"
                    >
                      {{ a == null ? "impossible" : deg(a) }}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </CardContent>
          </Card>

          <!-- Front courbé : le critère 5 ne s'applique pas, d'autres verdicts oui -->
          <Card v-if="curvedGuide">
            <CardHeader>
              <CardTitle class="flex items-center text-sm">
                Guide à front courbé — couverture &amp; raccord
                <InfoTip text="Le critère 5 du papier décrit la zone sans énergie qui s'ouvre entre deux fronts plans anglés. Un guide qui rayonne déjà un secteur n'a pas cette géométrie : les secteurs voisins se juxtaposent, il n'y a aucun trou à refermer. Ce qui le remplace : couvrir l'angle, se raccorder à plat, et courber le front au bon rayon." />
              </CardTitle>
            </CardHeader>
            <CardContent class="flex flex-col gap-3">
              <p class="text-xs text-muted-foreground">
                Le critère 5 ne s'applique pas à ce type de guide : il répondrait à une
                autre question, celle de la limite d'un guide <em>plan</em> monté au même
                angle.
              </p>

              <dl class="grid grid-cols-2 gap-x-6 gap-y-2 font-mono text-sm sm:grid-cols-4">
                <dt class="flex items-center text-muted-foreground">
                  Splay max
                  <InfoTip text="L'ouverture du guide elle-même : au-delà, les deux secteurs ne se touchent plus. Splay recommandé entre θ − 3° et θ, jamais au-dessus — un recouvrement modéré est un réglage acceptable, un trou angulaire est une faute." />
                </dt>
                <dd>
                  {{ deg(curvedGuide.maxSplayDeg) }}
                  <span class="text-xs text-muted-foreground">
                    (viser {{ deg(curvedGuide.recommendedSplayMinDeg) }}–{{
                      deg(curvedGuide.recommendedSplayMaxDeg)
                    }})
                  </span>
                </dd>
                <dt class="text-muted-foreground">Niveau à α/2</dt>
                <dd>{{ db(curvedGuide.levelAtHalfSplayDb) }}</dd>

                <dt class="text-muted-foreground">Raccord</dt>
                <dd :class="Math.abs(curvedGuide.spliceLevelDb) <= 2 ? 'text-status-ok' : 'text-status-alarm'">
                  {{ db(curvedGuide.spliceLevelDb) }}
                </dd>
                <dt class="flex items-center text-muted-foreground">
                  Vers isophase
                  <InfoTip text="Raccord entre une caisse à front plan et une caisse à guide courbé. La condition est la tangence des deux fronts à la jonction : chacun y arrive incliné de la moitié de son propre secteur, soit (θ_iso + θ_courbe) / 2. La forme θ/2 n'en est que le cas particulier d'un guide isophase parfaitement plan." />
                </dt>
                <dd>
                  {{ deg(curvedGuide.transitionSplayDeg) }}
                  <span v-if="curvedGuide.isophaseSectorDeg > 0" class="text-xs text-muted-foreground">
                    (θ_iso {{ deg(curvedGuide.isophaseSectorDeg) }})
                  </span>
                </dd>

                <template v-if="curvedGuide.actualRadiusM != null">
                  <dt class="flex items-center text-muted-foreground">
                    Rayon réel
                    <InfoTip text="Rayon du front identifié sur le guide, à comparer au rayon visé de chaque angle. R_réel < R_visé : le guide est trop courbé pour ce splay, les secteurs se recouvrent. R_réel > R_visé : il reste un trou." />
                  </dt>
                  <dd>{{ metres(curvedGuide.actualRadiusM) }}</dd>
                </template>
              </dl>

              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Angle</TableHead>
                    <TableHead>Recouvrement</TableHead>
                    <TableHead>1er creux</TableHead>
                    <TableHead>Rayon visé</TableHead>
                    <TableHead>Écart au réel</TableHead>
                    <TableHead>Courbure critique au-dessus de</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow v-for="row in curvedGuide.rows" :key="row.splayDeg">
                    <TableCell class="font-medium">{{ deg(row.splayDeg) }}</TableCell>
                    <TableCell :class="row.covered ? 'text-status-ok' : 'text-status-alarm'">
                      {{ row.covered ? "+" : "" }}{{ deg(row.overlapDeg) }}
                      <span class="text-xs">{{ row.covered ? "" : "· trou angulaire" }}</span>
                    </TableCell>
                    <TableCell>{{ row.overlapNotchHz == null ? "—" : khz(row.overlapNotchHz) }}</TableCell>
                    <TableCell>{{ row.targetRadiusM == null ? "plate" : metres(row.targetRadiusM) }}</TableCell>
                    <TableCell :class="row.radiusVerdict === 'matched' ? 'text-status-ok' : ''">
                      <template v-if="row.radiusErrorM == null">—</template>
                      <template v-else>
                        {{ row.radiusErrorM > 0 ? "+" : "" }}{{ metres(row.radiusErrorM) }}
                        <span class="text-xs text-muted-foreground">
                          {{
                            row.radiusVerdict === "tooCurved"
                              ? "· trop courbé"
                              : row.radiusVerdict === "tooFlat"
                                ? "· trop plat"
                                : ""
                          }}
                        </span>
                      </template>
                    </TableCell>
                    <TableCell>{{ khz(row.curvatureMattersAboveHz) }}</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
              <p class="text-xs text-muted-foreground">
                Recouvrement positif = les deux secteurs se chevauchent, réglage acceptable
                tant qu'il reste modéré ; négatif = trou angulaire entre eux, et ça c'est une
                faute. La fréquence indiquée en dernière colonne n'est pas une limite de
                fonctionnement : en dessous, des cordes plates approximent l'arc à mieux que
                λ/4 et un guide isophase donnerait le même résultat. Au-dessus seulement, la
                courbure du guide doit être juste.
              </p>

              <template v-if="curvedGuide.edgeLevels.length > 0">
                <p class="mt-1 text-xs text-muted-foreground">
                  Bosse au raccord relevée en fréquence (niveau au bord du secteur + 6 dB, cible
                  0 dB) :
                </p>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Fréquence</TableHead>
                      <TableHead>Bord du secteur</TableHead>
                      <TableHead>Lissé</TableHead>
                      <TableHead>Raccord</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow v-for="sample in curvedGuide.edgeLevels" :key="sample.frequencyHz">
                      <TableCell>{{ khz(sample.frequencyHz) }}</TableCell>
                      <TableCell :class="sample.isNarrowArtifact ? 'text-status-warn' : ''">
                        {{ db(sample.levelDb) }}
                        <span v-if="sample.isNarrowArtifact" class="text-xs">· accident étroit</span>
                      </TableCell>
                      <TableCell class="text-muted-foreground">{{ db(sample.smoothedLevelDb) }}</TableCell>
                      <TableCell
                        :class="Math.abs(sample.spliceLevelDb) <= 2 ? 'text-status-ok' : 'text-status-alarm'"
                      >
                        {{ db(sample.spliceLevelDb) }}
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
                <p v-if="curvedGuide.hasNarrowArtifact" class="text-xs text-muted-foreground">
                  Les points signalés sont des accidents étroits du guide, pas un comportement de
                  secteur : le verdict de bande est pris sur la médiane glissante, jamais sur le
                  minimum — sinon un seul creux jugerait tout le raccord.
                </p>
              </template>

              <template v-if="curvedGuide.guideDelayProfile.length > 0">
                <p class="mt-1 text-xs text-muted-foreground">
                  Profil de retard visé sur la bouche, à {{ deg(referenceSplayDeg) }} :
                </p>
                <div class="flex flex-wrap gap-x-4 gap-y-1 font-mono text-xs">
                  <span v-for="sample in curvedGuide.guideDelayProfile" :key="sample.yMm">
                    <span class="text-muted-foreground">y {{ sample.yMm.toFixed(0) }}</span>
                    → {{ sample.delayMm.toFixed(2) }} mm
                  </span>
                </div>
              </template>

              <template v-if="curvedGuide.actualDelayProfile.length > 0">
                <p class="mt-1 text-xs text-muted-foreground">
                  Profil du guide réel ({{ metres(curvedGuide.actualRadiusM) }}), sur la même
                  bouche :
                </p>
                <div class="flex flex-wrap gap-x-4 gap-y-1 font-mono text-xs">
                  <span v-for="sample in curvedGuide.actualDelayProfile" :key="sample.yMm">
                    <span class="text-muted-foreground">y {{ sample.yMm.toFixed(0) }}</span>
                    → {{ sample.delayMm.toFixed(2) }} mm
                  </span>
                </div>
              </template>
            </CardContent>
          </Card>

          <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
            <!-- Critère 2 -->
            <Card>
              <CardHeader>
                <CardTitle class="flex items-center text-sm">
                  Critère 2 — pas &lt; λ/2
                  <InfoTip text="Sous cette fréquence, aucun lobe de réseau ne peut exister quel que soit l'ARF : le critère 1 n'a alors pas lieu de s'appliquer." />
                </CardTitle>
              </CardHeader>
              <CardContent class="flex flex-col gap-2">
                <p class="text-xs text-muted-foreground">
                  Limite : <strong>{{ khz(report.criterion2.frequencyLimitHz) }}</strong>
                </p>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Fréquence</TableHead>
                      <TableHead>Lobe réseau</TableHead>
                      <TableHead>Premier creux</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow v-for="sample in report.criterion2.samples" :key="sample.frequencyHz">
                      <TableCell :class="sample.satisfied ? 'text-status-ok' : ''">
                        {{ khz(sample.frequencyHz) }}
                      </TableCell>
                      <TableCell>{{ sample.gratingLobeDeg == null ? "aucun" : deg(sample.gratingLobeDeg) }}</TableCell>
                      <TableCell>{{ deg(sample.firstDipDeg) }}</TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </CardContent>
            </Card>

            <!-- Champ proche / lointain + critère 3 -->
            <Card>
              <CardHeader>
                <CardTitle class="flex items-center text-sm">
                  Champ proche &amp; critère 3
                  <InfoTip text="La transition champ proche → lointain est progressive : la forme complète et la forme de Fresnel en donnent les deux bornes. Le critère 3 borne la déviation admissible d'un front plan à λ/4." />
                </CardTitle>
              </CardHeader>
              <CardContent class="flex flex-col gap-2">
                <p class="text-xs text-muted-foreground">
                  Aucun champ proche sous
                  <strong>{{ khz(report.nearField.noNearFieldBelowHz) }}</strong> · déviation
                  admissible à {{ khz(report.criterion3.fMaxHz) }} :
                  <strong>{{ mm(report.criterion3.maxDeviationMm) }}</strong>
                </p>
                <p
                  v-if="report.criterion3.wavefrontDeviationMm != null"
                  class="text-xs text-muted-foreground"
                >
                  Front relevé à {{ metres(report.criterion3.wavefrontRadiusM) }} de rayon : écart
                  au plan
                  <strong>{{ mm(report.criterion3.wavefrontDeviationMm) }}</strong> sur la bouche
                  (s = (D/2)² / 2R), soit isophase jusqu'à
                  <strong>{{ khz(report.criterion3.isophaseFrequencyLimitHz) }}</strong
                  >. C'est la vraie mesure de « à quel point le front est plan ».
                </p>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Fréquence</TableHead>
                      <TableHead>Frontière</TableHead>
                      <TableHead>Fresnel</TableHead>
                      <TableHead>1er creux</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow v-for="sample in report.nearField.samples" :key="sample.frequencyHz">
                      <TableCell>{{ khz(sample.frequencyHz) }}</TableCell>
                      <TableCell>{{ metres(sample.boundaryM) }}</TableCell>
                      <TableCell class="text-muted-foreground">{{ metres(sample.boundaryFresnelM) }}</TableCell>
                      <TableCell>{{ deg(sample.firstDipDeg) }}</TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </CardContent>
            </Card>

            <!-- Critère 4 -->
            <Card>
              <CardHeader>
                <CardTitle class="flex items-center text-sm">
                  Critère 4 — courbure
                  <InfoTip text="Une ligne bien courbée garde α·d constant. Le niveau chute de 3 dB quand α·d atteint le pas. En dessous de l'angle minimal indiqué, la ligne se comporte comme plate." />
                </CardTitle>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Angle</TableHead>
                      <TableHead>Rayon</TableHead>
                      <TableHead>α·d</TableHead>
                      <TableHead>Niveau</TableHead>
                      <TableHead>α mini</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow v-for="row in report.curvature" :key="row.splayDeg">
                      <TableCell class="font-medium">{{ deg(row.splayDeg) }}</TableCell>
                      <TableCell>{{ row.radiusM == null ? "plate" : metres(row.radiusM) }}</TableCell>
                      <TableCell>{{ row.angleDistanceProduct == null ? "—" : row.angleDistanceProduct.toFixed(2) }}</TableCell>
                      <TableCell>{{ db(row.relativeLevelDb) }}</TableCell>
                      <TableCell class="text-muted-foreground">{{ deg(row.curvedModelMinSplayDeg) }}</TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </CardContent>
            </Card>

            <!-- CCA -->
            <Card>
              <CardHeader>
                <CardTitle class="flex items-center text-sm">
                  Géométrie de l'arc (CCA)
                  <InfoTip text="Rayon et flèche ne dépendent que de l'angle et du pas : ils sont vrais quelle que soit la nature du guide. La flèche est l'écart entre une corde plate et l'arc sur une caisse." />
                </CardTitle>
              </CardHeader>
              <CardContent class="flex flex-col gap-2">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Angle</TableHead>
                      <TableHead>Rayon</TableHead>
                      <TableHead>Flèche</TableHead>
                      <TableHead>Courbure critique au-dessus de</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow v-for="row in report.cca.rows" :key="row.splayDeg">
                      <TableCell class="font-medium">{{ deg(row.splayDeg) }}</TableCell>
                      <TableCell>{{ row.radiusM == null ? "plate" : metres(row.radiusM) }}</TableCell>
                      <TableCell>{{ mm(row.sagittaMm) }}</TableCell>
                      <TableCell>{{ khz(row.curvatureMattersAboveHz) }}</TableCell>
                    </TableRow>
                  </TableBody>
                </Table>

              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
