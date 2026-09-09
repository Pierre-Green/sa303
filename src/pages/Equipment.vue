<script setup lang="ts">
// Page "Équipement et enceinte" : les enceintes et le matériel qui s'y attache
// (bumpers, à terme rigbars, etc.). Chaque catégorie d'équipement a sa propre
// liste + formulaire, sur le même principe que l'éditeur de grappes. Le
// bumper et sa barre de déport (SA303-BUMPER-BAR) partagent une seule
// catégorie : la barre n'existe jamais indépendamment d'un bumper actif
// (c'est lui qui déclare sa compatibilité), les regrouper évite de les
// présenter comme deux équipements indépendants au choix de l'utilisateur.
// Prévu pour accueillir des STL par équipement plus tard.

import { computed, onMounted, reactive, ref, watch } from "vue";
import { useSpeakerModelsStore } from "@/stores/speakerModels";
import { useBumperModelsStore } from "@/stores/bumperModels";
import { useBumperBarModelsStore } from "@/stores/bumperBarModels";
import { api } from "@/lib/api";
import type { BumperBarModel, BumperModel, SpeakerGeometryReport } from "@/lib/types";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Separator } from "@/components/ui/separator";
import InfoTip from "@/components/InfoTip.vue";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

type Category = "enceintes" | "bumpers";
const category = ref<Category>("enceintes");

const speakerModelsStore = useSpeakerModelsStore();
const bumperModelsStore = useBumperModelsStore();
const bumperBarModelsStore = useBumperBarModelsStore();

onMounted(async () => {
  await Promise.all([
    speakerModelsStore.fetchAll(),
    bumperModelsStore.fetchAll(),
    bumperBarModelsStore.fetchAll(),
  ]);
  if (speakerModelsStore.items.length > 0) selectedSpeakerId.value = speakerModelsStore.items[0].id;
});

function fmt(v: number, d = 2) {
  return v.toFixed(d);
}

// ---------- Enceintes (lecture seule des valeurs dérivées, brief §9) ----------

const selectedSpeakerId = ref<string | null>(null);
const report = ref<SpeakerGeometryReport | null>(null);
const speakerError = ref<string | null>(null);

const selectedSpeaker = computed(
  () => speakerModelsStore.items.find((s) => s.id === selectedSpeakerId.value) ?? null,
);

async function recomputeSpeakerReport() {
  speakerError.value = null;
  report.value = null;
  if (!selectedSpeakerId.value) return;
  try {
    report.value = await api.getSpeakerGeometryReport(selectedSpeakerId.value);
  } catch (e) {
    speakerError.value = String(e);
  }
}

watch(selectedSpeakerId, recomputeSpeakerReport);

// ---------- Bumpers (CRUD complet) ----------

const selectedBumperId = ref<string | null>(null);
const bumperError = ref<string | null>(null);
const savingBumper = ref(false);

interface BumperFormState {
  id: string;
  name: string;
  schemaVersion: number;
  depth: number;
  height: number;
  shackleHeightAboveBumper: number;
  maxDirectDeportMm: number;
  compatibility: Record<string, { flown: boolean; stacked: boolean }>;
}

function blankBumperForm(): BumperFormState {
  return {
    id: crypto.randomUUID(),
    name: "Nouveau bumper",
    schemaVersion: 1,
    depth: 702,
    height: 100,
    shackleHeightAboveBumper: 40,
    maxDirectDeportMm: 351,
    compatibility: {},
  };
}

const bumperForm = reactive<BumperFormState>(blankBumperForm());

function loadIntoBumperForm(b: BumperModel) {
  const compatibility: BumperFormState["compatibility"] = {};
  for (const c of b.compatibleSpeakers) {
    compatibility[c.speakerModelId] = { flown: c.flown, stacked: c.stacked };
  }
  Object.assign(bumperForm, {
    id: b.id,
    name: b.name,
    schemaVersion: b.schemaVersion,
    depth: b.depth,
    height: b.height,
    shackleHeightAboveBumper: b.shackleHeightAboveBumper,
    maxDirectDeportMm: b.maxDirectDeportMm,
    compatibility,
  });
}

watch(selectedBumperId, (id) => {
  const bumper = bumperModelsStore.items.find((b) => b.id === id);
  if (bumper) loadIntoBumperForm(bumper);
});

