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

// With the background tool active the swatches recolour the page background;
// otherwise they reflect the selected element's colour, else the pen colour.
const activeColour = computed(() => {
  if (store.activeTool === 'background') return store.selectedPage?.background ?? 'white'
  return selectedEl.value?.colour ?? opts.colour
})

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
</script>

<template>
  <div data-role="colour-panel" class="flex items-center gap-1">
    <button
      v-for="c in palette"
      :key="c"
      :data-colour="c"
      class="h-6 w-6 rounded-full border border-neutral-300"
      :class="activeColour === c ? 'ring-2 ring-blue-500 ring-offset-1' : ''"
      :style="{ backgroundColor: c }"
      @click="pickColour(c)"
    />
  </div>
</template>
