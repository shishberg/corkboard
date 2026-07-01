<script setup lang="ts">
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
  <div>
    <h2 class="mb-3 text-xs font-semibold tracking-wide text-neutral-500 uppercase">Calendar feeds</h2>
    <div v-if="feeds.length === 0" class="rounded-lg border bg-white p-4 text-sm italic text-neutral-500">
      no feeds configured
    </div>
    <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-2">
      <div v-for="f in feeds" :key="f.id" class="rounded-lg border bg-white p-4">
        <div class="mb-2 flex items-center justify-between gap-2">
          <span class="text-sm font-medium">{{ f.name }}</span>
          <span class="shrink-0 rounded-full px-2 py-0.5 text-xs" :class="badgeClass(f)">{{ badgeText(f) }}</span>
        </div>
        <div class="mb-2 text-xs text-neutral-500">
          last attempt {{ f.lastAttemptMs ? formatAgo(f.lastAttemptMs) : '—' }}
        </div>
        <p v-if="f.error" class="mb-2 text-sm text-red-600">{{ f.error }}</p>
        <p v-if="f.week.length === 0" class="text-sm italic text-neutral-500">not resolved yet</p>
        <div v-else class="space-y-1.5 text-sm">
          <div v-for="day in f.week" :key="day.label" class="flex gap-2">
            <span class="w-20 shrink-0 text-neutral-500">{{ day.label }}</span>
            <div v-if="day.events.length > 0" class="space-y-0.5">
              <div v-for="(e, i) in day.events" :key="i">
                <span class="text-neutral-500">{{ e.time || 'all day' }}</span> {{ e.title }}
              </div>
            </div>
            <span v-else class="text-neutral-400">—</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
