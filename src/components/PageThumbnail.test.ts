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
})
