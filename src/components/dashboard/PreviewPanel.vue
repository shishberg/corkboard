<script setup lang="ts">
import { computed } from 'vue'
import StatCard from './StatCard.vue'
import type { PreviewInfo } from '@/lib/dashboardTypes'

const props = defineProps<{ preview: PreviewInfo }>()

// Cache-busts on every new render — Vue re-evaluates this whenever
// updatedAtMs changes on the next poll, so the <img> swaps automatically.
const src = computed(() => `/preview.png?v=${props.preview.updatedAtMs}`)
</script>

<template>
  <StatCard title="Preview">
    <a href="/preview.png" target="_blank" class="block w-fit">
      <img
        :src="src"
        alt="live preview"
        class="block rounded border [image-rendering:pixelated] [zoom:0.5]"
      />
    </a>
  </StatCard>
</template>
