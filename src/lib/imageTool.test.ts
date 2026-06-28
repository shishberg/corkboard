import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { addImageFromFile } from './imageTool'
import { usePagesStore } from '@/stores/pages'
import * as deviceApi from '@/lib/deviceApi'

// Stub the browser Image so naturalSize resolves deterministically in jsdom.
class FakeImage {
  onload: () => void = () => {}
  onerror: () => void = () => {}
  naturalWidth = 400
  naturalHeight = 200
  set src(_v: string) {
    // Resolve on the next tick, like a real load.
    setTimeout(() => this.onload(), 0)
  }
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.stubGlobal('Image', FakeImage as unknown as typeof Image)
})
afterEach(() => vi.restoreAllMocks())

function pngFile() {
  return new File([new Uint8Array([1, 2, 3])], 'pic.png', { type: 'image/png' })
}

describe('addImageFromFile', () => {
  it('uploads, then adds a centred image element with the returned id', async () => {
    vi.spyOn(deviceApi, 'uploadImage').mockResolvedValue('img-1')
    const store = usePagesStore()

    const id = await addImageFromFile(pngFile())

    expect(id).not.toBeNull()
    const el = store.selectedPage!.elements.at(-1)!
    expect(el.type).toBe('image')
    expect((el as { src: string }).src).toBe('img-1')
    // 400×200 image → 2:1, fits to 400×200 on an 800×480 page, centred.
    expect(el.w / el.h).toBeCloseTo(2)
    expect(el.x).toBeCloseTo((800 - el.w) / 2)
    // The new element is selected and the select tool is active.
    expect(store.selectedElId).toBe(el.id)
    expect(store.activeTool).toBe('select')
  })

  it('does nothing and returns null when the upload fails', async () => {
    vi.spyOn(deviceApi, 'uploadImage').mockResolvedValue(null)
    const store = usePagesStore()

    const id = await addImageFromFile(pngFile())

    expect(id).toBeNull()
    expect(store.selectedPage!.elements).toHaveLength(0)
  })
})
