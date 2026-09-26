<script setup lang="ts">
// Nom, type, bumper, hauteur et assiette imposée.
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useEditor } from "../context";

const { form, result: res, bumper } = useEditor();
const { result } = res;
const { compatible: compatibleBumpers } = bumper;
</script>

<template>
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
      Déclare-le dans le JSON du bumper.
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
    <p v-if="result" class="text-xs text-muted-foreground">
      Bas de l'ensemble : {{ result.elevation.lowestPointMm.toFixed(0) }} mm ·
      haut : {{ result.elevation.highestPointMm.toFixed(0) }} mm
    </p>
  </div>

  <template v-if="form.compartment === 'flown'">
    <div class="flex items-center justify-between gap-2">
      <div class="flex items-center gap-2">
        <Checkbox id="imposed" v-model="form.imposedTiltEnabled" />
        <Label for="imposed" class="flex items-center text-xs">
          Assiette imposée
          <InfoTip
            text="Sans assiette imposée, la grappe pend librement : l'inclinaison initiale est calculée pour que le CG passe sous le point d'accroche. φ libre reste affiché même assiette imposée cochée, pour comparaison."
          />
        </Label>
      </div>
      <span v-if="result?.phiFreeHang != null" class="text-xs text-muted-foreground">
        φ libre : {{ ((result.phiFreeHang * 180) / Math.PI).toFixed(1) }}°
      </span>
    </div>

    <div v-if="form.imposedTiltEnabled" class="flex flex-col gap-1.5">
      <Label class="flex items-center text-xs">
        Assiette imposée
        <InfoTip text="Inclinaison nez vers le bas positive, en degrés, de l'enceinte du haut." />
      </Label>
      <Input v-model.number="form.imposedTilt" type="number" step="0.5" />
    </div>
  </template>
</template>
