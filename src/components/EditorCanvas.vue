<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { usePagesStore } from '@/stores/pages'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { makeElement, makeDrawingElement, defaultSize } from '@/stores/elementFactory'
import MovableElement from './MovableElement.vue'
import CalendarWidget from './widgets/CalendarWidget.vue'
import ImageWidget from './widgets/ImageWidget.vue'
import DrawingWidget from './widgets/DrawingWidget.vue'
import TextWidget from './widgets/TextWidget.vue'
import DrawingLayer from './widgets/DrawingLayer.vue'

const store = usePagesStore()
const opts = useToolOptionsStore()
const container = ref<HTMLElement | null>(null)
const surface = ref<HTMLElement | null>(null)
const scale = ref(1)

// The text element currently being edited (double-clicked). Only one at a time.
const editingElId = ref<string | null>(null)
function enterTextEdit(el: { id: string; type: string }) {
  if (el.type !== 'text' || store.activeTool !== 'select') return
  store.selectElement(el.id)
  editingElId.value = el.id
}
// Leave edit mode whenever the edited element is no longer the lone selection
// on the select tool (clicked away, switched tools, deleted, page changed).
watch(
  () => [store.selectedElId, store.activeTool] as const,
  ([sel, tool]) => {
    if (editingElId.value && (sel !== editingElId.value || tool !== 'select')) {
      editingElId.value = null
    }
  },
)

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
  creatingId = null
})
watch(size, recompute)

function clearSelection() {
  store.selectElement(null)
}

// --- Delete the selected element with Backspace/Delete ---
function onKeydown(e: KeyboardEvent) {
  if (e.key !== 'Backspace' && e.key !== 'Delete') return
  const t = e.target as HTMLElement | null
  // Never hijack Backspace/Delete while typing in a field or editing text — the
  // browser must handle them as character edits, not element deletion.
  const inField =
    !!t &&
    (t.tagName === 'INPUT' ||
      t.tagName === 'TEXTAREA' ||
      t.isContentEditable ||
      !!t.closest?.('[contenteditable="true"]'))
  if (inField) return
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
let createStart = { x: 0, y: 0 }
let createStartRaw = { x: 0, y: 0 }
let createTool: 'calendar' | 'image' | 'text' = 'calendar'
let creatingId: string | null = null
// Below this many screen pixels of movement we treat the gesture as a click,
// not a drag. Measured in raw CSS pixels so zoom doesn't change the decision.
const CLICK_SLOP = 8
const CREATE_MIN = 8

function onSurfacePointerDown(e: PointerEvent) {
  const tool = store.activeTool
  if (tool === 'calendar' || tool === 'image' || tool === 'text') {
    createTool = tool
    createStart = surfaceLocal(e)
    createStartRaw = { x: e.clientX, y: e.clientY }
    const el = makeElement(
      tool,
      { calendarVariant: opts.calendarVariant, colour: opts.colour, feedId: opts.feedId, font: opts.font, align: opts.align, imageId: opts.imageId },
      size.value,
      { x: createStart.x, y: createStart.y, w: CREATE_MIN, h: CREATE_MIN },
    )
    store.addElement(el)
    creatingId = el.id
    window.addEventListener('pointermove', onCreateMove)
    window.addEventListener('pointerup', onCreateUp)
    window.addEventListener('pointercancel', onCreateUp)
  } else {
    clearSelection()
  }
}
function stopCreateDrag() {
  window.removeEventListener('pointermove', onCreateMove)
  window.removeEventListener('pointerup', onCreateUp)
  window.removeEventListener('pointercancel', onCreateUp)
}
function onCreateMove(e: PointerEvent) {
  if (!creatingId) return
  const p = surfaceLocal(e)
  store.updateElement(creatingId, {
    x: Math.min(createStart.x, p.x),
    y: Math.min(createStart.y, p.y),
    w: Math.max(CREATE_MIN, Math.abs(p.x - createStart.x)),
    h: Math.max(CREATE_MIN, Math.abs(p.y - createStart.y)),
  })
}
function onCreateUp(e: PointerEvent) {
  stopCreateDrag()
  const id = creatingId
  creatingId = null
  if (!id) return
  // A click (movement under slop) sets a default size with top-left at the click point
  const moved = Math.hypot(e.clientX - createStartRaw.x, e.clientY - createStartRaw.y)
  if (moved < CLICK_SLOP) {
    const def = defaultSize(createTool)
    store.updateElement(id, { x: createStart.x, y: createStart.y, w: def.w, h: def.h })
  }
  // Switch to select so the user can immediately interact with what they made
  store.setActiveTool('select')
  // A new text box drops straight into edit mode so the user can just type.
  if (createTool === 'text') {
    editingElId.value = id
  }
}

// --- Pen: turn a finished stroke into a drawing element ---
function onStroke(points: { x: number; y: number }[]) {
  if (points.length === 0) return
  store.addElement(makeDrawingElement(points, opts.colour, opts.penSize))
}
</script>

<template>
  <div ref="container" class="flex h-full w-full items-center justify-center overflow-hidden bg-neutral-200">
    <div
      ref="surface"
      data-role="surface"
      class="relative shadow"
      :style="{
        width: `${size.w}px`,
        height: `${size.h}px`,
        backgroundColor: store.selectedPage?.background ?? 'white',
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
        :interactive="store.activeTool === 'select'"
        :editing="editingElId === el.id"
        @select="store.selectElement($event)"
        @update="store.updateElement(el.id, $event)"
        @dblclick="enterTextEdit(el)"
      >
        <CalendarWidget v-if="el.type === 'calendar'" :el="el" />
        <ImageWidget v-else-if="el.type === 'image'" :el="el" />
        <DrawingWidget v-else-if="el.type === 'drawing'" :el="el" />
        <TextWidget
          v-else-if="el.type === 'text'"
          :el="el"
          :editing="editingElId === el.id"
          @stop-editing="editingElId = null"
        />
      </MovableElement>

      <!-- Active drawing surface, only while the pen tool is selected -->
      <DrawingLayer v-if="store.activeTool === 'draw'" :size="size" :scale="scale" @stroke="onStroke" />
    </div>
  </div>
</template>
