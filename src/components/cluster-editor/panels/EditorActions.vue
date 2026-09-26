<script setup lang="ts">
// Enregistrer / supprimer, ou dupliquer une grappe livrée ; et l'erreur du
// dernier calcul ou de la dernière action.
import { Button } from "@/components/ui/button";
import { useEditor } from "../context";

const { library, result } = useEditor();
const { isBuiltin, isEditingExisting, saving } = library;
const { error } = result;
</script>

<template>
  <div class="flex gap-2">
    <p v-if="isBuiltin" class="w-full rounded-md border border-border bg-muted/40 p-2 text-xs text-muted-foreground">
      Grappe livrée avec le logiciel : en lecture seule, elle suit les mises à jour. Duplique-la pour en dériver la
      tienne.
    </p>
    <Button v-if="isBuiltin" class="flex-1" @click="library.duplicate">Dupliquer pour modifier</Button>
    <template v-else>
      <Button class="flex-1" :disabled="saving" @click="library.save">Enregistrer</Button>
      <Button v-if="isEditingExisting" variant="destructive" @click="library.remove">Supprimer</Button>
    </template>
  </div>

  <div v-if="error" class="rounded-md border border-destructive/50 bg-destructive/10 p-2 text-xs text-destructive">
    {{ error }}
  </div>
</template>
