import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import EditorCanvas from './EditorCanvas.vue'
import { usePagesStore } from '@/stores/pages'

beforeEach(() => setActivePinia(createPinia()))

describe('EditorCanvas', () => {
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
})
