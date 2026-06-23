<script setup lang="ts">
import type { ImageEl } from '@/stores/types'

defineProps<{ el: ImageEl }>()
</script>

<template>
  <img v-if="el.src" :src="el.src" class="h-full w-full object-contain" alt="" />
  <div
    v-else
    data-role="placeholder"
    class="flex h-full w-full items-center justify-center border-2 border-dashed border-neutral-400 bg-neutral-50 text-xs text-neutral-500"
  >
    Image
  </div>
</template>
