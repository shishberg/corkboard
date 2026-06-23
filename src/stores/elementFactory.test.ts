import { describe, it, expect } from 'vitest'
import { makeElement } from './elementFactory'

const opts = { clockVariant: 'time-date' as const, calendarVariant: 'week' as const }
const size = { w: 800, h: 480 }

describe('makeElement', () => {
  it('makes a centred clock element using the option variant', () => {
    const el = makeElement('clock', opts, size)
    expect(el.type).toBe('clock')
    if (el.type === 'clock') expect(el.variant).toBe('time-date')
    expect(el.x).toBe((800 - el.w) / 2)
    expect(el.y).toBe((480 - el.h) / 2)
    expect(el.id).toBeTruthy()
  })

  it('makes a calendar element using the option variant', () => {
    const el = makeElement('calendar', opts, size)
    expect(el.type).toBe('calendar')
    if (el.type === 'calendar') {
      expect(el.variant).toBe('week')
      expect(el.events).toEqual([])
    }
  })

  it('makes an image element with an empty src', () => {
    const el = makeElement('image', opts, size)
    expect(el.type).toBe('image')
    if (el.type === 'image') expect(el.src).toBe('')
  })
})
