import type { El } from './types'

let counter = 0
const uid = () => `el-${Date.now().toString(36)}-${(counter++).toString(36)}`

interface FactoryOpts {
  clockVariant: 'time' | 'time-date' | 'date'
  calendarVariant: 'today' | 'week'
}

const SIZES = {
  clock: { w: 240, h: 90 },
  calendar: { w: 300, h: 220 },
  image: { w: 200, h: 150 },
}

export function makeElement(
  tool: 'clock' | 'calendar' | 'image',
  opts: FactoryOpts,
  pageSize: { w: number; h: number },
): El {
  const { w, h } = SIZES[tool]
  const base = { id: uid(), x: (pageSize.w - w) / 2, y: (pageSize.h - h) / 2, w, h }
  switch (tool) {
    case 'clock':
      return { ...base, type: 'clock', variant: opts.clockVariant }
    case 'calendar':
      return { ...base, type: 'calendar', variant: opts.calendarVariant, events: [] }
    case 'image':
      return { ...base, type: 'image', src: '' }
  }
}
