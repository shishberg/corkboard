<script setup lang="ts">
import { computed } from 'vue'
import StatCard from './StatCard.vue'
import type { SystemInfo } from '@/lib/dashboardTypes'

const props = defineProps<{ system: SystemInfo }>()

function fmt(value: number | null, digits: number, suffix: string): string {
  return value === null ? '—' : `${value.toFixed(digits)}${suffix}`
}

// Oldest (15m) to newest (1m), left to right.
const loadAvgBars = computed(() => {
  const values = [props.system.loadAvg15, props.system.loadAvg5, props.system.loadAvg1]
  if (values.every((v) => v === null)) return null
  const max = Math.max(...values.map((v) => v ?? 0), 0.01)
  // A small minimum height so a near-zero load still shows a visible sliver.
  return values.map((v) => ({ value: v, heightPct: v === null ? 0 : Math.max(6, (v / max) * 100) }))
})

const loadAvgTooltip = computed(
  () => `15m: ${fmt(props.system.loadAvg15, 2, '')} · 5m: ${fmt(props.system.loadAvg5, 2, '')} · 1m: ${fmt(props.system.loadAvg1, 2, '')}`,
)

function fmtMem(usedKb: number | null, totalKb: number | null): string {
  if (usedKb === null || totalKb === null) return '—'
  const usedMb = Math.round(usedKb / 1024)
  const totalMb = Math.round(totalKb / 1024)
  return `${usedMb} / ${totalMb} MB`
}
</script>

<template>
  <StatCard title="System">
    <table class="w-full text-sm">
      <tbody>
        <tr>
          <td class="pr-4 text-neutral-500">CPU temp</td>
          <td>{{ fmt(system.cpuTempC, 1, '°C') }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">CPU freq</td>
          <td>{{ system.cpuFreqMhz === null ? '—' : `${system.cpuFreqMhz} MHz` }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Load avg</td>
          <td>
            <div v-if="loadAvgBars" class="flex h-6 w-fit items-end gap-1" :title="loadAvgTooltip">
              <div
                v-for="(bar, i) in loadAvgBars"
                :key="i"
                data-role="load-avg-bar"
                class="w-2 rounded-t bg-neutral-400"
                :style="{ height: bar.heightPct + '%' }"
              />
            </div>
            <span v-else>—</span>
          </td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Memory used</td>
          <td>
            {{
              fmtMem(
                system.memTotalKb !== null && system.memAvailableKb !== null
                  ? system.memTotalKb - system.memAvailableKb
                  : null,
                system.memTotalKb,
              )
            }}
          </td>
        </tr>
      </tbody>
    </table>
  </StatCard>
</template>
