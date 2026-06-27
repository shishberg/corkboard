<script setup lang="ts">
import { usePagesStore } from '@/stores/pages'
import PageThumbnail from './PageThumbnail.vue'
import { Plus, X } from '@lucide/vue'

const store = usePagesStore()
</script>

<template>
  <div class="flex w-36 flex-col gap-2 overflow-y-auto border-r bg-neutral-100 p-2">
    <div
      v-for="p in store.pages"
      :key="p.id"
      data-role="thumb"
      class="cursor-pointer rounded border-2 p-0.5"
      :class="[
        store.selectedPageId === p.id ? 'ring-2 ring-blue-500' : '',
        store.livePageId === p.id ? 'border-green-500' : 'border-transparent',
      ]"
      @click="store.selectPage(p.id)"
    >
      <PageThumbnail :page-id="p.id" />
      <p class="mt-0.5 truncate text-center text-xs text-neutral-600">{{ p.name }}</p>
      <div class="mt-0.5 flex items-center justify-between gap-1">
        <span
          v-if="store.livePageId === p.id"
          data-role="live-badge"
          class="rounded-full bg-green-500 px-1.5 py-0.5 text-xs font-medium text-white"
        >Live</span>
        <button
          v-else
          data-role="make-live"
          class="rounded bg-neutral-200 px-1.5 py-0.5 text-xs hover:bg-green-100"
          @click.stop="store.setLivePage(p.id)"
        >Make live</button>
        <button
          v-if="store.pages.length > 1"
          data-role="delete-page"
          class="rounded p-0.5 text-neutral-400 hover:bg-neutral-200 hover:text-neutral-700"
          @click.stop="store.deletePage(p.id)"
        ><X class="h-3 w-3" /></button>
      </div>
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
