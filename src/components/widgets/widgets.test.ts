import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ClockWidget from './ClockWidget.vue'
import CalendarWidget from './CalendarWidget.vue'
import ImageWidget from './ImageWidget.vue'
import DrawingWidget from './DrawingWidget.vue'
import { makeDrawingElement } from '@/stores/elementFactory'
import type { ClockEl, CalendarEl, ImageEl } from '@/stores/types'

describe('widgets', () => {
  it('ClockWidget shows a date line for the date variant', () => {
    const el: ClockEl = { id: 'c', type: 'clock', variant: 'date', x: 0, y: 0, w: 200, h: 80, colour: 'black' }
    const w = mount(ClockWidget, { props: { el } })
    expect(w.find('[data-role="date"]').exists()).toBe(true)
  })

  it('ClockWidget time font scales with element height', () => {
    // Wide elements so height is the binding dimension (width also caps the font).
    const small: ClockEl = { id: 'c', type: 'clock', variant: 'time', x: 0, y: 0, w: 2000, h: 100, colour: 'black' }
    const big: ClockEl = { ...small, h: 200 }
    const fontPx = (el: ClockEl) => {
      const w = mount(ClockWidget, { props: { el } })
      const style = w.get('[data-role="time"]').attributes('style') ?? ''
      return parseFloat(style.match(/font-size:\s*([\d.]+)px/)?.[1] ?? '0')
    }
    const a = fontPx(small)
    const b = fontPx(big)
    expect(a).toBeGreaterThan(0)
    expect(b).toBeCloseTo(a * 2, 1)
  })

  it('ClockWidget applies element colour as text colour', () => {
    const el: ClockEl = { id: 'c', type: 'clock', variant: 'time', x: 0, y: 0, w: 200, h: 80, colour: 'red' }
    const w = mount(ClockWidget, { props: { el } })
    const style = w.get('[data-role="time"]').attributes('style') ?? ''
    expect(style).toContain('color: red')
  })

  it('CalendarWidget renders a week layout with 7 day cells', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'week', x: 0, y: 0, w: 300, h: 200, events: [], colour: 'black' }
    const w = mount(CalendarWidget, { props: { el } })
    expect(w.findAll('[data-role="day"]').length).toBe(7)
  })

  it('CalendarWidget applies element colour as text colour', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'today', x: 0, y: 0, w: 300, h: 200, events: [], colour: 'blue' }
    const w = mount(CalendarWidget, { props: { el } })
    const style = w.find('[data-role="calendar-root"]').attributes('style') ?? ''
    expect(style).toContain('color: blue')
  })

  it('ImageWidget shows a placeholder when src is empty', () => {
    const el: ImageEl = { id: 'img', type: 'image', src: '', x: 0, y: 0, w: 200, h: 150, colour: 'black' }
    const w = mount(ImageWidget, { props: { el } })
    expect(w.find('[data-role="placeholder"]').exists()).toBe(true)
  })

  it('DrawingWidget viewBox uses natW/natH, not resized w/h', () => {
    const el = makeDrawingElement([{ x: 50, y: 60 }, { x: 90, y: 110 }], 'black', 4)
    // Simulate a resize — w and h change, natW/natH must not
    el.w = 200
    el.h = 300
    const w = mount(DrawingWidget, { props: { el } })
    const svg = w.find('[data-role="drawing"]')
    expect(svg.attributes('viewBox')).toBe(`0 0 ${el.natW} ${el.natH}`)
  })

  it('DrawingWidget renders a <path> with a non-empty d attribute', () => {
    const el = makeDrawingElement(
      [{ x: 10, y: 10 }, { x: 30, y: 20 }, { x: 50, y: 10 }],
      'black',
      4,
    )
    const w = mount(DrawingWidget, { props: { el } })
    const path = w.find('[data-role="drawing"] path')
    expect(path.exists()).toBe(true)
    expect(path.attributes('d')).toBeTruthy()
  })
})
