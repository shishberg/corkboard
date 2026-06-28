<script setup lang="ts">
import { computed } from 'vue'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { usePagesStore } from '@/stores/pages'
import { AlignLeft, AlignCenter } from '@lucide/vue'
import type { TextEl } from '@/stores/types'

const opts = useToolOptionsStore()
const store = usePagesStore()

const selectedTextEl = computed((): TextEl | null => {
  const id = store.selectedElId
  if (!id) return null
  const el = store.selectedPage?.elements.find((e) => e.id === id)
  return el?.type === 'text' ? (el as TextEl) : null
})

const value = computed(() => selectedTextEl.value?.align ?? opts.align)

function pickAlign(align: 'left' | 'center') {
  opts.align = align
  if (selectedTextEl.value) {
    store.setElementAlign(selectedTextEl.value.id, align)
  }
}
</script>

<template>
  <div class="flex items-center gap-1">
    <button
      data-role="align-left"
      class="flex h-8 w-8 items-center justify-center rounded hover:bg-neutral-100"
      :class="value === 'left' ? 'bg-neutral-100 ring-2 ring-blue-500' : ''"
      @click="pickAlign('left')"
    >
      <AlignLeft class="h-4 w-4" />
    </button>
    <button
      data-role="align-center"
      class="flex h-8 w-8 items-center justify-center rounded hover:bg-neutral-100"
      :class="value === 'center' ? 'bg-neutral-100 ring-2 ring-blue-500' : ''"
      @click="pickAlign('center')"
    >
      <AlignCenter class="h-4 w-4" />
    </button>
  </div>
</template>
