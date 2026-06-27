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

  it('the live page shows a live-badge; non-live pages show a make-live button', () => {
    const store = usePagesStore()
    store.addPage()
    // pages[0] is live by default
    const w = mount(PageSidebar)
    const thumbs = w.findAll('[data-role="thumb"]')
    expect(thumbs[0].find('[data-role="live-badge"]').exists()).toBe(true)
    expect(thumbs[0].find('[data-role="make-live"]').exists()).toBe(false)
    expect(thumbs[1].find('[data-role="live-badge"]').exists()).toBe(false)
    expect(thumbs[1].find('[data-role="make-live"]').exists()).toBe(true)
  })

  it('clicking make-live on a non-live page sets store.livePageId to that page', async () => {
    const store = usePagesStore()
    const b = store.addPage()
    const w = mount(PageSidebar)
    const thumbs = w.findAll('[data-role="thumb"]')
    await thumbs[1].find('[data-role="make-live"]').trigger('click')
    expect(store.livePageId).toBe(b)
  })

  it('clicking delete-page removes that page from store.pages', async () => {
    const store = usePagesStore()
    const b = store.addPage()
    const w = mount(PageSidebar)
    const thumbs = w.findAll('[data-role="thumb"]')
    await thumbs[1].find('[data-role="delete-page"]').trigger('click')
    expect(store.pages.some((p) => p.id === b)).toBe(false)
  })

  it('with one page, no delete-page button is rendered', () => {
    const w = mount(PageSidebar)
    expect(w.find('[data-role="delete-page"]').exists()).toBe(false)
  })

  it('a page that is both selected and live shows both the blue ring and the green border', () => {
    // Default state: pages[0] is both selected AND live — no addPage() needed.
    // addPage() would move selectedPageId to the new page and break the test.
    const w = mount(PageSidebar)
    const firstThumb = w.findAll('[data-role="thumb"]')[0]
    const classes = firstThumb.classes()
    // Selection indicator: blue ring
    expect(classes).toContain('ring-2')
    expect(classes).toContain('ring-blue-500')
    // Live indicator: green border (independent of ring)
    expect(classes).toContain('border-green-500')
  })
})
