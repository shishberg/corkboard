import { describe, it, expect, vi, afterEach } from 'vitest'
import { loadFontManifest, injectFontFaces, defaultFontId, DEFAULT_MANIFEST } from './fonts'
import type { FontDef } from './fonts'

afterEach(() => {
  vi.restoreAllMocks()
  // Clean up any injected style
  document.head.querySelector('style[data-role="font-faces"]')?.remove()
})

const SAMPLE: FontDef[] = [
  {
    id: 'my-font',
    name: 'My Font',
    default: true,
    faces: [{ weight: 400, style: 'normal', file: 'my-font/Regular.ttf' }],
  },
]

describe('loadFontManifest', () => {
  it('returns parsed fonts array on success', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ fonts: SAMPLE }),
    } as unknown as Response)
    const result = await loadFontManifest()
    expect(result).toEqual(SAMPLE)
    expect(global.fetch).toHaveBeenCalledWith('/fonts/manifest.json')
  })

  it('returns DEFAULT_MANIFEST on non-ok response', async () => {
    global.fetch = vi.fn().mockResolvedValue({ ok: false, status: 404 } as unknown as Response)
    const result = await loadFontManifest()
    expect(result).toEqual(DEFAULT_MANIFEST)
  })

  it('returns DEFAULT_MANIFEST when fetch throws', async () => {
    global.fetch = vi.fn().mockRejectedValue(new Error('offline'))
    const result = await loadFontManifest()
    expect(result).toEqual(DEFAULT_MANIFEST)
  })
})

describe('injectFontFaces', () => {
  it('adds a style[data-role="font-faces"] to document.head', () => {
    injectFontFaces(SAMPLE)
    const el = document.head.querySelector('style[data-role="font-faces"]')
    expect(el).not.toBeNull()
  })

  it('includes the font id and /fonts/ path in the injected CSS', () => {
    injectFontFaces(SAMPLE)
    const css = document.head.querySelector('style[data-role="font-faces"]')?.textContent ?? ''
    expect(css).toContain('my-font')
    expect(css).toContain('/fonts/')
  })

  it('replaces the existing style element on repeated calls (idempotent)', () => {
    injectFontFaces(SAMPLE)
    injectFontFaces(SAMPLE)
    const all = document.head.querySelectorAll('style[data-role="font-faces"]')
    expect(all.length).toBe(1)
  })

  it('injects an @font-face rule', () => {
    injectFontFaces(SAMPLE)
    const css = document.head.querySelector('style[data-role="font-faces"]')?.textContent ?? ''
    expect(css).toContain('@font-face')
  })
})

describe('defaultFontId', () => {
  it('returns the id of the font with default: true', () => {
    const fonts: FontDef[] = [
      { id: 'first', name: 'First', faces: [] },
      { id: 'second', name: 'Second', default: true, faces: [] },
    ]
    expect(defaultFontId(fonts)).toBe('second')
  })

  it('falls back to the first id when no default is marked', () => {
    const fonts: FontDef[] = [
      { id: 'first', name: 'First', faces: [] },
      { id: 'second', name: 'Second', faces: [] },
    ]
    expect(defaultFontId(fonts)).toBe('first')
  })

  it('falls back to atkinson-hyperlegible for an empty array', () => {
    expect(defaultFontId([])).toBe('atkinson-hyperlegible')
  })
})
