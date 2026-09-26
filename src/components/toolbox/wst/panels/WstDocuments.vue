<script setup lang="ts">
// Les textes de référence, lus dans une fenêtre par-dessus le calculateur, qui
// reste en place derrière. Une seule référence ouverte à la fois.
import { computed, ref } from "vue";
import { FileTextIcon } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { WST_DOCUMENTS, type WstDocument } from "../documents";

const activeDocument = ref<WstDocument | null>(null);
const documentOpen = computed({
  get: () => activeDocument.value !== null,
  set: (open: boolean) => {
    if (!open) activeDocument.value = null;
  },
});
</script>

<template>
  <div class="mt-2 flex flex-wrap gap-2">
    <Button
      v-for="document in WST_DOCUMENTS"
      :key="document.path"
      variant="outline"
      size="sm"
      @click="activeDocument = document"
    >
      <FileTextIcon />
      {{ document.label }}
    </Button>
  </div>

  <Dialog v-model:open="documentOpen">
    <DialogContent class="h-[88vh] grid-rows-[auto_1fr] gap-3 sm:max-w-5xl">
      <DialogHeader>
        <DialogTitle>{{ activeDocument?.label }}</DialogTitle>
        <DialogDescription>{{ activeDocument?.detail }}</DialogDescription>
      </DialogHeader>
      <iframe
        v-if="activeDocument"
        :src="activeDocument.path"
        :title="activeDocument.label"
        class="size-full rounded-md border border-border bg-white"
      />
    </DialogContent>
  </Dialog>
</template>
