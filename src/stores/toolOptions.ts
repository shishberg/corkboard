import { defineStore } from 'pinia'
import type { EpaperColour } from './types'

const KEY = 'corkboard.toolOptions'

interface ToolOptionsState {
  calendarVariant: 'date' | 'agenda'
  colour: EpaperColour
  // Outline (halo) colour for new text/calendar elements. undefined = no outline.
  outline?: EpaperColour
  penSize: number
  feedId: string
  font: string
  align: 'left' | 'center'
  daysAhead: number
}

const defaults: ToolOptionsState = {
  calendarVariant: 'agenda',
  colour: 'black',
  outline: undefined,
  penSize: 4,
  feedId: '',
  font: 'atkinson-hyperlegible',
  align: 'left',
  daysAhead: 7,
}

function load(): ToolOptionsState {
  try {
    const raw = localStorage.getItem(KEY)
    if (!raw) return { ...defaults }
    const parsed = JSON.parse(raw) as Record<string, unknown>
    // Migrate legacy key: drawColour → colour
    const colour = (parsed.colour ?? parsed.drawColour ?? defaults.colour) as ToolOptionsState['colour']
    // Migrate legacy calendar variants: 'week' → 'agenda', and the now-removed
    // 'today' folds into 'agenda' too.
    const rawVariant = parsed.calendarVariant ?? defaults.calendarVariant
    const calendarVariant = (rawVariant === 'week' || rawVariant === 'today' ? 'agenda' : rawVariant) as ToolOptionsState['calendarVariant']
    // Whitelist only known keys; unknown persisted keys (e.g. stale clockVariant) are dropped
    return {
      calendarVariant,
      colour,
      outline: (parsed.outline ?? defaults.outline) as ToolOptionsState['outline'],
      penSize: (parsed.penSize ?? defaults.penSize) as ToolOptionsState['penSize'],
      feedId: (parsed.feedId ?? defaults.feedId) as ToolOptionsState['feedId'],
      font: (parsed.font ?? defaults.font) as ToolOptionsState['font'],
      align: (parsed.align ?? defaults.align) as ToolOptionsState['align'],
      daysAhead: (parsed.daysAhead ?? defaults.daysAhead) as ToolOptionsState['daysAhead'],
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
