import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import AlignOptions from './AlignOptions.vue'
import { usePagesStore } from '@/stores/pages'
import { useToolOptionsStore } from '@/stores/toolOptions'

function textEl(id: string) {
  return { id, type: 'text' as const, x: 0, y: 0, w: 240, h: 80, colour: 'black' as const, text: 'Text', font: 'atkinson-hyperlegible', align: 'left' as const }
}

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

describe('AlignOptions', () => {
  it('renders left and center as icon buttons (no text labels)', () => {
    const w = mount(AlignOptions)
    const left = w.get('[data-role="align-left"]')
    const center = w.get('[data-role="align-center"]')
    expect(left.text()).toBe('')
    expect(center.text()).toBe('')
    expect(left.find('svg').exists()).toBe(true)
    expect(center.find('svg').exists()).toBe(true)
  })

  it('clicking an align button sets the tool default when nothing is selected', async () => {
    const opts = useToolOptionsStore()
    const w = mount(AlignOptions)
    await w.get('[data-role="align-center"]').trigger('click')
    expect(opts.align).toBe('center')
  })

  it('shows and edits the selected text element alignment', async () => {
    const store = usePagesStore()
    store.addElement({ ...textEl('t1'), align: 'center' })
    const w = mount(AlignOptions)
    // reflects the element's alignment
    expect(w.get('[data-role="align-center"]').classes().join(' ')).toContain('ring')
    await w.get('[data-role="align-left"]').trigger('click')
    expect((store.selectedPage?.elements[0] as { align: string }).align).toBe('left')
  })
})
