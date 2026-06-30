import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import PageThumbnail from './PageThumbnail.vue'
import { usePagesStore } from '@/stores/pages'
import { makeDrawingElement } from '@/stores/elementFactory'

beforeEach(() => setActivePinia(createPinia()))

function visualSize(el: HTMLElement): { width: number; height: number } {
  const m = el.style.transform.match(/scale\(([\d.]+)\)/)
  const scale = m ? parseFloat(m[1]) : 1
  return {
    width: parseFloat(el.style.width) * scale,
    height: parseFloat(el.style.height) * scale,
  }
}

describe('PageThumbnail', () => {
  it('renders a drawing element as an svg with data-role="drawing"', () => {
    const store = usePagesStore()
    const el = makeDrawingElement([{ x: 10, y: 10 }, { x: 30, y: 40 }], 'black', 4)
    store.addElement(el)
    const pageId = store.selectedPageId!
    const w = mount(PageThumbnail, { props: { pageId } })
    expect(w.find('[data-role="drawing"]').exists()).toBe(true)
  })

  it("uses a fixed 120×120 outer box for each thumbnail's own orientation", () => {
    const store = usePagesStore()
    const landscapeId = store.selectedPageId!
    const portraitId = store.addPage()
    store.toggleOrientation()
    store.selectPage(landscapeId)

    const landscape = mount(PageThumbnail, { props: { pageId: landscapeId } })
    const portrait = mount(PageThumbnail, { props: { pageId: portraitId } })

    const landscapeOuter = landscape.element as HTMLElement
    const portraitOuter = portrait.element as HTMLElement
    expect(landscapeOuter.style.width).toBe('120px')
    expect(landscapeOuter.style.height).toBe('120px')
    expect(portraitOuter.style.width).toBe('120px')
    expect(portraitOuter.style.height).toBe('120px')

    const landscapeInner = landscape.find('[data-role="thumbnail-inner"]').element as HTMLElement
    const portraitInner = portrait.find('[data-role="thumbnail-inner"]').element as HTMLElement
    expect(visualSize(landscapeInner)).toEqual({ width: 120, height: 72 })
    expect(visualSize(portraitInner)).toEqual({ width: 72, height: 120 })
  })

  it('centers the scaled content inside the 120×120 box', () => {
    const store = usePagesStore()
    const landscapeId = store.selectedPageId!
    const portraitId = store.addPage()
    store.toggleOrientation()
    store.selectPage(landscapeId)

    const landscape = mount(PageThumbnail, { props: { pageId: landscapeId } })
    const portrait = mount(PageThumbnail, { props: { pageId: portraitId } })

    const landscapeInner = landscape.find('[data-role="thumbnail-inner"]').element as HTMLElement
    const portraitInner = portrait.find('[data-role="thumbnail-inner"]').element as HTMLElement
    expect(landscapeInner.style.top).toBe('24px')
    expect(landscapeInner.style.left).toBe('0px')
    expect(portraitInner.style.top).toBe('0px')
    expect(portraitInner.style.left).toBe('24px')
  })

  it('paints the inner page rectangle with the page background colour', () => {
    const store = usePagesStore()
    store.setPageBackground('red')
    const pageId = store.selectedPageId!
    const w = mount(PageThumbnail, { props: { pageId } })
    const inner = w.find('[data-role="thumbnail-inner"]').element as HTMLElement
    expect(inner.style.backgroundColor).toBe('red')
  })

  it('defaults the inner page background to white when the page has none', () => {
    const store = usePagesStore()
    store.setPageBackground('yellow')
    const pageId = store.selectedPageId!
    store.setPageBackground('white')
    const w = mount(PageThumbnail, { props: { pageId } })
    const inner = w.find('[data-role="thumbnail-inner"]').element as HTMLElement
    expect(inner.style.backgroundColor).toBe('white')
  })
})
