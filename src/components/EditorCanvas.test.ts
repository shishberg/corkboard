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

describe('EditorCanvas', () => {
  it('dragging a movable element updates its position in the store', async () => {
    const store = usePagesStore()
    store.addElement({ id: 'e1', type: 'clock', variant: 'time', x: 0, y: 0, w: 200, h: 80 })
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
    store.addElement({ id: 'e1', type: 'clock', variant: 'time', x: 0, y: 0, w: 200, h: 80 })
    const w = mount(EditorCanvas, { global: { plugins: [] } })
    await w.vm.$nextTick()
    expect(w.findAll('[data-role="movable"]').length).toBe(1)
  })

  it('clears selection when the empty surface is clicked', async () => {
    const store = usePagesStore()
    store.addElement({ id: 'e1', type: 'clock', variant: 'time', x: 0, y: 0, w: 200, h: 80 })
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
})
