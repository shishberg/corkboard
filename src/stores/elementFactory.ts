import type { El, DrawingEl, EpaperColour, TextEl } from './types'

let counter = 0
const uid = () => `el-${Date.now().toString(36)}-${(counter++).toString(36)}`

interface FactoryOpts {
  calendarVariant: 'date' | 'today' | 'agenda'
  colour: EpaperColour
  feedId: string
  font: string
  align: 'left' | 'center'
  imageId?: string
}

interface Rect {
  x: number
  y: number
  w: number
  h: number
}

const SIZES = {
  calendar: { w: 300, h: 220 },
  image: { w: 200, h: 150 },
  text: { w: 240, h: 80 },
}

export function defaultSize(tool: 'calendar' | 'image' | 'text'): { w: number; h: number } {
  return { ...SIZES[tool] }
}

// Default rect for a freshly-uploaded image: scale its natural size to fit
// within half the page (in whichever dimension binds first), preserving aspect,
// and centre it. Keeps the placed image undistorted from the moment it appears.
export function imagePlacement(
  natW: number,
  natH: number,
  page: { w: number; h: number },
): Rect {
  const w0 = natW > 0 ? natW : 1
  const h0 = natH > 0 ? natH : 1
  const scale = Math.min((page.w * 0.5) / w0, (page.h * 0.5) / h0)
  const w = w0 * scale
  const h = h0 * scale
  return { x: (page.w - w) / 2, y: (page.h - h) / 2, w, h }
}

export function makeElement(
  tool: 'calendar' | 'image' | 'text',
  opts: FactoryOpts,
  pageSize: { w: number; h: number },
  rect?: Rect,
): El {
  const { w, h } = SIZES[tool]
  const geom: Rect = rect ?? { w, h, x: (pageSize.w - w) / 2, y: (pageSize.h - h) / 2 }
  const base = { id: uid(), ...geom, colour: opts.colour }
  switch (tool) {
    case 'calendar':
      return { ...base, type: 'calendar', variant: opts.calendarVariant, feedId: opts.feedId, font: opts.font }
    case 'image':
      return { ...base, type: 'image', src: opts.imageId ?? '' }
    case 'text':
      return { ...base, type: 'text', text: 'Text', font: opts.font, align: opts.align } satisfies TextEl
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
