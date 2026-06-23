import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import App from './App.vue'

describe('App', () => {
  it('mounts the full shell without throwing', () => {
    setActivePinia(createPinia())
    const wrapper = mount(App, { global: { plugins: [createPinia()] } })
    expect(wrapper.exists()).toBe(true)
  })
})
