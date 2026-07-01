<script setup lang="ts">
import { computed } from 'vue'
import StatCard from './StatCard.vue'
import { formatAgo } from '@/lib/dashboardFormat'
import type { PreviewInfo } from '@/lib/dashboardTypes'

const props = defineProps<{ preview: PreviewInfo }>()

// Cache-busts on every new render — Vue re-evaluates this whenever
// updatedAtMs changes on the next poll, so the <img> swaps automatically.
const src = computed(() => `/preview.png?v=${props.preview.updatedAtMs}`)
</script>

<template>
  <StatCard title="Preview">
    <div class="flex flex-wrap items-start gap-6">
      <a href="/preview.png" target="_blank" class="block shrink-0 overflow-x-auto">
        <img
          :src="src"
          alt="live preview"
          class="block rounded border [image-rendering:pixelated]"
        />
      </a>
      <table class="text-sm">
        <tbody>
          <tr>
            <td class="pr-4 text-neutral-500">Last rendered</td>
            <td>{{ preview.updatedAtMs ? formatAgo(preview.updatedAtMs) : 'never' }}</td>
          </tr>
          <tr>
            <td class="pr-4 text-neutral-500">Renders this run</td>
            <td>{{ preview.renderCount }}</td>
          </tr>
          <tr>
            <td class="pr-4 text-neutral-500">Connected listeners</td>
            <td>{{ preview.connectedListeners }}</td>
          </tr>
          <tr>
            <td class="pr-4 text-neutral-500">Last calendar poll</td>
            <td>{{ preview.lastPollAtMs ? formatAgo(preview.lastPollAtMs) : 'never' }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </StatCard>
</template>
