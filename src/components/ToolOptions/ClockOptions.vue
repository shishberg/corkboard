<script setup lang="ts">
import { useToolOptionsStore } from '@/stores/toolOptions'
import { Clock, CalendarClock, Calendar } from '@lucide/vue'

const opts = useToolOptionsStore()
const variants = [
  { id: 'time', icon: Clock, label: 'Time' },
  { id: 'time-date', icon: CalendarClock, label: 'Time + date' },
  { id: 'date', icon: Calendar, label: 'Date' },
] as const
</script>

<template>
  <div class="flex flex-col gap-1">
    <p class="mb-1 text-xs font-medium text-neutral-500">Clock style</p>
    <button
      v-for="v in variants"
      :key="v.id"
      class="flex items-center gap-2 rounded px-2 py-1 text-sm hover:bg-neutral-100"
      :class="opts.clockVariant === v.id ? 'bg-neutral-100 font-medium' : ''"
      @click="opts.clockVariant = v.id"
    >
      <component :is="v.icon" class="h-4 w-4" />
      {{ v.label }}
    </button>
  </div>
</template>
