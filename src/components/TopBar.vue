<script setup lang="ts">
import { ref } from 'vue'
import { usePagesStore } from '@/stores/pages'
import { Button } from '@/components/ui/button'
import { RectangleHorizontal, RectangleVertical } from '@lucide/vue'
import { refreshNow } from '@/lib/deviceApi'

const store = usePagesStore()
const toast = ref<string | null>(null)

function showToast(msg: string) {
  toast.value = msg
  setTimeout(() => (toast.value = null), 1500)
}

function publish() {
  // Stubbed this pass — no network call.
  showToast('Published (stub)')
}

async function handleRefresh() {
  const ok = await refreshNow()
  showToast(ok ? 'Refreshing…' : 'Device offline')
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
        <component :is="store.orientation === 'landscape' ? RectangleHorizontal : RectangleVertical" class="h-4 w-4" />
        {{ store.orientation }}
      </button>
      <Button data-role="refresh" size="sm" variant="outline" @click="handleRefresh">Refresh now</Button>
      <Button data-role="publish" size="sm" @click="publish">Publish</Button>
    </div>
  </header>
</template>
