<script setup lang="ts">
// Vignette d'un équipement, dessinée par l'ArrayViewer à partir d'un montage
// d'exemple calculé par le solveur.
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import type { Vec2 } from "@/lib/types";
import ArrayViewer from "@/components/array-viewer/ArrayViewer.vue";
import { useEquipmentPreview, type PreviewSpec } from "../composables/useEquipmentPreview";

const props = defineProps<{
  spec: PreviewSpec | null;
  /** Ce que montre l'aperçu : l'enceinte seule ou le bumper seul. */
  part: "speaker" | "bumper";
  /** Perçages de l'enceinte, repère enceinte, quand on les connaît. */
  speakerHoles?: Vec2[];
}>();

const { result, cluster, message } = useEquipmentPreview(() => props.spec);
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center text-sm">
        Aperçu
        <InfoTip
          text="Placé par le solveur dans un montage d'exemple, assiette libre : silhouette et perçages uniquement, aucune charge."
        />
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="h-80">
        <ArrayViewer
          v-if="result && cluster"
          :result="result"
          :compartment="cluster.compartment"
          :view-key="cluster.id"
          :preview="part"
          :speaker-holes="speakerHoles"
        />
        <div
          v-else
          class="flex h-full items-center justify-center rounded-md border border-dashed border-border text-xs text-muted-foreground"
        >
          {{ message ?? "Calcul…" }}
        </div>
      </div>
    </CardContent>
  </Card>
</template>
