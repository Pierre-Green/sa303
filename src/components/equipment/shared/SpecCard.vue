<script setup lang="ts">
// Fiche technique en lecture seule : une liste libellé / valeur.
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import type { SpecRow } from "../types";

defineProps<{
  title: string;
  tip?: string;
  rows: SpecRow[];
}>();
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center text-sm">
        {{ title }}
        <InfoTip v-if="tip" :text="tip" />
      </CardTitle>
    </CardHeader>
    <CardContent>
      <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
        <template v-for="row in rows" :key="row.label">
          <dt class="flex items-center text-muted-foreground">
            {{ row.label }}
            <InfoTip v-if="row.tip" :text="row.tip" />
          </dt>
          <dd class="font-mono">{{ row.value }}</dd>
        </template>
      </dl>
      <slot />
    </CardContent>
  </Card>
</template>
