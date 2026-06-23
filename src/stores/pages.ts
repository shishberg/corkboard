import { defineStore } from 'pinia'
import type { DocState, El, Page, ToolId, BaseEl } from './types'

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
      timeline: [],
      selectedPageId: first.id,
      selectedElId: null,
      activeTool: 'select',
    }
  },

  getters: {
    selectedPage(state): Page | null {
      return state.pages.find((p) => p.id === state.selectedPageId) ?? null
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
    addToTimeline(pageId: string) {
      if (!this.pages.some((p) => p.id === pageId)) return
      this.timeline.push({ pageId, delayMs: 5000 })
    },
    reorderTimeline(from: number, to: number) {
      if (from === to) return
      const len = this.timeline.length
      if (!Number.isInteger(from) || from < 0 || from >= len) return
      if (!Number.isInteger(to) || to < 0 || to >= len) return
      const [moved] = this.timeline.splice(from, 1)
      this.timeline.splice(to, 0, moved)
    },
    setTimelineDelay(index: number, delayMs: number) {
      const entry = this.timeline[index]
      if (entry) entry.delayMs = delayMs
    },
    removeFromTimeline(index: number) {
      if (!Number.isInteger(index) || index < 0 || index >= this.timeline.length) return
      this.timeline.splice(index, 1)
    },
  },
})
