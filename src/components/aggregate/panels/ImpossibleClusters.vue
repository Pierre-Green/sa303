<script setup lang="ts">
// Grappes écartées de l'agrégat parce que le solveur les a refusées.
import type { ImpossibleClusterReport } from "@/lib/types";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

defineProps<{ clusters: ImpossibleClusterReport[] }>();
</script>

<template>
  <Card class="border-destructive/50">
    <CardHeader>
      <CardTitle class="text-sm text-destructive">
        Grappes exclues — configuration physiquement impossible ({{ clusters.length }})
      </CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-2">
      <div
        v-for="(c, idx) in clusters"
        :key="idx"
        class="rounded-md border border-destructive/50 bg-destructive/10 p-2 text-xs text-destructive"
      >
        <span class="font-medium">{{ c.clusterName }}</span> — {{ c.reason }}
      </div>
    </CardContent>
  </Card>
</template>
