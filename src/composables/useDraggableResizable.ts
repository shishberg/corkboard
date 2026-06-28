interface Rect { x: number; y: number; w: number; h: number }

interface Opts {
  getRect: () => Rect
  onUpdate: (rect: Rect) => void
  scale: () => number
  // Optional width/height ratio to lock resizing to (e.g. an image's aspect).
  // When set, resize keeps this ratio so the box can't distort the content.
  aspect?: () => number | undefined
}

const MIN = 20

export function useDraggableResizable(opts: Opts) {
  function begin(e: PointerEvent, mode: 'drag' | 'resize') {
    e.preventDefault()
    e.stopPropagation()
    const startX = e.clientX
    const startY = e.clientY
    const start = { ...opts.getRect() }
    const scale = opts.scale() || 1

    const move = (ev: PointerEvent) => {
      const dx = (ev.clientX - startX) / scale
      const dy = (ev.clientY - startY) / scale
      if (mode === 'drag') {
        opts.onUpdate({ ...start, x: start.x + dx, y: start.y + dy })
      } else {
        const aspect = opts.aspect?.()
        if (aspect && aspect > 0) {
          // Drive width from the pointer, derive height to keep the ratio.
          let w = Math.max(MIN, start.w + dx)
          let h = w / aspect
          if (h < MIN) {
            h = MIN
            w = MIN * aspect
          }
          opts.onUpdate({ ...start, w, h })
        } else {
          opts.onUpdate({
            ...start,
            w: Math.max(MIN, start.w + dx),
            h: Math.max(MIN, start.h + dy),
          })
        }
      }
    }
    const up = () => {
      window.removeEventListener('pointermove', move)
      window.removeEventListener('pointerup', up)
    }
    window.addEventListener('pointermove', move)
    window.addEventListener('pointerup', up)
  }

  return {
    startDrag: (e: PointerEvent) => begin(e, 'drag'),
    startResize: (e: PointerEvent) => begin(e, 'resize'),
  }
}
