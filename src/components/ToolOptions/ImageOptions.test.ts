import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import ImageOptions from './ImageOptions.vue'
import { useToolOptionsStore } from '@/stores/toolOptions'
import { usePagesStore } from '@/stores/pages'
import * as deviceApi from '@/lib/deviceApi'
import type { ImageEl } from '@/stores/types'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})
afterEach(() => vi.restoreAllMocks())

function selectFile(w: ReturnType<typeof mount>) {
  const input = w.get('[data-role="image-file"]')
  const file = new File([new Uint8Array([1, 2, 3])], 'pic.png', { type: 'image/png' })
  Object.defineProperty(input.element, 'files', { value: [file], configurable: true })
  return input.trigger('change')
}

describe('ImageOptions', () => {
  it('uploads the chosen file and stores the returned id as the pending image', async () => {
    const spy = vi.spyOn(deviceApi, 'uploadImage').mockResolvedValue('img-new')
    const opts = useToolOptionsStore()
    const w = mount(ImageOptions)
    await selectFile(w)
    await w.vm.$nextTick()
    expect(spy).toHaveBeenCalledOnce()
    expect(opts.imageId).toBe('img-new')
  })

  it('applies the uploaded image to the selected image element', async () => {
    vi.spyOn(deviceApi, 'uploadImage').mockResolvedValue('img-new')
    const store = usePagesStore()
    const el: ImageEl = { id: 'i1', type: 'image', x: 0, y: 0, w: 200, h: 150, colour: 'black', src: '' }
    store.addElement(el)
    store.selectElement('i1')
    const w = mount(ImageOptions)
    await selectFile(w)
    await w.vm.$nextTick()
    expect((store.selectedPage?.elements[0] as ImageEl).src).toBe('img-new')
  })

  it('shows an offline message when the upload fails', async () => {
    vi.spyOn(deviceApi, 'uploadImage').mockResolvedValue(null)
    const opts = useToolOptionsStore()
    const w = mount(ImageOptions)
    await selectFile(w)
    await w.vm.$nextTick()
    expect(opts.imageId).toBe('')
    expect(w.find('[data-role="image-error"]').exists()).toBe(true)
  })
})
