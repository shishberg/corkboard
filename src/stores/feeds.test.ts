import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useFeedsStore } from './feeds'
import * as deviceApi from '@/lib/deviceApi'

beforeEach(() => {
  setActivePinia(createPinia())
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('useFeedsStore', () => {
  it('has a default stub feed list', () => {
    const store = useFeedsStore()
    expect(store.feeds).toEqual([{ id: 'family', name: 'Family' }])
  })

  it('replaces feeds when loadFeeds succeeds', async () => {
    const newFeeds = [{ id: 'work', name: 'Work' }, { id: 'family', name: 'Family' }]
    vi.spyOn(deviceApi, 'fetchFeeds').mockResolvedValue(newFeeds)
    const store = useFeedsStore()
    await store.loadFeeds()
    expect(store.feeds).toEqual(newFeeds)
  })

  it('keeps stub feeds when loadFeeds returns null (device unreachable)', async () => {
    vi.spyOn(deviceApi, 'fetchFeeds').mockResolvedValue(null)
    const store = useFeedsStore()
    await store.loadFeeds()
    expect(store.feeds).toEqual([{ id: 'family', name: 'Family' }])
  })
})
