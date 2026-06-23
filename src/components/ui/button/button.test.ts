import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import { Button } from './index'

describe('Button', () => {
  it('renders its slot text', () => {
    const wrapper = mount(Button, { slots: { default: 'Publish' } })
    expect(wrapper.text()).toContain('Publish')
  })
})
