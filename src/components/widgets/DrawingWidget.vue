<script setup lang="ts">
import type { DrawingEl } from '@/stores/types'
import { strokeToPath } from '@/lib/freehand'

defineProps<{ el: DrawingEl }>()
</script>

<template>
  <svg
    data-role="drawing"
    :viewBox="`0 0 ${el.natW} ${el.natH}`"
    class="h-full w-full overflow-visible"
    preserveAspectRatio="none"
  >
    <path
      v-for="(s, i) in el.strokes"
      :key="i"
      :d="strokeToPath(s.points, s.size)"
      :fill="el.colour"
      stroke="none"
    />
  </svg>
</template>
