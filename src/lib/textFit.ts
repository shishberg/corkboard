// Auto-size text to fill its box. The device renderer does the same thing in
// Rust (text::fit_font_size); here we measure the real DOM so the editor preview
// is true WYSIWYG — the browser's own font metrics and line-wrapping decide the
// size, which matches the panel closely since both use the same font.

export const LINE_HEIGHT = 1.25

// Largest integer px in [minPx, maxPx] at which `text`, wrapped at width `w` in
// `font`, still fits within `(w, h)`. Measures an offscreen element so the
// result reflects exactly how the browser will lay the text out.
export function fitFontSize(
  text: string,
  w: number,
  h: number,
  font: string,
  minPx = 10,
  maxPx = 240,
): number {
  if (!text.trim() || w <= 0 || h <= 0) return minPx
  if (typeof document === 'undefined') return minPx

  const el = document.createElement('div')
  Object.assign(el.style, {
    position: 'absolute',
    visibility: 'hidden',
    left: '-99999px',
    top: '0',
    width: `${w}px`,
    fontFamily: font,
    lineHeight: String(LINE_HEIGHT),
    whiteSpace: 'pre-wrap',
    overflowWrap: 'normal',
    wordBreak: 'normal',
  })
  el.textContent = text
  document.body.appendChild(el)

  // Binary search the largest fitting size (fit is monotonic in px).
  let best = minPx
  let lo = minPx
  let hi = maxPx
  while (lo <= hi) {
    const mid = (lo + hi) >> 1
    el.style.fontSize = `${mid}px`
    const fits = el.scrollHeight <= h + 0.5 && el.scrollWidth <= w + 0.5
    if (fits) {
      best = mid
      lo = mid + 1
    } else {
      hi = mid - 1
    }
  }

  el.remove()
  return best
}
