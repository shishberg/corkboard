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

  it("uses a fixed 120×120 square with a page-shaped outline for each thumbnail's own orientation", () => {
    const store = usePagesStore()
    const landscapeId = store.selectedPageId!
    const portraitId = store.addPage()
    store.toggleOrientation()
    store.selectPage(landscapeId)

    const landscape = mount(PageThumbnail, { props: { pageId: landscapeId } })
    const portrait = mount(PageThumbnail, { props: { pageId: portraitId } })

    // The outer is just a 120×120 positioning container — no border, no
    // background, so a white page isn't camouflaged against it.
    const landscapeOuter = landscape.element as HTMLElement
    const portraitOuter = portrait.element as HTMLElement
    expect(landscapeOuter.style.width).toBe('120px')
    expect(landscapeOuter.style.height).toBe('120px')
    expect(portraitOuter.style.width).toBe('120px')
    expect(portraitOuter.style.height).toBe('120px')
    expect(landscapeOuter.classList.contains('border')).toBe(false)
    expect(landscapeOuter.classList.contains('bg-white')).toBe(false)
    expect(portraitOuter.classList.contains('border')).toBe(false)
    expect(portraitOuter.classList.contains('bg-white')).toBe(false)

    // The outline is the visible page rectangle at display size, bordered, so
    // the page outline is visible even when the page background is white.
    const landscapeOutline = landscape.find('[data-role="thumbnail-outline"]').element as HTMLElement
    const portraitOutline = portrait.find('[data-role="thumbnail-outline"]').element as HTMLElement
    expect(landscapeOutline.style.width).toBe('120px')
    expect(landscapeOutline.style.height).toBe('72px')
    expect(portraitOutline.style.width).toBe('72px')
    expect(portraitOutline.style.height).toBe('120px')
    expect(landscapeOutline.classList.contains('border')).toBe(true)
    expect(portraitOutline.classList.contains('border')).toBe(true)
  })

  it('centers the page outline inside the 120×120 box', () => {
    const store = usePagesStore()
    const landscapeId = store.selectedPageId!
    const portraitId = store.addPage()
    store.toggleOrientation()
    store.selectPage(landscapeId)

    const landscape = mount(PageThumbnail, { props: { pageId: landscapeId } })
    const portrait = mount(PageThumbnail, { props: { pageId: portraitId } })

    const landscapeOutline = landscape.find('[data-role="thumbnail-outline"]').element as HTMLElement
    const portraitOutline = portrait.find('[data-role="thumbnail-outline"]').element as HTMLElement
    expect(landscapeOutline.style.top).toBe('24px')
    expect(landscapeOutline.style.left).toBe('0px')
    expect(portraitOutline.style.top).toBe('0px')
    expect(portraitOutline.style.left).toBe('24px')
  })

  it('paints the page outline with the page background colour', () => {
    const store = usePagesStore()
    store.setPageBackground('red')
    const pageId = store.selectedPageId!
    const w = mount(PageThumbnail, { props: { pageId } })
    const outline = w.find('[data-role="thumbnail-outline"]').element as HTMLElement
    expect(outline.style.backgroundColor).toBe('red')
  })

  it('defaults the page outline background to white when the page has none', () => {
    const store = usePagesStore()
    const pageId = store.selectedPageId!
    const w = mount(PageThumbnail, { props: { pageId } })
    const outline = w.find('[data-role="thumbnail-outline"]').element as HTMLElement
    expect(outline.style.backgroundColor).toBe('white')
  })
})
