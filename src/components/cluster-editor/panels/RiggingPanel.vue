<script setup lang="ts">
// Accroche en vol : support, points, montage de barre, charges, et pull-back.
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useEditor } from "../context";

const { form, result: res, rigging: rig } = useEditor();
const { result, bumperView } = res;
const {
  view: rigging,
  pointsModel,
  barMountModel,
  pullBackForced,
  pullBackActive,
  angleRange,
  angleField,
  tensionRange,
  tensionField,
} = rig;
</script>

<template>
  <div
    v-if="form.compartment === 'flown' && form.bumperModelId && bumperView"
    class="flex flex-col gap-1 rounded-md border border-border bg-muted/40 p-2 text-xs"
  >
    <div class="flex items-center text-muted-foreground">
      Accroche
      <InfoTip
        text="Le solveur choisit le trou parmi ceux réellement percés. À 1 point, l'assiette découle du trou : il prend celui qui approche le mieux l'assiette imposée et affiche l'écart. En Auto, le bumper seul passe avant la barre, qui n'est montée que si elle fait nettement mieux. À 2 points, l'assiette est tenue exactement par les longueurs de chaîne, et les deux trous sont choisis pour équilibrer les charges. Chaque point est comparé à sa charge maximale d'utilisation (poids × k_dyn)."
      />
    </div>
    <!-- On choisit la famille et le nombre de points, jamais le trou : c'est
         le solveur qui le choisit. -->
    <template v-if="rigging">
      <div class="grid grid-cols-2 gap-1.5">
        <Select v-model="form.rigging.support">
          <SelectTrigger class="h-7 text-xs"><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectItem value="auto">Auto</SelectItem>
            <SelectItem value="bumper">Bumper seul</SelectItem>
            <SelectItem value="bar">Barre de déport</SelectItem>
          </SelectContent>
        </Select>
        <Select v-model="pointsModel">
          <SelectTrigger class="h-7 text-xs"><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectItem value="1">1 point</SelectItem>
            <SelectItem value="2">2 points</SelectItem>
          </SelectContent>
        </Select>
      </div>
      <Select v-if="form.rigging.support === 'bar'" v-model="barMountModel">
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
      <div v-for="(f, i) in rigging.barLinkForces" :key="'link-' + i" class="flex justify-between text-muted-foreground">
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

    <!-- Pull-back : coché et grisé quand il est obligatoire (aucun trou
         n'approche l'assiette), sinon activable à la main pour répartir la
         charge. -->
    <div class="mt-1 flex items-center gap-2">
      <Checkbox
        id="pull-back"
        :model-value="pullBackForced || form.pullBackEnabled"
        :disabled="pullBackForced || form.rigging.points === 2"
        @update:model-value="rig.togglePullBack"
      />
      <Label for="pull-back" class="flex items-center text-xs">
        Pull-back (compression)
        <InfoTip
          :text="pullBackForced
            ? 'Obligatoire : aucun trou du bumper ni de la barre n\'approche l\'assiette imposée. Le pull-back est un second moteur accroché au trou de couronne 0° de l\'enceinte du bas, qui tire verticalement vers le haut.'
            : 'Second moteur accroché au trou de couronne 0° de l\'enceinte du bas, qui tire verticalement vers le haut. Il porte le bas de la grappe, met une partie de la chaîne en compression et décharge la manille principale. À activer quand les points d\'accroche sont faibles, pour répartir la charge. Demande une assiette imposée. On règle sa direction et sa tension ; le trou du moteur principal en découle.'"
        />
      </Label>
    </div>
    <div v-if="pullBackActive" class="flex flex-col gap-1.5" :class="pullBackForced ? 'text-status-alarm' : ''">
      <div v-if="result">
        Tension : {{ (result.pullBackTensionN / 1000).toFixed(2) }} kN, soit
        {{ (bumperView.pullBackLoadShare * 100).toFixed(0) }} % de la charge
        (manille : {{ (bumperView.supportForceN / 1000).toFixed(2) }} kN)
      </div>
      <!-- Toujours réglable, même obligatoire : tendre davantage le pull-back
           décharge la manille du haut. -->
      <div v-if="tensionRange" class="flex items-center gap-2">
        <Label class="flex shrink-0 items-center text-[11px]">
          Tension (entre {{ tensionRange[0].toFixed(2) }} et {{ tensionRange[1].toFixed(2) }} kN)
          <InfoTip
            text="Tension voulue dans le pull-back, sur la même base que les efforts affichés (poids × k_dyn). Tendre davantage décharge la manille du haut. Avec la direction, elle fixe l'équilibre : le solveur choisit le trou qui approche le mieux l'assiette imposée, et affiche l'écart. Sans saisie, la tension proposée tient exactement l'assiette."
          />
        </Label>
        <Input
          class="h-7 w-20"
          type="number"
          step="0.1"
          :min="tensionRange[0]"
          :max="tensionRange[1]"
          :model-value="form.pullBackTensionKn ?? (result ? (result.pullBackTensionN / 1000).toFixed(2) : undefined)"
          @update:model-value="tensionField.onInput"
          @change="tensionField.onCommit"
        />
      </div>
      <div class="flex items-center gap-2">
        <Label class="flex shrink-0 items-center text-[11px]">
          Direction (entre {{ angleRange?.[0].toFixed(1) }}° et {{ angleRange?.[1].toFixed(1) }}°)
          <InfoTip
            text="180° = verticale vers le haut. Le pull-back reste dans 180° ± la tolérance des réglages (10° par défaut, comme Meyer Sound). La suggestion par défaut est 180°."
          />
        </Label>
        <Input
          class="h-7 w-20"
          type="number"
          step="0.5"
          :min="angleRange?.[0]"
          :max="angleRange?.[1]"
          :model-value="form.pullBackAngle ?? undefined"
          @update:model-value="angleField.onInput"
          @change="angleField.onCommit"
        />
      </div>
    </div>
  </div>
</template>
