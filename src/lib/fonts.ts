export interface FontFace { weight: number; style: string; file: string }
export interface FontDef { id: string; name: string; default?: boolean; faces: FontFace[] }

export const DEFAULT_MANIFEST: FontDef[] = [
  {
    id: 'atkinson-hyperlegible',
    name: 'Atkinson Hyperlegible',
    default: true,
    faces: [
      { weight: 400, style: 'normal', file: 'atkinson-hyperlegible/Regular.ttf' },
      { weight: 700, style: 'normal', file: 'atkinson-hyperlegible/Bold.ttf' },
    ],
  },
  {
    id: 'dejavu-sans',
    name: 'DejaVu Sans',
    faces: [{ weight: 400, style: 'normal', file: 'dejavu-sans/DejaVuSans.ttf' }],
  },
  {
    id: 'carlito',
    name: 'Carlito',
    faces: [{ weight: 400, style: 'normal', file: 'carlito/Carlito-Regular.ttf' }],
  },
  {
    id: 'gelasio',
    name: 'Gelasio',
    faces: [{ weight: 400, style: 'normal', file: 'gelasio/Gelasio.ttf' }],
  },
]

/** GET /fonts/manifest.json; returns parsed fonts array on success, DEFAULT_MANIFEST on any failure. */
export async function loadFontManifest(): Promise<FontDef[]> {
  try {
    const res = await fetch('/fonts/manifest.json')
    if (!res.ok) return DEFAULT_MANIFEST
    const json = await res.json() as { fonts: FontDef[] }
    return json.fonts
  } catch {
    return DEFAULT_MANIFEST
  }
}

/** Inject @font-face rules for all faces into document.head (idempotent). */
export function injectFontFaces(fonts: FontDef[]): void {
  const css = fonts.flatMap((font) =>
    font.faces.map(
      (face) =>
        `@font-face { font-family: '${font.id}'; font-weight: ${face.weight}; font-style: ${face.style}; src: url('/fonts/${face.file}'); }`,
    ),
  ).join('\n')

  const existing = document.head.querySelector('style[data-role="font-faces"]')
  if (existing) {
    existing.textContent = css
  } else {
    const style = document.createElement('style')
    style.setAttribute('data-role', 'font-faces')
    style.textContent = css
    document.head.appendChild(style)
  }
}

/** Return the id of the default font, or the first font id, or 'atkinson-hyperlegible'. */
export function defaultFontId(fonts: FontDef[]): string {
  return fonts.find((f) => f.default)?.id ?? fonts[0]?.id ?? 'atkinson-hyperlegible'
}
