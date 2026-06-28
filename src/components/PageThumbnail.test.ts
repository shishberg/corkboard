import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import PageThumbnail from './PageThumbnail.vue'
import { usePagesStore } from '@/stores/pages'
import { makeDrawingElement } from '@/stores/elementFactory'

beforeEach(() => setActivePinia(createPinia()))

describe('PageThumbnail', () => {
  it('renders a drawing element as an svg with data-role="drawing"', () => {
    const store = usePagesStore()
    const el = makeDrawingElement([{ x: 10, y: 10 }, { x: 30, y: 40 }], 'black', 4)
    store.addElement(el)
    const pageId = store.selectedPageId!
    const w = mount(PageThumbnail, { props: { pageId } })
    expect(w.find('[data-role="drawing"]').exists()).toBe(true)
  })

  it("sizes each thumbnail from its own page's orientation, not the selected page", () => {
    const store = usePagesStore()
    // Page 1 (landscape, the default) is selected.
    const landscapeId = store.selectedPageId!
    // Add a second page and make it portrait.
    const portraitId = store.addPage()
    store.toggleOrientation()
    // Re-select the landscape page so it's the "active" one.
    store.selectPage(landscapeId)

    // THUMB_W is 120. Landscape (800×480) → height 120 * 480/800 = 72.
    // Portrait (480×800) → height 120 * 800/480 = 200.
    const landscape = mount(PageThumbnail, { props: { pageId: landscapeId } })
    const portrait = mount(PageThumbnail, { props: { pageId: portraitId } })

    expect((landscape.element as HTMLElement).style.height).toBe('72px')
    expect((portrait.element as HTMLElement).style.height).toBe('200px')
  })
})
