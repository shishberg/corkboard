<script setup lang="ts">
import { useToolOptionsStore } from '@/stores/toolOptions'

const opts = useToolOptionsStore()
const sizes = [2, 4, 8, 12]

// Map a pen size to a visible dot diameter so the circles read as distinct.
function dotPx(s: number): number {
  return s * 1.4 + 5
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <div>
      <div class="flex items-center gap-1">
        <button
          v-for="s in sizes"
          :key="s"
          :data-pen-size="s"
          class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-100"
          :class="opts.penSize === s ? 'bg-neutral-100 ring-2 ring-blue-500' : ''"
          @click="opts.penSize = s"
        >
          <span
            data-role="pen-dot"
            class="rounded-full bg-neutral-800"
            :style="{ width: `${dotPx(s)}px`, height: `${dotPx(s)}px` }"
          />
        </button>
      </div>
    </div>
  </div>
</template>
