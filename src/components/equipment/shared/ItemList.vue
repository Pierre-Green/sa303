<script setup lang="ts">
// Liste de sélection d'une catégorie d'équipement.
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

defineProps<{
  title: string;
  items: { id: string; name: string }[];
}>();
const selected = defineModel<string | null>({ required: true });
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle class="text-sm">{{ title }}</CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-1">
      <button
        v-for="item in items"
        :key="item.id"
        class="rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent"
        :class="item.id === selected ? 'bg-accent text-accent-foreground' : ''"
        @click="selected = item.id"
      >
        {{ item.name }}
        <span class="block font-mono text-[10px] text-muted-foreground">{{ item.id }}</span>
      </button>
      <p v-if="items.length === 0" class="text-xs text-muted-foreground">Aucun élément.</p>
    </CardContent>
  </Card>
</template>
