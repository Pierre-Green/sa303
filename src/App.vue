<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router";
import { TooltipProvider } from "@/components/ui/tooltip";
</script>

<template>
  <TooltipProvider :delay-duration="150">
    <div class="flex h-screen flex-col overflow-hidden bg-background">
      <header class="flex shrink-0 items-center gap-4 border-b border-border bg-card px-4 py-2">
        <span class="text-sm font-semibold text-primary">SA303 Rigging</span>
        <nav class="flex gap-1 text-sm">
          <RouterLink
            to="/clusters"
            class="rounded-md px-3 py-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground"
            active-class="bg-accent text-accent-foreground font-medium"
          >
            Grappes &amp; stacks
          </RouterLink>
          <RouterLink
            to="/equipment"
            class="rounded-md px-3 py-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground"
            active-class="bg-accent text-accent-foreground font-medium"
          >
            Équipement et enceinte
          </RouterLink>
          <RouterLink
            to="/aggregate"
            class="rounded-md px-3 py-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground"
            active-class="bg-accent text-accent-foreground font-medium"
          >
            Agrégat
          </RouterLink>
          <RouterLink
            to="/toolbox"
            class="rounded-md px-3 py-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground"
            active-class="bg-accent text-accent-foreground font-medium"
          >
            Boîte à outils
          </RouterLink>
        </nav>
      </header>
      <main class="min-h-0 flex-1">
        <RouterView />
      </main>
    </div>
  </TooltipProvider>
</template>
