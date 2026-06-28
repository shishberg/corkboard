import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { nextTick, reactive } from 'vue'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import { usePagesStore } from '@/stores/pages'
import CalendarWidget from './CalendarWidget.vue'
import ImageWidget from './ImageWidget.vue'
import DrawingWidget from './DrawingWidget.vue'
import TextWidget from './TextWidget.vue'
import { makeDrawingElement } from '@/stores/elementFactory'
import { formatSampleDate } from '@/lib/sampleCalendar'
import type { CalendarEl, ImageEl, TextEl } from '@/stores/types'

beforeEach(() => {
  setActivePinia(createPinia())
})

describe('widgets', () => {
  it('CalendarWidget agenda shows only days with events, skipping empty ones', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'agenda', x: 0, y: 0, w: 300, h: 400, feedId: '', colour: 'black', font: 'atkinson-hyperlegible', align: 'center' as const }
    const w = mount(CalendarWidget, { props: { el } })
    const days = w.findAll('[data-role="day"]').map((d) => d.text())
    // The sample has no events on Wednesday, so it is skipped.
    expect(days).toEqual(['Today', 'Tomorrow', 'Monday', 'Tuesday', 'Thursday', 'Friday'])
    expect(days).not.toContain('Wednesday')
  })

  it('CalendarWidget agenda formats event times as 12-hour and includes all-day items', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'agenda', x: 0, y: 0, w: 300, h: 400, feedId: '', colour: 'black', font: 'atkinson-hyperlegible', align: 'center' as const }
    const w = mount(CalendarWidget, { props: { el } })
    const text = w.get('[data-role="calendar-agenda"]').text()
    expect(text).toContain('Last day of term') // all-day, no time
    expect(text).toContain('8:15am Choir')
    expect(text).toContain('6:00pm Ballet')
  })

  it('CalendarWidget applies element colour as text colour', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'today', x: 0, y: 0, w: 300, h: 200, feedId: '', colour: 'blue', font: 'atkinson-hyperlegible', align: 'center' as const }
    const w = mount(CalendarWidget, { props: { el } })
    const style = w.find('[data-role="calendar-root"]').attributes('style') ?? ''
    expect(style).toContain('color: blue')
  })

  it('CalendarWidget applies the element font as font-family', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'today', x: 0, y: 0, w: 300, h: 200, feedId: '', colour: 'black', font: 'gelasio', align: 'center' as const }
    const w = mount(CalendarWidget, { props: { el } })
    const style = w.find('[data-role="calendar-root"]').attributes('style') ?? ''
    expect(style).toContain('font-family: gelasio')
  })

  it('CalendarWidget falls back to the default font when font is empty', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'today', x: 0, y: 0, w: 300, h: 200, feedId: '', colour: 'black', font: '', align: 'center' as const }
    const w = mount(CalendarWidget, { props: { el } })
    const style = w.find('[data-role="calendar-root"]').attributes('style') ?? ''
    expect(style).toContain('font-family: atkinson-hyperlegible')
  })

  it('CalendarWidget today variant shows 3 sample events', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'today', x: 0, y: 0, w: 300, h: 200, feedId: '', colour: 'black', font: 'atkinson-hyperlegible', align: 'center' as const }
    const w = mount(CalendarWidget, { props: { el } })
    expect(w.findAll('[data-role="event"]').length).toBe(3)
  })

  it('CalendarWidget date variant shows the formatted sample date', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'date', x: 0, y: 0, w: 300, h: 200, feedId: '', colour: 'black', font: 'atkinson-hyperlegible', align: 'center' as const }
    const w = mount(CalendarWidget, { props: { el } })
    const dateEl = w.find('[data-role="calendar-date"]')
    expect(dateEl.exists()).toBe(true)
    expect(dateEl.text()).toContain(formatSampleDate())
  })

  it('ImageWidget shows a placeholder when src is empty', () => {
    const el: ImageEl = { id: 'img', type: 'image', src: '', x: 0, y: 0, w: 200, h: 150, colour: 'black' }
    const w = mount(ImageWidget, { props: { el } })
    expect(w.find('[data-role="placeholder"]').exists()).toBe(true)
  })

  it('ImageWidget renders the device image URL when src holds an image id', () => {
    const el: ImageEl = { id: 'img', type: 'image', src: 'img-abc123', x: 0, y: 0, w: 200, h: 150, colour: 'black' }
    const w = mount(ImageWidget, { props: { el } })
    const img = w.find('img')
    expect(img.exists()).toBe(true)
    expect(img.attributes('src')).toBe('/api/images/img-abc123')
    expect(w.find('[data-role="placeholder"]').exists()).toBe(false)
  })

  it('DrawingWidget viewBox uses natW/natH, not resized w/h', () => {
    const el = makeDrawingElement([{ x: 50, y: 60 }, { x: 90, y: 110 }], 'black', 4)
    // Simulate a resize — w and h change, natW/natH must not
    el.w = 200
    el.h = 300
    const w = mount(DrawingWidget, { props: { el } })
    const svg = w.find('[data-role="drawing"]')
    expect(svg.attributes('viewBox')).toBe(`0 0 ${el.natW} ${el.natH}`)
  })

  it('DrawingWidget renders a <path> with a non-empty d attribute', () => {
    const el = makeDrawingElement(
      [{ x: 10, y: 10 }, { x: 30, y: 20 }, { x: 50, y: 10 }],
      'black',
      4,
    )
    const w = mount(DrawingWidget, { props: { el } })
    const path = w.find('[data-role="drawing"] path')
    expect(path.exists()).toBe(true)
    expect(path.attributes('d')).toBeTruthy()
  })
})

