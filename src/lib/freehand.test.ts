import { describe, it, expect } from 'vitest'
import { strokeToPath } from './freehand'

describe('strokeToPath', () => {
  it('returns empty string for zero points', () => {
    expect(strokeToPath([], 4)).toBe('')
  })

  it('returns a non-empty path for a single point (tap → dot)', () => {
    const d = strokeToPath([{ x: 50, y: 50 }], 4)
    expect(d).not.toBe('')
    expect(d.length).toBeGreaterThan(0)
  })

  it('returns a non-empty path for two or more points', () => {
    const d = strokeToPath([{ x: 0, y: 0 }, { x: 10, y: 10 }], 4)
    expect(d).not.toBe('')
  })
})
