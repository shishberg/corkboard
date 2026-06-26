<script setup lang="ts">
import { ref, computed, onBeforeUnmount } from 'vue'
import { useToolOptionsStore } from '@/stores/toolOptions'

const props = defineProps<{ size: { w: number; h: number }; scale: number }>()
const emit = defineEmits<{ stroke: [points: { x: number; y: number }[]] }>()

const opts = useToolOptionsStore()
const root = ref<HTMLElement | null>(null)
const points = ref<{ x: number; y: number }[]>([])
const livePath = computed(() => points.value.map((p) => `${p.x},${p.y}`).join(' '))

function toLocal(e: PointerEvent) {
  const r = root.value?.getBoundingClientRect()
  const s = props.scale || 1
  return { x: ((e.clientX - (r?.left ?? 0)) / s), y: ((e.clientY - (r?.top ?? 0)) / s) }
}

function onMove(e: PointerEvent) {
  points.value.push(toLocal(e))
}
function stopStroke() {
  window.removeEventListener('pointermove', onMove)
  window.removeEventListener('pointerup', onUp)
}
function onUp() {
  stopStroke()
  if (points.value.length > 1) emit('stroke', points.value.slice())
  points.value = []
}
function onDown(e: PointerEvent) {
  points.value = [toLocal(e)]
  window.addEventListener('pointermove', onMove)
  window.addEventListener('pointerup', onUp)
}
onBeforeUnmount(() => {
  stopStroke()
  points.value = []
})
</script>

<template>
  <div
    ref="root"
    data-role="draw-layer"
    class="absolute inset-0 cursor-crosshair"
    style="touch-action: none"
    @pointerdown.stop.prevent="onDown"
  >
    <svg :viewBox="`0 0 ${size.w} ${size.h}`" class="h-full w-full" preserveAspectRatio="none">
      <polyline
        v-if="points.length > 1"
        :points="livePath"
        fill="none"
        :stroke="opts.drawColour"
        :stroke-width="opts.penSize"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </div>
</template>
