import type { El, DrawingEl, EpaperColour } from './types'

let counter = 0
const uid = () => `el-${Date.now().toString(36)}-${(counter++).toString(36)}`

interface FactoryOpts {
  clockVariant: 'time' | 'time-date' | 'date'
  calendarVariant: 'today' | 'week'
  colour: EpaperColour
}

interface Rect {
  x: number
  y: number
  w: number
  h: number
}

const SIZES = {
  clock: { w: 240, h: 90 },
  calendar: { w: 300, h: 220 },
  image: { w: 200, h: 150 },
}

export function defaultSize(tool: 'clock' | 'calendar' | 'image'): { w: number; h: number } {
  return { ...SIZES[tool] }
}

export function makeElement(
  tool: 'clock' | 'calendar' | 'image',
  opts: FactoryOpts,
  pageSize: { w: number; h: number },
  rect?: Rect,
): El {
  const { w, h } = SIZES[tool]
  const geom: Rect = rect ?? { w, h, x: (pageSize.w - w) / 2, y: (pageSize.h - h) / 2 }
  const base = { id: uid(), ...geom, colour: opts.colour }
  switch (tool) {
    case 'clock':
      return { ...base, type: 'clock', variant: opts.clockVariant }
    case 'calendar':
      return { ...base, type: 'calendar', variant: opts.calendarVariant, events: [] }
    case 'image':
      return { ...base, type: 'image', src: '' }
  }
}

// Build a drawing element from page-logical stroke points. The bounding box is
// padded by the stroke size so thick strokes aren't clipped, and the points are
// stored relative to the box so they move with the element.
export function makeDrawingElement(
  points: { x: number; y: number }[],
  colour: EpaperColour,
  size: number,
): DrawingEl {
  const xs = points.map((p) => p.x)
  const ys = points.map((p) => p.y)
  const minX = Math.min(...xs) - size
  const minY = Math.min(...ys) - size
  const maxX = Math.max(...xs) + size
  const maxY = Math.max(...ys) + size
  const local = points.map((p) => ({ x: p.x - minX, y: p.y - minY }))
  const w = Math.max(1, maxX - minX)
  const h = Math.max(1, maxY - minY)
  return {
    id: uid(),
    type: 'drawing',
    x: minX,
    y: minY,
    w,
    h,
    natW: w,
    natH: h,
    colour,
    strokes: [{ colour, size, points: local }],
  }
}
