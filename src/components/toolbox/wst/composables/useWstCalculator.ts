// Critères WST (Urban, Heil & Bauman, AES 5488). Aucune formule ici : tout est
// calculé par `sa303-core::wst`, ce composable tient le formulaire et relance
// le calcul à chaque saisie. Sélectionner une enceinte fait dériver le pas
// entre centres acoustiques de sa géométrie réelle — le jour en façade
// s'ouvrant avec l'angle, chaque splay a son propre pas, donc son propre ARF.

import { computed, onMounted, reactive, ref, watch } from "vue";
import { useSpeakerModelsStore } from "@/stores/speakerModels";
import { api } from "@/lib/api";
import type { WstInputs, WstReport } from "@/lib/types";

const SPEED_OF_SOUND_REAL = 343;
/** Célérité implicite du papier (λ = 1/(3F), F en kHz) : à choisir pour
 * recouper ses tableaux au chiffre près. */
const SPEED_OF_SOUND_PAPER = 1000 / 3;
export const NO_SPEAKER = "__manual__";

export interface WstFormState {
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

/** Une liste saisie « 0, 5, 10 » → nombres exploitables, silencieusement
 * tolérante aux séparateurs et aux entrées vides en cours de frappe. */
function parseList(text: string): number[] {
  return text
    .split(/[,;\s]+/)
    .map((part) => Number(part.replace(",", ".")))
    .filter((value) => Number.isFinite(value));
}

export function useWstCalculator() {
  const speakers = useSpeakerModelsStore();

  const report = ref<WstReport | null>(null);
  const error = ref<string | null>(null);

  const form = reactive<WstFormState>({
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
      : (speakers.items.find((s) => s.id === form.speakerModelId) ?? null),
  );

  const splays = computed(() => parseList(form.splaysText));
  const distances = computed(() => parseList(form.distancesText).filter((d) => d > 0));
  /** Angle de référence : le premier de la grille, celui qui décrit la ligne. */
  const referenceSplayDeg = computed(() => splays.value[0] ?? 0);

  // Sélectionner une enceinte reprend ses caractéristiques. C'est un simple
  // report à l'écran : quand une enceinte est sélectionnée, c'est le backend
  // qui fait foi et ignore ces champs — d'où leur désactivation.
  watch(selectedSpeaker, (speaker) => {
    if (!speaker) return;
    const a = speaker.acoustics;
    form.boxHeightMm = speaker.height;
    form.guideKind = a.wgFront === "constantCurvature" ? "curved" : "isophase";
    form.guideCoverageDeg = a.directivityVertical;
    form.guideLevelAtHalfSplayDb = a.wgLevelAtHalfCoverageDb;
    if (a.wgOutputHeight > 0) form.radiatingHeightMm = a.wgOutputHeight;
    form.isophaseSectorDeg = a.wgIsophaseSectorDeg;
    // Bouche acoustique : elle n'a de sens qu'en front plan. Pour un guide
    // courbé, le `D` du relevé est corrélé au rayon et ne décrit aucune
    // bouche — le backend l'ignore, la case le montre en le laissant sur la
    // physique.
    const measured = a.guideMeasurement;
    if (a.wgFront === "constantCurvature") {
      form.acousticMouthHeightMm = form.radiatingHeightMm;
    } else if (measured && measured.acousticMouthHeightMm > 0) {
      form.acousticMouthHeightMm = measured.acousticMouthHeightMm;
    }
  });

  /** La bouche acoustique n'est ni saisissable ni utilisée en front courbé. */
  const acousticMouthApplies = computed(() => form.guideKind !== "curved");

  // Jour affiché dans le formulaire. Une enceinte sélectionnée ouvre un jour
  // différent à chaque angle : la case en montre celui de l'angle de
  // référence, pris du rapport plutôt que recopié dans `form` — sinon la
  // saisie de l'utilisateur serait écrasée dès qu'il désélectionne, et le
  // rapport s'alimenterait de sa propre sortie.
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

  let token = 0;
  async function recompute() {
    const mine = ++token;
    error.value = null;
    try {
      const r = await api.computeWstReport(buildInputs(), selectedSpeaker.value?.id ?? null);
      if (mine !== token) return;
      report.value = r;
    } catch (e) {
      if (mine === token) {
        report.value = null;
        error.value = String(e);
      }
    }
  }

  onMounted(async () => {
    await speakers.fetchAll();
    await recompute();
  });
  watch(form, recompute, { deep: true });

  // Le backend ne renseigne qu'un des deux : le critère 5 en front plan, les
  // verdicts de directivité en front courbé.
  const criterion5 = computed(() => report.value?.criterion5 ?? null);
  const curvedGuide = computed(() => report.value?.curvedGuide ?? null);

  return {
    form,
    report,
    error,
    selectedSpeaker,
    distances,
    referenceSplayDeg,
    acousticMouthApplies,
    displayedGapMm,
    criterion5,
    curvedGuide,
  };
}

export type WstCalculator = ReturnType<typeof useWstCalculator>;