const isEditingExistingBumper = computed(() =>
  bumperModelsStore.items.some((b) => b.id === bumperForm.id),
);

function newBumper() {
  selectedBumperId.value = null;
  Object.assign(bumperForm, blankBumperForm());
}

function compatFor(speakerId: string) {
  if (!bumperForm.compatibility[speakerId]) {
    bumperForm.compatibility[speakerId] = { flown: false, stacked: false };
  }
  return bumperForm.compatibility[speakerId];
}

function buildBumper(): BumperModel {
  return {
    id: bumperForm.id,
    name: bumperForm.name,
    schemaVersion: bumperForm.schemaVersion,
    depth: bumperForm.depth,
    height: bumperForm.height,
    shackleHeightAboveBumper: bumperForm.shackleHeightAboveBumper,
    maxDirectDeportMm: bumperForm.maxDirectDeportMm,
    compatibleSpeakers: Object.entries(bumperForm.compatibility)
      .filter(([, c]) => c.flown || c.stacked)
      .map(([speakerModelId, c]) => ({ speakerModelId, flown: c.flown, stacked: c.stacked })),
  };
}

async function saveBumper() {
  savingBumper.value = true;
  bumperError.value = null;
  try {
    const bumper = buildBumper();
    await bumperModelsStore.save(bumper);
    selectedBumperId.value = bumper.id;
  } catch (e) {
    bumperError.value = String(e);
  } finally {
    savingBumper.value = false;
  }
}

async function removeBumper() {
  if (!isEditingExistingBumper.value) return;
  await bumperModelsStore.remove(bumperForm.id);
  if (bumperModelsStore.items.length > 0) {
    selectedBumperId.value = bumperModelsStore.items[0].id;
  } else {
    newBumper();
  }
}

// ---------- Barres de déport (SA303-BUMPER-BAR, CRUD complet) ----------

const selectedBumperBarId = ref<string | null>(null);
const bumperBarError = ref<string | null>(null);
const savingBumperBar = ref(false);

interface BumperBarFormState {
  id: string;
  name: string;
  schemaVersion: number;
  maxDeportMm: number;
  compatibility: Record<string, boolean>;
}

function blankBumperBarForm(): BumperBarFormState {
  return {
    id: crypto.randomUUID(),
    name: "Nouvelle barre",
    schemaVersion: 1,
    maxDeportMm: 1500,
    compatibility: {},
  };
}

const bumperBarForm = reactive<BumperBarFormState>(blankBumperBarForm());

function loadIntoBumperBarForm(b: BumperBarModel) {
  const compatibility: BumperBarFormState["compatibility"] = {};
  for (const c of b.compatibleBumpers) {
    compatibility[c.bumperModelId] = true;
  }
  Object.assign(bumperBarForm, {
    id: b.id,
    name: b.name,
    schemaVersion: b.schemaVersion,
    maxDeportMm: b.maxDeportMm,
    compatibility,
  });
}

watch(selectedBumperBarId, (id) => {
  const bumperBar = bumperBarModelsStore.items.find((b) => b.id === id);
  if (bumperBar) loadIntoBumperBarForm(bumperBar);
});

const isEditingExistingBumperBar = computed(() =>
  bumperBarModelsStore.items.some((b) => b.id === bumperBarForm.id),
);

function newBumperBar() {
  selectedBumperBarId.value = null;
  Object.assign(bumperBarForm, blankBumperBarForm());
}

function buildBumperBar(): BumperBarModel {
  return {
    id: bumperBarForm.id,
    name: bumperBarForm.name,
    schemaVersion: bumperBarForm.schemaVersion,
    maxDeportMm: bumperBarForm.maxDeportMm,
    compatibleBumpers: Object.entries(bumperBarForm.compatibility)
      .filter(([, compatible]) => compatible)
      .map(([bumperModelId]) => ({ bumperModelId })),
  };
}

async function saveBumperBar() {
  savingBumperBar.value = true;
  bumperBarError.value = null;
  try {
    const bumperBar = buildBumperBar();
    await bumperBarModelsStore.save(bumperBar);
    selectedBumperBarId.value = bumperBar.id;
  } catch (e) {
    bumperBarError.value = String(e);
  } finally {
    savingBumperBar.value = false;
  }
}

