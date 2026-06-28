// Build a CSS text-shadow that mimics the device's 1px dilation outline: the
// glyph is haloed by copies offset 1px in all 8 directions, in the outline
// colour. This matches render.rs (text::draw_text outline pass) for WYSIWYG.
//
// Note: use this, NOT -webkit-text-stroke. text-stroke centres the stroke and
// eats into the glyph; the device grows the halo strictly outward, which is what
// offset shadow copies do.
export function outlineTextShadow(colour?: string): string {
  if (!colour) return 'none'
  const d = 1
  const parts: string[] = []
  for (let dx = -d; dx <= d; dx++) {
    for (let dy = -d; dy <= d; dy++) {
      if (dx === 0 && dy === 0) continue
      parts.push(`${dx}px ${dy}px 0 ${colour}`)
    }
  }
  return parts.join(', ')
}
