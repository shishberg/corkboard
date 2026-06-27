import { defineStore } from 'pinia'
import { loadFontManifest, injectFontFaces, defaultFontId, DEFAULT_MANIFEST } from '@/lib/fonts'
import type { FontDef } from '@/lib/fonts'

interface FontsState {
  fonts: FontDef[]
}

export const useFontsStore = defineStore('fonts', {
  state: (): FontsState => ({
    fonts: DEFAULT_MANIFEST,
  }),
  getters: {
    defaultId(state): string {
      return defaultFontId(state.fonts)
    },
  },
  actions: {
    async load() {
      this.fonts = await loadFontManifest()
      injectFontFaces(this.fonts)
    },
  },
})
