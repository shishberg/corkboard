import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { usePagesStore } from './pages'
import type { ClockEl } from './types'

beforeEach(() => setActivePinia(createPinia()))

function clockEl(id: string): ClockEl {
  return { id, type: 'clock', variant: 'time', x: 0, y: 0, w: 200, h: 80 }
}

describe('usePagesStore', () => {
  it('starts with one page selected', () => {
    const s = usePagesStore()
    expect(s.pages.length).toBe(1)
    expect(s.selectedPageId).toBe(s.pages[0].id)
  })

  it('addPage adds and selects a new page', () => {
    const s = usePagesStore()
    const id = s.addPage()
    expect(s.pages.length).toBe(2)
    expect(s.selectedPageId).toBe(id)
  })

  it('toggleOrientation flips orientation and pageSize', () => {
    const s = usePagesStore()
    expect(s.orientation).toBe('landscape')
    expect(s.pageSize).toEqual({ w: 800, h: 480 })
    s.toggleOrientation()
    expect(s.orientation).toBe('portrait')
    expect(s.pageSize).toEqual({ w: 480, h: 800 })
  })

  it('addElement pushes onto the selected page and selects it', () => {
    const s = usePagesStore()
    s.addElement(clockEl('e1'))
    expect(s.selectedPage?.elements.length).toBe(1)
    expect(s.selectedElId).toBe('e1')
  })

  it('updateElement patches geometry', () => {
    const s = usePagesStore()
    s.addElement(clockEl('e1'))
    s.updateElement('e1', { x: 50, y: 60 })
    const el = s.selectedPage?.elements[0]
    expect(el?.x).toBe(50)
    expect(el?.y).toBe(60)
  })

  it('addToTimeline appends with a default delay', () => {
    const s = usePagesStore()
    const pid = s.pages[0].id
    s.addToTimeline(pid)
    expect(s.timeline).toEqual([{ pageId: pid, delayMs: 5000 }])
  })

  it('reorderTimeline moves an entry', () => {
    const s = usePagesStore()
    const a = s.pages[0].id
    const b = s.addPage()
    s.addToTimeline(a)
    s.addToTimeline(b)
    s.reorderTimeline(0, 1)
    expect(s.timeline.map((t) => t.pageId)).toEqual([b, a])
  })

  it('setTimelineDelay updates one entry', () => {
    const s = usePagesStore()
    s.addToTimeline(s.pages[0].id)
    s.setTimelineDelay(0, 12000)
    expect(s.timeline[0].delayMs).toBe(12000)
  })

  // Guard tests (review findings)

  it('reorderTimeline: ignores negative from index', () => {
    const s = usePagesStore()
    const a = s.pages[0].id
    const b = s.addPage()
    s.addToTimeline(a)
    s.addToTimeline(b)
    const before = s.timeline.map((t) => t.pageId)
    s.reorderTimeline(-1, 0)
    expect(s.timeline.map((t) => t.pageId)).toEqual(before)
  })

  it('reorderTimeline: ignores out-of-range to index', () => {
    const s = usePagesStore()
    const a = s.pages[0].id
    const b = s.addPage()
    s.addToTimeline(a)
    s.addToTimeline(b)
    const before = s.timeline.map((t) => t.pageId)
    s.reorderTimeline(0, 99)
    expect(s.timeline.map((t) => t.pageId)).toEqual(before)
  })

  it('reorderTimeline: same-index is a no-op', () => {
    const s = usePagesStore()
    s.addToTimeline(s.pages[0].id)
    const before = [...s.timeline]
    s.reorderTimeline(0, 0)
    expect(s.timeline).toEqual(before)
  })

  it('addToTimeline: ignores unknown pageId', () => {
    const s = usePagesStore()
    s.addToTimeline('does-not-exist')
    expect(s.timeline.length).toBe(0)
  })

  it('selectPage: ignores unknown id, leaves selection unchanged', () => {
    const s = usePagesStore()
    const originalId = s.selectedPageId
    s.addElement(clockEl('e1'))
    s.selectPage('ghost-id')
    expect(s.selectedPageId).toBe(originalId)
    expect(s.selectedElId).toBe('e1')
  })

  it('updateElement: patch cannot change id or type', () => {
    const s = usePagesStore()
    s.addElement(clockEl('e1'))
    s.updateElement('e1', { id: 'hacked', type: 'calendar' } as Partial<import('./types').BaseEl>)
    const el = s.selectedPage?.elements[0]
    expect(el?.id).toBe('e1')
    expect(el?.type).toBe('clock')
  })

  it('removeFromTimeline: ignores negative index', () => {
    const s = usePagesStore()
    s.addToTimeline(s.pages[0].id)
    s.removeFromTimeline(-1)
    expect(s.timeline.length).toBe(1)
  })
})
