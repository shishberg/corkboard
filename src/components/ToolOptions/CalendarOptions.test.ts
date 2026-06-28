import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import { nextTick } from 'vue'
import CalendarOptions from './CalendarOptions.vue'
import { useFeedsStore } from '@/stores/feeds'
import { useToolOptionsStore } from '@/stores/toolOptions'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})
afterEach(() => vi.restoreAllMocks())

describe('CalendarOptions feed picker', () => {
  it('omits the (none) option when feeds are available', async () => {
    const feeds = useFeedsStore()
    feeds.feeds = [{ id: 'family', name: 'Family' }]
    const w = mount(CalendarOptions)
    await nextTick()
    const values = w.findAll('[data-role="feed-select"] option').map((o) => o.attributes('value'))
    expect(values).not.toContain('')
    expect(values).toContain('family')
  })

  it('shows the (none) option when there are no feeds', async () => {
    const feeds = useFeedsStore()
    feeds.feeds = []
    const w = mount(CalendarOptions)
    await nextTick()
    const values = w.findAll('[data-role="feed-select"] option').map((o) => o.attributes('value'))
    expect(values).toContain('')
  })

  it('defaults feedId to the first feed when one is available but none is selected', async () => {
    const feeds = useFeedsStore()
    feeds.feeds = [{ id: 'family', name: 'Family' }, { id: 'work', name: 'Work' }]
    const opts = useToolOptionsStore()
    opts.feedId = ''
    mount(CalendarOptions)
    await nextTick()
    expect(opts.feedId).toBe('family')
  })

  it('leaves an already-valid feedId alone', async () => {
    const feeds = useFeedsStore()
    feeds.feeds = [{ id: 'family', name: 'Family' }, { id: 'work', name: 'Work' }]
    const opts = useToolOptionsStore()
    opts.feedId = 'work'
    mount(CalendarOptions)
    await nextTick()
    expect(opts.feedId).toBe('work')
  })
})
