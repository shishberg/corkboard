<script setup lang="ts">
import { useDeviceStatus } from '@/composables/useDeviceStatus'
import { Button } from '@/components/ui/button'
import PreviewPanel from '@/components/dashboard/PreviewPanel.vue'
import PreviewStatsPanel from '@/components/dashboard/PreviewStatsPanel.vue'
import DocumentPanel from '@/components/dashboard/DocumentPanel.vue'
import DevicePanel from '@/components/dashboard/DevicePanel.vue'
import FontsPanel from '@/components/dashboard/FontsPanel.vue'
import FeedsPanel from '@/components/dashboard/FeedsPanel.vue'
import LogsPanel from '@/components/dashboard/LogsPanel.vue'
import SystemPanel from '@/components/dashboard/SystemPanel.vue'

const { status, unreachable, refresh } = useDeviceStatus()
</script>

<template>
  <div class="mx-auto max-w-6xl p-5">
    <header class="mb-4 flex items-start justify-between">
      <div>
        <h1 class="text-lg font-semibold">Corkboard</h1>
        <p v-if="status" class="text-sm text-neutral-500">
          {{ status.hostname }} · {{ status.display.os }}/{{ status.display.arch }}
        </p>
        <p v-else-if="unreachable" class="text-sm text-red-600">device unreachable</p>
        <p v-else class="text-sm text-neutral-500">loading…</p>
      </div>
      <div class="flex gap-2">
        <Button data-role="refresh" variant="outline" size="sm" @click="refresh">Refresh</Button>
        <Button data-role="editor" as="a" href="/" variant="outline" size="sm">Editor</Button>
      </div>
    </header>

    <div v-if="status" class="columns-1 gap-4 sm:columns-2 xl:columns-3">
      <div class="mb-4 break-inside-avoid"><PreviewPanel :preview="status.preview" /></div>
      <div class="mb-4 break-inside-avoid"><PreviewStatsPanel :preview="status.preview" /></div>
      <div class="mb-4 break-inside-avoid"><DocumentPanel :document="status.document" /></div>
      <div class="mb-4 break-inside-avoid"><DevicePanel :status="status" /></div>
      <div class="mb-4 break-inside-avoid"><SystemPanel :system="status.system" /></div>
      <div class="mb-4 break-inside-avoid"><FontsPanel :fonts="status.fonts" /></div>
      <div class="mb-4 break-inside-avoid"><FeedsPanel :feeds="status.feeds" /></div>
      <div class="mb-4 break-inside-avoid"><LogsPanel :logs="status.logs" /></div>
    </div>
  </div>
</template>
