import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import App from './App.vue'
import { usePagesStore } from '@/stores/pages'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

function winPointer(type: string, x: number, y: number): PointerEvent {
  const e = new Event(type) as unknown as { clientX: number; clientY: number; pointerId: number }
  e.clientX = x
  e.clientY = y
  e.pointerId = 1
  return e as unknown as PointerEvent
}

describe('App integration', () => {
  it('drag-creates a calendar element after picking the calendar tool, and toggles orientation', async () => {
    const w = mount(App, { attachTo: document.body })
    const store = usePagesStore()

    // Picking a tool only makes it active — nothing is added yet.
    await w.get('[data-tool="calendar"]').trigger('click')
    expect(store.activeTool).toBe('calendar')
    expect(store.selectedPage?.elements.length).toBe(0)

    // Dragging on the canvas creates the element.
    w.get('[data-role="surface"]').element.dispatchEvent(winPointer('pointerdown', 10, 20))
    window.dispatchEvent(winPointer('pointermove', 110, 140))
    window.dispatchEvent(winPointer('pointerup', 110, 140))
    await w.vm.$nextTick()
    expect(store.selectedPage?.elements.some((e) => e.type === 'calendar')).toBe(true)
    // After creation, tool automatically switches back to select
    expect(store.activeTool).toBe('select')

    expect(store.orientation).toBe('landscape')
    await w.get('[data-role="orientation"]').trigger('click')
    expect(store.orientation).toBe('portrait')
    w.unmount()
  })

  it('publish shows a toast', async () => {
    const w = mount(App)
    await w.get('[data-role="publish"]').trigger('click')
    expect(w.get('[data-role="toast"]').text()).toContain('Published')
  })
})
