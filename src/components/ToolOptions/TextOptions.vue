<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { useFontsStore } from '@/stores/fonts'
import { usePagesStore } from '@/stores/pages'
import type { TextEl } from '@/stores/types'

const opts = useToolOptionsStore()
const fonts = useFontsStore()
const store = usePagesStore()

onMounted(() => {
  fonts.load()
})

// Fall back to the manifest's default font if the stored font is not loaded.
// This avoids a blank <select> when a custom manifest doesn't include the
// previously saved font id.
const effectiveFont = computed(() =>
  fonts.fonts.some((f) => f.id === opts.font) ? opts.font : fonts.defaultId,
)

const selectedTextEl = computed((): TextEl | null => {
  const id = store.selectedElId
  if (!id) return null
  const el = store.selectedPage?.elements.find((e) => e.id === id)
  return el?.type === 'text' ? (el as TextEl) : null
})

function pickFont(font: string) {
  opts.font = font
  if (selectedTextEl.value) {
    store.setElementFont(selectedTextEl.value.id, font)
  }
}

function pickAlign(align: 'left' | 'center') {
  opts.align = align
  if (selectedTextEl.value) {
    store.setElementAlign(selectedTextEl.value.id, align)
  }
}
</script>

<template>
  <div class="flex flex-col gap-1">
    <p class="mb-1 text-xs font-medium text-neutral-500">Font</p>
    <select
      data-role="font-select"
      class="rounded border px-1 py-1 text-sm"
      :value="effectiveFont"
      @change="pickFont(($event.target as HTMLSelectElement).value)"
    >
      <option v-for="font in fonts.fonts" :key="font.id" :value="font.id">{{ font.name }}</option>
    </select>
    <p class="mb-1 mt-2 text-xs font-medium text-neutral-500">Align</p>
    <div class="flex gap-1">
      <button
        data-role="align-left"
        class="rounded px-2 py-1 text-sm hover:bg-neutral-100"
        :class="opts.align === 'left' ? 'bg-neutral-100 font-medium' : ''"
        @click="pickAlign('left')"
      >Left</button>
      <button
        data-role="align-center"
        class="rounded px-2 py-1 text-sm hover:bg-neutral-100"
        :class="opts.align === 'center' ? 'bg-neutral-100 font-medium' : ''"
        @click="pickAlign('center')"
      >Center</button>
    </div>
  </div>
</template>
