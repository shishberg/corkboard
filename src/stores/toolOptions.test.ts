import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useToolOptionsStore, ensureToolOptionsPersistence } from './toolOptions'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

describe('useToolOptionsStore', () => {
  it('has sensible defaults', () => {
    const s = useToolOptionsStore()
    expect(s.calendarVariant).toBe('agenda')
    expect(s.colour).toBe('black')
    expect(s.penSize).toBe(4)
    expect(s.daysAhead).toBe(7)
  })

  it('persists changes to localStorage', () => {
    const s = ensureToolOptionsPersistence()
    s.colour = 'red'
    const saved = JSON.parse(localStorage.getItem('corkboard.toolOptions') || '{}')
    expect(saved.colour).toBe('red')
  })

  it('restores from localStorage on init', () => {
    localStorage.setItem('corkboard.toolOptions', JSON.stringify({ penSize: 12 }))
    setActivePinia(createPinia())
    const s = useToolOptionsStore()
    expect(s.penSize).toBe(12)
  })

  it('migrates legacy drawColour key to colour on load', () => {
    localStorage.setItem('corkboard.toolOptions', JSON.stringify({ drawColour: 'red' }))
    setActivePinia(createPinia())
    const s = useToolOptionsStore()
    expect(s.colour).toBe('red')
  })

  it('colour takes precedence over drawColour when both are present', () => {
    localStorage.setItem('corkboard.toolOptions', JSON.stringify({ colour: 'blue', drawColour: 'red' }))
    setActivePinia(createPinia())
    const s = useToolOptionsStore()
    expect(s.colour).toBe('blue')
  })

  it('does not carry stale unknown keys (e.g. clockVariant) from persisted blob', () => {
    localStorage.setItem(
      'corkboard.toolOptions',
      JSON.stringify({ calendarVariant: 'week', colour: 'red', penSize: 8, clockVariant: 'time' }),
    )
    setActivePinia(createPinia())
    const s = useToolOptionsStore()
    expect('clockVariant' in s.$state).toBe(false)
    // Legacy 'week' variant migrates to 'agenda' on load.
    expect(s.calendarVariant).toBe('agenda')
    expect(s.colour).toBe('red')
    expect(s.penSize).toBe(8)
  })
})
