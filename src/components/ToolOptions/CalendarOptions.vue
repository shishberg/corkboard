<script setup lang="ts">
import { onMounted } from 'vue'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { useFeedsStore } from '@/stores/feeds'

const opts = useToolOptionsStore()
const feeds = useFeedsStore()

const variants = [
  { id: 'date', label: 'Date' },
  { id: 'today', label: 'Today' },
  { id: 'week', label: 'Week' },
] as const

onMounted(() => {
  feeds.loadFeeds()
})
</script>

<template>
  <div class="flex flex-col gap-1">
    <p class="mb-1 text-xs font-medium text-neutral-500">Calendar view</p>
    <button
      v-for="v in variants"
      :key="v.id"
      class="rounded px-2 py-1 text-left text-sm hover:bg-neutral-100"
      :class="opts.calendarVariant === v.id ? 'bg-neutral-100 font-medium' : ''"
      @click="opts.calendarVariant = v.id"
    >
      {{ v.label }}
    </button>
    <p class="mb-1 mt-2 text-xs font-medium text-neutral-500">Feed</p>
    <select
      data-role="feed-select"
      class="rounded border px-1 py-1 text-sm"
      :value="opts.feedId"
      @change="opts.feedId = ($event.target as HTMLSelectElement).value"
    >
      <option value="">(none)</option>
      <option v-for="feed in feeds.feeds" :key="feed.id" :value="feed.id">{{ feed.name }}</option>
    </select>
  </div>
</template>
