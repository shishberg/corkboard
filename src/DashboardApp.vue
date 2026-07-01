<script setup lang="ts">
import { useDeviceStatus } from '@/composables/useDeviceStatus'
import { Button } from '@/components/ui/button'
import PreviewPanel from '@/components/dashboard/PreviewPanel.vue'
import DocumentPanel from '@/components/dashboard/DocumentPanel.vue'
import DevicePanel from '@/components/dashboard/DevicePanel.vue'
import FontsPanel from '@/components/dashboard/FontsPanel.vue'
import FeedsPanel from '@/components/dashboard/FeedsPanel.vue'
import LogsPanel from '@/components/dashboard/LogsPanel.vue'

const { status, unreachable } = useDeviceStatus()
</script>

<template>
  <div class="mx-auto max-w-6xl p-5">
    <header class="mb-4 flex items-start justify-between">
      <div>
        <h1 class="text-lg font-semibold">Corkboard</h1>
        <p v-if="status" class="text-sm text-neutral-500">
          {{ status.hostname }} · {{ status.display.os }}/{{ status.display.arch }}
        </p>
        <p v-else-if="unreachable" class="text-sm text-red-600">device unreachable — retrying…</p>
        <p v-else class="text-sm text-neutral-500">loading…</p>
      </div>
      <Button data-role="editor" as="a" href="/" variant="outline" size="sm">Editor</Button>
    </header>

    <template v-if="status">
      <PreviewPanel :preview="status.preview" />

      <div class="mt-4 grid grid-cols-1 gap-4 md:grid-cols-3">
        <DocumentPanel :document="status.document" />
        <DevicePanel :status="status" />
        <FontsPanel :fonts="status.fonts" />
      </div>

      <div class="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-2">
        <FeedsPanel :feeds="status.feeds" />
        <LogsPanel :logs="status.logs" />
      </div>
    </template>
  </div>
</template>
