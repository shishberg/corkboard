import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import ToolRail from './ToolRail.vue'
import { usePagesStore } from '@/stores/pages'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

describe('ToolRail', () => {
  it('selecting the clock tool adds a clock element to the page', async () => {
    const store = usePagesStore()
    const w = mount(ToolRail)
    await w.get('[data-tool="clock"]').trigger('click')
    expect(store.activeTool).toBe('clock')
    expect(store.selectedPage?.elements.some((e) => e.type === 'clock')).toBe(true)
  })

  it('selecting the select tool does not add an element', async () => {
    const store = usePagesStore()
    const w = mount(ToolRail)
    await w.get('[data-tool="select"]').trigger('click')
    expect(store.activeTool).toBe('select')
    expect(store.selectedPage?.elements.length).toBe(0)
  })
})
