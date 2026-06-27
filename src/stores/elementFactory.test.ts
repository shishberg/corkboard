import { describe, it, expect } from 'vitest'
import { makeElement, makeDrawingElement } from './elementFactory'

const opts = { clockVariant: 'time-date' as const, calendarVariant: 'week' as const, colour: 'red' as const }
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

  it('uses an explicit rect when one is given (draw-to-place)', () => {
    const el = makeElement('image', opts, size, { x: 10, y: 20, w: 120, h: 90 })
    expect(el.x).toBe(10)
    expect(el.y).toBe(20)
    expect(el.w).toBe(120)
    expect(el.h).toBe(90)
  })

  it('carries the colour from opts onto the element', () => {
    const el = makeElement('clock', opts, size)
    expect(el.colour).toBe('red')
  })
})

describe('makeDrawingElement', () => {
  it('bounds the stroke and stores points relative to the element', () => {
    const el = makeDrawingElement([{ x: 50, y: 60 }, { x: 90, y: 110 }], 'red', 4)
    expect(el.type).toBe('drawing')
    // bbox is padded by the stroke size on every side
    expect(el.x).toBe(46)
    expect(el.y).toBe(56)
    expect(el.w).toBe(48) // (90-50) + 2*4
    expect(el.h).toBe(58) // (110-60) + 2*4
    expect(el.strokes[0].colour).toBe('red')
    expect(el.strokes[0].size).toBe(4)
    // first point sits at (50-46, 60-56) = (4, 4) in element-local space
    expect(el.strokes[0].points[0]).toEqual({ x: 4, y: 4 })
  })

  it('sets element-level colour from the colour argument', () => {
    const el = makeDrawingElement([{ x: 10, y: 10 }, { x: 20, y: 20 }], 'blue', 2)
    expect(el.colour).toBe('blue')
  })

  it('sets natW and natH equal to w and h at creation', () => {
    const el = makeDrawingElement([{ x: 50, y: 60 }, { x: 90, y: 110 }], 'red', 4)
    expect(el.natW).toBe(el.w)
    expect(el.natH).toBe(el.h)
  })
})
