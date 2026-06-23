<script setup lang="ts">
import { usePagesStore } from '@/stores/pages'
import PageThumbnail from './PageThumbnail.vue'
import { Plus } from '@lucide/vue'

const store = usePagesStore()
</script>

<template>
  <div class="flex w-36 flex-col gap-2 overflow-y-auto border-r bg-neutral-100 p-2">
    <div
      v-for="p in store.pages"
      :key="p.id"
      data-role="thumb"
      class="cursor-pointer rounded p-0.5"
      :class="store.selectedPageId === p.id ? 'ring-2 ring-blue-500' : ''"
      @click="store.selectPage(p.id)"
    >
      <PageThumbnail :page-id="p.id" />
      <p class="mt-0.5 truncate text-center text-xs text-neutral-600">{{ p.name }}</p>
    </div>
    <button
      data-role="add-page"
      class="flex items-center justify-center gap-1 rounded border border-dashed py-2 text-sm text-neutral-600 hover:bg-neutral-200"
      @click="store.addPage()"
    >
      <Plus class="h-4 w-4" /> Add page
    </button>
  </div>
</template>
