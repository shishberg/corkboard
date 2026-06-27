<script setup lang="ts">
import { computed } from 'vue'
import type { TextEl } from '@/stores/types'
import { usePagesStore } from '@/stores/pages'

const props = defineProps<{ el: TextEl }>()

const store = usePagesStore()

// Mirror CalendarWidget's baseSize: scale font with the smaller dimension
const baseSize = computed(() => `${Math.max(8, Math.min(props.el.w, props.el.h) * 0.25)}px`)

const isEditable = computed(
  () => store.selectedElId === props.el.id && store.activeTool === 'select',
)

function onInput(e: Event) {
  store.setElementText(props.el.id, (e.target as HTMLElement).innerText)
}

function onFocus() {
  store.selectElement(props.el.id)
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    ;(e.target as HTMLElement).blur()
  }
}
</script>

<template>
  <div
    data-role="text-root"
    class="h-full w-full overflow-hidden"
    :style="{ fontSize: baseSize }"
  >
    <div
      data-role="text-edit"
      class="h-full w-full"
      :style="{
        fontFamily: el.font,
        textAlign: el.align,
        color: el.colour,
        whiteSpace: 'pre-wrap',
        outline: 'none',
        cursor: isEditable ? 'text' : 'default',
      }"
      :contenteditable="isEditable ? 'true' : 'false'"
      @input="onInput"
      @focus="onFocus"
      @keydown="onKeydown"
    >{{ el.text }}</div>
  </div>
</template>
