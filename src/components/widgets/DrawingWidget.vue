<script setup lang="ts">
import type { DrawingEl } from '@/stores/types'

defineProps<{ el: DrawingEl }>()
const toPath = (points: { x: number; y: number }[]) => points.map((p) => `${p.x},${p.y}`).join(' ')
</script>

<template>
  <svg
    data-role="drawing"
    :viewBox="`0 0 ${el.natW} ${el.natH}`"
    class="h-full w-full overflow-visible"
    preserveAspectRatio="none"
  >
    <polyline
      v-for="(s, i) in el.strokes"
      :key="i"
      :points="toPath(s.points)"
      fill="none"
      :stroke="el.colour"
      :stroke-width="s.size"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
</template>
