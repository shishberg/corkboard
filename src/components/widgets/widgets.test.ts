import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import CalendarWidget from './CalendarWidget.vue'
import ImageWidget from './ImageWidget.vue'
import DrawingWidget from './DrawingWidget.vue'
import { makeDrawingElement } from '@/stores/elementFactory'
import { formatSampleDate } from '@/lib/sampleCalendar'
import type { CalendarEl, ImageEl } from '@/stores/types'

describe('widgets', () => {
  it('CalendarWidget renders a week layout with 7 day cells', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'week', x: 0, y: 0, w: 300, h: 200, feedId: '', colour: 'black' }
    const w = mount(CalendarWidget, { props: { el } })
    expect(w.findAll('[data-role="day"]').length).toBe(7)
  })

  it('CalendarWidget applies element colour as text colour', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'today', x: 0, y: 0, w: 300, h: 200, feedId: '', colour: 'blue' }
    const w = mount(CalendarWidget, { props: { el } })
    const style = w.find('[data-role="calendar-root"]').attributes('style') ?? ''
    expect(style).toContain('color: blue')
  })

  it('CalendarWidget today variant shows 3 sample events', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'today', x: 0, y: 0, w: 300, h: 200, feedId: '', colour: 'black' }
    const w = mount(CalendarWidget, { props: { el } })
    expect(w.findAll('[data-role="event"]').length).toBe(3)
  })

  it('CalendarWidget date variant shows the formatted sample date', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'date', x: 0, y: 0, w: 300, h: 200, feedId: '', colour: 'black' }
    const w = mount(CalendarWidget, { props: { el } })
    const dateEl = w.find('[data-role="calendar-date"]')
    expect(dateEl.exists()).toBe(true)
    expect(dateEl.text()).toContain(formatSampleDate())
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
