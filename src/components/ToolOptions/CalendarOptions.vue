<script setup lang="ts">
import { onMounted, computed, watch } from 'vue'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { useFeedsStore } from '@/stores/feeds'
import { usePagesStore } from '@/stores/pages'
import type { CalendarEl } from '@/stores/types'

const opts = useToolOptionsStore()
const feeds = useFeedsStore()
const store = usePagesStore()

const variants = [
  { id: 'date', label: 'Date' },
  { id: 'today', label: 'Today' },
  { id: 'agenda', label: 'Agenda' },
] as const

const hasFeeds = computed(() => feeds.feeds.length > 0)

const selectedCalEl = computed((): CalendarEl | null => {
  const id = store.selectedElId
  if (!id) return null
  const el = store.selectedPage?.elements.find((e) => e.id === id)
  return el?.type === 'calendar' ? (el as CalendarEl) : null
})

// Editing a selected calendar shows that element's settings; otherwise the tool
// defaults used for new calendars.
const variantValue = computed(() => selectedCalEl.value?.variant ?? opts.calendarVariant)
const feedValue = computed(() => selectedCalEl.value?.feedId ?? opts.feedId)

function pickVariant(variant: CalendarEl['variant']) {
  opts.calendarVariant = variant
  if (selectedCalEl.value) {
    store.setElementVariant(selectedCalEl.value.id, variant)
  }
}

function pickFeed(feedId: string) {
  opts.feedId = feedId
  if (selectedCalEl.value) {
    store.setElementFeed(selectedCalEl.value.id, feedId)
  }
}

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
  <div class="flex items-center gap-1">
    <button
      v-for="v in variants"
      :key="v.id"
      :data-variant="v.id"
      class="rounded px-2 py-1 text-sm hover:bg-neutral-100"
      :class="variantValue === v.id ? 'bg-neutral-100 font-medium' : ''"
      @click="pickVariant(v.id)"
    >
      {{ v.label }}
    </button>
    <select
      data-role="feed-select"
      class="ml-1 rounded border px-1 py-1 text-sm"
      :value="feedValue"
      @change="pickFeed(($event.target as HTMLSelectElement).value)"
    >
      <option v-if="!hasFeeds" value="">(none)</option>
      <option v-for="feed in feeds.feeds" :key="feed.id" :value="feed.id">{{ feed.name }}</option>
    </select>
  </div>
</template>
