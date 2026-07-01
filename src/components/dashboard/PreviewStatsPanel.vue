<script setup lang="ts">
import StatCard from './StatCard.vue'
import { formatAgo } from '@/lib/dashboardFormat'
import type { PreviewInfo } from '@/lib/dashboardTypes'

defineProps<{ preview: PreviewInfo }>()
</script>

<template>
  <StatCard title="Preview stats">
    <table class="w-full text-sm">
      <tbody>
        <tr>
          <td class="pr-4 text-neutral-500">Last rendered</td>
          <td>{{ preview.updatedAtMs ? formatAgo(preview.updatedAtMs) : 'never' }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Renders this run</td>
          <td>{{ preview.renderCount }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Connected listeners</td>
          <td>{{ preview.connectedListeners }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Last calendar poll</td>
          <td>{{ preview.lastPollAtMs ? formatAgo(preview.lastPollAtMs) : 'never' }}</td>
        </tr>
      </tbody>
    </table>
  </StatCard>
</template>
