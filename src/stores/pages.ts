import { defineStore } from 'pinia'
import type { DocState, El, Page, ToolId, BaseEl, EpaperColour, TextEl } from './types'

let counter = 0
const uid = (prefix: string) => `${prefix}-${Date.now().toString(36)}-${(counter++).toString(36)}`

function blankPage(): Page {
  return { id: uid('page'), name: 'Page', elements: [] }
}

export const usePagesStore = defineStore('pages', {
  state: (): DocState => {
    const first = blankPage()
    return {
      orientation: 'landscape',
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
    pageSize(state): { w: number; h: number } {
      return state.orientation === 'landscape' ? { w: 800, h: 480 } : { w: 480, h: 800 }
    },
  },

  actions: {
    addPage(): string {
      const page = blankPage()
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
      this.orientation = this.orientation === 'landscape' ? 'portrait' : 'landscape'
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
    setElementText(id: string, text: string) {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      const el = page?.elements.find((e) => e.id === id)
      if (!el || el.type !== 'text') return
      ;(el as TextEl).text = text
    },
    setElementFont(id: string, font: string) {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      const el = page?.elements.find((e) => e.id === id)
      if (!el || el.type !== 'text') return
      ;(el as TextEl).font = font
    },
    setElementAlign(id: string, align: 'left' | 'center') {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      const el = page?.elements.find((e) => e.id === id)
      if (!el || el.type !== 'text') return
      ;(el as TextEl).align = align
    },
  },
})
