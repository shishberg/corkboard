import { defineStore } from 'pinia'
import type { EpaperColour } from './types'

const KEY = 'corkboard.toolOptions'

interface ToolOptionsState {
  clockVariant: 'time' | 'time-date' | 'date'
  calendarVariant: 'today' | 'week'
  colour: EpaperColour
  penSize: number
}

const defaults: ToolOptionsState = {
  clockVariant: 'time',
  calendarVariant: 'today',
  colour: 'black',
  penSize: 4,
}

function load(): ToolOptionsState {
  try {
    const raw = localStorage.getItem(KEY)
    if (!raw) return { ...defaults }
    const parsed = JSON.parse(raw) as Record<string, unknown>
    // Migrate legacy key: drawColour → colour
    const colour = (parsed.colour ?? parsed.drawColour ?? defaults.colour) as ToolOptionsState['colour']
    return { ...defaults, ...parsed, colour }
  } catch {
    return { ...defaults }
  }
}

export const useToolOptionsStore = defineStore('toolOptions', {
  state: (): ToolOptionsState => load(),
})

// Persistence is opt-in per active store instance so it works everywhere,
// including tests. `ensureToolOptionsPersistence()` subscribes once and
// writes the whole state back to localStorage on every change.
const subscribed = new WeakSet<object>()
export function ensureToolOptionsPersistence() {
  const store = useToolOptionsStore()
  if (subscribed.has(store)) return store
  subscribed.add(store)
  store.$subscribe((_m, state) => {
    localStorage.setItem(KEY, JSON.stringify(state))
  }, { flush: 'sync' })
  return store
}
