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

  it('setting a delay updates the store in milliseconds', async () => {
    const store = usePagesStore()
    store.addToTimeline(store.pages[0].id)
    const w = mount(Timeline)
    const input = w.get('[data-role="delay-input"]')
    await input.setValue('8')
    expect(store.timeline[0].delayMs).toBe(8000)
  })
})
