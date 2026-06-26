<script setup lang="ts">
import { computed } from 'vue'
import type { ClockEl } from '@/stores/types'

const props = defineProps<{ el: ClockEl }>()
const showTime = computed(() => props.el.variant !== 'date')
const showDate = computed(() => props.el.variant !== 'time')

// Font sizes track the element height, but are also capped by width so wide
// text (e.g. "12:45") doesn't overflow a short, narrow element. ~0.62em per
// glyph is a rough width estimate for the bold sans font.
const timeSize = computed(
  () => `${Math.min(props.el.h * (showDate.value ? 0.42 : 0.6), props.el.w / (5 * 0.62))}px`,
)
const dateSize = computed(
  () => `${Math.min(props.el.h * (showTime.value ? 0.16 : 0.4), props.el.w / (10 * 0.6))}px`,
)
</script>

<template>
  <div class="flex h-full w-full flex-col items-center justify-center overflow-hidden bg-white">
    <div v-if="showTime" data-role="time" class="font-bold leading-none" :style="{ fontSize: timeSize }">12:45</div>
    <div v-if="showDate" data-role="date" class="text-neutral-600 leading-tight" :style="{ fontSize: dateSize }">Mon 23 Jun</div>
  </div>
</template>
