import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import TopBar from './TopBar.vue'
import * as deviceApi from '@/lib/deviceApi'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('TopBar', () => {
  it('shows Refreshing… toast when refreshNow returns true', async () => {
    vi.spyOn(deviceApi, 'refreshNow').mockResolvedValue(true)
    const w = mount(TopBar)
    await w.get('[data-role="refresh"]').trigger('click')
    await w.vm.$nextTick()
    await w.vm.$nextTick()
    expect(w.get('[data-role="toast"]').text()).toBe('Refreshing…')
  })

  it('shows Device offline toast when refreshNow returns false', async () => {
    vi.spyOn(deviceApi, 'refreshNow').mockResolvedValue(false)
    const w = mount(TopBar)
    await w.get('[data-role="refresh"]').trigger('click')
    await w.vm.$nextTick()
    await w.vm.$nextTick()
    expect(w.get('[data-role="toast"]').text()).toBe('Device offline')
  })

  it('calls refreshNow when Refresh now button is clicked', async () => {
    const spy = vi.spyOn(deviceApi, 'refreshNow').mockResolvedValue(true)
    const w = mount(TopBar)
    await w.get('[data-role="refresh"]').trigger('click')
    expect(spy).toHaveBeenCalledOnce()
  })
})
