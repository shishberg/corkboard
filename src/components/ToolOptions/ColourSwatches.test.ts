import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { setActivePinia, createPinia } from 'pinia'
import ColourSwatches from './ColourSwatches.vue'
import { usePagesStore } from '@/stores/pages'
import { useToolOptionsStore } from '@/stores/toolOptions'

function calendarEl(id: string) {
  return { id, type: 'calendar' as const, variant: 'agenda' as const, x: 0, y: 0, w: 200, h: 80, feedId: '', colour: 'black' as const, font: 'atkinson-hyperlegible', align: 'center' as const }
}

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

describe('ColourSwatches', () => {
  it('renders 6 swatches', () => {
    const w = mount(ColourSwatches)
    const panel = w.find('[data-role="colour-panel"]')
    expect(panel.exists()).toBe(true)
    expect(panel.findAll('[data-colour]').length).toBe(6)
  })

  it('clicking a swatch with no selection sets opts.colour', async () => {
    const opts = useToolOptionsStore()
    const w = mount(ColourSwatches)
    await w.get('[data-colour="red"]').trigger('click')
    expect(opts.colour).toBe('red')
  })

  it('clicking a swatch with a selection updates element colour and opts.colour', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('e1'))
    expect(store.selectedElId).toBe('e1')
    const opts = useToolOptionsStore()
    const w = mount(ColourSwatches)
    await w.get('[data-colour="green"]').trigger('click')
    expect(opts.colour).toBe('green')
    expect(store.selectedPage?.elements[0].colour).toBe('green')
  })

  it('highlights the selected element colour when one is selected', async () => {
    const store = usePagesStore()
    store.addElement({ ...calendarEl('e1'), colour: 'blue' })
    const w = mount(ColourSwatches)
    await nextTick()
    expect(w.get('[data-colour="blue"]').classes().join(' ')).toContain('ring')
  })

  it('highlights opts.colour when no element is selected', async () => {
    const opts = useToolOptionsStore()
    opts.colour = 'yellow'
    expect(usePagesStore().selectedElId).toBe(null)
    const w = mount(ColourSwatches)
    await nextTick()
    expect(w.get('[data-colour="yellow"]').classes().join(' ')).toContain('ring')
  })

  it('with the background tool active, a swatch sets the page background, not an element', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('e1')) // selected
    const opts = useToolOptionsStore()
    store.setActiveTool('background')
    const w = mount(ColourSwatches)
    await w.get('[data-colour="red"]').trigger('click')
    expect(store.selectedPage?.background).toBe('red')
    // the selected element keeps its colour; opts.colour is unchanged too
    expect(store.selectedPage?.elements[0].colour).toBe('black')
    expect(opts.colour).not.toBe('red')
  })

  it('with the background tool active, the panel highlights the page background', async () => {
    const store = usePagesStore()
    store.setPageBackground('green')
    store.setActiveTool('background')
    const w = mount(ColourSwatches)
    await nextTick()
    expect(w.get('[data-colour="green"]').classes().join(' ')).toContain('ring')
  })
})
