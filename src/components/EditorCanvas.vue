<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { usePagesStore } from '@/stores/pages'
import MovableElement from './MovableElement.vue'
import ClockWidget from './widgets/ClockWidget.vue'
import CalendarWidget from './widgets/CalendarWidget.vue'
import ImageWidget from './widgets/ImageWidget.vue'
import DrawingLayer from './widgets/DrawingLayer.vue'

const store = usePagesStore()
const container = ref<HTMLElement | null>(null)
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
})
onBeforeUnmount(() => ro?.disconnect())
watch(size, recompute)

function clearSelection() {
  store.selectElement(null)
}
</script>

<template>
  <div ref="container" class="flex h-full w-full items-center justify-center overflow-hidden bg-neutral-200">
    <div
      data-role="surface"
      class="relative bg-white shadow"
      :style="{
        width: `${size.w}px`,
        height: `${size.h}px`,
        transform: `scale(${scale})`,
        transformOrigin: 'center',
      }"
      @pointerdown.self="clearSelection"
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
      </MovableElement>
      <DrawingLayer :size="size" />
    </div>
  </div>
</template>
