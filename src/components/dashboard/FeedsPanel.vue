<script setup lang="ts">
import StatCard from './StatCard.vue'
import { formatAgo } from '@/lib/dashboardFormat'
import type { FeedInfo } from '@/lib/dashboardTypes'

defineProps<{ feeds: FeedInfo[] }>()

function badgeClass(f: FeedInfo): string {
  if (f.ok === true) return 'bg-green-100 text-green-700'
  if (f.ok === false) return 'bg-red-100 text-red-700'
  return 'bg-neutral-100 text-neutral-500'
}

function badgeText(f: FeedInfo): string {
  if (f.ok === true) return 'ok'
  if (f.ok === false) return 'error'
  return 'not fetched yet'
}
</script>

<template>
  <StatCard title="Calendar feeds">
    <table class="w-full text-sm">
      <thead>
        <tr class="text-left text-neutral-500">
          <th class="pb-1 pr-3 font-normal">Feed</th>
          <th class="pb-1 pr-3 font-normal">Status</th>
          <th class="pb-1 pr-3 font-normal">Last attempt</th>
          <th class="pb-1 pr-3 font-normal">Events today</th>
          <th class="pb-1 font-normal">Detail</th>
        </tr>
      </thead>
      <tbody>
        <tr v-if="feeds.length === 0">
          <td colspan="5" class="italic text-neutral-500">no feeds configured</td>
        </tr>
        <tr v-for="f in feeds" :key="f.id">
          <td class="pr-3">{{ f.name }}</td>
          <td class="pr-3"><span class="rounded-full px-2 py-0.5 text-xs" :class="badgeClass(f)">{{ badgeText(f) }}</span></td>
          <td class="pr-3">{{ f.lastAttemptMs ? formatAgo(f.lastAttemptMs) : '—' }}</td>
          <td class="pr-3">{{ f.todayEventCount ?? '—' }}</td>
          <td>{{ f.error ?? '' }}</td>
        </tr>
      </tbody>
    </table>
  </StatCard>
</template>
