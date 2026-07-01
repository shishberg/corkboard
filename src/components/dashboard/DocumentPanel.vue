<script setup lang="ts">
import StatCard from './StatCard.vue'
import { formatAgo } from '@/lib/dashboardFormat'
import type { DocumentInfo } from '@/lib/dashboardTypes'

defineProps<{ document: DocumentInfo }>()
</script>

<template>
  <StatCard title="Document">
    <table class="w-full text-sm">
      <tbody>
        <tr>
          <td class="pr-4 text-neutral-500">Pages</td>
          <td>{{ document.pageCount }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Live page</td>
          <td>{{ document.livePageName ?? document.livePageId ?? '(none)' }}</td>
        </tr>
        <tr>
          <td class="pr-4 text-neutral-500">Last published</td>
          <td>{{ document.savedAtMs ? formatAgo(document.savedAtMs) : 'never' }}</td>
        </tr>
      </tbody>
    </table>
  </StatCard>
</template>
