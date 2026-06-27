import { getStroke } from 'perfect-freehand'

export interface Point {
  x: number
  y: number
}

/** Convert a pressure-free point array to a filled SVG path `d` string using perfect-freehand. */
export function strokeToPath(points: Point[], size: number): string {
  if (points.length < 2) return ''

  const outline = getStroke(
    points.map((p) => [p.x, p.y]),
    {
      size,
      thinning: 0.5,
      smoothing: 0.5,
      streamline: 0.5,
      simulatePressure: true,
    },
  )

  if (outline.length < 2) return ''

  const d = outline.reduce<string>((acc, [x, y], i) => {
    if (i === 0) return `M ${x} ${y}`
    return `${acc} L ${x} ${y}`
  }, '')

  return `${d} Z`
}
