<script setup lang="ts">
import { computed } from 'vue'
import type { CalendarEl } from '@/stores/types'

const props = defineProps<{ el: CalendarEl }>()
const days = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']
const isWeek = computed(() => props.el.variant === 'week')
const rows = computed(() =>
  props.el.events.length ? props.el.events.map((e) => e.title) : ['Standup 9:00', 'Lunch 12:30'],
)
</script>

<template>
  <div class="h-full w-full overflow-hidden bg-white p-2 text-xs">
    <div v-if="isWeek" class="grid grid-cols-7 gap-1">
      <div v-for="d in days" :key="d" data-role="day" class="border p-1 text-center">{{ d }}</div>
    </div>
    <div v-else>
      <div class="mb-1 font-bold">Today</div>
      <div v-for="(r, i) in rows" :key="i" data-role="event" class="border-b py-0.5">{{ r }}</div>
    </div>
  </div>
</template>
