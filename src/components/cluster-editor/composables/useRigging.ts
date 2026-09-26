// Accroche en vol : famille de support, nombre de points, montage de barre, et
// le pull-back avec ses deux champs bornés. Tout ce qui est affiché vient du
// dernier `BumperView` calculé par Rust.

import { computed, watch } from "vue";
import type { FormState } from "../types";
import type { ClusterResultState } from "./useClusterResult";
import { inwardRange, useBoundedNumber } from "./useBoundedNumber";

export function useRigging(form: FormState, res: ClusterResultState) {
  const { result, bumperView } = res;

  // Suggestion de départ pour la direction du pull-back, posée une seule fois
  // au moment où elle devient nécessaire (jamais si l'utilisateur a déjà
  // choisi, jamais recopiée ensuite) — même principe que pour l'assiette
  // imposée : une valeur de départ pratique, pas un couplage permanent.
  watch(
    () => bumperView.value?.pullBackAngleRangeDeg != null,
    (needed, wasNeeded) => {
      const suggested = result.value?.pullBackDirectionAngleDeg;
      if (needed && !wasNeeded && form.pullBackAngle === null && suggested != null) {
        form.pullBackAngle = Number(suggested.toFixed(1));
      }
    },
  );

  /** Plage de direction du pull-back, au dixième de degré. */
  const angleRange = computed(() => inwardRange(bumperView.value?.pullBackAngleRangeDeg, 0.1));
  const angleField = useBoundedNumber(
    () => angleRange.value,
    () => form.pullBackAngle,
    (v) => (form.pullBackAngle = v),
  );

  /** Pull-back manuel : tensions saisissables, en kN au centième. */
  const tensionRange = computed(() => {
    const r = bumperView.value?.pullBackTensionRangeN;
    return r ? inwardRange([r[0] / 1000, r[1] / 1000], 0.01) : null;
  });
  const tensionField = useBoundedNumber(
    () => tensionRange.value,
    () => form.pullBackTensionKn,
    (v) => (form.pullBackTensionKn = v),
  );

  /** Aucun trou n'approche l'assiette : le pull-back est obligatoire, la case
   * est cochée et grisée — sa tension et sa direction restent réglables. */
  const pullBackForced = computed(() => !!bumperView.value?.pullBackForced);
  const pullBackActive = computed(() => bumperView.value?.pullBackAngleRangeDeg != null);

  function disablePullBack() {
    form.pullBackEnabled = false;
    form.pullBackTensionKn = null;
  }

  // Sans assiette imposée, un pull-back manuel n'a rien pour fixer la
  // répartition : décocher l'assiette le désactive aussi.
  watch(
    () => form.imposedTiltEnabled,
    (on) => {
      if (!on && form.pullBackEnabled) disablePullBack();
    },
  );

  // À 2 points, la charge se répartit déjà entre les deux moteurs : le
  // pull-back manuel n'a plus de sens, le solveur le refuse.
  watch(
    () => form.rigging.points,
    (n) => {
      if (n === 2 && form.pullBackEnabled) disablePullBack();
    },
  );

  /** Activer le pull-back à la main demande une assiette imposée : c'est elle
   * qui fixe la répartition. On part de l'assiette actuelle, pour ne rien
   * changer à la grappe au moment où on coche. */
  function togglePullBack(v: boolean | "indeterminate") {
    const on = v === true;
    form.pullBackEnabled = on;
    if (on && !form.imposedTiltEnabled) {
      const phi = result.value?.phiFreeHang;
      if (phi != null) form.imposedTilt = Number(((phi * 180) / Math.PI).toFixed(1));
      form.imposedTiltEnabled = true;
    }
    if (!on) form.pullBackTensionKn = null;
  }

  /** Accroche sur trous déclarés : présente dès que le bumper cote ses trous. */
  const view = computed(() => bumperView.value?.rigging ?? null);

  /** Les `Select` ne portent que des chaînes. */
  const pointsModel = computed({
    get: () => String(form.rigging.points),
    set: (v: string) => (form.rigging.points = Number(v)),
  });
  const barMountModel = computed({
    get: () => (form.rigging.barMountIndex === null ? "auto" : String(form.rigging.barMountIndex)),
    set: (v: string) => (form.rigging.barMountIndex = v === "auto" ? null : Number(v)),
  });

  return {
    view,
    pointsModel,
    barMountModel,
    pullBackForced,
    pullBackActive,
    togglePullBack,
    angleRange,
    angleField,
    tensionRange,
    tensionField,
  };
}
