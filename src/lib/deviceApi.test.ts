import { describe, it, expect, vi, afterEach } from 'vitest'
import { fetchFeeds, refreshNow, getDocument, putDocument, uploadImage } from './deviceApi'
import type { DocState } from '@/stores/types'

afterEach(() => {
  vi.restoreAllMocks()
})

describe('fetchFeeds', () => {
  it('returns parsed feeds on 200 ok', async () => {
    const feeds = [{ id: 'family', name: 'Family' }]
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => feeds,
    } as unknown as Response)
    const result = await fetchFeeds()
    expect(result).toEqual(feeds)
    expect(global.fetch).toHaveBeenCalledWith('/api/feeds')
  })

  it('returns null on non-ok response', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 503,
    } as unknown as Response)
    expect(await fetchFeeds()).toBeNull()
  })

  it('returns null when fetch throws', async () => {
    global.fetch = vi.fn().mockRejectedValue(new Error('Network error'))
    expect(await fetchFeeds()).toBeNull()
  })
})

describe('refreshNow', () => {
  it('returns true on 200 ok', async () => {
    global.fetch = vi.fn().mockResolvedValue({ ok: true } as unknown as Response)
    expect(await refreshNow()).toBe(true)
    expect(global.fetch).toHaveBeenCalledWith('/api/refresh', { method: 'POST' })
  })

  it('returns false on non-ok response', async () => {
    global.fetch = vi.fn().mockResolvedValue({ ok: false } as unknown as Response)
    expect(await refreshNow()).toBe(false)
  })

  it('returns false when fetch throws', async () => {
    global.fetch = vi.fn().mockRejectedValue(new Error('offline'))
    expect(await refreshNow()).toBe(false)
  })
})

describe('getDocument', () => {
  const doc: DocState = {
    orientation: 'landscape',
    pages: [{ id: 'p1', name: 'Page', elements: [] }],
    livePageId: 'p1',
    selectedPageId: 'p1',
    selectedElId: null,
    activeTool: 'select',
  }

  it('returns parsed DocState on 200 ok', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => doc,
    } as unknown as Response)
    const result = await getDocument()
    expect(result).toEqual(doc)
    expect(global.fetch).toHaveBeenCalledWith('/api/document')
  })

  it('returns null on non-ok response', async () => {
    global.fetch = vi.fn().mockResolvedValue({ ok: false, status: 503 } as unknown as Response)
    expect(await getDocument()).toBeNull()
  })

  it('returns null when fetch throws', async () => {
    global.fetch = vi.fn().mockRejectedValue(new Error('offline'))
    expect(await getDocument()).toBeNull()
  })
})

describe('putDocument', () => {
  const doc: DocState = {
    orientation: 'portrait',
    pages: [{ id: 'p1', name: 'Page', elements: [] }],
    livePageId: 'p1',
    selectedPageId: 'p1',
    selectedElId: null,
    activeTool: 'select',
  }

  it('returns true on 2xx response', async () => {
    global.fetch = vi.fn().mockResolvedValue({ ok: true } as unknown as Response)
    const result = await putDocument(doc)
    expect(result).toBe(true)
    expect(global.fetch).toHaveBeenCalledWith('/api/document', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(doc),
    })
  })

  it('returns false on non-ok response', async () => {
    global.fetch = vi.fn().mockResolvedValue({ ok: false, status: 503 } as unknown as Response)
    expect(await putDocument(doc)).toBe(false)
  })

  it('returns false when fetch throws', async () => {
    global.fetch = vi.fn().mockRejectedValue(new Error('offline'))
    expect(await putDocument(doc)).toBe(false)
  })
})

describe('uploadImage', () => {
  const file = new Blob([new Uint8Array([1, 2, 3])], { type: 'image/png' })

  it('POSTs the file to /api/images and returns the new id', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ id: 'img-abc123' }),
    } as unknown as Response)
    global.fetch = fetchMock
    const id = await uploadImage(file)
    expect(id).toBe('img-abc123')
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe('/api/images')
    expect(init.method).toBe('POST')
    expect(init.body).toBe(file)
  })

  it('returns null on non-ok response', async () => {
    global.fetch = vi.fn().mockResolvedValue({ ok: false, status: 500 } as unknown as Response)
    expect(await uploadImage(file)).toBeNull()
  })

  it('returns null when fetch throws', async () => {
    global.fetch = vi.fn().mockRejectedValue(new Error('offline'))
    expect(await uploadImage(file)).toBeNull()
  })

  it('returns null when the response has no id', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({}),
    } as unknown as Response)
    expect(await uploadImage(file)).toBeNull()
  })
})
