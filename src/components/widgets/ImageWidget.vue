<script setup lang="ts">
import type { ImageEl } from '@/stores/types'
import { imageUrl } from '@/lib/deviceApi'

defineProps<{ el: ImageEl }>()

const emit = defineEmits<{ loaded: [size: { w: number; h: number }] }>()

function onLoad(e: Event) {
  const img = e.target as HTMLImageElement
  if (img.naturalWidth > 0 && img.naturalHeight > 0) {
    emit('loaded', { w: img.naturalWidth, h: img.naturalHeight })
  }
}
</script>

<template>
  <img v-if="el.src" :src="imageUrl(el.src)" class="h-full w-full object-contain" alt="" @load="onLoad" />
  <div
    v-else
    data-role="placeholder"
    class="flex h-full w-full items-center justify-center border-2 border-dashed border-neutral-400 bg-neutral-50 text-xs text-neutral-500"
  >
    Image
  </div>
</template>
