import { defineStore } from 'pinia'
import type { DocState, LoadedDoc, El, Page, ToolId, BaseEl, EpaperColour, TextEl, CalendarEl, ImageEl, Orientation, Size } from './types'
import { pageSize } from './types'

let counter = 0
const uid = (prefix: string) => `${prefix}-${Date.now().toString(36)}-${(counter++).toString(36)}`

function blankPage(orientation: Orientation = 'landscape'): Page {
  return { id: uid('page'), name: 'Page', elements: [], background: 'white', orientation }
}

export const usePagesStore = defineStore('pages', {
  state: (): DocState => {
    const first = blankPage()
    first.name = 'Page 1'
    return {
      pages: [first],
      livePageId: first.id,
      selectedPageId: first.id,
      selectedElId: null,
      activeTool: 'select',
    }
  },

  getters: {
    selectedPage(state): Page | null {
      return state.pages.find((p) => p.id === state.selectedPageId) ?? null
    },
    livePage(state): Page | null {
      return state.pages.find((p) => p.id === state.livePageId) ?? null
    },
    // Canvas size for the page being edited. Orientation is per-page now, so
    // this follows the selected page (defaulting to landscape if none).
    pageSize(): Size {
      return pageSize(this.selectedPage)
    },
  },

  actions: {
    addPage(): string {
      // Inherit the orientation of the page you're on, so adding to a portrait
      // board keeps it portrait.
      const page = blankPage(this.selectedPage?.orientation ?? 'landscape')
      page.name = `Page ${this.pages.length + 1}`
      this.pages.push(page)
      this.selectedPageId = page.id
      this.selectedElId = null
      return page.id
    },
    selectPage(id: string) {
      if (!this.pages.some((p) => p.id === id)) return
      this.selectedPageId = id
      this.selectedElId = null
    },
    toggleOrientation() {
      const page = this.selectedPage
      if (!page) return
      page.orientation = page.orientation === 'landscape' ? 'portrait' : 'landscape'
    },
    setActiveTool(t: ToolId) {
      this.activeTool = t
    },
    addElement(el: El) {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      if (!page) return
      page.elements.push(el)
      this.selectedElId = el.id
    },
    selectElement(id: string | null) {
      this.selectedElId = id
    },
    // Elements render in array order, so the last element is drawn on top.
    // "Front" = end of the array, "back" = start.
    bringToFront(id?: string) {
      const el = this._takeElement(id)
      if (el) this._currentElements()?.push(el)
    },
    sendToBack(id?: string) {
      const el = this._takeElement(id)
      if (el) this._currentElements()?.unshift(el)
    },
    // Remove and return the target element from the current page (or null).
    _takeElement(id?: string): El | null {
      const targetId = id ?? this.selectedElId
      if (!targetId) return null
      const els = this._currentElements()
      const i = els?.findIndex((e) => e.id === targetId) ?? -1
      if (!els || i === -1) return null
      return els.splice(i, 1)[0]
    },
    _currentElements(): El[] | null {
      return this.pages.find((p) => p.id === this.selectedPageId)?.elements ?? null
    },
    deleteElement(id?: string) {
      const targetId = id ?? this.selectedElId
      if (!targetId) return
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      if (!page) return
      const i = page.elements.findIndex((e) => e.id === targetId)
      if (i === -1) return
      page.elements.splice(i, 1)
      if (this.selectedElId === targetId) this.selectedElId = null
    },
    updateElement(id: string, patch: Partial<BaseEl>) {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      const el = page?.elements.find((e) => e.id === id)
      if (el) {
        const { x, y, w, h } = patch
        if (x !== undefined) el.x = x
        if (y !== undefined) el.y = y
        if (w !== undefined) el.w = w
        if (h !== undefined) el.h = h
      }
    },
    deletePage(id: string) {
      if (this.pages.length <= 1) return
      const i = this.pages.findIndex((p) => p.id === id)
      if (i === -1) return
      this.pages.splice(i, 1)
      if (this.livePageId === id) this.livePageId = this.pages[0].id
      if (this.selectedPageId === id) {
        this.selectedPageId = this.pages[0].id
        this.selectedElId = null
      }
    },
    setLivePage(id: string) {
      if (!this.pages.some((p) => p.id === id)) return
      this.livePageId = id
    },
    setElementColour(id: string, colour: EpaperColour) {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      const el = page?.elements.find((e) => e.id === id)
      if (!el) return
      el.colour = colour
      if (el.type === 'drawing') {
        for (const stroke of el.strokes) stroke.colour = colour
      }
    },
    setElementSrc(id: string, src: string) {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      const el = page?.elements.find((e) => e.id === id)
      if (!el || el.type !== 'image') return
      ;(el as ImageEl).src = src
    },
    setPageBackground(colour: EpaperColour) {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      if (page) page.background = colour
    },
    setElementText(id: string, text: string) {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      const el = page?.elements.find((e) => e.id === id)
      if (!el || el.type !== 'text') return
      ;(el as TextEl).text = text
    },
    setElementFont(id: string, font: string) {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      const el = page?.elements.find((e) => e.id === id)
      // Font applies to any element that contains text.
      if (!el || (el.type !== 'text' && el.type !== 'calendar')) return
      ;(el as TextEl | CalendarEl).font = font
    },
    setElementVariant(id: string, variant: CalendarEl['variant']) {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      const el = page?.elements.find((e) => e.id === id)
      if (!el || el.type !== 'calendar') return
      ;(el as CalendarEl).variant = variant
    },
    setElementFeed(id: string, feedId: string) {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      const el = page?.elements.find((e) => e.id === id)
      if (!el || el.type !== 'calendar') return
      ;(el as CalendarEl).feedId = feedId
    },
    setElementAlign(id: string, align: 'left' | 'center') {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      const el = page?.elements.find((e) => e.id === id)
      // Alignment applies to any element that contains text.
      if (!el || (el.type !== 'text' && el.type !== 'calendar')) return
      ;(el as TextEl | CalendarEl).align = align
    },
    hydrate(doc: LoadedDoc) {
      if (!doc.pages || doc.pages.length === 0) return
      // Older documents kept orientation at the document level; carry it onto
      // any page that lacks its own. Also default backgrounds (also once global).
      const legacyOrientation = doc.orientation ?? 'landscape'
      this.pages = doc.pages.map((p) => ({
        ...p,
        background: p.background ?? 'white',
        orientation: p.orientation ?? legacyOrientation,
        // Normalise legacy calendars: migrate the 'week' variant to 'agenda'
        // and default a missing align to 'center' (the old fixed date layout).
        elements: p.elements.map((el) =>
          el.type === 'calendar'
            ? {
                ...el,
                variant: ((el.variant as string) === 'week' ? 'agenda' : el.variant) as CalendarEl['variant'],
                align: (el as CalendarEl).align ?? 'center',
              }
            : el,
        ),
      }))
      this.activeTool = doc.activeTool
      this.selectedElId = doc.selectedElId
      const pageIds = new Set(doc.pages.map((p) => p.id))
      this.livePageId = doc.livePageId && pageIds.has(doc.livePageId) ? doc.livePageId : doc.pages[0].id
      this.selectedPageId = doc.selectedPageId && pageIds.has(doc.selectedPageId) ? doc.selectedPageId : doc.pages[0].id
    },
  },
})
