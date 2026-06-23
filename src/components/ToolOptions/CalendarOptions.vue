<script setup lang="ts">
import { useToolOptionsStore } from '@/stores/toolOptions'

const opts = useToolOptionsStore()
const variants = [
  { id: 'today', label: 'Today' },
  { id: 'week', label: 'Week' },
] as const
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
  </div>
</template>
