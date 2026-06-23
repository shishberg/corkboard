import { describe, it, expect, vi } from 'vitest'
import { useDraggableResizable } from './useDraggableResizable'

function pointer(type: string, x: number, y: number): PointerEvent {
  const e = new Event(type) as any
  e.clientX = x
  e.clientY = y
  e.pointerId = 1
  e.preventDefault = vi.fn()
  e.stopPropagation = vi.fn()
  Object.defineProperty(e, 'target', { value: { setPointerCapture: vi.fn(), releasePointerCapture: vi.fn() }, configurable: true })
  return e as PointerEvent
}

describe('useDraggableResizable', () => {
  it('translates a drag into an updated position', () => {
    let rect = { x: 10, y: 10, w: 100, h: 50 }
    const onUpdate = vi.fn((r) => (rect = r))
    const dr = useDraggableResizable({
      getRect: () => rect,
      onUpdate,
      scale: () => 1,
    })

    dr.startDrag(pointer('pointerdown', 0, 0))
    window.dispatchEvent(pointer('pointermove', 20, 30))
    window.dispatchEvent(pointer('pointerup', 20, 30))

    expect(onUpdate).toHaveBeenCalled()
    expect(rect).toEqual({ x: 30, y: 40, w: 100, h: 50 })
  })

  it('divides movement by scale', () => {
    let rect = { x: 0, y: 0, w: 100, h: 50 }
    const onUpdate = vi.fn((r) => (rect = r))
    const dr = useDraggableResizable({ getRect: () => rect, onUpdate, scale: () => 0.5 })

    dr.startDrag(pointer('pointerdown', 0, 0))
    window.dispatchEvent(pointer('pointermove', 50, 0))
    window.dispatchEvent(pointer('pointerup', 50, 0))

    expect(rect.x).toBe(100) // 50px on screen / 0.5 scale = 100 logical px
  })

  it('resizes from the bottom-right handle', () => {
    let rect = { x: 0, y: 0, w: 100, h: 50 }
    const onUpdate = vi.fn((r) => (rect = r))
    const dr = useDraggableResizable({ getRect: () => rect, onUpdate, scale: () => 1 })

    dr.startResize(pointer('pointerdown', 100, 50))
    window.dispatchEvent(pointer('pointermove', 130, 90))
    window.dispatchEvent(pointer('pointerup', 130, 90))

    expect(rect).toEqual({ x: 0, y: 0, w: 130, h: 90 })
  })
})
