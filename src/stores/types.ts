export type Orientation = 'landscape' | 'portrait'
export type ToolId = 'select' | 'calendar' | 'draw' | 'image' | 'text'
export type EpaperColour = 'black' | 'white' | 'red' | 'yellow' | 'blue' | 'green'

export interface BaseEl { id: string; type: string; x: number; y: number; w: number; h: number; colour: EpaperColour }
export interface CalendarEl extends BaseEl { type: 'calendar'; variant: 'date' | 'today' | 'week'; feedId: string }
export interface ImageEl extends BaseEl { type: 'image'; src: string }
export interface DrawingEl extends BaseEl { type: 'drawing'; natW: number; natH: number; strokes: Stroke[] }
export interface TextEl extends BaseEl { type: 'text'; text: string; font: string; align: 'left' | 'center' }

export interface Stroke { colour: EpaperColour; size: number; points: { x: number; y: number }[] }

export type El = CalendarEl | ImageEl | DrawingEl | TextEl

export interface Page { id: string; name: string; elements: El[] }

export interface DocState {
  orientation: Orientation
  pages: Page[]
  livePageId: string | null
  selectedPageId: string | null
  selectedElId: string | null
  activeTool: ToolId
}
