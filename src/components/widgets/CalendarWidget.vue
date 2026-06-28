<script setup lang="ts">
import { computed } from 'vue'
import type { CalendarEl } from '@/stores/types'
import { useFontsStore } from '@/stores/fonts'
import { formatSampleDate, SAMPLE_TODAY_EVENTS, sampleAgenda, format12h } from '@/lib/sampleCalendar'

const props = defineProps<{ el: CalendarEl }>()

const fonts = useFontsStore()
const variant = computed(() => props.el.variant)
// Older documents may carry no font; fall back to the default so the preview
// matches the device, which falls back to its default face on an empty id.
const effectiveFont = computed(() => props.el.font || fonts.defaultId)
const formattedDate = computed(() => formatSampleDate())

// Date and Today keep the simple min-dimension base size.
const baseSize = computed(() => `${Math.max(6, Math.min(props.el.w, props.el.h) * 0.06)}px`)

// Agenda: choose one font size so every day heading and event fits the height,
// using the same arithmetic as the device renderer (render.rs draw_agenda) so
// the preview matches the panel.
const AGENDA_LH = 1.3
const AGENDA_MIN = 11
const AGENDA_MAX = 22
const AGENDA_INSET = 4
// Only days that have events are shown — empty days are skipped entirely.
const agenda = computed(() => sampleAgenda().filter((d) => d.events.length > 0))
const agendaPx = computed(() => {
  const days = agenda.value
  if (days.length === 0) return AGENDA_MIN
  const totalEvents = days.reduce((n, d) => n + d.events.length, 0)
  const linesEquiv = days.length + totalEvents + 0.5 * (days.length - 1)
  const availH = Math.max(1, props.el.h - 2 * AGENDA_INSET)
  return Math.min(AGENDA_MAX, Math.max(AGENDA_MIN, Math.floor(availH / (AGENDA_LH * linesEquiv))))
})
const agendaGap = computed(() => `${agendaPx.value * AGENDA_LH * 0.5}px`)
const agendaIndent = computed(() => `${agendaPx.value * 0.9}px`)

function eventLine(ev: { time: string; title: string }): string {
  return ev.time === '' ? ev.title : `${format12h(ev.time)} ${ev.title}`
}
</script>

<template>
  <div
    data-role="calendar-root"
    class="h-full w-full overflow-hidden bg-white"
    :style="{ color: el.colour, fontFamily: effectiveFont }"
  >
    <!-- Date variant: large centred formatted date -->
    <div
      v-if="variant === 'date'"
      data-role="calendar-date"
      class="flex h-full items-center justify-center p-2 text-center font-bold"
      :style="{ fontSize: baseSize }"
    >
      {{ formattedDate }}
    </div>

    <!-- Today variant: heading + event list -->
    <div v-else-if="variant === 'today'" class="p-2" :style="{ fontSize: baseSize }">
      <div class="mb-1 font-bold">Today</div>
      <div v-for="ev in SAMPLE_TODAY_EVENTS" :key="ev.time" data-role="event" class="border-b py-0.5">
        {{ ev.time }}&nbsp;&nbsp;{{ ev.title }}
      </div>
    </div>

    <!-- Agenda variant: 7-day list (Today, Tomorrow, then weekday names) -->
    <div
      v-else
      data-role="calendar-agenda"
      :style="{
        fontSize: `${agendaPx}px`,
        lineHeight: String(AGENDA_LH),
        padding: `${AGENDA_INSET}px`,
      }"
    >
      <div v-for="(day, i) in agenda" :key="day.heading" data-role="agenda-day">
        <div
          data-role="day"
          class="overflow-hidden whitespace-nowrap font-medium"
          :style="{ marginTop: i === 0 ? '0' : agendaGap }"
        >
          {{ day.heading }}
        </div>
        <div
          v-for="ev in day.events"
          :key="ev.time + ev.title"
          data-role="event"
          class="overflow-hidden whitespace-nowrap"
          :style="{ paddingLeft: agendaIndent }"
        >
          {{ eventLine(ev) }}
        </div>
      </div>
    </div>
  </div>
</template>
