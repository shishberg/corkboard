import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { setActivePinia, createPinia } from 'pinia'
import ToolOptionsBar from './ToolOptionsBar.vue'
import { usePagesStore } from '@/stores/pages'
import * as fontsLib from '@/lib/fonts'
import type { ToolId } from '@/stores/types'

function calendarEl(id: string) {
  return { id, type: 'calendar' as const, variant: 'agenda' as const, x: 0, y: 0, w: 200, h: 80, feedId: '', colour: 'black' as const, font: 'atkinson-hyperlegible', align: 'center' as const, daysAhead: 7 as const }
}
function textEl(id: string) {
  return { id, type: 'text' as const, x: 0, y: 0, w: 240, h: 80, colour: 'black' as const, text: 'Text', font: 'atkinson-hyperlegible', align: 'left' as const }
}
function imageEl(id: string) {
  return { id, type: 'image' as const, x: 0, y: 0, w: 200, h: 150, colour: 'black' as const, src: '' }
}

// Panels are identified by a stable data-role on each.
const FONT = '[data-role="font-select"]'
const ALIGN = '[data-role="align-left"]'
const PEN = '[data-pen-size]'
const FEED = '[data-role="feed-select"]'
const COLOUR = '[data-role="colour-panel"]'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
  vi.spyOn(fontsLib, 'loadFontManifest').mockResolvedValue(fontsLib.DEFAULT_MANIFEST)
  vi.spyOn(fontsLib, 'injectFontFaces').mockImplementation(() => {})
})
afterEach(() => vi.restoreAllMocks())

async function mountWithTool(tool: ToolId) {
  const store = usePagesStore()
  store.setActiveTool(tool)
  const w = mount(ToolOptionsBar)
  await nextTick()
  return w
}

describe('ToolOptionsBar visibility by active tool', () => {
  it('select tool with nothing selected shows an empty bar', async () => {
    const w = await mountWithTool('select')
    for (const sel of [FONT, ALIGN, PEN, FEED, COLOUR]) {
      expect(w.find(sel).exists()).toBe(false)
    }
  })

  it('calendar tool shows calendar, font, align and colour panels (no pen)', async () => {
    const w = await mountWithTool('calendar')
    expect(w.find(FEED).exists()).toBe(true)
    expect(w.find(FONT).exists()).toBe(true)
    expect(w.find(COLOUR).exists()).toBe(true)
    expect(w.find(ALIGN).exists()).toBe(true)
    expect(w.find(PEN).exists()).toBe(false)
  })

  it('text tool shows font, align and colour panels', async () => {
    const w = await mountWithTool('text')
    expect(w.find(FONT).exists()).toBe(true)
    expect(w.find(ALIGN).exists()).toBe(true)
    expect(w.find(COLOUR).exists()).toBe(true)
    expect(w.find(FEED).exists()).toBe(false)
    expect(w.find(PEN).exists()).toBe(false)
  })

  it('draw tool shows pen size and colour panels only', async () => {
    const w = await mountWithTool('draw')
    expect(w.find(PEN).exists()).toBe(true)
    expect(w.find(COLOUR).exists()).toBe(true)
    expect(w.find(FONT).exists()).toBe(false)
    expect(w.find(FEED).exists()).toBe(false)
  })

  it('background tool shows colour panel only', async () => {
    const w = await mountWithTool('background')
    expect(w.find(COLOUR).exists()).toBe(true)
    expect(w.find(PEN).exists()).toBe(false)
    expect(w.find(FONT).exists()).toBe(false)
  })
})

describe('ToolOptionsBar visibility by selected element', () => {
  it('a selected calendar (select tool) shows calendar, font, align and colour panels', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('c1'))
    store.setActiveTool('select')
    const w = mount(ToolOptionsBar)
    await nextTick()
    expect(w.find(FEED).exists()).toBe(true)
    expect(w.find(FONT).exists()).toBe(true)
    expect(w.find(COLOUR).exists()).toBe(true)
    expect(w.find(ALIGN).exists()).toBe(true)
  })

  it('a selected text element shows font, align and colour panels', async () => {
    const store = usePagesStore()
    store.addElement(textEl('t1'))
    store.setActiveTool('select')
    const w = mount(ToolOptionsBar)
    await nextTick()
    expect(w.find(FONT).exists()).toBe(true)
    expect(w.find(ALIGN).exists()).toBe(true)
    expect(w.find(COLOUR).exists()).toBe(true)
  })

  it('a selected image shows no panels (image has no editable text settings or colour)', async () => {
    const store = usePagesStore()
    store.addElement(imageEl('i1'))
    store.setActiveTool('select')
    const w = mount(ToolOptionsBar)
    await nextTick()
    for (const sel of [FONT, ALIGN, PEN, FEED, COLOUR]) {
      expect(w.find(sel).exists()).toBe(false)
    }
  })
})
