import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { usePagesStore } from './pages'
import type { ImageEl, DrawingEl, TextEl, DocState } from './types'

beforeEach(() => setActivePinia(createPinia()))

function imageEl(id: string): ImageEl {
  return { id, type: 'image', x: 0, y: 0, w: 200, h: 150, colour: 'black', src: '' }
}

function textEl(id: string): TextEl {
  return { id, type: 'text', x: 0, y: 0, w: 240, h: 80, colour: 'black', text: 'Text', font: 'atkinson-hyperlegible', align: 'left' }
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

  it('setElementText sets text on a text element', () => {
    const s = usePagesStore()
    s.addElement(textEl('t1'))
    s.setElementText('t1', 'Hello')
    const el = s.selectedPage?.elements[0] as TextEl
    expect(el.text).toBe('Hello')
  })

  it('setElementText is a no-op for an unknown id', () => {
    const s = usePagesStore()
    s.addElement(textEl('t1'))
    s.setElementText('ghost', 'ignored')
    const el = s.selectedPage?.elements[0] as TextEl
    expect(el.text).toBe('Text')
  })

  it('setElementText is a no-op for a non-text element', () => {
    const s = usePagesStore()
    s.addElement(imageEl('i1'))
    s.setElementText('i1', 'ignored')
    expect(s.selectedPage?.elements[0].type).toBe('image')
  })

  it('a blank page starts with a white background', () => {
    const s = usePagesStore()
    expect(s.selectedPage?.background).toBe('white')
  })

  it('setPageBackground sets the current page background colour', () => {
    const s = usePagesStore()
    s.setPageBackground('blue')
    expect(s.selectedPage?.background).toBe('blue')
  })

  it('hydrate defaults a missing page background to white', () => {
    const s = usePagesStore()
    const doc: DocState = {
      orientation: 'landscape',
      pages: [{ id: 'p1', name: 'Page', elements: [] }],
      livePageId: 'p1',
      selectedPageId: 'p1',
      selectedElId: null,
      activeTool: 'select',
    }
    s.hydrate(doc)
    expect(s.selectedPage?.background).toBe('white')
  })

  it('bringToFront moves the element to the end of the array (drawn last = on top)', () => {
    const s = usePagesStore()
    s.addElement(imageEl('a'))
    s.addElement(imageEl('b'))
    s.addElement(imageEl('c'))
    s.bringToFront('a')
    expect(s.selectedPage?.elements.map((e) => e.id)).toEqual(['b', 'c', 'a'])
  })

  it('sendToBack moves the element to the start of the array (drawn first = behind)', () => {
    const s = usePagesStore()
    s.addElement(imageEl('a'))
    s.addElement(imageEl('b'))
    s.addElement(imageEl('c'))
    s.sendToBack('c')
    expect(s.selectedPage?.elements.map((e) => e.id)).toEqual(['c', 'a', 'b'])
  })

  it('bringToFront/sendToBack default to the selected element', () => {
    const s = usePagesStore()
    s.addElement(imageEl('a'))
    s.addElement(imageEl('b'))
    s.selectElement('a')
    s.bringToFront()
    expect(s.selectedPage?.elements.map((e) => e.id)).toEqual(['b', 'a'])
    s.selectElement('a')
    s.sendToBack()
    expect(s.selectedPage?.elements.map((e) => e.id)).toEqual(['a', 'b'])
  })

  it('bringToFront is a no-op for an unknown id', () => {
    const s = usePagesStore()
    s.addElement(imageEl('a'))
    s.addElement(imageEl('b'))
    s.bringToFront('ghost')
    expect(s.selectedPage?.elements.map((e) => e.id)).toEqual(['a', 'b'])
  })

  it('setElementSrc sets src on an image element', () => {
    const s = usePagesStore()
    s.addElement(imageEl('i1'))
    s.setElementSrc('i1', 'img-abc123')
    const el = s.selectedPage?.elements[0] as ImageEl
    expect(el.src).toBe('img-abc123')
  })

  it('setElementSrc is a no-op for a non-image element', () => {
    const s = usePagesStore()
    s.addElement(textEl('t1'))
    s.setElementSrc('t1', 'img-abc123')
    const el = s.selectedPage?.elements[0] as TextEl
    expect(el.text).toBe('Text')
  })

  it('setElementFont sets font on a text element', () => {
    const s = usePagesStore()
    s.addElement(textEl('t1'))
    s.setElementFont('t1', 'other-font')
    const el = s.selectedPage?.elements[0] as TextEl
    expect(el.font).toBe('other-font')
  })

  it('setElementFont is a no-op for an unknown id', () => {
    const s = usePagesStore()
    s.addElement(textEl('t1'))
    s.setElementFont('ghost', 'other-font')
    const el = s.selectedPage?.elements[0] as TextEl
    expect(el.font).toBe('atkinson-hyperlegible')
  })

  it('setElementFont is a no-op for a non-text element', () => {
    const s = usePagesStore()
    s.addElement(imageEl('i1'))
    // should not throw
    s.setElementFont('i1', 'any-font')
    expect(s.selectedPage?.elements[0].type).toBe('image')
  })

  it('setElementAlign sets align on a text element', () => {
    const s = usePagesStore()
    s.addElement(textEl('t1'))
    s.setElementAlign('t1', 'center')
    const el = s.selectedPage?.elements[0] as TextEl
    expect(el.align).toBe('center')
  })

  it('setElementAlign is a no-op for an unknown id', () => {
    const s = usePagesStore()
    s.addElement(textEl('t1'))
    s.setElementAlign('ghost', 'center')
    const el = s.selectedPage?.elements[0] as TextEl
    expect(el.align).toBe('left')
  })

  it('setElementAlign is a no-op for a non-text element', () => {
    const s = usePagesStore()
    s.addElement(imageEl('i1'))
    // should not throw
    s.setElementAlign('i1', 'center')
    expect(s.selectedPage?.elements[0].type).toBe('image')
  })

  // deletePage
  it('deletePage removes a page (length drops, page is gone)', () => {
    const s = usePagesStore()
    const b = s.addPage()
    expect(s.pages.length).toBe(2)
    s.deletePage(b)
    expect(s.pages.length).toBe(1)
    expect(s.pages.some((p) => p.id === b)).toBe(false)
  })

  it('deletePage reassigns livePageId when the live page is deleted', () => {
    const s = usePagesStore()
    const b = s.addPage()
    s.setLivePage(b)
    expect(s.livePageId).toBe(b)
    s.deletePage(b)
    expect(s.pages.length).toBe(1)
    expect(s.livePageId).toBe(s.pages[0].id)
    expect(s.livePage?.id).toBe(s.pages[0].id)
  })

  it('deletePage reassigns selectedPageId and clears selectedElId when selected page is deleted', () => {
    const s = usePagesStore()
    const a = s.pages[0].id
    const b = s.addPage()
    // Select page b and add an element to it
    s.selectPage(b)
    s.addElement(imageEl('e1'))
    expect(s.selectedPageId).toBe(b)
    expect(s.selectedElId).toBe('e1')
    // Delete b — selection should fall back to first remaining page
    s.deletePage(b)
    expect(s.selectedPageId).toBe(a)
    expect(s.selectedElId).toBe(null)
  })

  it('deletePage leaves livePageId and selectedPageId unchanged when they point at a different page', () => {
    const s = usePagesStore()
    const a = s.pages[0].id
    s.addPage()
    const c = s.addPage()
    s.setLivePage(a)
    s.selectPage(a)
    // Delete page c which is neither live nor selected
    s.deletePage(c)
    expect(s.livePageId).toBe(a)
    expect(s.selectedPageId).toBe(a)
  })

  it('deletePage is a no-op when only one page remains', () => {
    const s = usePagesStore()
    expect(s.pages.length).toBe(1)
    const id = s.pages[0].id
    s.deletePage(id)
    expect(s.pages.length).toBe(1)
    expect(s.livePageId).toBe(id)
  })

  it('deletePage is a no-op for an unknown id', () => {
    const s = usePagesStore()
    s.addPage()
    const before = s.pages.length
    s.deletePage('ghost')
    expect(s.pages.length).toBe(before)
  })

  // hydrate
  describe('hydrate', () => {
    it('replaces pages and livePageId from a loaded doc', () => {
      const s = usePagesStore()
      const loaded: DocState = {
        orientation: 'portrait',
        pages: [{ id: 'loaded-p1', name: 'Loaded', elements: [] }],
        livePageId: 'loaded-p1',
        selectedPageId: 'loaded-p1',
        selectedElId: null,
        activeTool: 'draw',
      }
      s.hydrate(loaded)
      expect(s.pages.length).toBe(1)
      expect(s.pages[0].id).toBe('loaded-p1')
      expect(s.livePageId).toBe('loaded-p1')
      expect(s.orientation).toBe('portrait')
      expect(s.activeTool).toBe('draw')
    })

    it('is a no-op when doc.pages is empty', () => {
      const s = usePagesStore()
      const originalPageId = s.pages[0].id
      const empty: DocState = {
        orientation: 'portrait',
        pages: [],
        livePageId: null,
        selectedPageId: null,
        selectedElId: null,
        activeTool: 'draw',
      }
      s.hydrate(empty)
      expect(s.pages.length).toBe(1)
      expect(s.pages[0].id).toBe(originalPageId)
    })

    it('fixes a dangling selectedPageId to the first page', () => {
      const s = usePagesStore()
      const loaded: DocState = {
        orientation: 'landscape',
        pages: [{ id: 'real-p1', name: 'Page', elements: [] }],
        livePageId: 'real-p1',
        selectedPageId: 'ghost-id',
        selectedElId: null,
        activeTool: 'select',
      }
      s.hydrate(loaded)
      expect(s.selectedPageId).toBe('real-p1')
    })
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
