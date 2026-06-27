<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { usePagesStore } from '@/stores/pages'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { makeElement, makeDrawingElement, defaultSize } from '@/stores/elementFactory'
import MovableElement from './MovableElement.vue'
import ClockWidget from './widgets/ClockWidget.vue'
import CalendarWidget from './widgets/CalendarWidget.vue'
import ImageWidget from './widgets/ImageWidget.vue'
import DrawingWidget from './widgets/DrawingWidget.vue'
import DrawingLayer from './widgets/DrawingLayer.vue'

const store = usePagesStore()
const opts = useToolOptionsStore()
const container = ref<HTMLElement | null>(null)
const surface = ref<HTMLElement | null>(null)
const scale = ref(1)

const size = computed(() => store.pageSize)
const elements = computed(() => store.selectedPage?.elements ?? [])

function recompute() {
  const el = container.value
  if (!el) return
  const fit = Math.min(el.clientWidth / size.value.w, el.clientHeight / size.value.h)
  scale.value = fit > 0 ? fit : 1
}

let ro: ResizeObserver | null = null
onMounted(() => {
  recompute()
  if (typeof ResizeObserver !== 'undefined' && container.value) {
    ro = new ResizeObserver(recompute)
    ro.observe(container.value)
  }
  window.addEventListener('keydown', onKeydown)
})
onBeforeUnmount(() => {
  ro?.disconnect()
  window.removeEventListener('keydown', onKeydown)
  stopCreateDrag()
  creating.value = null
})
watch(size, recompute)

function clearSelection() {
  store.selectElement(null)
}

// --- Delete the selected element with Backspace/Delete ---
function onKeydown(e: KeyboardEvent) {
  if (e.key !== 'Backspace' && e.key !== 'Delete') return
  const t = e.target as HTMLElement | null
  if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return
  if (store.selectedElId) {
    e.preventDefault()
    store.deleteElement()
  }
}

// --- Map a pointer event to surface-local (logical) coordinates ---
function surfaceLocal(e: PointerEvent) {
  const r = surface.value?.getBoundingClientRect()
  const s = scale.value || 1
  return { x: (e.clientX - (r?.left ?? 0)) / s, y: (e.clientY - (r?.top ?? 0)) / s }
}

// --- Draw-to-place: drag on the surface to create the active tool's element ---
const creating = ref<{ x: number; y: number; w: number; h: number } | null>(null)
let createStart = { x: 0, y: 0 }
let createStartRaw = { x: 0, y: 0 }
let createTool: 'clock' | 'calendar' | 'image' = 'clock'
// Below this many screen pixels of movement we treat the gesture as a click,
// not a drag. Measured in raw CSS pixels so zoom doesn't change the decision.
const CLICK_SLOP = 8

function onSurfacePointerDown(e: PointerEvent) {
  const tool = store.activeTool
  if (tool === 'clock' || tool === 'calendar' || tool === 'image') {
    createTool = tool
    createStart = surfaceLocal(e)
    createStartRaw = { x: e.clientX, y: e.clientY }
    creating.value = { ...createStart, w: 0, h: 0 }
    window.addEventListener('pointermove', onCreateMove)
    window.addEventListener('pointerup', onCreateUp)
  } else {
    clearSelection()
  }
}
function stopCreateDrag() {
  window.removeEventListener('pointermove', onCreateMove)
  window.removeEventListener('pointerup', onCreateUp)
}
function onCreateMove(e: PointerEvent) {
  if (!creating.value) return
  const p = surfaceLocal(e)
  creating.value = {
    x: Math.min(createStart.x, p.x),
    y: Math.min(createStart.y, p.y),
    w: Math.abs(p.x - createStart.x),
    h: Math.abs(p.y - createStart.y),
  }
}
function onCreateUp(e: PointerEvent) {
  stopCreateDrag()
  const c = creating.value
  creating.value = null
  if (!c) return
  // A click (movement under the slop) drops a default-sized element at the click point.
  const moved = Math.hypot(e.clientX - createStartRaw.x, e.clientY - createStartRaw.y)
  const def = defaultSize(createTool)
  const rect = moved < CLICK_SLOP ? { x: createStart.x, y: createStart.y, w: def.w, h: def.h } : c
  store.addElement(
    makeElement(createTool, { clockVariant: opts.clockVariant, calendarVariant: opts.calendarVariant, colour: opts.colour }, size.value, rect),
  )
}

// --- Pen: turn a finished stroke into a drawing element ---
function onStroke(points: { x: number; y: number }[]) {
  store.addElement(makeDrawingElement(points, opts.colour, opts.penSize))
}
</script>

<template>
  <div ref="container" class="flex h-full w-full items-center justify-center overflow-hidden bg-neutral-200">
    <div
      ref="surface"
      data-role="surface"
      class="relative bg-white shadow"
      :style="{
        width: `${size.w}px`,
        height: `${size.h}px`,
        transform: `scale(${scale})`,
        transformOrigin: 'center',
      }"
      @pointerdown.self="onSurfacePointerDown"
    >
      <MovableElement
        v-for="el in elements"
        :key="el.id"
        data-role="movable"
        :id="el.id"
        :x="el.x"
        :y="el.y"
        :w="el.w"
        :h="el.h"
        :selected="store.selectedElId === el.id"
        :scale="scale"
        @select="store.selectElement($event)"
        @update="store.updateElement(el.id, $event)"
      >
        <ClockWidget v-if="el.type === 'clock'" :el="el" />
        <CalendarWidget v-else-if="el.type === 'calendar'" :el="el" />
        <ImageWidget v-else-if="el.type === 'image'" :el="el" />
        <DrawingWidget v-else-if="el.type === 'drawing'" :el="el" />
      </MovableElement>

      <!-- Rubber-band preview while drag-creating an element -->
      <div
        v-if="creating"
        class="pointer-events-none absolute border-2 border-dashed border-blue-400 bg-blue-100/30"
        :style="{ left: `${creating.x}px`, top: `${creating.y}px`, width: `${creating.w}px`, height: `${creating.h}px` }"
      />

      <!-- Active drawing surface, only while the pen tool is selected -->
      <DrawingLayer v-if="store.activeTool === 'draw'" :size="size" :scale="scale" @stroke="onStroke" />
    </div>
  </div>
</template>
