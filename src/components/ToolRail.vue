<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { usePagesStore } from '@/stores/pages'
import { useToolOptionsStore, ensureToolOptionsPersistence } from '@/stores/toolOptions'
import { addImageFromFile } from '@/lib/imageTool'
import type { ToolId } from '@/stores/types'
import { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider } from '@/components/ui/tooltip'
import {
  MousePointer2, Calendar, Pencil, Image as ImageIcon, Trash2, Type,
  BringToFront, SendToBack, PaintBucket,
} from '@lucide/vue'

const store = usePagesStore()
const opts = useToolOptionsStore()
onMounted(() => ensureToolOptionsPersistence())

const imageInput = ref<HTMLInputElement | null>(null)

function pickTool(tool: ToolId) {
  // Selecting a tool only makes it active; elements are created by drawing on
  // the canvas (see EditorCanvas). Tool settings live in the ToolOptionsBar.
  store.setActiveTool(tool)
}

// The image tool isn't a draw mode — it opens the file dialog immediately and
// drops the uploaded image onto the page.
function pickImage() {
  const input = imageInput.value
  if (!input) return
  input.value = '' // allow re-picking the same file
  input.click()
}

async function onImageChosen(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  await addImageFromFile(file)
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
      <Tooltip>
        <TooltipTrigger as-child>
          <button
            data-tool="calendar"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'calendar' ? 'bg-neutral-200' : ''"
            @click="pickTool('calendar')"
          >
            <Calendar class="h-5 w-5" />
          </button>
        </TooltipTrigger>
        <TooltipContent side="right">Calendar</TooltipContent>
      </Tooltip>

      <!-- Draw -->
      <Tooltip>
        <TooltipTrigger as-child>
          <button
            data-tool="draw"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'draw' ? 'bg-neutral-200' : ''"
            @click="pickTool('draw')"
          >
            <Pencil class="h-5 w-5" :style="{ color: opts.colour }" />
          </button>
        </TooltipTrigger>
        <TooltipContent side="right">Draw</TooltipContent>
      </Tooltip>

      <!-- Image: one click opens the file dialog; uploading drops the image
           straight onto the page (no draw-to-place). -->
      <Tooltip>
        <TooltipTrigger as-child>
          <button
            data-tool="image"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            @click="pickImage"
          >
            <ImageIcon class="h-5 w-5" />
          </button>
        </TooltipTrigger>
        <TooltipContent side="right">Add image</TooltipContent>
      </Tooltip>
      <input
        ref="imageInput"
        data-role="image-file"
        type="file"
        accept="image/*"
        class="hidden"
        @change="onImageChosen"
      />

      <!-- Text -->
      <Tooltip>
        <TooltipTrigger as-child>
          <button
            data-tool="text"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'text' ? 'bg-neutral-200' : ''"
            @click="pickTool('text')"
          >
            <Type class="h-5 w-5" />
          </button>
        </TooltipTrigger>
        <TooltipContent side="right">Text</TooltipContent>
      </Tooltip>

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
    </div>
  </TooltipProvider>
</template>
