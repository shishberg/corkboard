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

  it('locks resize to the given aspect ratio', () => {
    let rect = { x: 0, y: 0, w: 100, h: 50 } // 2:1, but aspect lock overrides
    const onUpdate = vi.fn((r) => (rect = r))
    const dr = useDraggableResizable({
      getRect: () => rect,
      onUpdate,
      scale: () => 1,
      aspect: () => 2, // width = 2 × height
    })

    dr.startResize(pointer('pointerdown', 100, 50))
    // Drag width +60 (→160) and height +40 (height is ignored under lock).
    window.dispatchEvent(pointer('pointermove', 160, 90))
    window.dispatchEvent(pointer('pointerup', 160, 90))

    expect(rect.w).toBe(160)
    expect(rect.h).toBe(80) // 160 / 2, derived from width — not 90
  })

  it('respects the minimum size while keeping aspect', () => {
    let rect = { x: 0, y: 0, w: 100, h: 50 }
    const onUpdate = vi.fn((r) => (rect = r))
    const dr = useDraggableResizable({
      getRect: () => rect,
      onUpdate,
      scale: () => 1,
      aspect: () => 4, // wide: width = 4 × height
    })

    // Shrink width to the minimum; the derived height (20/4 = 5) would fall
    // below MIN, so the box is pushed back up to keep both ≥ MIN at the ratio.
    dr.startResize(pointer('pointerdown', 100, 50))
    window.dispatchEvent(pointer('pointermove', 0, 0))
    window.dispatchEvent(pointer('pointerup', 0, 0))

    expect(rect.h).toBe(20) // MIN
    expect(rect.w).toBe(80) // MIN * 4
  })
})
