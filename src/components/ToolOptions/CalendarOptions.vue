<script setup lang="ts">
import { onMounted, computed, watch } from 'vue'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { useFeedsStore } from '@/stores/feeds'

const opts = useToolOptionsStore()
const feeds = useFeedsStore()

const variants = [
  { id: 'date', label: 'Date' },
  { id: 'today', label: 'Today' },
  { id: 'agenda', label: 'Agenda' },
] as const

const hasFeeds = computed(() => feeds.feeds.length > 0)

// "(none)" is only an option when there are no feeds. So whenever feeds are
// available but the current selection isn't one of them (e.g. the default
// empty value), fall back to the first feed so the picker shows a real choice.
watch(
  () => feeds.feeds,
  (list) => {
    if (list.length > 0 && !list.some((f) => f.id === opts.feedId)) {
      opts.feedId = list[0].id
    }
  },
  { immediate: true },
)

onMounted(() => {
  feeds.loadFeeds()
})
</script>

<template>
  <div class="flex flex-col gap-1">
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
      <option v-if="!hasFeeds" value="">(none)</option>
      <option v-for="feed in feeds.feeds" :key="feed.id" :value="feed.id">{{ feed.name }}</option>
    </select>
  </div>
</template>
