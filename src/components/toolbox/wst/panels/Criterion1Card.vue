<script setup lang="ts">
// Ligne dérivée et critère 1 (ARF).
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import type { WstReport } from "@/lib/types";
import { db, deg, metres, mm, ratio } from "../format";

defineProps<{ report: WstReport }>();
</script>

<template>
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
</template>
