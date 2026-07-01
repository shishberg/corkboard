<script setup lang="ts">
import StatCard from './StatCard.vue'
import type { FontsInfo } from '@/lib/dashboardTypes'

defineProps<{ fonts: FontsInfo }>()
</script>

<template>
  <StatCard title="Fonts">
    <div class="flex flex-wrap gap-1.5">
      <span
        v-for="id in fonts.ids"
        :key="id"
        class="rounded px-2 py-0.5 text-xs"
        :class="id === fonts.defaultId ? 'bg-neutral-900 text-white' : 'bg-neutral-100 text-neutral-700'"
      >
        {{ id }}{{ fonts.boldIds.includes(id) ? ' (bold)' : '' }}
      </span>
    </div>
  </StatCard>
</template>
