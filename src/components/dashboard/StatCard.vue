<script setup lang="ts">
defineProps<{ title: string }>()
</script>

<template>
  <div class="rounded-lg border bg-white p-4">
    <h2 class="mb-3 text-xs font-semibold tracking-wide text-neutral-500 uppercase">{{ title }}</h2>
    <slot />
  </div>
</template>
