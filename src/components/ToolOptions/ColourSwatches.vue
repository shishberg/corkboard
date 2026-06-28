<script setup lang="ts">
import { computed } from 'vue'
import { usePagesStore } from '@/stores/pages'
import { useToolOptionsStore } from '@/stores/toolOptions'
import type { EpaperColour } from '@/stores/types'

const store = usePagesStore()
const opts = useToolOptionsStore()

const palette: EpaperColour[] = ['black', 'white', 'red', 'yellow', 'blue', 'green']

const selectedEl = computed(() => {
  const id = store.selectedElId
  if (!id) return null
  return store.selectedPage?.elements.find((e) => e.id === id) ?? null
})

// Outline only applies to the text-bearing elements (or, with nothing selected,
// to the tool default that new text/calendars inherit).
const outlineApplies = computed(() => {
  if (store.activeTool === 'background') return false
  const el = selectedEl.value
  if (el) return el.type === 'text' || el.type === 'calendar'
  return true
})

// With the background tool active the swatches recolour the page background;
// otherwise they reflect the selected element's colour, else the pen colour.
const activeColour = computed(() => {
  if (store.activeTool === 'background') return store.selectedPage?.background ?? 'white'
  return selectedEl.value?.colour ?? opts.colour
})

// The outline colour a shift-click would toggle: the selected text/calendar's
// outline, else the tool default. Undefined when outline doesn't apply.
const activeOutline = computed<EpaperColour | undefined>(() => {
  if (!outlineApplies.value) return undefined
  const el = selectedEl.value
  if (el && (el.type === 'text' || el.type === 'calendar')) return el.outline
  return opts.outline
})

function onSwatch(c: EpaperColour, e: MouseEvent) {
  if (e.shiftKey) {
    setOutline(c)
    return
  }
  pickColour(c)
}

function pickColour(c: EpaperColour) {
  if (store.activeTool === 'background') {
    store.setPageBackground(c)
    return
  }
  opts.colour = c
  if (selectedEl.value) {
    store.setElementColour(selectedEl.value.id, c)
  }
}

// Shift-click sets the outline to that colour; shift-clicking the colour that's
// already the outline removes it (toggle).
function setOutline(c: EpaperColour) {
  if (!outlineApplies.value) return
  const next = activeOutline.value === c ? undefined : c
  opts.outline = next
  if (selectedEl.value) {
    store.setElementOutline(selectedEl.value.id, next)
  }
}

// An inset double ring marks the colour currently used as the outline. White +
// dark so it reads on any of the six swatch colours.
function swatchStyle(c: EpaperColour): Record<string, string> {
  const style: Record<string, string> = { backgroundColor: c }
  if (activeOutline.value === c) {
    style.boxShadow = 'inset 0 0 0 2px #fff, inset 0 0 0 4px #000'
  }
  return style
}
</script>

<template>
  <div data-role="colour-panel" class="flex items-center gap-1">
    <button
      v-for="c in palette"
      :key="c"
      :data-colour="c"
      :data-outline="activeOutline === c ? 'true' : undefined"
      class="h-6 w-6 rounded-full border border-neutral-300"
      :class="activeColour === c ? 'ring-2 ring-blue-500 ring-offset-1' : ''"
      :style="swatchStyle(c)"
      :title="outlineApplies ? 'Click: colour · Shift-click: outline' : 'Click: colour'"
      @click="onSwatch(c, $event)"
    />
  </div>
</template>
