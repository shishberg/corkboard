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
    expect(s.drawColour).toBe('black')
    expect(s.penSize).toBe(4)
  })

  it('persists changes to localStorage', () => {
    const s = ensureToolOptionsPersistence()
    s.drawColour = 'red'
    const saved = JSON.parse(localStorage.getItem('corkboard.toolOptions') || '{}')
    expect(saved.drawColour).toBe('red')
  })

  it('restores from localStorage on init', () => {
    localStorage.setItem('corkboard.toolOptions', JSON.stringify({ penSize: 12 }))
    setActivePinia(createPinia())
    const s = useToolOptionsStore()
    expect(s.penSize).toBe(12)
  })
})
