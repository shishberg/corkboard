<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { useFontsStore } from '@/stores/fonts'
import { usePagesStore } from '@/stores/pages'
import type { CalendarEl, TextEl } from '@/stores/types'

const opts = useToolOptionsStore()
const fonts = useFontsStore()
const store = usePagesStore()

onMounted(() => {
  fonts.load()
})

// The selected element, if it carries text (text or calendar).
const selectedTextish = computed((): TextEl | CalendarEl | null => {
  const id = store.selectedElId
  if (!id) return null
  const el = store.selectedPage?.elements.find((e) => e.id === id)
  return el && (el.type === 'text' || el.type === 'calendar') ? el : null
})

// Resolve a font id to one present in the loaded manifest, falling back to the
// manifest default. Avoids a blank <select> when a stored/element font isn't
// loaded (e.g. a custom manifest, or an older document with no font).
function resolve(font: string): string {
  return fonts.fonts.some((f) => f.id === font) ? font : fonts.defaultId
}

// Editing a selected element shows that element's font; otherwise the tool
// default used for new elements.
const value = computed(() =>
  resolve(selectedTextish.value ? selectedTextish.value.font : opts.font),
)

function pickFont(font: string) {
  opts.font = font
  if (selectedTextish.value) {
    store.setElementFont(selectedTextish.value.id, font)
  }
}
</script>

<template>
  <select
    data-role="font-select"
    class="rounded border px-1 py-1 text-sm"
    :value="value"
    @change="pickFont(($event.target as HTMLSelectElement).value)"
  >
    <option v-for="font in fonts.fonts" :key="font.id" :value="font.id">{{ font.name }}</option>
  </select>
</template>
