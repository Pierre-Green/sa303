<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

defineProps<{ text: string }>();
</script>

<template>
  <Tooltip>
    <TooltipTrigger as-child>
      <button
        type="button"
        tabindex="0"
        class="ml-1 inline-flex size-3.5 shrink-0 items-center justify-center rounded-full border border-border bg-card text-[9px] font-bold text-muted-foreground hover:bg-foreground hover:text-background focus-visible:outline-none"
        @click.prevent
      >
        i
      </button>
    </TooltipTrigger>
    <TooltipContent class="max-w-70 text-left leading-relaxed" side="top">
      {{ text }}
    </TooltipContent>
  </Tooltip>
</template>
