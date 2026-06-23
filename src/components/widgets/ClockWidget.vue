<script setup lang="ts">
import { computed } from 'vue'
import type { ClockEl } from '@/stores/types'

const props = defineProps<{ el: ClockEl }>()
const showTime = computed(() => props.el.variant !== 'date')
const showDate = computed(() => props.el.variant !== 'time')
</script>

<template>
  <div class="flex h-full w-full flex-col items-center justify-center bg-white">
    <div v-if="showTime" data-role="time" class="text-3xl font-bold leading-none">12:45</div>
    <div v-if="showDate" data-role="date" class="text-sm text-neutral-600">Mon 23 Jun</div>
  </div>
</template>
