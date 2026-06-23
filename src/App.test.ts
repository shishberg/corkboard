import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import App from './App.vue'
import { usePagesStore } from '@/stores/pages'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

describe('App integration', () => {
  it('adds a clock element when the clock tool is picked, and toggles orientation', async () => {
    const w = mount(App)
    const store = usePagesStore()

    await w.get('[data-tool="clock"]').trigger('click')
    expect(store.selectedPage?.elements.some((e) => e.type === 'clock')).toBe(true)

    expect(store.orientation).toBe('landscape')
    await w.get('[data-role="orientation"]').trigger('click')
    expect(store.orientation).toBe('portrait')
  })

  it('publish shows a toast', async () => {
    const w = mount(App)
    await w.get('[data-role="publish"]').trigger('click')
    expect(w.get('[data-role="toast"]').text()).toContain('Published')
  })
})