describe('TextWidget', () => {
  // Tracks a cleanup function to restore document.activeElement after stubs.
  let cleanupActiveElement: (() => void) | null = null

  afterEach(() => {
    if (cleanupActiveElement) {
      cleanupActiveElement()
      cleanupActiveElement = null
    }
  })

  const sampleEl: TextEl = {
    id: 'text1',
    type: 'text',
    x: 0,
    y: 0,
    w: 240,
    h: 80,
    colour: 'black',
    text: 'Hello world',
    font: 'atkinson-hyperlegible',
    align: 'left',
  }

  it('renders the element text inside data-role="text-root"', () => {
    const w = mount(TextWidget, { props: { el: sampleEl } })
    const root = w.find('[data-role="text-root"]')
    expect(root.exists()).toBe(true)
    expect(root.text()).toContain('Hello world')
  })

  it('applies font-family from el.font', () => {
    const w = mount(TextWidget, { props: { el: sampleEl } })
    const edit = w.find('[data-role="text-edit"]')
    expect(edit.attributes('style')).toContain('atkinson-hyperlegible')
  })

  it('applies text-align from el.align', () => {
    const el: TextEl = { ...sampleEl, align: 'center' as const }
    const w = mount(TextWidget, { props: { el } })
    const edit = w.find('[data-role="text-edit"]')
    expect(edit.attributes('style')).toContain('center')
  })

  it('is NOT editable when editing is false', () => {
    const w = mount(TextWidget, { props: { el: sampleEl, editing: false } })
    const edit = w.find('[data-role="text-edit"]')
    expect(edit.attributes('contenteditable')).toBe('false')
  })

  it('defaults to not editable when editing prop is omitted', () => {
    const w = mount(TextWidget, { props: { el: sampleEl } })
    const edit = w.find('[data-role="text-edit"]')
    expect(edit.attributes('contenteditable')).toBe('false')
  })

  it('is editable when editing is true', () => {
    const w = mount(TextWidget, { props: { el: sampleEl, editing: true } })
    const edit = w.find('[data-role="text-edit"]')
    expect(edit.attributes('contenteditable')).toBe('true')
  })

  it('focuses the contenteditable when editing turns on', async () => {
    const w = mount(TextWidget, { props: { el: sampleEl, editing: false }, attachTo: document.body })
    const node = w.find('[data-role="text-edit"]').element as HTMLElement
    expect(document.activeElement).not.toBe(node)
    await w.setProps({ editing: true })
    await nextTick()
    expect(document.activeElement).toBe(node)
    w.unmount()
  })

  it('emits stop-editing when the contenteditable blurs', async () => {
    const w = mount(TextWidget, { props: { el: sampleEl, editing: true } })
    await w.find('[data-role="text-edit"]').trigger('blur')
    expect(w.emitted('stopEditing')).toBeTruthy()
  })

  it('input event calls store.setElementText with the innerText', async () => {
    const store = usePagesStore()
    store.addElement(sampleEl)

    const w = mount(TextWidget, { props: { el: sampleEl, editing: true } })
    const edit = w.find('[data-role="text-edit"]')

    // Simulate innerText being set and an input event fired
    Object.defineProperty(edit.element, 'innerText', {
      value: 'New text',
      configurable: true,
      writable: true,
    })
    await edit.trigger('input')

    const el = store.selectedPage?.elements[0]
    expect(el?.type === 'text' && (el as TextEl).text).toBe('New text')
  })

  // --- caret-safe contenteditable regression tests ---

  it('node textContent is set imperatively on mount (no reactive interpolation)', () => {
    // Use a fresh copy: sampleEl can be mutated by earlier tests that add it to the store
    const freshEl: TextEl = { ...sampleEl, text: 'Hello world' }
    const w = mount(TextWidget, { props: { el: freshEl } })
    const node = w.find('[data-role="text-edit"]').element
    expect(node.textContent).toBe('Hello world')
  })

  it('watcher does NOT overwrite node content while the node has focus', async () => {
    const store = usePagesStore()
    store.selectedElId = sampleEl.id
    store.activeTool = 'select'

    const reactiveEl = reactive({ ...sampleEl })
    const w = mount(TextWidget, { props: { el: reactiveEl } })
    const node = w.find('[data-role="text-edit"]').element as HTMLElement

    // Stub document.activeElement to look like this node is focused
    Object.defineProperty(document, 'activeElement', {
      get: () => node,
      configurable: true,
    })
    cleanupActiveElement = () => {
      // Remove the own-property stub so the prototype getter takes over again
      Reflect.deleteProperty(document, 'activeElement')
    }

    // Simulate what the browser does when the user types (sets DOM directly)
    node.textContent = 'user typed this'

    // An external el.text change arrives (e.g. undo or collaboration)
    reactiveEl.text = 'external update'
    await nextTick()

    // Watcher must have skipped — the user's in-progress edit is intact
    expect(node.textContent).toBe('user typed this')
  })

  it('watcher DOES update node content when el.text changes and node is not focused', async () => {
    const reactiveEl = reactive({ ...sampleEl })
    const w = mount(TextWidget, { props: { el: reactiveEl } })
    const node = w.find('[data-role="text-edit"]').element as HTMLElement

    // No focus stub — jsdom's document.activeElement defaults to document.body,
    // which is not this node, so the watcher should proceed.

    reactiveEl.text = 'external update'
    await nextTick()

    expect(node.textContent).toBe('external update')
  })
})
