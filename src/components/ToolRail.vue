<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { usePagesStore } from '@/stores/pages'
import { useToolOptionsStore, ensureToolOptionsPersistence } from '@/stores/toolOptions'
import type { ToolId, EpaperColour } from '@/stores/types'
import { Popover, PopoverTrigger, PopoverContent } from '@/components/ui/popover'
import { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider } from '@/components/ui/tooltip'
import {
  MousePointer2, Calendar, Pencil, Image as ImageIcon, Trash2, Type,
  BringToFront, SendToBack, PaintBucket,
} from '@lucide/vue'
import CalendarOptions from './ToolOptions/CalendarOptions.vue'
import DrawOptions from './ToolOptions/DrawOptions.vue'
import ImageOptions from './ToolOptions/ImageOptions.vue'
import TextOptions from './ToolOptions/TextOptions.vue'

const store = usePagesStore()
const opts = useToolOptionsStore()
onMounted(() => ensureToolOptionsPersistence())

function pickTool(tool: ToolId) {
  // Selecting a tool only makes it active; elements are created by drawing on
  // the canvas (see EditorCanvas).
  store.setActiveTool(tool)
}

const palette: EpaperColour[] = ['black', 'white', 'red', 'yellow', 'blue', 'green']

const selectedEl = computed(() => {
  const id = store.selectedElId
  if (!id) return null
  return store.selectedPage?.elements.find((e) => e.id === id) ?? null
})

// With the background tool active the swatches recolour the page background;
// otherwise they reflect the selected element's colour, else the pen colour.
const activeColour = computed(() => {
  if (store.activeTool === 'background') return store.selectedPage?.background ?? 'white'
  return selectedEl.value?.colour ?? opts.colour
})

function pickColour(c: EpaperColour) {
  if (store.activeTool === 'background') {
    store.setPageBackground(c)
    return
  }
  opts.colour = c
  if (selectedEl.value) {
    store.setElementColour(selectedEl.value.id, c)
  }
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
            <Pencil class="h-5 w-5" :style="{ color: opts.colour }" />
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

      <!-- Text -->
      <Popover>
        <PopoverTrigger as-child>
          <button
            data-tool="text"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'text' ? 'bg-neutral-200' : ''"
            @click="pickTool('text')"
          >
            <Type class="h-5 w-5" />
          </button>
        </PopoverTrigger>
        <PopoverContent side="right" class="w-48"><TextOptions /></PopoverContent>
      </Popover>

      <!-- Background: while active, the colour swatches recolour the page -->
      <Tooltip>
        <TooltipTrigger as-child>
          <button
            data-tool="background"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'background' ? 'bg-neutral-200' : ''"
            @click="pickTool('background')"
          >
            <PaintBucket class="h-5 w-5" />
          </button>
        </TooltipTrigger>
        <TooltipContent side="right">Background colour</TooltipContent>
      </Tooltip>

      <!-- Z-order: move selected element in front of / behind the others -->
      <Tooltip>
        <TooltipTrigger as-child>
          <button
            data-role="bring-to-front"
            class="mt-2 flex h-9 w-9 items-center justify-center rounded text-neutral-600 hover:bg-neutral-200 disabled:cursor-not-allowed disabled:opacity-30 disabled:hover:bg-transparent"
            :disabled="!store.selectedElId"
            @click="store.bringToFront()"
          >
            <BringToFront class="h-5 w-5" />
          </button>
        </TooltipTrigger>
        <TooltipContent side="right">Bring to front</TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger as-child>
          <button
            data-role="send-to-back"
            class="flex h-9 w-9 items-center justify-center rounded text-neutral-600 hover:bg-neutral-200 disabled:cursor-not-allowed disabled:opacity-30 disabled:hover:bg-transparent"
            :disabled="!store.selectedElId"
            @click="store.sendToBack()"
          >
            <SendToBack class="h-5 w-5" />
          </button>
        </TooltipTrigger>
        <TooltipContent side="right">Send to back</TooltipContent>
      </Tooltip>

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

      <!-- Colour panel -->
      <div data-role="colour-panel" class="mt-2 flex flex-col items-center gap-1">
        <button
          v-for="c in palette"
          :key="c"
          :data-colour="c"
          class="h-6 w-6 rounded-full border border-neutral-300"
          :class="activeColour === c ? 'ring-2 ring-blue-500 ring-offset-1' : ''"
          :style="{ backgroundColor: c }"
          @click="pickColour(c)"
        />
      </div>
    </div>
  </TooltipProvider>
</template>
