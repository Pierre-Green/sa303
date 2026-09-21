<script setup lang="ts">
// Le réglage de la ligne d'écoute. Hauteur d'oreille d'un public debout par
// défaut (1700 mm) : gradins, assis ou balcon se règlent ici.

const model = defineModel<number>({ required: true });

const MIN_MM = 0;
const MAX_MM = 20000;

function onInput(e: Event) {
  const value = Number((e.target as HTMLInputElement).value);
  if (!Number.isFinite(value)) return;
  model.value = Math.min(Math.max(value, MIN_MM), MAX_MM);
}
</script>

<template>
  <label
    class="absolute right-2 top-2 flex items-center gap-1.5 rounded-md border border-border bg-card/90 px-2 py-1 font-mono text-[11px] text-muted-foreground"
  >
    <span>écoute</span>
    <input
      :value="model"
      type="number"
      :min="MIN_MM"
      :max="MAX_MM"
      step="50"
      class="w-16 rounded-sm border border-border bg-background px-1 py-0.5 text-right font-mono text-[11px] text-foreground"
      @input="onInput"
    />
    <span>mm</span>
  </label>
</template>
