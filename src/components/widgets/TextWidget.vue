<script setup lang="ts">
import { computed, ref, onMounted, watch, nextTick } from 'vue'
import type { TextEl } from '@/stores/types'
import { usePagesStore } from '@/stores/pages'
import { fitFontSize, LINE_HEIGHT } from '@/lib/textFit'

const props = withDefaults(defineProps<{ el: TextEl; editing?: boolean }>(), {
  editing: false,
})

const emit = defineEmits<{ stopEditing: [] }>()

const store = usePagesStore()
const editRef = ref<HTMLElement | null>(null)

// Auto-size the text to fill the box (same fit as the device renderer) so the
// preview matches the panel and text isn't perpetually tiny. Recompute whenever
// the text, box or font changes.
const fontPx = ref(10)
function recomputeFit() {
  fontPx.value = fitFontSize(props.el.text || '', props.el.w, props.el.h, props.el.font)
}
watch(
  () => [props.el.text, props.el.w, props.el.h, props.el.font] as const,
  recomputeFit,
  { immediate: true },
)
// Text measurement only reflects the real font once it has loaded; before that
// the browser falls back to narrower metrics and the fit is a pixel or two too
// large — enough to flip a line and overflow. No single font-ready signal is
// reliable on its own (`fonts.ready` can resolve before the font is even
// requested), so recompute on every signal we have: an explicit font load,
// `fonts.ready`, and a double rAF after the next layout.
onMounted(() => {
  recomputeFit()
  const fonts = (document as Document & { fonts?: FontFaceSet }).fonts
  if (fonts) {
    fonts.load(`16px ${props.el.font}`).then(recomputeFit, recomputeFit)
    fonts.ready.then(recomputeFit)
  }
  requestAnimationFrame(() => requestAnimationFrame(recomputeFit))
})

const baseSize = computed(() => `${fontPx.value}px`)

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
        lineHeight: String(LINE_HEIGHT),
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
