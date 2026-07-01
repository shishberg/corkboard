import type { Feed } from '@/stores/feeds'
import type { DocState } from '@/stores/types'
import type { DashboardStatus } from '@/lib/dashboardTypes'

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

/** The same-origin URL the device serves a stored image at, by its id. */
export function imageUrl(id: string): string {
  return `/api/images/${id}`
}

/**
 * POST /api/images — uploads raw image bytes; returns the device-assigned image
 * id (`img-<uuid>`) to store as an element's `src`, or null on any failure.
 * The Blob's type sets the request Content-Type; the device sniffs the real
 * format from the bytes on read.
 */
export async function uploadImage(file: Blob): Promise<string | null> {
  try {
    const res = await fetch('/api/images', { method: 'POST', body: file })
    if (!res.ok) return null
    const data = (await res.json()) as { id?: string }
    return typeof data.id === 'string' ? data.id : null
  } catch {
    return null
  }
}

/** GET /api/status — device health snapshot for the dashboard, or null on any failure. */
export async function fetchStatus(): Promise<DashboardStatus | null> {
  try {
    const res = await fetch('/api/status')
    if (!res.ok) return null
    return (await res.json()) as DashboardStatus
  } catch {
    return null
  }
}
