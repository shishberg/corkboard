import { defineStore } from 'pinia'
import type { EpaperColour } from './types'

const KEY = 'corkboard.toolOptions'

interface ToolOptionsState {
  calendarVariant: 'date' | 'today' | 'agenda'
  colour: EpaperColour
  penSize: number
  feedId: string
  font: string
  align: 'left' | 'center'
}

const defaults: ToolOptionsState = {
  calendarVariant: 'today',
  colour: 'black',
  penSize: 4,
  feedId: '',
  font: 'atkinson-hyperlegible',
  align: 'left',
}

function load(): ToolOptionsState {
  try {
    const raw = localStorage.getItem(KEY)
    if (!raw) return { ...defaults }
    const parsed = JSON.parse(raw) as Record<string, unknown>
    // Migrate legacy key: drawColour → colour
    const colour = (parsed.colour ?? parsed.drawColour ?? defaults.colour) as ToolOptionsState['colour']
    // Migrate legacy calendar variant: 'week' was renamed to 'agenda'.
    const rawVariant = parsed.calendarVariant ?? defaults.calendarVariant
    const calendarVariant = (rawVariant === 'week' ? 'agenda' : rawVariant) as ToolOptionsState['calendarVariant']
    // Whitelist only known keys; unknown persisted keys (e.g. stale clockVariant) are dropped
    return {
      calendarVariant,
      colour,
      penSize: (parsed.penSize ?? defaults.penSize) as ToolOptionsState['penSize'],
      feedId: (parsed.feedId ?? defaults.feedId) as ToolOptionsState['feedId'],
      font: (parsed.font ?? defaults.font) as ToolOptionsState['font'],
      align: (parsed.align ?? defaults.align) as ToolOptionsState['align'],
    }
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
