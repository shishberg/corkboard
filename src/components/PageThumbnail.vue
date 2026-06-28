<script setup lang="ts">
import { computed } from 'vue'
import { usePagesStore } from '@/stores/pages'
import { pageSize } from '@/stores/types'
import CalendarWidget from './widgets/CalendarWidget.vue'
import ImageWidget from './widgets/ImageWidget.vue'
import DrawingWidget from './widgets/DrawingWidget.vue'
import TextWidget from './widgets/TextWidget.vue'

const props = defineProps<{ pageId: string }>()
const store = usePagesStore()

const THUMB_W = 120
const page = computed(() => store.pages.find((p) => p.id === props.pageId) ?? null)
// Size from this thumbnail's OWN page, not the selected page (store.pageSize
// follows the selected page).
const size = computed(() => pageSize(page.value))
const scale = computed(() => THUMB_W / size.value.w)

function onDragStart(e: DragEvent) {
  e.dataTransfer?.setData('text/plain', props.pageId)
}
</script>

<template>
  <div
    class="relative overflow-hidden border bg-white"
    :style="{ width: `${THUMB_W}px`, height: `${size.h * scale}px` }"
    draggable="true"
    @dragstart="onDragStart"
  >
    <div
      class="absolute left-0 top-0 origin-top-left"
      :style="{ width: `${size.w}px`, height: `${size.h}px`, transform: `scale(${scale})` }"
    >
      <div
        v-for="el in page?.elements ?? []"
        :key="el.id"
        class="absolute"
        :style="{ left: `${el.x}px`, top: `${el.y}px`, width: `${el.w}px`, height: `${el.h}px` }"
      >
        <CalendarWidget v-if="el.type === 'calendar'" :el="el" />
        <ImageWidget v-else-if="el.type === 'image'" :el="el" />
        <DrawingWidget v-else-if="el.type === 'drawing'" :el="el" />
        <TextWidget v-else-if="el.type === 'text'" :el="el" />
      </div>
    </div>
  </div>
</template>
