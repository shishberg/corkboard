<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue'
import type { TextEl } from '@/stores/types'
import { usePagesStore } from '@/stores/pages'

const props = defineProps<{ el: TextEl }>()

const store = usePagesStore()
const editRef = ref<HTMLElement | null>(null)

// Mirror CalendarWidget's baseSize: scale font with the smaller dimension
const baseSize = computed(() => `${Math.max(8, Math.min(props.el.w, props.el.h) * 0.25)}px`)

const isEditable = computed(
  () => store.selectedElId === props.el.id && store.activeTool === 'select',
)

onMounted(() => {
  if (editRef.value) {
    editRef.value.textContent = props.el.text
  }
})

// Keep DOM in sync with external el.text changes (e.g. undo, collaboration),
// but skip while the node is focused so an active edit is never clobbered.
watch(
  () => props.el.text,
  (newText) => {
    const node = editRef.value
    if (!node) return
    if (document.activeElement === node) return
    node.textContent = newText
  },
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
    <!-- No reactive interpolation inside contenteditable — content is set
         imperatively via editRef so the browser caret is never reset. -->
    <div
      ref="editRef"
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
    ></div>
  </div>
</template>
