import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { setActivePinia, createPinia } from 'pinia'
import ToolRail from './ToolRail.vue'
import { usePagesStore } from '@/stores/pages'
import { useToolOptionsStore } from '@/stores/toolOptions'

function calendarEl(id: string) {
  return { id, type: 'calendar' as const, variant: 'agenda' as const, x: 0, y: 0, w: 200, h: 80, feedId: '', colour: 'black' as const, font: 'atkinson-hyperlegible' }
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

  it('the rail no longer holds tool settings or the colour panel', () => {
    const w = mount(ToolRail)
    expect(w.find('[data-role="colour-panel"]').exists()).toBe(false)
    expect(w.find('[data-role="feed-select"]').exists()).toBe(false)
    expect(w.find('[data-pen-size]').exists()).toBe(false)
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
})
