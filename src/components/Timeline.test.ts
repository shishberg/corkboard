import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import Timeline from './Timeline.vue'
import { usePagesStore } from '@/stores/pages'

beforeEach(() => setActivePinia(createPinia()))

describe('Timeline', () => {
  it('renders one item per timeline entry', () => {
    const store = usePagesStore()
    store.addToTimeline(store.pages[0].id)
    const w = mount(Timeline)
    expect(w.findAll('[data-role="timeline-item"]').length).toBe(1)
  })

  it('dropping a page id appends it to the timeline', async () => {
    const store = usePagesStore()
    const pid = store.pages[0].id
    const w = mount(Timeline)
    const dt = { getData: (t: string) => (t === 'text/plain' ? pid : '') }
    await w.get('[data-role="timeline-strip"]').trigger('drop', { dataTransfer: dt })
    expect(store.timeline.map((e) => e.pageId)).toEqual([pid])
  })

  it('dragging an item left onto an earlier item moves it to that slot, not the end', async () => {
    const store = usePagesStore()
    const p0 = store.pages[0].id
    const p1 = store.addPage()
    const p2 = store.addPage()
    store.addToTimeline(p0)
    store.addToTimeline(p1)
    store.addToTimeline(p2)
    const w = mount(Timeline)
    const items = w.findAll('[data-role="timeline-item"]')
    // Drag the last entry (idx 2) onto the first item (index 0).
    const dt = { getData: (t: string) => (t === 'text/plain' ? 'idx:2' : '') }
    await items[0].trigger('drop', { dataTransfer: dt })
    expect(store.timeline.map((e) => e.pageId)).toEqual([p2, p0, p1])
  })

  it('dragging an item right onto a later item moves it to that slot', async () => {
    const store = usePagesStore()
    const p0 = store.pages[0].id
    const p1 = store.addPage()
    const p2 = store.addPage()
    store.addToTimeline(p0)
    store.addToTimeline(p1)
    store.addToTimeline(p2)
    const w = mount(Timeline)
    const items = w.findAll('[data-role="timeline-item"]')
    // Drag the first entry (idx 0) onto the last item (index 2).
    const dt = { getData: (t: string) => (t === 'text/plain' ? 'idx:0' : '') }
    await items[2].trigger('drop', { dataTransfer: dt })
    expect(store.timeline.map((e) => e.pageId)).toEqual([p1, p2, p0])
  })

  it('setting a delay updates the store in milliseconds', async () => {
    const store = usePagesStore()
    store.addToTimeline(store.pages[0].id)
    const w = mount(Timeline)
    const input = w.get('[data-role="delay-input"]')
    await input.setValue('8')
    expect(store.timeline[0].delayMs).toBe(8000)
  })
})
