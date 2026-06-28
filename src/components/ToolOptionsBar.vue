<script setup lang="ts">
import { computed, type Component } from 'vue'
import { usePagesStore } from '@/stores/pages'
import type { ToolId, El } from '@/stores/types'
import CalendarOptions from './ToolOptions/CalendarOptions.vue'
import FontOptions from './ToolOptions/FontOptions.vue'
import AlignOptions from './ToolOptions/AlignOptions.vue'
import DrawOptions from './ToolOptions/DrawOptions.vue'
import ColourSwatches from './ToolOptions/ColourSwatches.vue'

// A settings panel and the contexts it applies to. A panel shows when the
// active tool is in `tools` OR the selected element's type is in `elementTypes`
// — so the same wiring drives "settings for the current tool" and "settings for
// the selected element". `align: 'end'` pushes a panel to the right of the bar.
interface PanelDef {
  component: Component
  tools: ToolId[]
  elementTypes: El['type'][]
  align?: 'end'
}

const panels: PanelDef[] = [
  { component: CalendarOptions, tools: ['calendar'], elementTypes: ['calendar'] },
  { component: FontOptions, tools: ['calendar', 'text'], elementTypes: ['calendar', 'text'] },
  { component: AlignOptions, tools: ['text'], elementTypes: ['text'] },
  // Pen size has no per-element edit: a placed drawing has per-stroke sizes.
  { component: DrawOptions, tools: ['draw'], elementTypes: [] },
  {
    component: ColourSwatches,
    tools: ['draw', 'calendar', 'text', 'background'],
    elementTypes: ['calendar', 'text', 'drawing'],
    align: 'end',
  },
]

const store = usePagesStore()

const selectedEl = computed((): El | null => {
  const id = store.selectedElId
  if (!id) return null
  return store.selectedPage?.elements.find((e) => e.id === id) ?? null
})

const visiblePanels = computed(() =>
  panels.filter(
    (p) =>
      p.tools.includes(store.activeTool) ||
      (selectedEl.value !== null && p.elementTypes.includes(selectedEl.value.type)),
  ),
)
</script>

<template>
  <div
    data-role="tool-options-bar"
    class="flex h-12 items-center gap-3 border-b bg-white px-3"
  >
    <component
      :is="panel.component"
      v-for="(panel, i) in visiblePanels"
      :key="i"
      :class="panel.align === 'end' ? 'ml-auto' : ''"
    />
  </div>
</template>
