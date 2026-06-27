import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useFontsStore } from './fonts'
import * as fontsLib from '@/lib/fonts'
import type { FontDef } from '@/lib/fonts'

beforeEach(() => {
  setActivePinia(createPinia())
})

afterEach(() => {
  vi.restoreAllMocks()
  document.head.querySelector('style[data-role="font-faces"]')?.remove()
})

const MOCK_FONTS: FontDef[] = [
  {
    id: 'test-font',
    name: 'Test Font',
    default: true,
    faces: [{ weight: 400, style: 'normal', file: 'test-font/Regular.ttf' }],
  },
]

describe('useFontsStore', () => {
  it('starts with the DEFAULT_MANIFEST', () => {
    const store = useFontsStore()
    expect(store.fonts).toEqual(fontsLib.DEFAULT_MANIFEST)
  })

  it('defaultId returns the default font id from DEFAULT_MANIFEST', () => {
    const store = useFontsStore()
    expect(store.defaultId).toBe('atkinson-hyperlegible')
  })

  it('load() replaces fonts with the fetched manifest', async () => {
    vi.spyOn(fontsLib, 'loadFontManifest').mockResolvedValue(MOCK_FONTS)
    vi.spyOn(fontsLib, 'injectFontFaces').mockImplementation(() => {})
    const store = useFontsStore()
    await store.load()
    expect(store.fonts).toEqual(MOCK_FONTS)
  })

  it('load() calls injectFontFaces with the loaded fonts', async () => {
    vi.spyOn(fontsLib, 'loadFontManifest').mockResolvedValue(MOCK_FONTS)
    const inject = vi.spyOn(fontsLib, 'injectFontFaces').mockImplementation(() => {})
    const store = useFontsStore()
    await store.load()
    expect(inject).toHaveBeenCalledWith(MOCK_FONTS)
  })
})
