<script setup lang="ts">
defineProps<{ size: { w: number; h: number } }>()
</script>

<template>
  <!-- Placeholder overlay this pass; stroke capture comes later. -->
  <canvas
    :width="size.w"
    :height="size.h"
    class="pointer-events-none absolute inset-0 h-full w-full"
  />
</template>
