<script setup lang="ts">
import { computed } from 'vue'
import { usePagesStore } from '@/stores/pages'
import { X } from '@lucide/vue'

const props = defineProps<{ index: number; pageId: string; delayMs: number }>()
const emit = defineEmits<{ setDelay: [ms: number]; remove: [] }>()

const store = usePagesStore()
const name = computed(() => store.pages.find((p) => p.id === props.pageId)?.name ?? 'Page')
const seconds = computed(() => Math.round(props.delayMs / 1000))

function onDelay(e: Event) {
  const v = Number((e.target as HTMLInputElement).value)
  emit('setDelay', Math.max(0, v) * 1000)
}
function onDragStart(e: DragEvent) {
  e.dataTransfer?.setData('text/plain', `idx:${props.index}`)
}
</script>

<template>
  <div
    data-role="timeline-item"
    class="flex shrink-0 flex-col items-center gap-1 rounded border bg-white p-2"
    draggable="true"
    @dragstart="onDragStart"
  >
    <div class="flex items-center gap-1">
      <span class="text-xs font-medium">{{ name }}</span>
      <button class="text-neutral-400 hover:text-red-500" @click="emit('remove')">
        <X class="h-3 w-3" />
      </button>
    </div>
    <label class="flex items-center gap-1 text-xs text-neutral-500">
      <input
        data-role="delay-input"
        type="number"
        min="0"
        class="w-12 rounded border px-1 text-right"
        :value="seconds"
        @change="onDelay"
      />
      s
    </label>
  </div>
</template>
