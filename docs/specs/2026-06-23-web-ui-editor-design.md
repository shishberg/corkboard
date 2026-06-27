# Corkboard Web UI — Editor (frontend-only) Design

**Date:** 2026-06-23
**Scope:** First pass of the page-editing web UI. Layout + basic in-memory interactions. No device wiring.

## Goal
Build the noticeboard page editor as a frontend-only Vue app. Get the layout and rough
appearance right, with core interactions working in memory (select page, switch tools,
add/drag/resize elements, reorder timeline). Publishing, freeform drawing, and real image
upload are stubbed in this pass.

## Stack
- Vite + Vue 3 (`<script setup>`, TypeScript)
- Tailwind + shadcn-vue (components, `Popover`, `Tooltip`, `Button`, etc.)
- Pinia for state — the store shape is the seed of the future page-state JSON contract
- Drag/resize: hand-rolled with pointer events (no extra library yet; adopt one only if it gets fiddly)
- lucide-vue-next for icons (ships with shadcn-vue)

## Target device (shapes constraints, not wired yet)
- Waveshare 7.3" E6 (Spectra 6) colour e-paper, **800×480**.
- 6-colour palette only: **black, white, red, yellow, blue, green**. The drawing colour
  options are exactly these six — no freeform colour picker. Canvas background defaults to white.
- Logical page size: landscape **800×480**, portrait **480×800**. Canvas scales to fit its
  container with letterboxing.

## Layout
```
+--------------------------------------------------------------+
| TopBar:  Corkboard            [orientation]   [Publish]      |
+-------+--------+---------------------------------------------+
| Pages | Tool   |                                             |
| side  | rail   |        EDITOR CANVAS (800x480 / 480x800)     |
| bar   | Select |        widgets = draggable DOM              |
| thumb | Clock  |        + drawing canvas overlay             |
| nails | Cal    |   popout: options for active tool           |
| [+]   | Draw   |                                             |
|       | Image  |                                             |
+-------+--------+---------------------------------------------+
| Timeline:  [pg1]-[pg2]-[pg3] ...    per-item delay control   |
+--------------------------------------------------------------+
```
- **TopBar** — app title; right side: orientation toggle + Publish button.
- **Pages sidebar** — vertical list of page thumbnails (real page tree scaled with CSS
  `transform: scale(...)`), plus an add-page button.
- **Tool rail** — vertical icon buttons with tooltips: Select, Clock, Calendar, Draw, Image.
- **Editor canvas** — the page surface; DOM widgets on top of a transparent drawing canvas overlay.
- **Timeline** — horizontal strip of pages the device loops through, with per-page delay.

## Tool options as popouts
- Clicking a tool **selects it and opens a popout** (`Popover`) anchored to that tool's icon,
  showing the tool's options.
- The popout **closes on outside-click or when the user starts using the tool** on the canvas.
- Option choices are **persisted to localStorage** per tool (clock variant, draw colour, pen
  size, calendar variant) and restored on load.
- **The tool icon reflects the current choice** where possible: the Draw icon is tinted with
  the selected colour; the Clock icon swaps to the chosen variation's glyph. Each clock
  variation is its own icon inside the popout.

## Components
- `App.vue` — grid shell tying the regions together.
- `TopBar.vue` — title, orientation toggle, Publish button.
- `PageSidebar.vue` + `PageThumbnail.vue` — page list and add-page.
- `ToolRail.vue` — tool icon buttons + their option popouts.
- `ToolOptions/` — one options panel per tool, rendered inside the popout
  (`ClockOptions.vue`, `CalendarOptions.vue`, `DrawOptions.vue`, `ImageOptions.vue`).
- `EditorCanvas.vue` — the page surface; renders widgets + the drawing overlay; handles selection.
- Widgets: `ClockWidget.vue`, `CalendarWidget.vue`, `ImageWidget.vue`; `DrawingLayer.vue`
  (transparent `<canvas>` overlay — placeholder this pass).
- `MovableElement.vue` — wraps any widget to provide drag + resize handles, backed by a
  `useDraggableResizable` composable (pointer events).
- `Timeline.vue` + `TimelineItem.vue` — loop order and delays.

## State (Pinia `usePagesStore`)
```ts
type Orientation = 'landscape' | 'portrait'
type ToolId = 'select' | 'clock' | 'calendar' | 'draw' | 'image'
type EpaperColour = 'black' | 'white' | 'red' | 'yellow' | 'blue' | 'green'

interface BaseEl { id: string; type: string; x: number; y: number; w: number; h: number }
interface ClockEl    extends BaseEl { type: 'clock';    variant: 'time' | 'time-date' | 'date' }
interface CalendarEl extends BaseEl { type: 'calendar'; variant: 'today' | 'week'; events: CalEvent[] }
interface ImageEl    extends BaseEl { type: 'image';    src: string }        // object URL for now
interface DrawingEl  extends BaseEl { type: 'drawing';  strokes: Stroke[] }  // stub this pass

interface CalEvent { id: string; title: string; start: string; recur?: 'none' | 'daily' | 'weekly' }
interface Stroke   { colour: EpaperColour; size: number; points: { x: number; y: number }[] }

type El = ClockEl | CalendarEl | ImageEl | DrawingEl

interface Page { id: string; name: string; elements: El[] }
interface TimelineEntry { pageId: string; delayMs: number }

interface DocState {
  orientation: Orientation
  pages: Page[]
  timeline: TimelineEntry[]
  selectedPageId: string | null
  selectedElId: string | null
  activeTool: ToolId
}
```
Tool option defaults (clock variant, draw colour, pen size, calendar variant) live in a
separate persisted slice backed by localStorage, not in `DocState`.

## Interactions in this pass
- Select a page from the sidebar opens it in the canvas. `[+]` adds a blank page.
- Selecting Clock / Calendar / Image adds a default element of that type to the page (placed
  at centre) using the tool's current options. **Select** picks/moves existing elements.
- Drag + resize elements on the canvas via `MovableElement` handles.
- Tool popouts edit the active tool's options (clock style, calendar today/week, pen
  size/colour, image placeholder) — UI only, persisted to localStorage.
- Timeline: drag a page from the sidebar onto it, drag within it to reorder, set per-page delay.
- Orientation toggle flips the canvas aspect ratio (800x480 <-> 480x800).
- **Publish** is stubbed — shows a toast, no network call.

## Explicitly out of scope this pass
- Device wiring: no GET load, no Publish POST, no image upload to the device.
- Freeform drawing engine (the overlay and `DrawingLayer` are placeholders; stroke capture comes later).
- Real image upload (image elements use a local object URL / placeholder).
- The page-state JSON contract and endpoints (the Pinia shape is a draft, not the committed contract).
- Auth (none — private-network assumption holds).

## Testing
Kept light, matching the project's "grow from real work" stance. Vitest is set up; add a few
sanity tests for the store (add page, add element, reorder timeline, toggle orientation) and a
smoke render of `App.vue`. Not full coverage this pass.

## Open decisions deferred (recorded in .mex/context/decisions.md)
- Whether the web UI is served from the device or hosted elsewhere.
- The final page-state JSON schema (this Pinia shape is the draft input to it).
- Device-side language/framework.
