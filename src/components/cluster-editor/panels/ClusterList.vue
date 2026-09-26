<script setup lang="ts">
// Grappes enregistrées, cases d'export d'audit et bouton d'export.
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import { useEditor } from "../context";

const { library, audit } = useEditor();
const { sorted, selectedId, items } = library;
const { selection, scope, exporting, message } = audit;
</script>

<template>
  <div>
    <div class="mb-2 flex items-center justify-between">
      <h2 class="text-sm font-semibold">Grappes &amp; stacks</h2>
      <Button size="sm" variant="outline" @click="library.create">+ Ajouter</Button>
    </div>
    <div class="flex flex-col gap-1">
      <div
        v-for="c in sorted"
        :key="c.id"
        class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm hover:bg-accent"
        :class="c.id === selectedId ? 'bg-accent text-accent-foreground' : ''"
      >
        <Checkbox
          :model-value="selection.has(c.id)"
          :aria-label="`Cocher ${c.name} pour l'export d'audit`"
          @update:model-value="audit.toggle(c.id)"
        />
        <button class="flex flex-1 items-center justify-between text-left" @click="selectedId = c.id">
          <span>{{ c.name }}</span>
          <Badge :class="c.compartment === 'flown' ? 'bg-zone-orientation' : 'bg-zone-lift'" class="text-white">
            {{ c.compartment === "flown" ? "vol" : "stack" }}
          </Badge>
        </button>
      </div>
    </div>

    <div class="mt-3 flex flex-col gap-1.5">
      <Button size="sm" variant="outline" :disabled="exporting || items.length === 0" @click="audit.run">
        {{ exporting ? "Export en cours…" : "Exporter pour audit" }}
      </Button>
      <p class="flex items-center text-xs text-muted-foreground">
        Portée : {{ scope.label }}
        <InfoTip
          text="JSON autoportant : d'abord les définitions des enceintes, bumpers et barres utilisés, puis chaque grappe avec le détail de ses jonctions — positions, efforts, moments de barre, efforts sur chaque goupille et taux de travail des cinq chemins. Cocher des grappes exporte la sélection ; sans coche, la grappe affichée ; sans grappe enregistrée affichée, tout le catalogue."
        />
      </p>
      <p v-if="message" class="break-all text-xs text-status-ok">{{ message }}</p>
    </div>
  </div>
</template>
