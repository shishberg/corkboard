import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import { nextTick } from 'vue'
import CalendarOptions from './CalendarOptions.vue'
import { useFeedsStore } from '@/stores/feeds'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { usePagesStore } from '@/stores/pages'

function calendarEl(id: string) {
  return { id, type: 'calendar' as const, variant: 'agenda' as const, x: 0, y: 0, w: 200, h: 80, feedId: '', colour: 'black' as const, font: 'atkinson-hyperlegible', align: 'center' as const, daysAhead: 7 as const }
}

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

describe('CalendarOptions selected-element editing', () => {
  it('shows and edits the variant of a selected calendar (no "Variant"/"Feed" labels)', async () => {
    const store = usePagesStore()
    store.addElement({ ...calendarEl('c1'), variant: 'agenda' })
    const w = mount(CalendarOptions)
    await nextTick()
    expect(w.text()).not.toContain('Feed')
    // reflects the element's variant
    expect(w.get('[data-variant="agenda"]').classes().join(' ')).toContain('font-medium')
    await w.get('[data-variant="date"]').trigger('click')
    expect((store.selectedPage?.elements[0] as { variant: string }).variant).toBe('date')
  })

  it('edits the feed of a selected calendar', async () => {
    const feeds = useFeedsStore()
    feeds.feeds = [{ id: 'family', name: 'Family' }, { id: 'work', name: 'Work' }]
    const store = usePagesStore()
    store.addElement({ ...calendarEl('c1'), feedId: 'family' })
    const w = mount(CalendarOptions)
    await nextTick()
    await w.get('[data-role="feed-select"]').setValue('work')
    expect((store.selectedPage?.elements[0] as { feedId: string }).feedId).toBe('work')
  })
})
