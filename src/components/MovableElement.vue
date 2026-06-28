<script setup lang="ts">
import { useDraggableResizable } from '@/composables/useDraggableResizable'

const props = defineProps<{
  id: string
  x: number
  y: number
  w: number
  h: number
  selected: boolean
  scale: number
  interactive?: boolean
  editing?: boolean
}>()

const emit = defineEmits<{
  select: [id: string]
  update: [rect: { x: number; y: number; w: number; h: number }]
}>()

const dr = useDraggableResizable({
  getRect: () => ({ x: props.x, y: props.y, w: props.w, h: props.h }),
  onUpdate: (rect) => emit('update', rect),
  scale: () => props.scale,
})

function onPointerDown(e: PointerEvent) {
  // While editing text, let the pointer reach the contenteditable so the caret
  // lands where the user clicks — don't start a drag (which preventDefaults
  // focus) or it's impossible to place the cursor or select text.
  if (props.editing) return
  emit('select', props.id)
  dr.startDrag(e)
}
</script>

<template>
  <div
    class="absolute select-none"
    :class="selected ? 'outline outline-2 outline-blue-500' : ''"
    :style="{ left: `${x}px`, top: `${y}px`, width: `${w}px`, height: `${h}px`, touchAction: 'none', pointerEvents: interactive === false ? 'none' : undefined }"
    @pointerdown="onPointerDown"
  >
    <slot />
    <div
      v-if="selected"
      class="absolute -right-1.5 -bottom-1.5 h-3 w-3 cursor-se-resize rounded-sm bg-blue-500"
      @pointerdown.stop="dr.startResize($event)"
    />
  </div>
</template>
