import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import PageSidebar from './PageSidebar.vue'
import { usePagesStore } from '@/stores/pages'

beforeEach(() => setActivePinia(createPinia()))

describe('PageSidebar', () => {
  it('shows one thumbnail per page', () => {
    const store = usePagesStore()
    store.addPage()
    const w = mount(PageSidebar)
    expect(w.findAll('[data-role="thumb"]').length).toBe(2)
  })

  it('the add button adds a page', async () => {
    const store = usePagesStore()
    const w = mount(PageSidebar)
    await w.get('[data-role="add-page"]').trigger('click')
    expect(store.pages.length).toBe(2)
  })

  it('clicking a thumbnail selects that page', async () => {
    const store = usePagesStore()
    const second = store.addPage()
    store.selectPage(store.pages[0].id)
    const w = mount(PageSidebar)
    await w.findAll('[data-role="thumb"]')[1].trigger('click')
    expect(store.selectedPageId).toBe(second)
  })
})
