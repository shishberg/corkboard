<script setup lang="ts">
import StatCard from './StatCard.vue'
import { formatDuration } from '@/lib/dashboardFormat'
import type { DashboardStatus } from '@/lib/dashboardTypes'

defineProps<{ status: DashboardStatus }>()
</script>

<template>
  <StatCard title="Device">
    <table class="w-full text-sm">
      <tbody>
        <tr>
          <td class="pr-4 text-neutral-500">Hostname</td>
          <td>{{ status.hostname }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Platform</td>
          <td>{{ status.display.os }} / {{ status.display.arch }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Display</td>
          <td>{{ status.display.kind }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Uptime</td>
          <td>{{ formatDuration(status.uptimeSecs) }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Poll interval</td>
          <td>{{ status.pollIntervalMinutes }} min</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Data dir</td>
          <td>{{ status.env.dataDir }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Dist dir</td>
          <td>{{ status.env.distDir }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Fonts dir</td>
          <td>{{ status.env.fontsDir }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Port</td>
          <td>{{ status.env.port }}</td>
        </tr>
      </tbody>
    </table>
  </StatCard>
</template>
