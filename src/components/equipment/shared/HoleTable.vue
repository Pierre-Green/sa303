<script setup lang="ts">
// Liste de trous en lecture seule, coordonnées dans le repère de l'équipement.
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import InfoTip from "@/components/ui/info-tip/InfoTip.vue";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { fmt } from "../format";

defineProps<{
  title: string;
  tip?: string;
  holes: [number, number][];
}>();
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center text-sm">
        {{ title }} ({{ holes.length }})
        <InfoTip v-if="tip" :text="tip" />
      </CardTitle>
    </CardHeader>
    <CardContent class="max-h-72 overflow-y-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>#</TableHead>
            <TableHead>X (mm)</TableHead>
            <TableHead>Y (mm)</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody class="font-mono">
          <TableRow v-for="([x, y], i) in holes" :key="i">
            <TableCell>{{ i + 1 }}</TableCell>
            <TableCell>{{ fmt(x, 3) }}</TableCell>
            <TableCell>{{ fmt(y, 3) }}</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </CardContent>
  </Card>
</template>
