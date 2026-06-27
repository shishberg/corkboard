import type { Feed } from '@/stores/feeds'

/** GET /api/feeds — returns parsed Feed array, or null on any network/parse failure. */
export async function fetchFeeds(): Promise<Feed[] | null> {
  try {
    const res = await fetch('/api/feeds')
    if (!res.ok) return null
    return (await res.json()) as Feed[]
  } catch {
    return null
  }
}

/** POST /api/refresh — returns true on 2xx, false on any error. */
export async function refreshNow(): Promise<boolean> {
  try {
    const res = await fetch('/api/refresh', { method: 'POST' })
    return res.ok
  } catch {
    return false
  }
}
