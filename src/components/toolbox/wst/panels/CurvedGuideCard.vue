<script setup lang="ts">
// Front courbé : le critère 5 ne s'applique pas, d'autres verdicts oui.
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { WstReport } from "@/lib/types";
import { db, deg, khz, metres } from "../format";

defineProps<{ curvedGuide: NonNullable<WstReport["curvedGuide"]>; referenceSplayDeg: number }>();
</script>

<template>
  <Card>
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
</template>
