import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import TopBar from './TopBar.vue'
import * as deviceApi from '@/lib/deviceApi'
import { usePagesStore } from '@/stores/pages'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('TopBar', () => {
  it('has no Refresh button — Publish re-fetches and refreshes on its own', () => {
    const w = mount(TopBar)
    expect(w.find('[data-role="refresh"]').exists()).toBe(false)
  })

  it('calls putDocument with the store state and shows Published toast on success', async () => {
    const spy = vi.spyOn(deviceApi, 'putDocument').mockResolvedValue(true)
    const store = usePagesStore()
    const w = mount(TopBar)
    await w.get('[data-role="publish"]').trigger('click')
    await w.vm.$nextTick()
    await w.vm.$nextTick()
    expect(spy).toHaveBeenCalledWith(store.$state)
    expect(w.get('[data-role="toast"]').text()).toBe('Published')
  })

  it('shows Device offline toast when putDocument returns false', async () => {
    vi.spyOn(deviceApi, 'putDocument').mockResolvedValue(false)
    const w = mount(TopBar)
    await w.get('[data-role="publish"]').trigger('click')
    await w.vm.$nextTick()
    await w.vm.$nextTick()
    expect(w.get('[data-role="toast"]').text()).toBe('Device offline')
  })

  it('Publish makes the currently selected page live', async () => {
    vi.spyOn(deviceApi, 'putDocument').mockResolvedValue(true)
    const store = usePagesStore()
    const second = store.addPage() // addPage selects the new page
    expect(store.livePageId).not.toBe(second)
    const w = mount(TopBar)
    await w.get('[data-role="publish"]').trigger('click')
    expect(store.livePageId).toBe(second)
  })

  it('has a Preview link that opens the live preview page in a new tab', () => {
    const w = mount(TopBar)
    const preview = w.get('[data-role="preview"]')
    expect(preview.attributes('href')).toBe('/preview')
    expect(preview.attributes('target')).toBe('_blank')
  })
})
