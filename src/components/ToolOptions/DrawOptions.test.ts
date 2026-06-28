import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import DrawOptions from './DrawOptions.vue'
import { useToolOptionsStore } from '@/stores/toolOptions'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

describe('DrawOptions', () => {
  it('renders one circle button per pen size', () => {
    const w = mount(DrawOptions)
    expect(w.findAll('[data-pen-size]').length).toBe(4)
    // Each button shows a circle, not a number.
    expect(w.findAll('[data-role="pen-dot"]').length).toBe(4)
  })

  it('circles get larger as the pen size grows', () => {
    const w = mount(DrawOptions)
    const widths = w.findAll('[data-role="pen-dot"]').map((d) => {
      const m = (d.attributes('style') ?? '').match(/width:\s*([\d.]+)px/)
      return m ? parseFloat(m[1]) : 0
    })
    const sorted = [...widths].sort((a, b) => a - b)
    expect(widths).toEqual(sorted)
    expect(widths[0]).toBeLessThan(widths[widths.length - 1])
  })

  it('clicking a circle sets the pen size', async () => {
    const opts = useToolOptionsStore()
    const w = mount(DrawOptions)
    await w.get('[data-pen-size="8"]').trigger('click')
    expect(opts.penSize).toBe(8)
  })

  it('marks the active pen size', async () => {
    const opts = useToolOptionsStore()
    opts.penSize = 12
    const w = mount(DrawOptions)
    const active = w.get('[data-pen-size="12"]')
    expect(active.classes().join(' ')).toContain('ring')
  })
})
