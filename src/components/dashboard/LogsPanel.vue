<script setup lang="ts">
import StatCard from './StatCard.vue'
import type { LogEntry } from '@/lib/dashboardTypes'

defineProps<{ logs: LogEntry[] }>()
</script>

<template>
  <StatCard title="Error log">
    <div class="max-h-64 overflow-y-auto font-mono text-xs">
      <div v-if="logs.length === 0" class="italic text-neutral-500">no warnings or errors this run</div>
      <div v-for="(l, i) in logs" :key="i" class="flex gap-2 border-b py-1 last:border-b-0">
        <span class="whitespace-nowrap text-neutral-500">{{ new Date(l.timeMs).toLocaleTimeString() }}</span>
        <span class="whitespace-nowrap font-semibold" :class="l.level === 'ERROR' ? 'text-red-600' : 'text-amber-600'">{{ l.level }}</span>
        <span class="whitespace-nowrap text-neutral-500">{{ l.target }}</span>
        <span>{{ l.message }}</span>
      </div>
    </div>
  </StatCard>
</template>
