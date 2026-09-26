<script setup lang="ts">
// Ce qui se monte sous cette enceinte, et les bumpers qui l'acceptent. Tout
// vient des déclarations des JSON : rien n'est déduit.
import { computed } from "vue";
import type { BumperModel, SpeakerModel } from "@/lib/types";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { yesNo } from "../format";

const props = defineProps<{
  speaker: SpeakerModel;
  bumpers: BumperModel[];
  speakerName: (id: string) => string;
}>();

const acceptingBumpers = computed(() =>
  props.bumpers.flatMap((b) => {
    const c = b.compatibleSpeakers.find((c) => c.speakerModelId === props.speaker.id);
    return c && (c.flown || c.stacked) ? [{ bumper: b, compat: c }] : [];
  }),
);
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center text-sm">
        Compatibilités
        <InfoTip text="Déclaré dans un seul sens, vers le bas : ce qui peut aller au-dessus se déduit des modèles qui déclarent celui-ci. Hors plage recommandée, une jonction reste mécaniquement valable." />
      </CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-4">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Enceinte en dessous</TableHead>
            <TableHead>Vol</TableHead>
            <TableHead>Stack</TableHead>
            <TableHead>Splay recommandé</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="c in speaker.compatibleBelow" :key="c.speakerModelId">
            <TableCell>{{ speakerName(c.speakerModelId) }}</TableCell>
            <TableCell>{{ yesNo(c.flown) }}</TableCell>
            <TableCell>{{ yesNo(c.stacked) }}</TableCell>
            <TableCell class="font-mono">
              {{ c.recommendedSplay ? `${c.recommendedSplay.minDeg}° – ${c.recommendedSplay.maxDeg}°` : "—" }}
            </TableCell>
          </TableRow>
          <TableRow v-if="speaker.compatibleBelow.length === 0">
            <TableCell colspan="4" class="text-muted-foreground">Rien ne se monte en dessous.</TableCell>
          </TableRow>
        </TableBody>
      </Table>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Bumper</TableHead>
            <TableHead>Vol</TableHead>
            <TableHead>Stack</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="{ bumper, compat } in acceptingBumpers" :key="bumper.id">
            <TableCell>{{ bumper.name }}</TableCell>
            <TableCell>{{ yesNo(compat.flown) }}</TableCell>
            <TableCell>{{ yesNo(compat.stacked) }}</TableCell>
          </TableRow>
          <TableRow v-if="acceptingBumpers.length === 0">
            <TableCell colspan="3" class="text-muted-foreground">Aucun bumper compatible.</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </CardContent>
  </Card>
</template>
