import { describe, it, expect, vi, afterEach } from 'vitest'
import { fetchFeeds, refreshNow } from './deviceApi'

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
