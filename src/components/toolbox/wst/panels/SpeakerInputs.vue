<script setup lang="ts">
// Caractéristiques de la caisse : reprises du modèle dès qu'une enceinte est
// sélectionnée, et alors désactivées.
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useSpeakerModelsStore } from "@/stores/speakerModels";
import { useWst } from "../context";
import { NO_SPEAKER } from "../composables/useWstCalculator";

const speakerModelsStore = useSpeakerModelsStore();
const { form, selectedSpeaker, acousticMouthApplies, displayedGapMm } = useWst();
</script>

<template>
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
</template>
