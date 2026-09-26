<script setup lang="ts">
// Éditeur de grappe : la sidebar de saisie et l'ArrayViewer. Ce fichier ne
// fait qu'assembler — l'état vit dans `useClusterEditor`, fourni aux panneaux
// par provide/inject ; le viewer, lui, reçoit le résultat par props comme
// n'importe où ailleurs.

import { Separator } from "@/components/ui/separator";
import ArrayViewer from "@/components/array-viewer/ArrayViewer.vue";
import { useClusterEditor } from "./composables/useClusterEditor";
import { provideEditor } from "./context";
import ClusterList from "./panels/ClusterList.vue";
import GeneralFields from "./panels/GeneralFields.vue";
import RiggingPanel from "./panels/RiggingPanel.vue";
import SpeakerChain from "./panels/SpeakerChain.vue";
import EditorActions from "./panels/EditorActions.vue";

const editor = useClusterEditor();
provideEditor(editor);

const { form } = editor;
const { result } = editor.result;
const { isEditingExisting } = editor.library;
</script>

<template>
  <div class="flex h-full">
    <aside class="flex w-95 shrink-0 flex-col gap-4 overflow-y-auto border-r border-border bg-card p-4">
      <ClusterList />

      <Separator />

      <div class="flex flex-col gap-3">
        <h3 class="text-sm font-semibold">{{ isEditingExisting ? "Éditer" : "Créer" }}</h3>
        <GeneralFields />
        <RiggingPanel />
        <Separator />
        <SpeakerChain />
        <EditorActions />
      </div>
    </aside>

    <div class="min-h-0 flex-1 p-3">
      <ArrayViewer
        v-if="result"
        :result="result"
        :compartment="form.compartment"
        :view-key="form.id"
        class="h-full"
      />
      <div v-else class="flex h-full items-center justify-center text-sm text-muted-foreground">
        Enregistre la grappe pour voir la visualisation.
      </div>
    </div>
  </div>
</template>
