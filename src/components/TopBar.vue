<script setup lang="ts">
import { ref } from 'vue'
import { usePagesStore } from '@/stores/pages'
import { Button } from '@/components/ui/button'
import { RectangleHorizontal, RectangleVertical } from '@lucide/vue'
import { putDocument } from '@/lib/deviceApi'
import type { DocState } from '@/stores/types'

const store = usePagesStore()
const toast = ref<string | null>(null)

function showToast(msg: string) {
  toast.value = msg
  setTimeout(() => (toast.value = null), 1500)
}

async function publish() {
  // Publish promotes the page you're looking at to the live (displayed) page,
  // then re-sends the document; the device re-resolves the calendar feeds and
  // re-renders, so a no-op Publish doubles as "refresh now".
  if (store.selectedPageId) store.setLivePage(store.selectedPageId)
  const ok = await putDocument(store.$state as DocState)
  showToast(ok ? 'Published' : 'Device offline')
}
</script>

<template>
  <header class="flex h-12 items-center justify-between border-b bg-white px-3">
    <h1 class="text-sm font-semibold">Corkboard</h1>
    <div class="flex items-center gap-2">
      <span v-if="toast" data-role="toast" class="text-xs text-green-600">{{ toast }}</span>
      <button
        data-role="orientation"
        class="flex items-center gap-1 rounded border px-2 py-1 text-xs hover:bg-neutral-100"
        @click="store.toggleOrientation()"
      >
        <component :is="(store.selectedPage?.orientation ?? 'landscape') === 'landscape' ? RectangleHorizontal : RectangleVertical" class="h-4 w-4" />
        {{ store.selectedPage?.orientation ?? 'landscape' }}
      </button>
      <Button data-role="preview" as="a" href="/preview.png" target="_blank" rel="noopener" variant="outline" size="sm">Preview</Button>
      <Button data-role="publish" size="sm" @click="publish">Publish</Button>
    </div>
  </header>
</template>
