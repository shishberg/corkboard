<script setup lang="ts">
import { computed } from 'vue'
import type { CalendarEl } from '@/stores/types'
import { formatSampleDate, SAMPLE_TODAY_EVENTS, SAMPLE_WEEK } from '@/lib/sampleCalendar'

const props = defineProps<{ el: CalendarEl }>()

// Base font tracks the smaller dimension so text scales with the container
// without overflowing a narrow week grid.
const baseSize = computed(() => `${Math.max(6, Math.min(props.el.w, props.el.h) * 0.06)}px`)
const variant = computed(() => props.el.variant)
const formattedDate = computed(() => formatSampleDate())
</script>

<template>
  <div data-role="calendar-root" class="h-full w-full overflow-hidden bg-white p-2" :style="{ fontSize: baseSize, color: el.colour }">
    <!-- Date variant: large centred formatted date -->
    <div v-if="variant === 'date'" data-role="calendar-date" class="flex h-full items-center justify-center text-center font-bold">
      {{ formattedDate }}
    </div>

    <!-- Week variant: 7-day grid -->
    <div v-else-if="variant === 'week'" class="grid grid-cols-7 gap-1">
      <div v-for="d in SAMPLE_WEEK" :key="d.day" class="border p-1">
        <div data-role="day" class="text-center font-medium">{{ d.day }}</div>
        <div v-for="ev in d.events" :key="ev.time" class="truncate text-xs">{{ ev.title }}</div>
      </div>
    </div>

    <!-- Today variant: heading + event list -->
    <div v-else>
      <div class="mb-1 font-bold">Today</div>
      <div v-for="ev in SAMPLE_TODAY_EVENTS" :key="ev.time" data-role="event" class="border-b py-0.5">
        {{ ev.time }}&nbsp;&nbsp;{{ ev.title }}
      </div>
    </div>
  </div>
</template>
