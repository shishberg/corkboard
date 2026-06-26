<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { usePagesStore } from '@/stores/pages'
import { useToolOptionsStore, ensureToolOptionsPersistence } from '@/stores/toolOptions'
import type { ToolId } from '@/stores/types'
import { Popover, PopoverTrigger, PopoverContent } from '@/components/ui/popover'
import { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider } from '@/components/ui/tooltip'
import {
  MousePointer2, Clock, CalendarClock, Calendar, Pencil, Image as ImageIcon, Trash2,
} from '@lucide/vue'
import ClockOptions from './ToolOptions/ClockOptions.vue'
import CalendarOptions from './ToolOptions/CalendarOptions.vue'
import DrawOptions from './ToolOptions/DrawOptions.vue'
import ImageOptions from './ToolOptions/ImageOptions.vue'

const store = usePagesStore()
const opts = useToolOptionsStore()
onMounted(() => ensureToolOptionsPersistence())

const clockGlyph = computed(() =>
  opts.clockVariant === 'date' ? Calendar : opts.clockVariant === 'time-date' ? CalendarClock : Clock,
)

function pickTool(tool: ToolId) {
  // Selecting a tool only makes it active; elements are created by drawing on
  // the canvas (see EditorCanvas).
  store.setActiveTool(tool)
}
</script>

<template>
  <TooltipProvider>
    <div class="flex w-12 flex-col items-center gap-1 border-r bg-neutral-50 py-2">
      <!-- Select -->
      <Tooltip>
        <TooltipTrigger as-child>
          <button
            data-tool="select"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'select' ? 'bg-neutral-200' : ''"
            @click="pickTool('select')"
          >
            <MousePointer2 class="h-5 w-5" />
          </button>
        </TooltipTrigger>
        <TooltipContent side="right">Select</TooltipContent>
      </Tooltip>

      <!-- Clock -->
      <Popover>
        <PopoverTrigger as-child>
          <button
            data-tool="clock"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'clock' ? 'bg-neutral-200' : ''"
            @click="pickTool('clock')"
          >
            <component :is="clockGlyph" class="h-5 w-5" />
          </button>
        </PopoverTrigger>
        <PopoverContent side="right" class="w-44"><ClockOptions /></PopoverContent>
      </Popover>

      <!-- Calendar -->
      <Popover>
        <PopoverTrigger as-child>
          <button
            data-tool="calendar"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'calendar' ? 'bg-neutral-200' : ''"
            @click="pickTool('calendar')"
          >
            <Calendar class="h-5 w-5" />
          </button>
        </PopoverTrigger>
        <PopoverContent side="right" class="w-44"><CalendarOptions /></PopoverContent>
      </Popover>

      <!-- Draw -->
      <Popover>
        <PopoverTrigger as-child>
          <button
            data-tool="draw"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'draw' ? 'bg-neutral-200' : ''"
            @click="pickTool('draw')"
          >
            <Pencil class="h-5 w-5" :style="{ color: opts.drawColour }" />
          </button>
        </PopoverTrigger>
        <PopoverContent side="right" class="w-48"><DrawOptions /></PopoverContent>
      </Popover>

      <!-- Image -->
      <Popover>
        <PopoverTrigger as-child>
          <button
            data-tool="image"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'image' ? 'bg-neutral-200' : ''"
            @click="pickTool('image')"
          >
            <ImageIcon class="h-5 w-5" />
          </button>
        </PopoverTrigger>
        <PopoverContent side="right" class="w-48"><ImageOptions /></PopoverContent>
      </Popover>

      <!-- Delete selected element -->
      <Tooltip>
        <TooltipTrigger as-child>
          <button
            data-role="delete-element"
            class="mt-2 flex h-9 w-9 items-center justify-center rounded text-neutral-600 hover:bg-red-100 hover:text-red-600 disabled:cursor-not-allowed disabled:opacity-30 disabled:hover:bg-transparent disabled:hover:text-neutral-600"
            :disabled="!store.selectedElId"
            @click="store.deleteElement()"
          >
            <Trash2 class="h-5 w-5" />
          </button>
        </TooltipTrigger>
        <TooltipContent side="right">Delete selected (Del)</TooltipContent>
      </Tooltip>
    </div>
  </TooltipProvider>
</template>
