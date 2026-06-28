export type Orientation = 'landscape' | 'portrait'
export type ToolId = 'select' | 'calendar' | 'draw' | 'image' | 'text' | 'background'
export type EpaperColour = 'black' | 'white' | 'red' | 'yellow' | 'blue' | 'green'

export interface BaseEl { id: string; type: string; x: number; y: number; w: number; h: number; colour: EpaperColour }
export interface CalendarEl extends BaseEl { type: 'calendar'; variant: 'date' | 'today' | 'agenda'; feedId: string; font: string; align: 'left' | 'center' }
export interface ImageEl extends BaseEl { type: 'image'; src: string }
export interface DrawingEl extends BaseEl { type: 'drawing'; natW: number; natH: number; strokes: Stroke[] }
export interface TextEl extends BaseEl { type: 'text'; text: string; font: string; align: 'left' | 'center' }

export interface Stroke { colour: EpaperColour; size: number; points: { x: number; y: number }[] }

export type El = CalendarEl | ImageEl | DrawingEl | TextEl

export interface Page { id: string; name: string; elements: El[]; background?: EpaperColour; orientation: Orientation }

export interface Size { w: number; h: number }

// Canvas dimensions for a page, derived from its orientation. Size is a function
// of the page (not a stored field, not a document-global) — null defaults to
// landscape. Use this everywhere a page's pixel size is needed.
export function pageSize(page: Pick<Page, 'orientation'> | null | undefined): Size {
  return (page?.orientation ?? 'landscape') === 'portrait'
    ? { w: 480, h: 800 }
    : { w: 800, h: 480 }
}

export interface DocState {
  pages: Page[]
  livePageId: string | null
  selectedPageId: string | null
  selectedElId: string | null
  activeTool: ToolId
}

// Shape accepted when loading a saved document. Covers current documents plus
// older ones that kept `orientation` at the document level and had pages
// without their own `orientation`. `hydrate` migrates these onto each page.
export type LoadedDoc = Omit<DocState, 'pages'> & {
  orientation?: Orientation
  pages: (Omit<Page, 'orientation'> & { orientation?: Orientation })[]
}
