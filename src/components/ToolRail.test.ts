import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { setActivePinia, createPinia } from 'pinia'
import ToolRail from './ToolRail.vue'
import { usePagesStore } from '@/stores/pages'
import { useToolOptionsStore } from '@/stores/toolOptions'

function calendarEl(id: string) {
  return { id, type: 'calendar' as const, variant: 'agenda' as const, x: 0, y: 0, w: 200, h: 80, feedId: '', colour: 'black' as const }
}

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

describe('ToolRail', () => {
  it('selecting the select tool does not add an element', async () => {
    const store = usePagesStore()
    const w = mount(ToolRail)
    await w.get('[data-tool="select"]').trigger('click')
    expect(store.activeTool).toBe('select')
    expect(store.selectedPage?.elements.length).toBe(0)
  })

  it('the delete button removes the selected element and is disabled otherwise', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('e1'))
    const w = mount(ToolRail)
    const del = w.get('[data-role="delete-element"]')
    await del.trigger('click')
    expect(store.selectedPage?.elements.length).toBe(0)
    // nothing selected now → button disabled
    expect((del.element as HTMLButtonElement).disabled).toBe(true)
  })

  it('bring-to-front moves the selected element to the end of the array', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('a'))
    store.addElement(calendarEl('b'))
    store.selectElement('a')
    const w = mount(ToolRail)
    await w.get('[data-role="bring-to-front"]').trigger('click')
    expect(store.selectedPage?.elements.map((e) => e.id)).toEqual(['b', 'a'])
  })

  it('send-to-back moves the selected element to the start of the array', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('a'))
    store.addElement(calendarEl('b'))
    store.selectElement('b')
    const w = mount(ToolRail)
    await w.get('[data-role="send-to-back"]').trigger('click')
    expect(store.selectedPage?.elements.map((e) => e.id)).toEqual(['b', 'a'])
  })

  it('z-order buttons are disabled when nothing is selected', () => {
    const w = mount(ToolRail)
    expect((w.get('[data-role="bring-to-front"]').element as HTMLButtonElement).disabled).toBe(true)
    expect((w.get('[data-role="send-to-back"]').element as HTMLButtonElement).disabled).toBe(true)
  })

  it('selecting the background tool sets it active', async () => {
    const store = usePagesStore()
    const w = mount(ToolRail)
    await w.get('[data-tool="background"]').trigger('click')
    expect(store.activeTool).toBe('background')
  })

  it('with the background tool active, a swatch sets the page background, not an element', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('e1')) // selected
    const opts = useToolOptionsStore()
    store.setActiveTool('background')
    const w = mount(ToolRail)
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
    const w = mount(ToolRail)
    await nextTick()
    expect(w.get('[data-colour="green"]').classes().join(' ')).toContain('ring')
  })

  it('mounting ToolRail wires tool-option persistence to localStorage', async () => {
    const opts = useToolOptionsStore()
    mount(ToolRail)
    // ToolRail calls ensureToolOptionsPersistence() onMounted; changing an
    // option should now write through to localStorage.
    opts.calendarVariant = 'agenda'
    await nextTick()
    const saved = JSON.parse(localStorage.getItem('corkboard.toolOptions') || '{}')
    expect(saved.calendarVariant).toBe('agenda')
  })

  it('colour panel renders 6 swatches', async () => {
    const w = mount(ToolRail)
    const panel = w.find('[data-role="colour-panel"]')
    expect(panel.exists()).toBe(true)
    expect(panel.findAll('[data-colour]').length).toBe(6)
  })

  it('clicking a swatch with no selection sets opts.colour', async () => {
    const opts = useToolOptionsStore()
    const w = mount(ToolRail)
    await w.get('[data-colour="red"]').trigger('click')
    expect(opts.colour).toBe('red')
  })

  it('clicking a swatch with a selection updates element colour and opts.colour', async () => {
    const store = usePagesStore()
    store.addElement(calendarEl('e1'))
    expect(store.selectedElId).toBe('e1')
    const opts = useToolOptionsStore()
    const w = mount(ToolRail)
    await w.get('[data-colour="green"]').trigger('click')
    expect(opts.colour).toBe('green')
    expect(store.selectedPage?.elements[0].colour).toBe('green')
  })

  it('panel highlights the selected element colour when one is selected', async () => {
    const store = usePagesStore()
    store.addElement({ ...calendarEl('e1'), colour: 'blue' })
    const w = mount(ToolRail)
    await nextTick()
    // The blue swatch should have the ring class indicating it is highlighted
    const blueSwatch = w.get('[data-colour="blue"]')
    expect(blueSwatch.classes().join(' ')).toContain('ring')
  })

  it('panel highlights opts.colour when no element is selected', async () => {
    const opts = useToolOptionsStore()
    opts.colour = 'yellow'
    expect(usePagesStore().selectedElId).toBe(null)
    const w = mount(ToolRail)
    await nextTick()
    const yellowSwatch = w.get('[data-colour="yellow"]')
    expect(yellowSwatch.classes().join(' ')).toContain('ring')
  })
})
