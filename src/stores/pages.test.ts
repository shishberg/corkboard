import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { usePagesStore } from './pages'
import type { ImageEl, DrawingEl } from './types'

beforeEach(() => setActivePinia(createPinia()))

function imageEl(id: string): ImageEl {
  return { id, type: 'image', x: 0, y: 0, w: 200, h: 150, colour: 'black', src: '' }
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
    s.addElement(imageEl('e1'))
    expect(s.selectedPage?.elements.length).toBe(1)
    expect(s.selectedElId).toBe('e1')
  })

  it('deleteElement removes the selected element and clears selection', () => {
    const s = usePagesStore()
    s.addElement(imageEl('e1'))
    expect(s.selectedElId).toBe('e1')
    s.deleteElement()
    expect(s.selectedPage?.elements.length).toBe(0)
    expect(s.selectedElId).toBe(null)
  })

  it('deleteElement(id) removes a specific element', () => {
    const s = usePagesStore()
    s.addElement(imageEl('e1'))
    s.addElement(imageEl('e2'))
    s.deleteElement('e1')
    expect(s.selectedPage?.elements.map((e) => e.id)).toEqual(['e2'])
    // e2 was the selection; deleting e1 leaves it intact
    expect(s.selectedElId).toBe('e2')
  })

  it('updateElement patches geometry', () => {
    const s = usePagesStore()
    s.addElement(imageEl('e1'))
    s.updateElement('e1', { x: 50, y: 60 })
    const el = s.selectedPage?.elements[0]
    expect(el?.x).toBe(50)
    expect(el?.y).toBe(60)
  })

  it('starts with the first page live', () => {
    const s = usePagesStore()
    expect(s.livePageId).toBe(s.pages[0].id)
    expect(s.livePage?.id).toBe(s.pages[0].id)
  })

  it('setLivePage changes which page is live', () => {
    const s = usePagesStore()
    const b = s.addPage()
    s.setLivePage(b)
    expect(s.livePageId).toBe(b)
  })

  it('setLivePage ignores an unknown id', () => {
    const s = usePagesStore()
    const before = s.livePageId
    s.setLivePage('ghost')
    expect(s.livePageId).toBe(before)
  })

  it('selectPage: ignores unknown id, leaves selection unchanged', () => {
    const s = usePagesStore()
    const originalId = s.selectedPageId
    s.addElement(imageEl('e1'))
    s.selectPage('ghost-id')
    expect(s.selectedPageId).toBe(originalId)
    expect(s.selectedElId).toBe('e1')
  })

  it('updateElement: patch cannot change id or type', () => {
    const s = usePagesStore()
    s.addElement(imageEl('e1'))
    s.updateElement('e1', { id: 'hacked', type: 'calendar' } as Partial<import('./types').BaseEl>)
    const el = s.selectedPage?.elements[0]
    expect(el?.id).toBe('e1')
    expect(el?.type).toBe('image')
  })

  it('setElementColour sets the colour on an element', () => {
    const s = usePagesStore()
    s.addElement(imageEl('e1'))
    s.setElementColour('e1', 'red')
    expect(s.selectedPage?.elements[0].colour).toBe('red')
  })

  it('setElementColour is a no-op for an unknown id', () => {
    const s = usePagesStore()
    s.addElement(imageEl('e1'))
    s.setElementColour('ghost', 'blue')
    expect(s.selectedPage?.elements[0].colour).toBe('black')
  })

  it('setElementColour on a drawing element also updates each stroke colour', () => {
    const s = usePagesStore()
    const drawing: DrawingEl = {
      id: 'd1',
      type: 'drawing',
      x: 0,
      y: 0,
      w: 100,
      h: 100,
      natW: 100,
      natH: 100,
      colour: 'black',
      strokes: [
        { colour: 'black', size: 4, points: [{ x: 10, y: 10 }] },
        { colour: 'black', size: 4, points: [{ x: 20, y: 20 }] },
      ],
    }
    s.addElement(drawing)
    s.setElementColour('d1', 'red')
    const el = s.selectedPage?.elements[0] as DrawingEl
    expect(el.colour).toBe('red')
    expect(el.strokes[0].colour).toBe('red')
    expect(el.strokes[1].colour).toBe('red')
  })
})
