export type Orientation = 'landscape' | 'portrait'
export type ToolId = 'select' | 'clock' | 'calendar' | 'draw' | 'image'
export type EpaperColour = 'black' | 'white' | 'red' | 'yellow' | 'blue' | 'green'

export interface BaseEl { id: string; type: string; x: number; y: number; w: number; h: number; colour: EpaperColour }
export interface ClockEl extends BaseEl { type: 'clock'; variant: 'time' | 'time-date' | 'date' }
export interface CalendarEl extends BaseEl { type: 'calendar'; variant: 'today' | 'week'; events: CalEvent[] }
export interface ImageEl extends BaseEl { type: 'image'; src: string }
export interface DrawingEl extends BaseEl { type: 'drawing'; strokes: Stroke[] }

export interface CalEvent { id: string; title: string; start: string; recur?: 'none' | 'daily' | 'weekly' }
export interface Stroke { colour: EpaperColour; size: number; points: { x: number; y: number }[] }

export type El = ClockEl | CalendarEl | ImageEl | DrawingEl

export interface Page { id: string; name: string; elements: El[] }
export interface TimelineEntry { pageId: string; delayMs: number }

export interface DocState {
  orientation: Orientation
  pages: Page[]
  timeline: TimelineEntry[]
  selectedPageId: string | null
  selectedElId: string | null
  activeTool: ToolId
}
