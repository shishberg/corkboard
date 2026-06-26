import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ClockWidget from './ClockWidget.vue'
import CalendarWidget from './CalendarWidget.vue'
import ImageWidget from './ImageWidget.vue'
import type { ClockEl, CalendarEl, ImageEl } from '@/stores/types'

describe('widgets', () => {
  it('ClockWidget shows a date line for the date variant', () => {
    const el: ClockEl = { id: 'c', type: 'clock', variant: 'date', x: 0, y: 0, w: 200, h: 80 }
    const w = mount(ClockWidget, { props: { el } })
    expect(w.find('[data-role="date"]').exists()).toBe(true)
  })

  it('ClockWidget time font scales with element height', () => {
    // Wide elements so height is the binding dimension (width also caps the font).
    const small: ClockEl = { id: 'c', type: 'clock', variant: 'time', x: 0, y: 0, w: 2000, h: 100 }
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

  it('CalendarWidget renders a week layout with 7 day cells', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'week', x: 0, y: 0, w: 300, h: 200, events: [] }
    const w = mount(CalendarWidget, { props: { el } })
    expect(w.findAll('[data-role="day"]').length).toBe(7)
  })

  it('ImageWidget shows a placeholder when src is empty', () => {
    const el: ImageEl = { id: 'img', type: 'image', src: '', x: 0, y: 0, w: 200, h: 150 }
    const w = mount(ImageWidget, { props: { el } })
    expect(w.find('[data-role="placeholder"]').exists()).toBe(true)
  })
})
