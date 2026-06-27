import type { Feed } from '@/stores/feeds'
import type { DocState } from '@/stores/types'

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

/** GET /api/document — returns parsed DocState, or null on any failure. */
export async function getDocument(): Promise<DocState | null> {
  try {
    const res = await fetch('/api/document')
    if (!res.ok) return null
    return (await res.json()) as DocState
  } catch {
    return null
  }
}

/** PUT /api/document — returns true on 2xx, false on any error. */
export async function putDocument(doc: DocState): Promise<boolean> {
  try {
    const res = await fetch('/api/document', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(doc),
    })
    return res.ok
  } catch {
    return false
  }
}
