import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import FontOptions from './FontOptions.vue'
import { usePagesStore } from '@/stores/pages'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { useFontsStore } from '@/stores/fonts'
import * as fontsLib from '@/lib/fonts'

function textEl(id: string) {
  return { id, type: 'text' as const, x: 0, y: 0, w: 240, h: 80, colour: 'black' as const, text: 'Text', font: 'atkinson-hyperlegible', align: 'left' as const }
}
function calendarEl(id: string) {
  return { id, type: 'calendar' as const, variant: 'agenda' as const, x: 0, y: 0, w: 200, h: 80, feedId: '', colour: 'black' as const, font: 'atkinson-hyperlegible', align: 'center' as const, daysAhead: 7 as const }
}

beforeEach(() => {
  setActivePinia(createPinia())
  // Prevent real network calls from onMounted fonts.load()
  vi.spyOn(fontsLib, 'loadFontManifest').mockResolvedValue(fontsLib.DEFAULT_MANIFEST)
  vi.spyOn(fontsLib, 'injectFontFaces').mockImplementation(() => {})
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('FontOptions', () => {
  it('font-select shows the stored tool default when no element is selected', () => {
    const opts = useToolOptionsStore()
    opts.font = 'atkinson-hyperlegible'
    const w = mount(FontOptions)
    const select = w.find('[data-role="font-select"]').element as HTMLSelectElement
    expect(select.value).toBe('atkinson-hyperlegible')
  })

  it('font-select falls back to the manifest default when the stored font is not loaded', () => {
    const opts = useToolOptionsStore()
    opts.font = 'not-in-manifest'
    const fontsStore = useFontsStore()
    expect(fontsStore.defaultId).toBe('atkinson-hyperlegible')
    const w = mount(FontOptions)
    const select = w.find('[data-role="font-select"]').element as HTMLSelectElement
    expect(select.value).toBe('atkinson-hyperlegible')
  })

  it('font-select falls back to manifest default after load() replaces fonts with a custom manifest', async () => {
    const customManifest: fontsLib.FontDef[] = [
      {
        id: 'custom-font',
        name: 'Custom Font',
        default: true,
        faces: [{ weight: 400, style: 'normal', file: 'custom/Regular.ttf' }],
      },
    ]
    vi.mocked(fontsLib.loadFontManifest).mockResolvedValue(customManifest)
    const opts = useToolOptionsStore()
    opts.font = 'atkinson-hyperlegible' // not in custom manifest
    const w = mount(FontOptions)
    const fontsStore = useFontsStore()
    await fontsStore.load()
    await w.vm.$nextTick()
    const select = w.find('[data-role="font-select"]').element as HTMLSelectElement
    expect(select.value).toBe('custom-font')
  })

  it('shows the selected text element font and edits it', async () => {
    const store = usePagesStore()
    store.addElement({ ...textEl('t1'), font: 'gelasio' })
    const w = mount(FontOptions)
    const select = w.find('[data-role="font-select"]').element as HTMLSelectElement
    expect(select.value).toBe('gelasio')
    await w.get('[data-role="font-select"]').setValue('carlito')
    expect((store.selectedPage?.elements[0] as { font: string }).font).toBe('carlito')
  })

  it('edits the font of a selected calendar element', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('c1'))
    const w = mount(FontOptions)
    await w.get('[data-role="font-select"]').setValue('gelasio')
    expect((store.selectedPage?.elements[0] as { font: string }).font).toBe('gelasio')
  })
})
