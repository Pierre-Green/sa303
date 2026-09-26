<script setup lang="ts">
// La chaîne d'enceintes, une ligne par enceinte dans l'ordre de montage.
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useEditor } from "../context";
import { STACK_TILT_OPTIONS } from "../composables/useClusterForm";

const { form, chain } = useEditor();
const { rows } = chain;
</script>

<template>
  <div>
    <div class="mb-1.5 flex items-center justify-between">
      <Label class="flex items-center text-xs">
        Enceintes — {{ form.compartment === "stacked" ? "du bas vers le haut" : "du haut vers le bas" }}
        <InfoTip
          text="Listées dans l'ordre de montage : en stack on part du sol (l'enceinte n°1 est celle qui porte le bumper et l'angle de calage), en grappe on part de l'accroche. L'enceinte de référence n'a pas de splay propre : son inclinaison ancre toute la chaîne."
        />
      </Label>
      <Button size="sm" variant="outline" @click="chain.add">+ enceinte</Button>
    </div>
    <div class="flex flex-col gap-1.5">
      <div v-for="(row, idx) in rows" :key="idx" class="flex items-center gap-2">
        <span class="w-20 shrink-0 text-xs text-muted-foreground">{{ row.label }}</span>

        <div class="flex-1">
          <Select v-if="row.kind === 'splay'" v-model="form.splays[row.splayIndex!]">
            <SelectTrigger class="w-full"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem v-for="s in chain.splayGridAt(row.splayIndex!)" :key="s" :value="s">{{ s }}°</SelectItem>
            </SelectContent>
          </Select>

          <Select v-else-if="row.kind === 'stack-reference'" v-model="form.imposedTilt">
            <SelectTrigger class="w-full"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem v-for="t in STACK_TILT_OPTIONS" :key="t" :value="t">{{ t }}°</SelectItem>
            </SelectContent>
          </Select>

          <Select v-else disabled :model-value="0">
            <SelectTrigger class="w-full text-muted-foreground"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem :value="0">0°</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div class="w-40 shrink-0">
          <Select v-model="form.speakerModelIds[row.dataIndex]">
            <SelectTrigger class="w-full"><SelectValue placeholder="Enceinte" /></SelectTrigger>
            <SelectContent>
              <SelectItem v-for="s in chain.compatibleAt(row.dataIndex)" :key="s.id" :value="s.id">
                {{ s.name }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- Conseil de calage, jamais une erreur : la jonction reste
             mécaniquement valable même hors de la plage recommandée. -->
        <span
          v-if="row.splayIndex !== null && chain.acousticWarning(row.splayIndex)"
          class="w-4 shrink-0 cursor-help text-center text-status-alarm"
          :title="chain.acousticWarning(row.splayIndex)!"
        >
          ⚠
        </span>
        <span v-else class="w-4 shrink-0" />

        <Button
          v-if="row.kind === 'splay'"
          size="sm"
          variant="ghost"
          :disabled="form.splays.length <= 1"
          @click="chain.remove(row.dataIndex, row.splayIndex!)"
        >
          ✕
        </Button>
        <span v-else class="w-9 shrink-0" />
      </div>
    </div>
  </div>
</template>
