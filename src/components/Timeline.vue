<script setup lang="ts">
import { usePagesStore } from '@/stores/pages'
import TimelineItem from './TimelineItem.vue'

const store = usePagesStore()

function onDrop(e: DragEvent) {
  // Strip-level drop (empty area): append to the end.
  const data = e.dataTransfer?.getData('text/plain') ?? ''
  if (data.startsWith('idx:')) {
    const from = Number(data.slice(4))
    store.reorderTimeline(from, store.timeline.length - 1)
  } else if (data) {
    store.addToTimeline(data)
  }
}

function onItemDrop(data: string, index: number) {
  // Drop onto a specific item: the dragged item takes that item's slot index
  // (the others shift to fill the gap). Page ids from the sidebar still append.
  if (data.startsWith('idx:')) {
    store.reorderTimeline(Number(data.slice(4)), index)
  } else if (data) {
    store.addToTimeline(data)
  }
}
</script>

<template>
  <div
    data-role="timeline-strip"
    class="flex h-24 items-center gap-2 overflow-x-auto border-t bg-neutral-50 p-2"
    @dragover.prevent
    @drop.prevent="onDrop"
  >
    <p v-if="!store.timeline.length" class="text-xs text-neutral-400">
      Drag pages here to set the loop order.
    </p>
    <TimelineItem
      v-for="(entry, i) in store.timeline"
      :key="`${entry.pageId}-${i}`"
      :index="i"
      :page-id="entry.pageId"
      :delay-ms="entry.delayMs"
      @set-delay="store.setTimelineDelay(i, $event)"
      @remove="store.removeFromTimeline(i)"
      @drop-data="onItemDrop($event, i)"
    />
  </div>
</template>
