import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import TextOptions from './TextOptions.vue'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { useFontsStore } from '@/stores/fonts'
import * as fontsLib from '@/lib/fonts'

beforeEach(() => {
  setActivePinia(createPinia())
  // Prevent real network calls from onMounted fonts.load()
  vi.spyOn(fontsLib, 'loadFontManifest').mockResolvedValue(fontsLib.DEFAULT_MANIFEST)
  vi.spyOn(fontsLib, 'injectFontFaces').mockImplementation(() => {})
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('TextOptions', () => {
  it('font-select shows the stored font when it exists in the loaded manifest', () => {
    const opts = useToolOptionsStore()
    opts.font = 'atkinson-hyperlegible'

    const w = mount(TextOptions)
    const select = w.find('[data-role="font-select"]').element as HTMLSelectElement
    expect(select.value).toBe('atkinson-hyperlegible')
  })

  it('font-select falls back to the manifest default when the stored font is not loaded', () => {
    const opts = useToolOptionsStore()
    opts.font = 'not-in-manifest'

    // DEFAULT_MANIFEST has atkinson-hyperlegible as default
    const fontsStore = useFontsStore()
    expect(fontsStore.defaultId).toBe('atkinson-hyperlegible')

    const w = mount(TextOptions)
    const select = w.find('[data-role="font-select"]').element as HTMLSelectElement
    // The select should show the manifest default, not blank
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

    const w = mount(TextOptions)
    // Trigger load via the store directly (simulates onMounted completing)
    const fontsStore = useFontsStore()
    await fontsStore.load()
    await w.vm.$nextTick()

    const select = w.find('[data-role="font-select"]').element as HTMLSelectElement
    expect(select.value).toBe('custom-font')
  })
})
