<script setup lang="ts">
import StatCard from './StatCard.vue'
import type { SystemInfo } from '@/lib/dashboardTypes'

defineProps<{ system: SystemInfo }>()

function fmt(value: number | null, digits: number, suffix: string): string {
  return value === null ? '—' : `${value.toFixed(digits)}${suffix}`
}

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
          <td>{{ fmt(system.loadAvg1, 2, '') }} / {{ fmt(system.loadAvg5, 2, '') }} / {{ fmt(system.loadAvg15, 2, '') }}
            <span class="text-neutral-400">(1/5/15m)</span></td>
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