async function removeBumperBar() {
  if (!isEditingExistingBumperBar.value) return;
  await bumperBarModelsStore.remove(bumperBarForm.id);
  if (bumperBarModelsStore.items.length > 0) {
    selectedBumperBarId.value = bumperBarModelsStore.items[0].id;
  } else {
    newBumperBar();
  }
}
</script>

<template>
  <div class="flex h-full">
    <aside class="flex w-[220px] shrink-0 flex-col gap-1 border-r border-border bg-card p-3">
      <button
        class="rounded-md px-3 py-2 text-left text-sm hover:bg-accent"
        :class="category === 'enceintes' ? 'bg-accent font-medium text-accent-foreground' : ''"
        @click="category = 'enceintes'"
      >
        Enceintes
      </button>
      <button
        class="rounded-md px-3 py-2 text-left text-sm hover:bg-accent"
        :class="category === 'bumpers' ? 'bg-accent font-medium text-accent-foreground' : ''"
        @click="category = 'bumpers'"
      >
        Bumper &amp; Barres de déport
      </button>
    </aside>

    <div class="flex-1 overflow-y-auto p-4">
      <!-- ================= ENCEINTES ================= -->
      <div v-if="category === 'enceintes'" class="grid grid-cols-1 gap-4 lg:grid-cols-[280px_1fr]">
        <Card>
          <CardHeader>
            <CardTitle class="text-sm">Enceintes</CardTitle>
          </CardHeader>
          <CardContent class="flex flex-col gap-1">
            <button
              v-for="s in speakerModelsStore.items"
              :key="s.id"
              class="rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent"
              :class="s.id === selectedSpeakerId ? 'bg-accent text-accent-foreground' : ''"
              @click="selectedSpeakerId = s.id"
            >
              {{ s.name }}
            </button>
          </CardContent>
        </Card>

        <div class="flex flex-col gap-4">
          <p v-if="speakerError" class="text-sm text-destructive">
            Incohérence géométrique : {{ speakerError }}
          </p>

          <div v-if="report && selectedSpeaker" class="grid grid-cols-1 gap-4 lg:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle class="text-sm">Valeurs dérivées</CardTitle>
              </CardHeader>
              <CardContent>
                <dl class="grid grid-cols-2 gap-x-4 gap-y-2 font-mono text-sm">
                  <dt class="flex items-center text-muted-foreground">
                    demi-angle (ha)
                    <InfoTip text="Moitié du dièdre de l'enceinte. Sert de référence pour toutes les positions angulaires de la zone orientation." />
                  </dt>
                  <dd>{{ fmt(report.ha, 1) }}°</dd>

                  <dt class="flex items-center text-muted-foreground">
                    HT (charnière av.-haut)
                    <InfoTip text="Trou avant-haut de l'enceinte. Goupille ronde dans trou rond : effort de direction quelconque, moment nul autour de l'axe." />
                  </dt>
                  <dd>{{ fmt(report.ht.x) }} ; {{ fmt(report.ht.y) }}</dd>

                  <dt class="flex items-center text-muted-foreground">
                    HB (charnière av.-bas)
                    <InfoTip text="Symétrique de HT. La bielle avant relie ce trou au pivot effectif de l'enceinte du dessous." />
                  </dt>
                  <dd>{{ fmt(report.hb.x) }} ; {{ fmt(report.hb.y) }}</dd>

                  <dt class="flex items-center text-muted-foreground">
                    PV (pivot effectif)
                    <InfoTip text="Trou avant-haut de l'enceinte inférieure, exprimé dans le repère de l'enceinte supérieure : c'est le centre de rotation réel de la jonction." />
                  </dt>
                  <dd>{{ fmt(report.pv.x) }} ; {{ fmt(report.pv.y) }}</dd>

                  <dt class="flex items-center text-muted-foreground">
                    Ancrage local
                    <InfoTip text="Ancrage de la barre orientation, sous la face supérieure. Un signe inversé ici renverserait complètement la direction de l'effort pivot." />
                  </dt>
                  <dd>{{ fmt(report.anchorLocal.x) }} ; {{ fmt(report.anchorLocal.y) }}</dd>

                  <dt class="flex items-center text-muted-foreground">
                    Entraxe bielle
                    <InfoTip text="Distance entre la goupille de bielle et le pivot effectif. Cet écart crée le moment que la butée de forme doit reprendre." />
                  </dt>
                  <dd>{{ fmt(report.bielleEntraxe) }} mm</dd>
                </dl>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle class="flex items-center text-sm">
                  Trous de couronne ({{ report.holes.length }})
                  <InfoTip text="Un trou par angle de la grille percée. Les splays impairs sont sur la couronne intérieure (retrait), donc avec un bras de levier différent des splays pairs — jamais une valeur unique." />
                </CardTitle>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Splay</TableHead>
                      <TableHead>Couronne</TableHead>
                      <TableHead>Rayon</TableHead>
                      <TableHead>Bras</TableHead>
                      <TableHead>
                        <span class="flex items-center">
                          Recoupement
                          <InfoTip text="Écart entre le bras reconstruit géométriquement et la formule trigonométrique de contrôle. Au-delà de 0,5 mm, la géométrie serait incohérente." />
                        </span>
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow v-for="h in report.holes" :key="h.splayDeg">
                      <TableCell>{{ h.splayDeg }}°</TableCell>
                      <TableCell>{{ h.row === "int" ? "intérieure" : "extérieure" }}</TableCell>
                      <TableCell>{{ fmt(h.radius, 0) }} mm</TableCell>
                      <TableCell>{{ fmt(h.leverMm, 1) }} mm</TableCell>
                      <TableCell>{{ fmt(h.discrepancyMm, 3) }} mm</TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>

      <!-- ================= BUMPER & BARRES DE DÉPORT ================= -->
      <div v-else class="flex flex-col gap-6">
        <div>
          <h3 class="mb-2 text-sm font-semibold text-muted-foreground">Bumpers</h3>
          <div class="grid grid-cols-1 gap-4 lg:grid-cols-[280px_1fr]">
            <Card>
              <CardHeader>
                <div class="flex items-center justify-between">
                  <CardTitle class="text-sm">Bumpers</CardTitle>
                  <Button size="sm" variant="outline" @click="newBumper">+ Ajouter</Button>
                </div>
              </CardHeader>
              <CardContent class="flex flex-col gap-1">
                <button
                  v-for="b in bumperModelsStore.items"
                  :key="b.id"
                  class="rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent"
                  :class="b.id === selectedBumperId ? 'bg-accent text-accent-foreground' : ''"
                  @click="selectedBumperId = b.id"
                >
                  {{ b.name }}
                </button>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle class="text-sm">{{ isEditingExistingBumper ? "Éditer" : "Créer" }} un bumper</CardTitle>
              </CardHeader>
              <CardContent class="flex flex-col gap-3">
                <div class="flex flex-col gap-1.5">
                  <Label class="text-xs">Nom</Label>
                  <Input v-model="bumperForm.name" class="w-72" />
                </div>

                <div class="grid grid-cols-3 gap-2 w-96">
                  <div class="flex flex-col gap-1.5">
                    <Label class="flex items-center text-xs">
                      Profondeur<InfoTip text="Étendue avant-arrière du bumper. Le SA303-BUMPER fait 702 mm, tube 100×50." />
                    </Label>
                    <Input v-model.number="bumperForm.depth" type="number" />
                  </div>
                  <div class="flex flex-col gap-1.5">
                    <Label class="flex items-center text-xs">
                      Hauteur<InfoTip text="Épaisseur du bumper (dimension verticale du tube)." />
                    </Label>
                    <Input v-model.number="bumperForm.height" type="number" />
                  </div>
                  <div class="flex flex-col gap-1.5">
                    <Label class="flex items-center text-xs">
                      Manille<InfoTip text="Hauteur à laquelle la manille se ferme au-dessus du dessus du bumper. La SA303-BUMPER-BAR n'entre volontairement pas dans ce modèle : ses perçages ne sont pas à hauteur constante." />
                    </Label>
                    <Input v-model.number="bumperForm.shackleHeightAboveBumper" type="number" />
                  </div>
                </div>

                <div class="grid grid-cols-2 gap-2 w-64">
                  <div class="flex flex-col gap-1.5">
                    <Label class="flex items-center text-xs">
                      Déport direct max<InfoTip text="Décalage maximal de l'accroche par rapport au centre du bumper avant qu'une SA303-BUMPER-BAR ne soit nécessaire. Par défaut la moitié de la profondeur." />
                    </Label>
                    <Input v-model.number="bumperForm.maxDirectDeportMm" type="number" />
                  </div>
                </div>

                <Separator />

                <div>
                  <Label class="mb-1.5 flex items-center text-xs">
                    Compatibilité par enceinte
                    <InfoTip text="Un bumper peut être posable en vol, en stack, ou les deux, selon l'enceinte. Décoché des deux côtés : le bumper n'apparaît pas comme option pour cette enceinte." />
                  </Label>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Enceinte</TableHead>
                        <TableHead>Vol</TableHead>
                        <TableHead>Stack</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      <TableRow v-for="speaker in speakerModelsStore.items" :key="speaker.id">
                        <TableCell>{{ speaker.name }}</TableCell>
                        <TableCell><Checkbox v-model="compatFor(speaker.id).flown" /></TableCell>
                        <TableCell><Checkbox v-model="compatFor(speaker.id).stacked" /></TableCell>
                      </TableRow>
                    </TableBody>
                  </Table>
                </div>

                <div class="flex gap-2">
                  <Button :disabled="savingBumper" @click="saveBumper">Enregistrer</Button>
                  <Button v-if="isEditingExistingBumper" variant="destructive" @click="removeBumper">Supprimer</Button>
                </div>
                <p v-if="bumperError" class="text-xs text-destructive">{{ bumperError }}</p>
              </CardContent>
            </Card>
          </div>
        </div>

        <Separator />

        <div>
          <h3 class="mb-2 text-sm font-semibold text-muted-foreground">Barres de déport</h3>
          <div class="grid grid-cols-1 gap-4 lg:grid-cols-[280px_1fr]">
            <Card>
              <CardHeader>
                <div class="flex items-center justify-between">
                  <CardTitle class="text-sm">Barres</CardTitle>
                  <Button size="sm" variant="outline" @click="newBumperBar">+ Ajouter</Button>
                </div>
              </CardHeader>
              <CardContent class="flex flex-col gap-1">
                <button
                  v-for="b in bumperBarModelsStore.items"
                  :key="b.id"
                  class="rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent"
                  :class="b.id === selectedBumperBarId ? 'bg-accent text-accent-foreground' : ''"
                  @click="selectedBumperBarId = b.id"
                >
                  {{ b.name }}
                </button>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle class="text-sm">{{ isEditingExistingBumperBar ? "Éditer" : "Créer" }} une barre</CardTitle>
              </CardHeader>
              <CardContent class="flex flex-col gap-3">
                <div class="flex flex-col gap-1.5">
                  <Label class="text-xs">Nom</Label>
                  <Input v-model="bumperBarForm.name" class="w-72" />
                </div>

                <div class="flex flex-col gap-1.5 w-64">
                  <Label class="flex items-center text-xs">
                    Portée max<InfoTip text="Portée maximale de déport depuis le centre du bumper. Au-delà, la barre seule ne suffit plus : une tirette est automatiquement mise en place par le solveur, sur le point 0° arrière-bas de l'enceinte du bas." />
                  </Label>
                  <Input v-model.number="bumperBarForm.maxDeportMm" type="number" />
                </div>

                <Separator />

                <div>
                  <Label class="mb-1.5 flex items-center text-xs">
                    Compatibilité par bumper
                    <InfoTip text="Une barre ne se greffe que sur les bumpers listés ici. Décoché : la barre n'apparaît pas comme option pour ce bumper." />
                  </Label>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Bumper</TableHead>
                        <TableHead>Compatible</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      <TableRow v-for="bumper in bumperModelsStore.items" :key="bumper.id">
                        <TableCell>{{ bumper.name }}</TableCell>
                        <TableCell><Checkbox v-model="bumperBarForm.compatibility[bumper.id]" /></TableCell>
                      </TableRow>
                    </TableBody>
                  </Table>
                </div>

                <div class="flex gap-2">
                  <Button :disabled="savingBumperBar" @click="saveBumperBar">Enregistrer</Button>
                  <Button v-if="isEditingExistingBumperBar" variant="destructive" @click="removeBumperBar">Supprimer</Button>
                </div>
                <p v-if="bumperBarError" class="text-xs text-destructive">{{ bumperBarError }}</p>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
