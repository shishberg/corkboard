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
    expect(s.clockVariant).toBe('time')
    expect(s.calendarVariant).toBe('today')
    expect(s.colour).toBe('black')
    expect(s.penSize).toBe(4)
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
})
