<script setup lang="ts">
import { computed, ref, onMounted, watch, nextTick } from 'vue'
import type { TextEl } from '@/stores/types'
import { usePagesStore } from '@/stores/pages'

const props = withDefaults(defineProps<{ el: TextEl; editing?: boolean }>(), {
  editing: false,
})

const emit = defineEmits<{ stopEditing: [] }>()

const store = usePagesStore()
const editRef = ref<HTMLElement | null>(null)

// Mirror CalendarWidget's baseSize: scale font with the smaller dimension
const baseSize = computed(() => `${Math.max(8, Math.min(props.el.w, props.el.h) * 0.25)}px`)

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

// When edit mode turns on, focus the box and drop the caret at the end so the
// user can type immediately (the contenteditable only became focusable now).
watch(
  () => props.editing,
  async (on) => {
    if (!on) return
    await nextTick()
    const node = editRef.value
    if (!node) return
    node.focus()
    const range = document.createRange()
    range.selectNodeContents(node)
    range.collapse(false)
    const sel = window.getSelection()
    sel?.removeAllRanges()
    sel?.addRange(range)
  },
)

function onInput(e: Event) {
  store.setElementText(props.el.id, (e.target as HTMLElement).innerText)
}

function onBlur() {
  emit('stopEditing')
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
        cursor: editing ? 'text' : 'default',
      }"
      :contenteditable="editing ? 'true' : 'false'"
      @input="onInput"
      @blur="onBlur"
      @keydown="onKeydown"
    ></div>
  </div>
</template>
