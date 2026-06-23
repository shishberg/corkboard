<script setup lang="ts">
import { useToolOptionsStore } from '@/stores/toolOptions'
import type { EpaperColour } from '@/stores/types'

const opts = useToolOptionsStore()
const palette: EpaperColour[] = ['black', 'white', 'red', 'yellow', 'blue', 'green']
const sizes = [2, 4, 8, 12]
</script>

<template>
  <div class="flex flex-col gap-2">
    <div>
      <p class="mb-1 text-xs font-medium text-neutral-500">Colour</p>
      <div class="flex gap-1">
        <button
          v-for="c in palette"
          :key="c"
          class="h-5 w-5 rounded-full border"
          :class="opts.drawColour === c ? 'ring-2 ring-blue-500 ring-offset-1' : ''"
          :style="{ backgroundColor: c }"
          @click="opts.drawColour = c"
        />
      </div>
    </div>
    <div>
      <p class="mb-1 text-xs font-medium text-neutral-500">Pen size</p>
      <div class="flex gap-1">
        <button
          v-for="s in sizes"
          :key="s"
          class="rounded px-2 py-1 text-xs hover:bg-neutral-100"
          :class="opts.penSize === s ? 'bg-neutral-100 font-medium' : ''"
          @click="opts.penSize = s"
        >
          {{ s }}
        </button>
      </div>
    </div>
  </div>
</template>
