// Catalogue d'équipement en lecture seule : les modèles viennent des JSON
// livrés, l'application ne fait que les montrer. Charge les trois stores une
// fois et tient la sélection courante de chaque catégorie.

import { computed, onMounted, ref, watch } from "vue";
import { useSpeakerModelsStore } from "@/stores/speakerModels";
import { useBumperModelsStore } from "@/stores/bumperModels";
import { useBumperBarModelsStore } from "@/stores/bumperBarModels";

export function useEquipmentCatalog() {
  const speakers = useSpeakerModelsStore();
  const bumpers = useBumperModelsStore();
  const bumperBars = useBumperBarModelsStore();

  const selectedSpeakerId = ref<string | null>(null);
  const selectedBumperId = ref<string | null>(null);
  const selectedBumperBarId = ref<string | null>(null);

  onMounted(() =>
    Promise.all([speakers.fetchAll(), bumpers.fetchAll(), bumperBars.fetchAll()]),
  );

  // Premier élément sélectionné d'office : une fiche vide n'apprend rien.
  watch(() => speakers.items, (items) => (selectedSpeakerId.value ??= items[0]?.id ?? null));
  watch(() => bumpers.items, (items) => (selectedBumperId.value ??= items[0]?.id ?? null));
  watch(() => bumperBars.items, (items) => (selectedBumperBarId.value ??= items[0]?.id ?? null));

  const selectedSpeaker = computed(
    () => speakers.items.find((s) => s.id === selectedSpeakerId.value) ?? null,
  );
  const selectedBumper = computed(
    () => bumpers.items.find((b) => b.id === selectedBumperId.value) ?? null,
  );
  const selectedBumperBar = computed(
    () => bumperBars.items.find((b) => b.id === selectedBumperBarId.value) ?? null,
  );

  const speakerName = (id: string) => speakers.items.find((s) => s.id === id)?.name ?? id;
  const bumperName = (id: string) => bumpers.items.find((b) => b.id === id)?.name ?? id;

  return {
    speakers,
    bumpers,
    bumperBars,
    selectedSpeakerId,
    selectedBumperId,
    selectedBumperBarId,
    selectedSpeaker,
    selectedBumper,
    selectedBumperBar,
    speakerName,
    bumperName,
  };
}

export type EquipmentCatalog = ReturnType<typeof useEquipmentCatalog>;
