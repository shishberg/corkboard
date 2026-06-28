<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue'
import type { CalendarEl } from '@/stores/types'
import { useFontsStore } from '@/stores/fonts'
import { fitFontSize, LINE_HEIGHT } from '@/lib/textFit'
import { formatSampleDate, sampleAgenda, format12h } from '@/lib/sampleCalendar'

const props = defineProps<{ el: CalendarEl }>()

const fonts = useFontsStore()
const variant = computed(() => props.el.variant)
// Older documents may carry no font; fall back to the default so the preview
// matches the device, which falls back to its default face on an empty id.
const effectiveFont = computed(() => props.el.font || fonts.defaultId)
const formattedDate = computed(() => formatSampleDate())

// Date variant: auto-size the date to fill its box, exactly like a text field
// (same fit as the device renderer) so it isn't perpetually tiny and the preview
// matches the panel. Recompute when the box, font or date changes.
const datePx = ref(10)
function recomputeDateFit() {
  if (variant.value !== 'date') return
  datePx.value = fitFontSize(formattedDate.value, props.el.w, props.el.h, effectiveFont.value)
}
watch(
  () => [variant.value, props.el.w, props.el.h, effectiveFont.value, formattedDate.value] as const,
  recomputeDateFit,
  { immediate: true },
)
// Text measurement only reflects the real font once it has loaded; recompute on
// every font-ready signal we have (mirrors TextWidget).
onMounted(() => {
  recomputeDateFit()
  const fontSet = (document as Document & { fonts?: FontFaceSet }).fonts
  if (fontSet) {
    fontSet.load(`16px ${effectiveFont.value}`).then(recomputeDateFit, recomputeDateFit)
    fontSet.ready.then(recomputeDateFit)
  }
  requestAnimationFrame(() => requestAnimationFrame(recomputeDateFit))
})
const datePxSize = computed(() => `${datePx.value}px`)

// Agenda: bold day headings, indented events, a thin divider under each event,
// and a font size chosen to fit the height. The layout is computed as absolutely
// positioned rows so it truncates at a line boundary when it would overflow —
// the exact same algorithm as the device renderer (render.rs draw_agenda), so
// the editor preview matches the panel.
const AGENDA_LH = 1.3
const AGENDA_MIN = 11
const AGENDA_MAX = 22
const AGENDA_INSET = 4

function eventLine(ev: { time: string; title: string }): string {
  return ev.time === '' ? ev.title : `${format12h(ev.time)} ${ev.title}`
}

// The days to lay out: the first `daysAhead` calendar days that have events.
const agendaDays = computed(() => {
  const n = Math.max(1, Math.min(7, props.el.daysAhead || 7))
  return sampleAgenda().slice(0, n).filter((d) => d.events.length > 0)
})

interface Row {
  key: string
  text: string
  top: number
  heading: boolean
}

const agenda = computed(() => {
  const days = agendaDays.value
  const availH = Math.max(1, props.el.h - 2 * AGENDA_INSET)
  if (days.length === 0) return { px: AGENDA_MIN, lineH: AGENDA_MIN * AGENDA_LH, indent: 0, rows: [] as Row[] }

  const totalEvents = days.reduce((n, d) => n + d.events.length, 0)
  const linesEquiv = days.length + totalEvents + 0.5 * (days.length - 1)
  const px = Math.min(AGENDA_MAX, Math.max(AGENDA_MIN, Math.floor(availH / (AGENDA_LH * linesEquiv))))
  const lineH = px * AGENDA_LH
  const gap = lineH * 0.5
  const indent = px * 0.9

  const rows: Row[] = []
  let y = 0
  days.forEach((day, n) => {
    if (n > 0) y += gap
    if (y + lineH > availH) return
    rows.push({ key: `h-${day.heading}`, text: day.heading, top: y, heading: true })
    y += lineH
    for (const ev of day.events) {
      if (y + lineH > availH) break
      rows.push({ key: `e-${day.heading}-${ev.time}-${ev.title}`, text: eventLine(ev), top: y, heading: false })
      y += lineH
    }
  })
  return { px, lineH, indent, rows }
})
</script>

<template>
  <div
    data-role="calendar-root"
    class="h-full w-full overflow-hidden"
    :style="{ color: el.colour, fontFamily: effectiveFont }"
  >
    <!-- Date variant: large formatted date filling the box. Top-aligned with
         horizontal alignment from el.align, exactly like a text element and the
         device renderer (text::draw_text), so the editor and panel match. -->
    <div
      v-if="variant === 'date'"
      data-role="calendar-date"
      class="h-full w-full font-bold"
      :style="{
        fontSize: datePxSize,
        textAlign: el.align,
        lineHeight: String(LINE_HEIGHT),
        whiteSpace: 'pre-wrap',
      }"
    >
      {{ formattedDate }}
    </div>

    <!-- Agenda variant: bold day headings, indented events, thin dividers. -->
    <div
      v-else
      data-role="calendar-agenda"
      class="relative h-full w-full"
      :style="{
        fontSize: `${agenda.px}px`,
        lineHeight: String(AGENDA_LH),
        padding: `${AGENDA_INSET}px`,
      }"
    >
      <div
        v-for="row in agenda.rows"
        :key="row.key"
        :data-role="row.heading ? 'day' : 'event'"
        class="absolute overflow-hidden whitespace-nowrap"
        :style="{
          top: `${AGENDA_INSET + row.top}px`,
          left: `${AGENDA_INSET + (row.heading ? 0 : agenda.indent)}px`,
          right: `${AGENDA_INSET}px`,
          height: `${agenda.lineH}px`,
          fontWeight: row.heading ? 700 : 400,
          borderBottom: row.heading ? 'none' : `1px solid ${el.colour}`,
        }"
      >
        {{ row.text }}
      </div>
    </div>
  </div>
</template>
