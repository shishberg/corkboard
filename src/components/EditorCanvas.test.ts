import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { setActivePinia, createPinia } from 'pinia'
import EditorCanvas from './EditorCanvas.vue'
import { usePagesStore } from '@/stores/pages'

beforeEach(() => setActivePinia(createPinia()))

function winPointer(type: string, x: number, y: number): PointerEvent {
  const e = new Event(type) as unknown as { clientX: number; clientY: number; pointerId: number }
  e.clientX = x
  e.clientY = y
  e.pointerId = 1
  return e as unknown as PointerEvent
}

function calendarEl(id: string) {
  return { id, type: 'calendar' as const, variant: 'week' as const, x: 0, y: 0, w: 200, h: 80, feedId: '', colour: 'black' as const }
}

describe('EditorCanvas', () => {
  it('dragging a movable element updates its position in the store', async () => {
    const store = usePagesStore()
    store.setActiveTool('select')
    store.addElement(calendarEl('e1'))
    const w = mount(EditorCanvas, { attachTo: document.body })
    await nextTick()
    // Dispatch the pointerdown directly: test-utils' trigger builds a real
    // MouseEvent whose clientX is read-only, but the composable needs clientX.
    w.get('[data-role="movable"]').element.dispatchEvent(winPointer('pointerdown', 0, 0))
    window.dispatchEvent(winPointer('pointermove', 20, 30))
    window.dispatchEvent(winPointer('pointerup', 20, 30))
    const moved = store.selectedPage?.elements.find((e) => e.id === 'e1')
    expect(moved?.x).toBe(20)
    expect(moved?.y).toBe(30)
    w.unmount()
  })

  it('renders one MovableElement per element on the selected page', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('e1'))
    const w = mount(EditorCanvas, { global: { plugins: [] } })
    await w.vm.$nextTick()
    expect(w.findAll('[data-role="movable"]').length).toBe(1)
  })

  it('Backspace deletes the selected element', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('e1'))
    const w = mount(EditorCanvas, { attachTo: document.body })
    await nextTick()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Backspace' }))
    expect(store.selectedPage?.elements.length).toBe(0)
    w.unmount()
  })

  it('Delete key deletes the selected element', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('e1'))
    const w = mount(EditorCanvas, { attachTo: document.body })
    await nextTick()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Delete' }))
    expect(store.selectedPage?.elements.length).toBe(0)
    w.unmount()
  })

  it('drag-creates an element of the active tool at the dragged rect', async () => {
    const store = usePagesStore()
    store.setActiveTool('calendar')
    const w = mount(EditorCanvas, { attachTo: document.body })
    await nextTick()
    // jsdom getBoundingClientRect is all-zero and scale stays 1, so client
    // coords map straight to surface-local coords.
    w.get('[data-role="surface"]').element.dispatchEvent(winPointer('pointerdown', 10, 20))
    window.dispatchEvent(winPointer('pointermove', 110, 140))
    window.dispatchEvent(winPointer('pointerup', 110, 140))
    await nextTick()
    const els = store.selectedPage?.elements ?? []
    expect(els.length).toBe(1)
    expect(els[0].type).toBe('calendar')
    expect({ x: els[0].x, y: els[0].y, w: els[0].w, h: els[0].h }).toEqual({ x: 10, y: 20, w: 100, h: 120 })
    // After creating, tool switches to select
    expect(store.activeTool).toBe('select')
    w.unmount()
  })

  it('pen tool draws a stroke and creates a drawing element', async () => {
    const store = usePagesStore()
    store.setActiveTool('draw')
    const w = mount(EditorCanvas, { attachTo: document.body })
    await nextTick()
    const layer = w.get('[data-role="draw-layer"]').element
    layer.dispatchEvent(winPointer('pointerdown', 10, 10))
    window.dispatchEvent(winPointer('pointermove', 30, 40))
    window.dispatchEvent(winPointer('pointerup', 30, 40))
    await nextTick()
    const els = store.selectedPage?.elements ?? []
    expect(els.length).toBe(1)
    expect(els[0].type).toBe('drawing')
    // Pen tool stays active after drawing
    expect(store.activeTool).toBe('draw')
    w.unmount()
  })

  it('pen tap (pointerdown then pointerup, no move) creates a drawing dot', async () => {
    const store = usePagesStore()
    store.setActiveTool('draw')
    const w = mount(EditorCanvas, { attachTo: document.body })
    await nextTick()
    const layer = w.get('[data-role="draw-layer"]').element
    layer.dispatchEvent(winPointer('pointerdown', 50, 50))
    // No pointermove — just a tap
    window.dispatchEvent(winPointer('pointerup', 50, 50))
    await nextTick()
    const els = store.selectedPage?.elements ?? []
    expect(els.length).toBe(1)
    expect(els[0].type).toBe('drawing')
    w.unmount()
  })

  it('clears selection when the empty surface is clicked', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('e1'))
    expect(store.selectedElId).toBe('e1')
    const w = mount(EditorCanvas)
    await w.get('[data-role="surface"]').trigger('pointerdown')
    expect(store.selectedElId).toBe(null)
  })

  it('recomputes scale when orientation changes', async () => {
    // Container 1000×600:
    //   landscape {w:800, h:480}: fit = min(1000/800, 600/480) = min(1.25, 1.25) = 1.25
    //   portrait  {w:480, h:800}: fit = min(1000/480, 600/800) = min(2.08, 0.75)  = 0.75
    // The two scales differ, so the test fails if the watch is absent.
    const store = usePagesStore()
    const w = mount(EditorCanvas, { attachTo: document.body })

    // Mock the container element's clientWidth/clientHeight
    const containerEl = w.element as HTMLElement
    Object.defineProperty(containerEl, 'clientWidth', { get: () => 1000, configurable: true })
    Object.defineProperty(containerEl, 'clientHeight', { get: () => 600, configurable: true })

    // Trigger initial recompute with the mocked dimensions (landscape)
    const surface = w.get('[data-role="surface"]')
    // Force recompute by reading scale after manually triggering via store
    // Start in landscape — call toggleOrientation to portrait and back to get a clean baseline
    // Actually, just read current transform after a tick and then toggle
    await nextTick()

    // Manually trigger recompute by toggling to portrait
    store.toggleOrientation() // now portrait {w:480, h:800}
    await nextTick()
    const portraitTransform = surface.attributes('style') ?? ''
    const portraitMatch = portraitTransform.match(/scale\(([^)]+)\)/)
    const portraitScale = portraitMatch ? parseFloat(portraitMatch[1]) : null

    // Toggle back to landscape
    store.toggleOrientation() // now landscape {w:800, h:480}
    await nextTick()
    const landscapeTransform = surface.attributes('style') ?? ''
    const landscapeMatch = landscapeTransform.match(/scale\(([^)]+)\)/)
    const landscapeScale = landscapeMatch ? parseFloat(landscapeMatch[1]) : null

    // With 1000×600 container: landscape=1.25, portrait=0.75 — must differ
    expect(portraitScale).not.toBeNull()
    expect(landscapeScale).not.toBeNull()
    expect(landscapeScale).not.toBeCloseTo(portraitScale as number, 5)

    w.unmount()
  })

  it('with a creation tool active, pointerdown on an existing element creates a NEW element', async () => {
    const store = usePagesStore()
    // Pre-place an existing element
    store.addElement(calendarEl('e1'))
    store.setActiveTool('calendar')
    const w = mount(EditorCanvas, { attachTo: document.body })
    await nextTick()

    // pointer-events:none on MovableElement means event hits the surface instead.
    // Simulate via surface pointerdown (as if it passed through).
    w.get('[data-role="surface"]').element.dispatchEvent(winPointer('pointerdown', 10, 20))
    window.dispatchEvent(winPointer('pointermove', 110, 140))
    window.dispatchEvent(winPointer('pointerup', 110, 140))
    await nextTick()

    const els = store.selectedPage?.elements ?? []
    // A second element was created — count is 2
    expect(els.length).toBe(2)
    // The selected element is the newly created one (not 'e1')
    expect(store.selectedElId).not.toBe('e1')
    w.unmount()
  })

  it('creation tool: movable has pointer-events:none; select tool: movable has no pointer-events restriction', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('e1'))

    // With a creation tool active
    store.setActiveTool('calendar')
    const w = mount(EditorCanvas, { attachTo: document.body })
    await nextTick()
    const movable = w.get('[data-role="movable"]')
    expect((movable.element as HTMLElement).style.pointerEvents).toBe('none')

    // Switch to select tool
    store.setActiveTool('select')
    await nextTick()
    expect((movable.element as HTMLElement).style.pointerEvents).not.toBe('none')

    w.unmount()
  })

  it('pointercancel during live creation finalises the element without leaking listeners', async () => {
    const store = usePagesStore()
    store.setActiveTool('calendar')
    const w = mount(EditorCanvas, { attachTo: document.body })
    await nextTick()

    w.get('[data-role="surface"]').element.dispatchEvent(winPointer('pointerdown', 10, 20))
    await nextTick()
    // One element added during pointerdown
    expect(store.selectedPage?.elements.length).toBe(1)

    // Cancel the gesture — should finalise just like pointerup
    window.dispatchEvent(winPointer('pointercancel', 10, 20))
    await nextTick()
    // Element still exists (not removed), tool switches to select
    expect(store.selectedPage?.elements.length).toBe(1)
    expect(store.activeTool).toBe('select')

    // Further pointermove should have no effect (listeners cleaned up)
    const before = { ...store.selectedPage!.elements[0] }
    window.dispatchEvent(winPointer('pointermove', 200, 300))
    await nextTick()
    const after = store.selectedPage!.elements[0]
    expect(after.x).toBe(before.x)
    expect(after.y).toBe(before.y)

    w.unmount()
  })
})
