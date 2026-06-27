# Corkboard Web UI Editor (frontend-only) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the noticeboard page editor as a frontend-only Vue app — layout plus in-memory interactions (select page, switch tools, add/drag/resize elements, reorder timeline), with publish/drawing/upload stubbed.

**Architecture:** Vite + Vue 3 single-page app. A Pinia store (`usePagesStore`) holds the document state (the seed of the future page-state JSON contract); a second persisted slice (`useToolOptionsStore`) holds sticky tool defaults backed by localStorage. The editor surface is absolutely-positioned DOM widgets layered over a transparent `<canvas>` overlay; thumbnails reuse the same DOM tree scaled with CSS `transform`. Drag/resize is hand-rolled with pointer events via a `useDraggableResizable` composable.

**Tech Stack:** Vite, Vue 3 (`<script setup>`, TypeScript), Tailwind CSS v4 (`@tailwindcss/vite` plugin, CSS-first config), shadcn-vue (Button, Popover, Tooltip), Pinia, @lucide/vue, Vitest + @vue/test-utils + jsdom.

## Global Constraints

- Target device: Waveshare 7.3" E6 colour e-paper, logical page size landscape **800×480**, portrait **480×800**. Canvas scales to fit its container with letterboxing.
- Drawing colour palette is exactly six values: `black, white, red, yellow, blue, green`. No freeform colour picker. Canvas background defaults to white.
- No auth, no network calls this pass. Publish is a stubbed toast. Device wiring, the drawing engine, real image upload, and the JSON contract/endpoints are out of scope.
- Tool option defaults (clock variant, draw colour, pen size, calendar variant) persist to localStorage in a slice **separate** from `DocState`.
- Use shadcn-vue components rather than hand-rolling UI where one exists. Don't add a drag/resize library this pass.
- Vue SFCs `<script setup lang="ts">`. Component files are `PascalCase.vue`. Tests live next to or under `src/` as `*.test.ts` / `*.spec.ts` and run under Vitest.

---

### Task 1: Scaffold the Vite + Vue + TS project with Vitest

**Files:**
- Create: `package.json`, `vite.config.ts`, `vitest.config.ts`, `tsconfig.json`, `tsconfig.node.json`, `index.html`, `src/main.ts`, `src/App.vue`, `src/vite-env.d.ts`
- Test: `src/smoke.test.ts`

**Interfaces:**
- Consumes: nothing (first task).
- Produces: a booting Vue app mounted at `#app`, Pinia installed in `main.ts`, and a working `npm test` (Vitest + jsdom) command.

- [ ] **Step 1: Create the project with the Vue + TS template**

Run from the repo root (`/Users/agent/src/corkboard`):

```bash
npm create vite@latest . -- --template vue-ts
```

If the directory is non-empty, choose "Ignore files and continue". This generates `package.json`, `vite.config.ts`, `tsconfig*.json`, `index.html`, `src/main.ts`, `src/App.vue`, `src/vite-env.d.ts`.

- [ ] **Step 2: Install runtime and test dependencies**

```bash
npm install pinia @lucide/vue
npm install -D vitest @vue/test-utils jsdom @vitejs/plugin-vue
```

- [ ] **Step 3: Add a Vitest config with jsdom**

Create `vitest.config.ts`:

```ts
import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath } from 'node:url'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  test: {
    environment: 'jsdom',
    globals: true,
  },
})
```

- [ ] **Step 4: Add the `@` alias to Vite and TS configs**

In `vite.config.ts`, add the same `resolve.alias` block as above. In `tsconfig.json` (or `tsconfig.app.json` if the template split it), add under `compilerOptions`:

```json
"baseUrl": ".",
"paths": { "@/*": ["src/*"] }
```

- [ ] **Step 5: Install Pinia in `main.ts`**

Replace `src/main.ts` with:

```ts
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './style.css'

createApp(App).use(createPinia()).mount('#app')
```

- [ ] **Step 6: Add test scripts to `package.json`**

In the `"scripts"` block add:

```json
"test": "vitest run",
"test:watch": "vitest"
```

- [ ] **Step 7: Write a failing smoke test**

Create `src/smoke.test.ts`:

```ts
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia } from 'pinia'
import App from './App.vue'

describe('App', () => {
  it('mounts without throwing', () => {
    const wrapper = mount(App, { global: { plugins: [createPinia()] } })
    expect(wrapper.exists()).toBe(true)
  })
})
```

- [ ] **Step 8: Run the test to verify it passes**

Run: `npm test`
Expected: PASS (the Vite template `App.vue` mounts fine).

- [ ] **Step 9: Verify the dev server boots**

Run: `npm run build`
Expected: build completes with no type errors.

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "chore: scaffold Vite + Vue + TS project with Vitest"
```

---

### Task 2: Add Tailwind v4 and the shadcn-vue base components

**Files:**
- Create: `components.json`, `src/lib/utils.ts`, `src/components/ui/button/`, `src/components/ui/popover/`, `src/components/ui/tooltip/` (the last four are written by the shadcn-vue CLI)
- Modify: `src/style.css`, `vite.config.ts`, `vitest.config.ts`, `tsconfig.app.json`
- Test: `src/components/ui/button/button.test.ts`

**Interfaces:**
- Consumes: the booting app from Task 1.
- Produces: importable shadcn-vue components — `Button` from `@/components/ui/button`, `Popover`/`PopoverTrigger`/`PopoverContent` from `@/components/ui/popover`, `Tooltip`/`TooltipTrigger`/`TooltipContent`/`TooltipProvider` from `@/components/ui/tooltip`, and the `cn()` class-merge helper from `@/lib/utils`.

This task uses **Tailwind v4**: a Vite plugin instead of PostCSS, and CSS-first config (no `tailwind.config.js`, no `postcss.config.js`). The shadcn-vue `init` CLI writes `components.json`, `src/lib/utils.ts`, and the theme tokens into `src/style.css` — reproducing the v4 theme variables by hand is the error-prone part, so let the CLI do it.

- [ ] **Step 1: Install Tailwind v4 and the shadcn-vue runtime deps**

```bash
npm install tailwindcss @tailwindcss/vite
npm install class-variance-authority clsx tailwind-merge tw-animate-css reka-ui
```

(`@tailwindcss/vite` is the v4 Vite plugin — it replaces the v3 PostCSS pipeline, so no `postcss`/`autoprefixer`/`tailwind.config.js`. `reka-ui` is the headless primitive library shadcn-vue builds on.)

- [ ] **Step 2: Add the Tailwind plugin to the Vite and Vitest configs**

In `vite.config.ts`, import and register the plugin alongside `vue()`:

```ts
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { fileURLToPath } from 'node:url'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
})
```

Do the same in `vitest.config.ts` so component tests get the compiled styles (harmless if unused in jsdom):

```ts
import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { fileURLToPath } from 'node:url'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  test: {
    environment: 'jsdom',
    globals: true,
  },
})
```

- [ ] **Step 3: Replace `src/style.css` with the v4 import**

```css
@import 'tailwindcss';

html, body, #app {
  height: 100%;
  margin: 0;
  font-family: system-ui, Avenir, Helvetica, Arial, sans-serif;
}
```

(`shadcn-vue init` in Step 4 prepends `@import 'tw-animate-css';`, the `@custom-variant dark`, and the `:root` / `.dark` theme token blocks above your rules. Leave room for it — don't hand-write those tokens.)

- [ ] **Step 4: Run shadcn-vue init**

```bash
npx shadcn-vue@latest init
```

This is interactive. Expected prompts and answers:
- **Which color would you like to use as base color?** → `Slate`
- (If asked) **Configure import alias / components / utils** → accept defaults (`@/components`, `@/lib/utils`).

It writes `components.json`, creates `src/lib/utils.ts` with the `cn()` helper, and injects the Tailwind v4 theme variables into `src/style.css`. If the CLI cannot detect the framework, confirm it's run from the repo root where `package.json` and `vite.config.ts` live.

- [ ] **Step 5: Verify `components.json` and the `cn` helper exist**

Confirm `src/lib/utils.ts` contains:

```ts
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
```

If `init` did not create it (e.g. it only wrote `components.json`), create it by hand with the contents above. Confirm `components.json` has `"aliases": { "utils": "@/lib/utils", "ui": "@/components/ui", "components": "@/components" }`. The `tailwind.config` field should be `""` for v4 — that's expected, not a mistake.

- [ ] **Step 6: Add the Button, Popover, and Tooltip components via the CLI**

```bash
npx shadcn-vue@latest add button popover tooltip
```

If the CLI prompts, accept the defaults. This writes `src/components/ui/button/`, `src/components/ui/popover/`, and `src/components/ui/tooltip/`, each with an `index.ts` barrel export.

- [ ] **Step 7: Write a failing test for the Button render**

Create `src/components/ui/button/button.test.ts`:

```ts
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import { Button } from './index'

describe('Button', () => {
  it('renders its slot text', () => {
    const wrapper = mount(Button, { slots: { default: 'Publish' } })
    expect(wrapper.text()).toContain('Publish')
  })
})
```

- [ ] **Step 8: Run the test**

Run: `npm test -- button`
Expected: PASS.

- [ ] **Step 9: Verify the build still type-checks**

Run: `npm run build`
Expected: build succeeds.

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "chore: add Tailwind and shadcn-vue base components (button, popover, tooltip)"
```

---

### Task 3: Define the document store types and core actions

**Files:**
- Create: `src/stores/types.ts`, `src/stores/pages.ts`
- Test: `src/stores/pages.test.ts`

**Interfaces:**
- Consumes: Pinia from Task 1.
- Produces: the shared types in `@/stores/types` and `usePagesStore` from `@/stores/pages`.
  - Types (exported from `@/stores/types`): `Orientation = 'landscape' | 'portrait'`; `ToolId = 'select' | 'clock' | 'calendar' | 'draw' | 'image'`; `EpaperColour = 'black' | 'white' | 'red' | 'yellow' | 'blue' | 'green'`; `BaseEl { id: string; type: string; x: number; y: number; w: number; h: number }`; `ClockEl extends BaseEl { type: 'clock'; variant: 'time' | 'time-date' | 'date' }`; `CalendarEl extends BaseEl { type: 'calendar'; variant: 'today' | 'week'; events: CalEvent[] }`; `ImageEl extends BaseEl { type: 'image'; src: string }`; `DrawingEl extends BaseEl { type: 'drawing'; strokes: Stroke[] }`; `CalEvent { id: string; title: string; start: string; recur?: 'none' | 'daily' | 'weekly' }`; `Stroke { colour: EpaperColour; size: number; points: { x: number; y: number }[] }`; `El = ClockEl | CalendarEl | ImageEl | DrawingEl`; `Page { id: string; name: string; elements: El[] }`; `TimelineEntry { pageId: string; delayMs: number }`.
  - `usePagesStore` state matches `DocState`: `orientation`, `pages`, `timeline`, `selectedPageId`, `selectedElId`, `activeTool`.
  - `usePagesStore` getters: `selectedPage: Page | null`, `pageSize: { w: number; h: number }`.
  - `usePagesStore` actions: `addPage(): string` (returns new page id, selects it), `selectPage(id: string): void`, `toggleOrientation(): void`, `setActiveTool(t: ToolId): void`, `addElement(el: El): void` (pushes onto selected page, selects it), `selectElement(id: string | null): void`, `updateElement(id: string, patch: Partial<BaseEl>): void`, `addToTimeline(pageId: string): void` (appends with `delayMs: 5000`), `reorderTimeline(from: number, to: number): void`, `setTimelineDelay(index: number, delayMs: number): void`, `removeFromTimeline(index: number): void`.

- [ ] **Step 1: Write the type definitions**

Create `src/stores/types.ts`:

```ts
export type Orientation = 'landscape' | 'portrait'
export type ToolId = 'select' | 'clock' | 'calendar' | 'draw' | 'image'
export type EpaperColour = 'black' | 'white' | 'red' | 'yellow' | 'blue' | 'green'

export interface BaseEl { id: string; type: string; x: number; y: number; w: number; h: number }
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
```

- [ ] **Step 2: Write failing tests for the store**

Create `src/stores/pages.test.ts`:

```ts
import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { usePagesStore } from './pages'
import type { ClockEl } from './types'

beforeEach(() => setActivePinia(createPinia()))

function clockEl(id: string): ClockEl {
  return { id, type: 'clock', variant: 'time', x: 0, y: 0, w: 200, h: 80 }
}

describe('usePagesStore', () => {
  it('starts with one page selected', () => {
    const s = usePagesStore()
    expect(s.pages.length).toBe(1)
    expect(s.selectedPageId).toBe(s.pages[0].id)
  })

  it('addPage adds and selects a new page', () => {
    const s = usePagesStore()
    const id = s.addPage()
    expect(s.pages.length).toBe(2)
    expect(s.selectedPageId).toBe(id)
  })

  it('toggleOrientation flips orientation and pageSize', () => {
    const s = usePagesStore()
    expect(s.orientation).toBe('landscape')
    expect(s.pageSize).toEqual({ w: 800, h: 480 })
    s.toggleOrientation()
    expect(s.orientation).toBe('portrait')
    expect(s.pageSize).toEqual({ w: 480, h: 800 })
  })

  it('addElement pushes onto the selected page and selects it', () => {
    const s = usePagesStore()
    s.addElement(clockEl('e1'))
    expect(s.selectedPage?.elements.length).toBe(1)
    expect(s.selectedElId).toBe('e1')
  })

  it('updateElement patches geometry', () => {
    const s = usePagesStore()
    s.addElement(clockEl('e1'))
    s.updateElement('e1', { x: 50, y: 60 })
    const el = s.selectedPage?.elements[0]
    expect(el?.x).toBe(50)
    expect(el?.y).toBe(60)
  })

  it('addToTimeline appends with a default delay', () => {
    const s = usePagesStore()
    const pid = s.pages[0].id
    s.addToTimeline(pid)
    expect(s.timeline).toEqual([{ pageId: pid, delayMs: 5000 }])
  })

  it('reorderTimeline moves an entry', () => {
    const s = usePagesStore()
    const a = s.pages[0].id
    const b = s.addPage()
    s.addToTimeline(a)
    s.addToTimeline(b)
    s.reorderTimeline(0, 1)
    expect(s.timeline.map((t) => t.pageId)).toEqual([b, a])
  })

  it('setTimelineDelay updates one entry', () => {
    const s = usePagesStore()
    s.addToTimeline(s.pages[0].id)
    s.setTimelineDelay(0, 12000)
    expect(s.timeline[0].delayMs).toBe(12000)
  })
})
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `npm test -- pages`
Expected: FAIL with "Cannot find module './pages'".

- [ ] **Step 4: Write the store**

Create `src/stores/pages.ts`:

```ts
import { defineStore } from 'pinia'
import type { DocState, El, Page, ToolId, BaseEl } from './types'

let counter = 0
const uid = (prefix: string) => `${prefix}-${Date.now().toString(36)}-${(counter++).toString(36)}`

function blankPage(): Page {
  return { id: uid('page'), name: 'Page', elements: [] }
}

export const usePagesStore = defineStore('pages', {
  state: (): DocState => {
    const first = blankPage()
    return {
      orientation: 'landscape',
      pages: [first],
      timeline: [],
      selectedPageId: first.id,
      selectedElId: null,
      activeTool: 'select',
    }
  },

  getters: {
    selectedPage(state): Page | null {
      return state.pages.find((p) => p.id === state.selectedPageId) ?? null
    },
    pageSize(state): { w: number; h: number } {
      return state.orientation === 'landscape' ? { w: 800, h: 480 } : { w: 480, h: 800 }
    },
  },

  actions: {
    addPage(): string {
      const page = blankPage()
      page.name = `Page ${this.pages.length + 1}`
      this.pages.push(page)
      this.selectedPageId = page.id
      this.selectedElId = null
      return page.id
    },
    selectPage(id: string) {
      this.selectedPageId = id
      this.selectedElId = null
    },
    toggleOrientation() {
      this.orientation = this.orientation === 'landscape' ? 'portrait' : 'landscape'
    },
    setActiveTool(t: ToolId) {
      this.activeTool = t
    },
    addElement(el: El) {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      if (!page) return
      page.elements.push(el)
      this.selectedElId = el.id
    },
    selectElement(id: string | null) {
      this.selectedElId = id
    },
    updateElement(id: string, patch: Partial<BaseEl>) {
      const page = this.pages.find((p) => p.id === this.selectedPageId)
      const el = page?.elements.find((e) => e.id === id)
      if (el) Object.assign(el, patch)
    },
    addToTimeline(pageId: string) {
      this.timeline.push({ pageId, delayMs: 5000 })
    },
    reorderTimeline(from: number, to: number) {
      if (from === to) return
      const [moved] = this.timeline.splice(from, 1)
      this.timeline.splice(to, 0, moved)
    },
    setTimelineDelay(index: number, delayMs: number) {
      const entry = this.timeline[index]
      if (entry) entry.delayMs = delayMs
    },
    removeFromTimeline(index: number) {
      this.timeline.splice(index, 1)
    },
  },
})
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `npm test -- pages`
Expected: PASS (all 8 tests).

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add pages store with document state and core actions"
```

---

### Task 4: Add the persisted tool-options store and element factories

**Files:**
- Create: `src/stores/toolOptions.ts`, `src/stores/elementFactory.ts`
- Test: `src/stores/toolOptions.test.ts`, `src/stores/elementFactory.test.ts`

**Interfaces:**
- Consumes: types from `@/stores/types`.
- Produces:
  - `useToolOptionsStore` from `@/stores/toolOptions` with state `{ clockVariant: 'time' | 'time-date' | 'date'; calendarVariant: 'today' | 'week'; drawColour: EpaperColour; penSize: number }`, defaults `clockVariant: 'time'`, `calendarVariant: 'today'`, `drawColour: 'black'`, `penSize: 4`. It reads its initial state from `localStorage['corkboard.toolOptions']` and writes back on every change via a `$subscribe`.
  - `makeElement(tool, opts, pageSize)` from `@/stores/elementFactory` where `tool: 'clock' | 'calendar' | 'image'`, `opts: { clockVariant; calendarVariant }`, `pageSize: { w: number; h: number }` — returns a fully-formed `El` centred on the page with a generated id.

- [ ] **Step 1: Write failing tests for the tool-options store**

Create `src/stores/toolOptions.test.ts`:

```ts
import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useToolOptionsStore, ensureToolOptionsPersistence } from './toolOptions'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

describe('useToolOptionsStore', () => {
  it('has sensible defaults', () => {
    const s = useToolOptionsStore()
    expect(s.clockVariant).toBe('time')
    expect(s.calendarVariant).toBe('today')
    expect(s.drawColour).toBe('black')
    expect(s.penSize).toBe(4)
  })

  it('persists changes to localStorage', () => {
    const s = ensureToolOptionsPersistence()
    s.drawColour = 'red'
    const saved = JSON.parse(localStorage.getItem('corkboard.toolOptions') || '{}')
    expect(saved.drawColour).toBe('red')
  })

  it('restores from localStorage on init', () => {
    localStorage.setItem('corkboard.toolOptions', JSON.stringify({ penSize: 12 }))
    setActivePinia(createPinia())
    const s = useToolOptionsStore()
    expect(s.penSize).toBe(12)
  })
})
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `npm test -- toolOptions`
Expected: FAIL with "Cannot find module './toolOptions'".

- [ ] **Step 3: Write the tool-options store**

Create `src/stores/toolOptions.ts`:

```ts
import { defineStore } from 'pinia'
import type { EpaperColour } from './types'

const KEY = 'corkboard.toolOptions'

interface ToolOptionsState {
  clockVariant: 'time' | 'time-date' | 'date'
  calendarVariant: 'today' | 'week'
  drawColour: EpaperColour
  penSize: number
}

const defaults: ToolOptionsState = {
  clockVariant: 'time',
  calendarVariant: 'today',
  drawColour: 'black',
  penSize: 4,
}

function load(): ToolOptionsState {
  try {
    const raw = localStorage.getItem(KEY)
    return raw ? { ...defaults, ...JSON.parse(raw) } : { ...defaults }
  } catch {
    return { ...defaults }
  }
}

export const useToolOptionsStore = defineStore('toolOptions', {
  state: (): ToolOptionsState => load(),
})

// Persistence is opt-in per active store instance so it works everywhere,
// including tests. `ensureToolOptionsPersistence()` subscribes once and
// writes the whole state back to localStorage on every change.
const subscribed = new WeakSet<object>()
export function ensureToolOptionsPersistence() {
  const store = useToolOptionsStore()
  if (subscribed.has(store)) return store
  subscribed.add(store)
  store.$subscribe((_m, state) => {
    localStorage.setItem(KEY, JSON.stringify(state))
  })
  return store
}
```

The Step 1 test imports `ensureToolOptionsPersistence` and calls it in the "persists changes" case — so the single definition above is all that's needed; there is no separate `persistToolOptions` export.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `npm test -- toolOptions`
Expected: PASS (3 tests).

- [ ] **Step 5: Write failing tests for the element factory**

Create `src/stores/elementFactory.test.ts`:

```ts
import { describe, it, expect } from 'vitest'
import { makeElement } from './elementFactory'

const opts = { clockVariant: 'time-date' as const, calendarVariant: 'week' as const }
const size = { w: 800, h: 480 }

describe('makeElement', () => {
  it('makes a centred clock element using the option variant', () => {
    const el = makeElement('clock', opts, size)
    expect(el.type).toBe('clock')
    if (el.type === 'clock') expect(el.variant).toBe('time-date')
    expect(el.x).toBe((800 - el.w) / 2)
    expect(el.y).toBe((480 - el.h) / 2)
    expect(el.id).toBeTruthy()
  })

  it('makes a calendar element using the option variant', () => {
    const el = makeElement('calendar', opts, size)
    expect(el.type).toBe('calendar')
    if (el.type === 'calendar') {
      expect(el.variant).toBe('week')
      expect(el.events).toEqual([])
    }
  })

  it('makes an image element with an empty src', () => {
    const el = makeElement('image', opts, size)
    expect(el.type).toBe('image')
    if (el.type === 'image') expect(el.src).toBe('')
  })
})
```

- [ ] **Step 6: Run the tests to verify they fail**

Run: `npm test -- elementFactory`
Expected: FAIL with "Cannot find module './elementFactory'".

- [ ] **Step 7: Write the element factory**

Create `src/stores/elementFactory.ts`:

```ts
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
```

- [ ] **Step 8: Run the tests to verify they pass**

Run: `npm test -- elementFactory`
Expected: PASS (3 tests).

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat: add persisted tool-options store and element factory"
```

---

### Task 5: Build the drag/resize composable and MovableElement wrapper

**Files:**
- Create: `src/composables/useDraggableResizable.ts`, `src/components/MovableElement.vue`
- Test: `src/composables/useDraggableResizable.test.ts`

**Interfaces:**
- Consumes: nothing from earlier tasks except types.
- Produces:
  - `useDraggableResizable(opts)` from `@/composables/useDraggableResizable` where `opts: { getRect: () => { x: number; y: number; w: number; h: number }; onUpdate: (rect: { x: number; y: number; w: number; h: number }) => void; scale: () => number }`. Returns `{ startDrag(e: PointerEvent): void; startResize(e: PointerEvent): void }`. Movement deltas are divided by `scale()` so dragging tracks the pointer when the canvas is letterboxed/scaled.
  - `MovableElement.vue` — props `{ id: string; x: number; y: number; w: number; h: number; selected: boolean; scale: number }`, emits `select` (on pointerdown) and `update` (with the new rect). Renders a positioned box with a default slot for the widget and a resize handle at the bottom-right shown only when `selected`.

- [ ] **Step 1: Write a failing test for the composable**

Create `src/composables/useDraggableResizable.test.ts`:

```ts
import { describe, it, expect, vi } from 'vitest'
import { useDraggableResizable } from './useDraggableResizable'

function pointer(type: string, x: number, y: number): PointerEvent {
  const e = new Event(type) as any
  e.clientX = x
  e.clientY = y
  e.pointerId = 1
  e.preventDefault = vi.fn()
  e.stopPropagation = vi.fn()
  e.target = { setPointerCapture: vi.fn(), releasePointerCapture: vi.fn() }
  return e as PointerEvent
}

describe('useDraggableResizable', () => {
  it('translates a drag into an updated position', () => {
    let rect = { x: 10, y: 10, w: 100, h: 50 }
    const onUpdate = vi.fn((r) => (rect = r))
    const dr = useDraggableResizable({
      getRect: () => rect,
      onUpdate,
      scale: () => 1,
    })

    dr.startDrag(pointer('pointerdown', 0, 0))
    window.dispatchEvent(pointer('pointermove', 20, 30))
    window.dispatchEvent(pointer('pointerup', 20, 30))

    expect(onUpdate).toHaveBeenCalled()
    expect(rect).toEqual({ x: 30, y: 40, w: 100, h: 50 })
  })

  it('divides movement by scale', () => {
    let rect = { x: 0, y: 0, w: 100, h: 50 }
    const onUpdate = vi.fn((r) => (rect = r))
    const dr = useDraggableResizable({ getRect: () => rect, onUpdate, scale: () => 0.5 })

    dr.startDrag(pointer('pointerdown', 0, 0))
    window.dispatchEvent(pointer('pointermove', 50, 0))
    window.dispatchEvent(pointer('pointerup', 50, 0))

    expect(rect.x).toBe(100) // 50px on screen / 0.5 scale = 100 logical px
  })

  it('resizes from the bottom-right handle', () => {
    let rect = { x: 0, y: 0, w: 100, h: 50 }
    const onUpdate = vi.fn((r) => (rect = r))
    const dr = useDraggableResizable({ getRect: () => rect, onUpdate, scale: () => 1 })

    dr.startResize(pointer('pointerdown', 100, 50))
    window.dispatchEvent(pointer('pointermove', 130, 90))
    window.dispatchEvent(pointer('pointerup', 130, 90))

    expect(rect).toEqual({ x: 0, y: 0, w: 130, h: 90 })
  })
})
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- useDraggableResizable`
Expected: FAIL with "Cannot find module './useDraggableResizable'".

- [ ] **Step 3: Write the composable**

Create `src/composables/useDraggableResizable.ts`:

```ts
interface Rect { x: number; y: number; w: number; h: number }

interface Opts {
  getRect: () => Rect
  onUpdate: (rect: Rect) => void
  scale: () => number
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
        opts.onUpdate({
          ...start,
          w: Math.max(MIN, start.w + dx),
          h: Math.max(MIN, start.h + dy),
        })
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
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `npm test -- useDraggableResizable`
Expected: PASS (3 tests).

- [ ] **Step 5: Write MovableElement.vue**

Create `src/components/MovableElement.vue`:

```vue
<script setup lang="ts">
import { useDraggableResizable } from '@/composables/useDraggableResizable'

const props = defineProps<{
  id: string
  x: number
  y: number
  w: number
  h: number
  selected: boolean
  scale: number
}>()

const emit = defineEmits<{
  select: [id: string]
  update: [rect: { x: number; y: number; w: number; h: number }]
}>()

const dr = useDraggableResizable({
  getRect: () => ({ x: props.x, y: props.y, w: props.w, h: props.h }),
  onUpdate: (rect) => emit('update', rect),
  scale: () => props.scale,
})

function onPointerDown(e: PointerEvent) {
  emit('select', props.id)
  dr.startDrag(e)
}
</script>

<template>
  <div
    class="absolute select-none"
    :class="selected ? 'outline outline-2 outline-blue-500' : ''"
    :style="{ left: `${x}px`, top: `${y}px`, width: `${w}px`, height: `${h}px`, touchAction: 'none' }"
    @pointerdown="onPointerDown"
  >
    <slot />
    <div
      v-if="selected"
      class="absolute -right-1.5 -bottom-1.5 h-3 w-3 cursor-se-resize rounded-sm bg-blue-500"
      @pointerdown.stop="dr.startResize($event)"
    />
  </div>
</template>
```

- [ ] **Step 6: Run the full suite to confirm nothing broke**

Run: `npm test`
Expected: PASS (all suites so far).

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: add drag/resize composable and MovableElement wrapper"
```

---

### Task 6: Build the widget components

**Files:**
- Create: `src/components/widgets/ClockWidget.vue`, `src/components/widgets/CalendarWidget.vue`, `src/components/widgets/ImageWidget.vue`, `src/components/widgets/DrawingLayer.vue`
- Test: `src/components/widgets/widgets.test.ts`

**Interfaces:**
- Consumes: types from `@/stores/types`.
- Produces four presentational components:
  - `ClockWidget.vue` — prop `el: ClockEl`. Renders a static sample time/date based on `el.variant`.
  - `CalendarWidget.vue` — prop `el: CalendarEl`. Renders a `today` or `week` layout with `el.events` (sample placeholder rows if empty).
  - `ImageWidget.vue` — prop `el: ImageEl`. Renders the `<img>` if `el.src` is set, else a dashed placeholder.
  - `DrawingLayer.vue` — prop `size: { w: number; h: number }`. Renders a transparent full-canvas `<canvas>` placeholder (no stroke capture this pass).

- [ ] **Step 1: Write failing tests for the widgets**

Create `src/components/widgets/widgets.test.ts`:

```ts
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ClockWidget from './ClockWidget.vue'
import CalendarWidget from './CalendarWidget.vue'
import ImageWidget from './ImageWidget.vue'
import type { ClockEl, CalendarEl, ImageEl } from '@/stores/types'

describe('widgets', () => {
  it('ClockWidget shows a date line for the date variant', () => {
    const el: ClockEl = { id: 'c', type: 'clock', variant: 'date', x: 0, y: 0, w: 200, h: 80 }
    const w = mount(ClockWidget, { props: { el } })
    expect(w.get('[data-role="date"]').exists()).toBe(true)
  })

  it('CalendarWidget renders a week layout with 7 day cells', () => {
    const el: CalendarEl = { id: 'cal', type: 'calendar', variant: 'week', x: 0, y: 0, w: 300, h: 200, events: [] }
    const w = mount(CalendarWidget, { props: { el } })
    expect(w.findAll('[data-role="day"]').length).toBe(7)
  })

  it('ImageWidget shows a placeholder when src is empty', () => {
    const el: ImageEl = { id: 'img', type: 'image', src: '', x: 0, y: 0, w: 200, h: 150 }
    const w = mount(ImageWidget, { props: { el } })
    expect(w.get('[data-role="placeholder"]').exists()).toBe(true)
  })
})
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `npm test -- widgets`
Expected: FAIL with "Cannot find module './ClockWidget.vue'".

- [ ] **Step 3: Write ClockWidget.vue**

Create `src/components/widgets/ClockWidget.vue`:

```vue
<script setup lang="ts">
import { computed } from 'vue'
import type { ClockEl } from '@/stores/types'

const props = defineProps<{ el: ClockEl }>()
const showTime = computed(() => props.el.variant !== 'date')
const showDate = computed(() => props.el.variant !== 'time')
</script>

<template>
  <div class="flex h-full w-full flex-col items-center justify-center bg-white">
    <div v-if="showTime" data-role="time" class="text-3xl font-bold leading-none">12:45</div>
    <div v-if="showDate" data-role="date" class="text-sm text-neutral-600">Mon 23 Jun</div>
  </div>
</template>
```

- [ ] **Step 4: Write CalendarWidget.vue**

Create `src/components/widgets/CalendarWidget.vue`:

```vue
<script setup lang="ts">
import { computed } from 'vue'
import type { CalendarEl } from '@/stores/types'

const props = defineProps<{ el: CalendarEl }>()
const days = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']
const isWeek = computed(() => props.el.variant === 'week')
const rows = computed(() =>
  props.el.events.length ? props.el.events.map((e) => e.title) : ['Standup 9:00', 'Lunch 12:30'],
)
</script>

<template>
  <div class="h-full w-full overflow-hidden bg-white p-2 text-xs">
    <div v-if="isWeek" class="grid grid-cols-7 gap-1">
      <div v-for="d in days" :key="d" data-role="day" class="border p-1 text-center">{{ d }}</div>
    </div>
    <div v-else>
      <div class="mb-1 font-bold">Today</div>
      <div v-for="(r, i) in rows" :key="i" data-role="event" class="border-b py-0.5">{{ r }}</div>
    </div>
  </div>
</template>
```

- [ ] **Step 5: Write ImageWidget.vue**

Create `src/components/widgets/ImageWidget.vue`:

```vue
<script setup lang="ts">
import type { ImageEl } from '@/stores/types'

defineProps<{ el: ImageEl }>()
</script>

<template>
  <img v-if="el.src" :src="el.src" class="h-full w-full object-contain" alt="" />
  <div
    v-else
    data-role="placeholder"
    class="flex h-full w-full items-center justify-center border-2 border-dashed border-neutral-400 bg-neutral-50 text-xs text-neutral-500"
  >
    Image
  </div>
</template>
```

- [ ] **Step 6: Write DrawingLayer.vue**

Create `src/components/widgets/DrawingLayer.vue`:

```vue
<script setup lang="ts">
defineProps<{ size: { w: number; h: number } }>()
</script>

<template>
  <!-- Placeholder overlay this pass; stroke capture comes later. -->
  <canvas
    :width="size.w"
    :height="size.h"
    class="pointer-events-none absolute inset-0 h-full w-full"
  />
</template>
```

- [ ] **Step 7: Run the tests to verify they pass**

Run: `npm test -- widgets`
Expected: PASS (3 tests).

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: add clock, calendar, image widgets and drawing-layer placeholder"
```

---

### Task 7: Build the EditorCanvas

**Files:**
- Create: `src/components/EditorCanvas.vue`
- Test: `src/components/EditorCanvas.test.ts`

**Interfaces:**
- Consumes: `usePagesStore`, `MovableElement`, the four widgets, `DrawingLayer`.
- Produces: `EditorCanvas.vue` — no props; reads everything from `usePagesStore`. Renders a letterboxed page surface sized to `pageSize`, scaled to fit a `ref`-measured container. Each element renders inside a `MovableElement`; selecting/dragging updates the store. Clicking empty canvas clears selection. Exposes nothing.

- [ ] **Step 1: Write a failing test for EditorCanvas**

Create `src/components/EditorCanvas.test.ts`:

```ts
import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import EditorCanvas from './EditorCanvas.vue'
import { usePagesStore } from '@/stores/pages'

beforeEach(() => setActivePinia(createPinia()))

describe('EditorCanvas', () => {
  it('renders one MovableElement per element on the selected page', async () => {
    const store = usePagesStore()
    store.addElement({ id: 'e1', type: 'clock', variant: 'time', x: 0, y: 0, w: 200, h: 80 })
    const w = mount(EditorCanvas, { global: { plugins: [] } })
    await w.vm.$nextTick()
    expect(w.findAll('[data-role="movable"]').length).toBe(1)
  })

  it('clears selection when the empty surface is clicked', async () => {
    const store = usePagesStore()
    store.addElement({ id: 'e1', type: 'clock', variant: 'time', x: 0, y: 0, w: 200, h: 80 })
    expect(store.selectedElId).toBe('e1')
    const w = mount(EditorCanvas)
    await w.get('[data-role="surface"]').trigger('pointerdown')
    expect(store.selectedElId).toBe(null)
  })
})
```

Note: `setActivePinia` already installs the store globally, so no `plugins` array is needed; the first test's `global.plugins` is harmless. Keep both tests as written.

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- EditorCanvas`
Expected: FAIL with "Cannot find module './EditorCanvas.vue'".

- [ ] **Step 3: Write EditorCanvas.vue**

Create `src/components/EditorCanvas.vue`:

```vue
<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { usePagesStore } from '@/stores/pages'
import MovableElement from './MovableElement.vue'
import ClockWidget from './widgets/ClockWidget.vue'
import CalendarWidget from './widgets/CalendarWidget.vue'
import ImageWidget from './widgets/ImageWidget.vue'
import DrawingLayer from './widgets/DrawingLayer.vue'

const store = usePagesStore()
const container = ref<HTMLElement | null>(null)
const scale = ref(1)

const size = computed(() => store.pageSize)
const elements = computed(() => store.selectedPage?.elements ?? [])

function recompute() {
  const el = container.value
  if (!el) return
  const fit = Math.min(el.clientWidth / size.value.w, el.clientHeight / size.value.h)
  scale.value = fit > 0 ? fit : 1
}

let ro: ResizeObserver | null = null
onMounted(() => {
  recompute()
  if (typeof ResizeObserver !== 'undefined' && container.value) {
    ro = new ResizeObserver(recompute)
    ro.observe(container.value)
  }
})
onBeforeUnmount(() => ro?.disconnect())

function clearSelection() {
  store.selectElement(null)
}
</script>

<template>
  <div ref="container" class="flex h-full w-full items-center justify-center overflow-hidden bg-neutral-200">
    <div
      data-role="surface"
      class="relative bg-white shadow"
      :style="{
        width: `${size.w}px`,
        height: `${size.h}px`,
        transform: `scale(${scale})`,
        transformOrigin: 'center',
      }"
      @pointerdown.self="clearSelection"
    >
      <MovableElement
        v-for="el in elements"
        :key="el.id"
        data-role="movable"
        :id="el.id"
        :x="el.x"
        :y="el.y"
        :w="el.w"
        :h="el.h"
        :selected="store.selectedElId === el.id"
        :scale="scale"
        @select="store.selectElement($event)"
        @update="store.updateElement(el.id, $event)"
      >
        <ClockWidget v-if="el.type === 'clock'" :el="el" />
        <CalendarWidget v-else-if="el.type === 'calendar'" :el="el" />
        <ImageWidget v-else-if="el.type === 'image'" :el="el" />
      </MovableElement>
      <DrawingLayer :size="size" />
    </div>
  </div>
</template>
```

- [ ] **Step 4: Pass the `data-role` through MovableElement's root**

`data-role="movable"` is set on `MovableElement` in the template above, but Vue applies fallthrough attributes to the component root only when there is a single root element. `MovableElement.vue` has a single root `<div>`, so `data-role="movable"` lands on it automatically. No change needed — confirm by re-reading `MovableElement.vue` from Task 5 (single root div).

- [ ] **Step 5: Run the test to verify it passes**

Run: `npm test -- EditorCanvas`
Expected: PASS (2 tests).

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add EditorCanvas with letterboxed scaling and selection"
```

---

### Task 8: Build the ToolRail and tool option popouts

**Files:**
- Create: `src/components/ToolRail.vue`, `src/components/ToolOptions/ClockOptions.vue`, `src/components/ToolOptions/CalendarOptions.vue`, `src/components/ToolOptions/DrawOptions.vue`, `src/components/ToolOptions/ImageOptions.vue`
- Test: `src/components/ToolRail.test.ts`

**Interfaces:**
- Consumes: `usePagesStore`, `useToolOptionsStore` + `ensureToolOptionsPersistence`, `makeElement`, shadcn-vue `Popover`/`Tooltip`/`Button`, lucide icons.
- Produces: `ToolRail.vue` — vertical icon buttons (Select, Clock, Calendar, Draw, Image). Clicking a tool calls `store.setActiveTool` and opens its popover; choosing Clock/Calendar/Image also adds a default element via `makeElement` using current options. The Draw icon is tinted with `drawColour`; the Clock icon swaps glyph by `clockVariant`. Each `*Options.vue` edits its slice of `useToolOptionsStore`.

- [ ] **Step 1: Write a failing test for ToolRail**

Create `src/components/ToolRail.test.ts`:

```ts
import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import ToolRail from './ToolRail.vue'
import { usePagesStore } from '@/stores/pages'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

describe('ToolRail', () => {
  it('selecting the clock tool adds a clock element to the page', async () => {
    const store = usePagesStore()
    const w = mount(ToolRail)
    await w.get('[data-tool="clock"]').trigger('click')
    expect(store.activeTool).toBe('clock')
    expect(store.selectedPage?.elements.some((e) => e.type === 'clock')).toBe(true)
  })

  it('selecting the select tool does not add an element', async () => {
    const store = usePagesStore()
    const w = mount(ToolRail)
    await w.get('[data-tool="select"]').trigger('click')
    expect(store.activeTool).toBe('select')
    expect(store.selectedPage?.elements.length).toBe(0)
  })
})
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- ToolRail`
Expected: FAIL with "Cannot find module './ToolRail.vue'".

- [ ] **Step 3: Write the four option components**

Create `src/components/ToolOptions/ClockOptions.vue`:

```vue
<script setup lang="ts">
import { useToolOptionsStore } from '@/stores/toolOptions'
import { Clock, CalendarClock, Calendar } from '@lucide/vue'

const opts = useToolOptionsStore()
const variants = [
  { id: 'time', icon: Clock, label: 'Time' },
  { id: 'time-date', icon: CalendarClock, label: 'Time + date' },
  { id: 'date', icon: Calendar, label: 'Date' },
] as const
</script>

<template>
  <div class="flex flex-col gap-1">
    <p class="mb-1 text-xs font-medium text-neutral-500">Clock style</p>
    <button
      v-for="v in variants"
      :key="v.id"
      class="flex items-center gap-2 rounded px-2 py-1 text-sm hover:bg-neutral-100"
      :class="opts.clockVariant === v.id ? 'bg-neutral-100 font-medium' : ''"
      @click="opts.clockVariant = v.id"
    >
      <component :is="v.icon" class="h-4 w-4" />
      {{ v.label }}
    </button>
  </div>
</template>
```

Create `src/components/ToolOptions/CalendarOptions.vue`:

```vue
<script setup lang="ts">
import { useToolOptionsStore } from '@/stores/toolOptions'

const opts = useToolOptionsStore()
const variants = [
  { id: 'today', label: 'Today' },
  { id: 'week', label: 'Week' },
] as const
</script>

<template>
  <div class="flex flex-col gap-1">
    <p class="mb-1 text-xs font-medium text-neutral-500">Calendar view</p>
    <button
      v-for="v in variants"
      :key="v.id"
      class="rounded px-2 py-1 text-left text-sm hover:bg-neutral-100"
      :class="opts.calendarVariant === v.id ? 'bg-neutral-100 font-medium' : ''"
      @click="opts.calendarVariant = v.id"
    >
      {{ v.label }}
    </button>
  </div>
</template>
```

Create `src/components/ToolOptions/DrawOptions.vue`:

```vue
<script setup lang="ts">
import { useToolOptionsStore } from '@/stores/toolOptions'
import type { EpaperColour } from '@/stores/types'

const opts = useToolOptionsStore()
const palette: EpaperColour[] = ['black', 'white', 'red', 'yellow', 'blue', 'green']
const sizes = [2, 4, 8, 12]
</script>

<template>
  <div class="flex flex-col gap-2">
    <div>
      <p class="mb-1 text-xs font-medium text-neutral-500">Colour</p>
      <div class="flex gap-1">
        <button
          v-for="c in palette"
          :key="c"
          class="h-5 w-5 rounded-full border"
          :class="opts.drawColour === c ? 'ring-2 ring-blue-500 ring-offset-1' : ''"
          :style="{ backgroundColor: c }"
          @click="opts.drawColour = c"
        />
      </div>
    </div>
    <div>
      <p class="mb-1 text-xs font-medium text-neutral-500">Pen size</p>
      <div class="flex gap-1">
        <button
          v-for="s in sizes"
          :key="s"
          class="rounded px-2 py-1 text-xs hover:bg-neutral-100"
          :class="opts.penSize === s ? 'bg-neutral-100 font-medium' : ''"
          @click="opts.penSize = s"
        >
          {{ s }}
        </button>
      </div>
    </div>
  </div>
</template>
```

Create `src/components/ToolOptions/ImageOptions.vue`:

```vue
<script setup lang="ts">
// Placeholder this pass — real upload comes later (device wiring out of scope).
</script>

<template>
  <div class="text-xs text-neutral-500">
    Image upload is stubbed in this pass. A blank image placeholder is added to the page.
  </div>
</template>
```

- [ ] **Step 4: Write ToolRail.vue**

Create `src/components/ToolRail.vue`:

```vue
<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { usePagesStore } from '@/stores/pages'
import { useToolOptionsStore, ensureToolOptionsPersistence } from '@/stores/toolOptions'
import { makeElement } from '@/stores/elementFactory'
import type { ToolId } from '@/stores/types'
import { Popover, PopoverTrigger, PopoverContent } from '@/components/ui/popover'
import { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider } from '@/components/ui/tooltip'
import {
  MousePointer2, Clock, CalendarClock, Calendar, Pencil, Image as ImageIcon,
} from '@lucide/vue'
import ClockOptions from './ToolOptions/ClockOptions.vue'
import CalendarOptions from './ToolOptions/CalendarOptions.vue'
import DrawOptions from './ToolOptions/DrawOptions.vue'
import ImageOptions from './ToolOptions/ImageOptions.vue'

const store = usePagesStore()
const opts = useToolOptionsStore()
onMounted(() => ensureToolOptionsPersistence())

const clockGlyph = computed(() =>
  opts.clockVariant === 'date' ? Calendar : opts.clockVariant === 'time-date' ? CalendarClock : Clock,
)

function pickTool(tool: ToolId) {
  store.setActiveTool(tool)
  if (tool === 'clock' || tool === 'calendar' || tool === 'image') {
    store.addElement(
      makeElement(tool, { clockVariant: opts.clockVariant, calendarVariant: opts.calendarVariant }, store.pageSize),
    )
  }
}
</script>

<template>
  <TooltipProvider>
    <div class="flex w-12 flex-col items-center gap-1 border-r bg-neutral-50 py-2">
      <!-- Select -->
      <Tooltip>
        <TooltipTrigger as-child>
          <button
            data-tool="select"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'select' ? 'bg-neutral-200' : ''"
            @click="pickTool('select')"
          >
            <MousePointer2 class="h-5 w-5" />
          </button>
        </TooltipTrigger>
        <TooltipContent side="right">Select</TooltipContent>
      </Tooltip>

      <!-- Clock -->
      <Popover>
        <PopoverTrigger as-child>
          <button
            data-tool="clock"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'clock' ? 'bg-neutral-200' : ''"
            @click="pickTool('clock')"
          >
            <component :is="clockGlyph" class="h-5 w-5" />
          </button>
        </PopoverTrigger>
        <PopoverContent side="right" class="w-44"><ClockOptions /></PopoverContent>
      </Popover>

      <!-- Calendar -->
      <Popover>
        <PopoverTrigger as-child>
          <button
            data-tool="calendar"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'calendar' ? 'bg-neutral-200' : ''"
            @click="pickTool('calendar')"
          >
            <Calendar class="h-5 w-5" />
          </button>
        </PopoverTrigger>
        <PopoverContent side="right" class="w-44"><CalendarOptions /></PopoverContent>
      </Popover>

      <!-- Draw -->
      <Popover>
        <PopoverTrigger as-child>
          <button
            data-tool="draw"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'draw' ? 'bg-neutral-200' : ''"
            @click="pickTool('draw')"
          >
            <Pencil class="h-5 w-5" :style="{ color: opts.drawColour }" />
          </button>
        </PopoverTrigger>
        <PopoverContent side="right" class="w-48"><DrawOptions /></PopoverContent>
      </Popover>

      <!-- Image -->
      <Popover>
        <PopoverTrigger as-child>
          <button
            data-tool="image"
            class="flex h-9 w-9 items-center justify-center rounded hover:bg-neutral-200"
            :class="store.activeTool === 'image' ? 'bg-neutral-200' : ''"
            @click="pickTool('image')"
          >
            <ImageIcon class="h-5 w-5" />
          </button>
        </PopoverTrigger>
        <PopoverContent side="right" class="w-48"><ImageOptions /></PopoverContent>
      </Popover>
    </div>
  </TooltipProvider>
</template>
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `npm test -- ToolRail`
Expected: PASS (2 tests).

If shadcn-vue's `Tooltip`/`Popover` warn about missing provider context in jsdom, the tests still pass because they only click `[data-tool=...]`. If a render error occurs, confirm `reka-ui` is installed (Task 2 Step 1).

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add tool rail with option popouts and icon reflection"
```

---

### Task 9: Build the PageSidebar and PageThumbnail

**Files:**
- Create: `src/components/PageSidebar.vue`, `src/components/PageThumbnail.vue`
- Test: `src/components/PageSidebar.test.ts`

**Interfaces:**
- Consumes: `usePagesStore`, the widgets (for thumbnail rendering), shadcn-vue `Button`, lucide `Plus`.
- Produces:
  - `PageThumbnail.vue` — props `{ pageId: string }`. Renders a scaled-down read-only copy of the page (same widget DOM under `transform: scale`), draggable (`draggable="true"`) with `dragstart` setting `dataTransfer` text to the page id.
  - `PageSidebar.vue` — lists thumbnails (click selects, highlights the selected one) and an add-page button.

- [ ] **Step 1: Write a failing test for PageSidebar**

Create `src/components/PageSidebar.test.ts`:

```ts
import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import PageSidebar from './PageSidebar.vue'
import { usePagesStore } from '@/stores/pages'

beforeEach(() => setActivePinia(createPinia()))

describe('PageSidebar', () => {
  it('shows one thumbnail per page', () => {
    const store = usePagesStore()
    store.addPage()
    const w = mount(PageSidebar)
    expect(w.findAll('[data-role="thumb"]').length).toBe(2)
  })

  it('the add button adds a page', async () => {
    const store = usePagesStore()
    const w = mount(PageSidebar)
    await w.get('[data-role="add-page"]').trigger('click')
    expect(store.pages.length).toBe(2)
  })

  it('clicking a thumbnail selects that page', async () => {
    const store = usePagesStore()
    const second = store.addPage()
    store.selectPage(store.pages[0].id)
    const w = mount(PageSidebar)
    await w.findAll('[data-role="thumb"]')[1].trigger('click')
    expect(store.selectedPageId).toBe(second)
  })
})
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- PageSidebar`
Expected: FAIL with "Cannot find module './PageSidebar.vue'".

- [ ] **Step 3: Write PageThumbnail.vue**

Create `src/components/PageThumbnail.vue`:

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { usePagesStore } from '@/stores/pages'
import ClockWidget from './widgets/ClockWidget.vue'
import CalendarWidget from './widgets/CalendarWidget.vue'
import ImageWidget from './widgets/ImageWidget.vue'

const props = defineProps<{ pageId: string }>()
const store = usePagesStore()

const THUMB_W = 120
const page = computed(() => store.pages.find((p) => p.id === props.pageId) ?? null)
const size = computed(() => store.pageSize)
const scale = computed(() => THUMB_W / size.value.w)

function onDragStart(e: DragEvent) {
  e.dataTransfer?.setData('text/plain', props.pageId)
}
</script>

<template>
  <div
    class="relative overflow-hidden border bg-white"
    :style="{ width: `${THUMB_W}px`, height: `${size.h * scale}px` }"
    draggable="true"
    @dragstart="onDragStart"
  >
    <div
      class="absolute left-0 top-0 origin-top-left"
      :style="{ width: `${size.w}px`, height: `${size.h}px`, transform: `scale(${scale})` }"
    >
      <div
        v-for="el in page?.elements ?? []"
        :key="el.id"
        class="absolute"
        :style="{ left: `${el.x}px`, top: `${el.y}px`, width: `${el.w}px`, height: `${el.h}px` }"
      >
        <ClockWidget v-if="el.type === 'clock'" :el="el" />
        <CalendarWidget v-else-if="el.type === 'calendar'" :el="el" />
        <ImageWidget v-else-if="el.type === 'image'" :el="el" />
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 4: Write PageSidebar.vue**

Create `src/components/PageSidebar.vue`:

```vue
<script setup lang="ts">
import { usePagesStore } from '@/stores/pages'
import PageThumbnail from './PageThumbnail.vue'
import { Plus } from '@lucide/vue'

const store = usePagesStore()
</script>

<template>
  <div class="flex w-36 flex-col gap-2 overflow-y-auto border-r bg-neutral-100 p-2">
    <div
      v-for="p in store.pages"
      :key="p.id"
      data-role="thumb"
      class="cursor-pointer rounded p-0.5"
      :class="store.selectedPageId === p.id ? 'ring-2 ring-blue-500' : ''"
      @click="store.selectPage(p.id)"
    >
      <PageThumbnail :page-id="p.id" />
      <p class="mt-0.5 truncate text-center text-xs text-neutral-600">{{ p.name }}</p>
    </div>
    <button
      data-role="add-page"
      class="flex items-center justify-center gap-1 rounded border border-dashed py-2 text-sm text-neutral-600 hover:bg-neutral-200"
      @click="store.addPage()"
    >
      <Plus class="h-4 w-4" /> Add page
    </button>
  </div>
</template>
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `npm test -- PageSidebar`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add page sidebar with scaled thumbnails and add-page"
```

---

### Task 10: Build the Timeline and TimelineItem

**Files:**
- Create: `src/components/Timeline.vue`, `src/components/TimelineItem.vue`
- Test: `src/components/Timeline.test.ts`

**Interfaces:**
- Consumes: `usePagesStore`, lucide `X`.
- Produces:
  - `TimelineItem.vue` — props `{ index: number; pageId: string; delayMs: number }`. Shows the page name, a delay input in seconds (emits `setDelay` with milliseconds), a remove button (emits `remove`), and is `draggable` for reordering (sets `dataTransfer` to its index).
  - `Timeline.vue` — a horizontal strip. Accepts a drop from a sidebar thumbnail (page id string) to append via `addToTimeline`, and a drop from another timeline item (numeric index) to reorder via `reorderTimeline`. Renders one `TimelineItem` per entry.

- [ ] **Step 1: Write a failing test for the Timeline**

Create `src/components/Timeline.test.ts`:

```ts
import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import Timeline from './Timeline.vue'
import { usePagesStore } from '@/stores/pages'

beforeEach(() => setActivePinia(createPinia()))

describe('Timeline', () => {
  it('renders one item per timeline entry', () => {
    const store = usePagesStore()
    store.addToTimeline(store.pages[0].id)
    const w = mount(Timeline)
    expect(w.findAll('[data-role="timeline-item"]').length).toBe(1)
  })

  it('dropping a page id appends it to the timeline', async () => {
    const store = usePagesStore()
    const pid = store.pages[0].id
    const w = mount(Timeline)
    const dt = { getData: (t: string) => (t === 'text/plain' ? pid : '') }
    await w.get('[data-role="timeline-strip"]').trigger('drop', { dataTransfer: dt })
    expect(store.timeline.map((e) => e.pageId)).toEqual([pid])
  })

  it('setting a delay updates the store in milliseconds', async () => {
    const store = usePagesStore()
    store.addToTimeline(store.pages[0].id)
    const w = mount(Timeline)
    const input = w.get('[data-role="delay-input"]')
    await input.setValue('8')
    expect(store.timeline[0].delayMs).toBe(8000)
  })
})
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- Timeline`
Expected: FAIL with "Cannot find module './Timeline.vue'".

- [ ] **Step 3: Write TimelineItem.vue**

Create `src/components/TimelineItem.vue`:

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { usePagesStore } from '@/stores/pages'
import { X } from '@lucide/vue'

const props = defineProps<{ index: number; pageId: string; delayMs: number }>()
const emit = defineEmits<{ setDelay: [ms: number]; remove: []; reorderFrom: [index: number] }>()

const store = usePagesStore()
const name = computed(() => store.pages.find((p) => p.id === props.pageId)?.name ?? 'Page')
const seconds = computed(() => Math.round(props.delayMs / 1000))

function onDelay(e: Event) {
  const v = Number((e.target as HTMLInputElement).value)
  emit('setDelay', Math.max(0, v) * 1000)
}
function onDragStart(e: DragEvent) {
  e.dataTransfer?.setData('text/plain', `idx:${props.index}`)
}
</script>

<template>
  <div
    data-role="timeline-item"
    class="flex shrink-0 flex-col items-center gap-1 rounded border bg-white p-2"
    draggable="true"
    @dragstart="onDragStart"
  >
    <div class="flex items-center gap-1">
      <span class="text-xs font-medium">{{ name }}</span>
      <button class="text-neutral-400 hover:text-red-500" @click="emit('remove')">
        <X class="h-3 w-3" />
      </button>
    </div>
    <label class="flex items-center gap-1 text-xs text-neutral-500">
      <input
        data-role="delay-input"
        type="number"
        min="0"
        class="w-12 rounded border px-1 text-right"
        :value="seconds"
        @change="onDelay"
      />
      s
    </label>
  </div>
</template>
```

- [ ] **Step 4: Write Timeline.vue**

Create `src/components/Timeline.vue`:

```vue
<script setup lang="ts">
import { usePagesStore } from '@/stores/pages'
import TimelineItem from './TimelineItem.vue'

const store = usePagesStore()

function onDrop(e: DragEvent) {
  const data = e.dataTransfer?.getData('text/plain') ?? ''
  if (data.startsWith('idx:')) {
    const from = Number(data.slice(4))
    store.reorderTimeline(from, store.timeline.length - 1)
  } else if (data) {
    store.addToTimeline(data)
  }
}
</script>

<template>
  <div
    data-role="timeline-strip"
    class="flex h-24 items-center gap-2 overflow-x-auto border-t bg-neutral-50 p-2"
    @dragover.prevent
    @drop.prevent="onDrop"
  >
    <p v-if="!store.timeline.length" class="text-xs text-neutral-400">
      Drag pages here to set the loop order.
    </p>
    <TimelineItem
      v-for="(entry, i) in store.timeline"
      :key="`${entry.pageId}-${i}`"
      :index="i"
      :page-id="entry.pageId"
      :delay-ms="entry.delayMs"
      @set-delay="store.setTimelineDelay(i, $event)"
      @remove="store.removeFromTimeline(i)"
    />
  </div>
</template>
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `npm test -- Timeline`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add timeline with drop-to-add, reorder, and per-page delay"
```

---

### Task 11: Build the TopBar, assemble App.vue, and add the integration smoke test

**Files:**
- Create: `src/components/TopBar.vue`
- Modify: `src/App.vue`, `src/smoke.test.ts`
- Test: `src/App.test.ts`

**Interfaces:**
- Consumes: every component built so far, `usePagesStore`.
- Produces:
  - `TopBar.vue` — app title, an orientation toggle button (calls `toggleOrientation`, shows current orientation), and a Publish button that shows a transient toast message (in-component `ref`, no network).
  - `App.vue` — the grid shell: TopBar across the top, PageSidebar + ToolRail + EditorCanvas in the middle row, Timeline across the bottom.

- [ ] **Step 1: Write TopBar.vue**

Create `src/components/TopBar.vue`:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { usePagesStore } from '@/stores/pages'
import { Button } from '@/components/ui/button'
import { RectangleHorizontal, RectangleVertical } from '@lucide/vue'

const store = usePagesStore()
const toast = ref<string | null>(null)

function publish() {
  // Stubbed this pass — no network call.
  toast.value = 'Published (stub)'
  setTimeout(() => (toast.value = null), 1500)
}
</script>

<template>
  <header class="flex h-12 items-center justify-between border-b bg-white px-3">
    <h1 class="text-sm font-semibold">Corkboard</h1>
    <div class="flex items-center gap-2">
      <span v-if="toast" data-role="toast" class="text-xs text-green-600">{{ toast }}</span>
      <button
        data-role="orientation"
        class="flex items-center gap-1 rounded border px-2 py-1 text-xs hover:bg-neutral-100"
        @click="store.toggleOrientation()"
      >
        <component :is="store.orientation === 'landscape' ? RectangleHorizontal : RectangleVertical" class="h-4 w-4" />
        {{ store.orientation }}
      </button>
      <Button data-role="publish" size="sm" @click="publish">Publish</Button>
    </div>
  </header>
</template>
```

- [ ] **Step 2: Replace App.vue with the grid shell**

Overwrite `src/App.vue`:

```vue
<script setup lang="ts">
import TopBar from './components/TopBar.vue'
import PageSidebar from './components/PageSidebar.vue'
import ToolRail from './components/ToolRail.vue'
import EditorCanvas from './components/EditorCanvas.vue'
import Timeline from './components/Timeline.vue'
</script>

<template>
  <div class="flex h-full w-full flex-col">
    <TopBar />
    <div class="flex min-h-0 flex-1">
      <PageSidebar />
      <ToolRail />
      <div class="min-w-0 flex-1">
        <EditorCanvas />
      </div>
    </div>
    <Timeline />
  </div>
</template>
```

- [ ] **Step 3: Replace the placeholder smoke test with an integration test**

The Task 1 `src/smoke.test.ts` mounted the template `App.vue`; the new `App.vue` pulls in stores, so give it Pinia. Overwrite `src/smoke.test.ts`:

```ts
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import App from './App.vue'

describe('App', () => {
  it('mounts the full shell without throwing', () => {
    setActivePinia(createPinia())
    const wrapper = mount(App, { global: { plugins: [createPinia()] } })
    expect(wrapper.exists()).toBe(true)
  })
})
```

- [ ] **Step 4: Write an App integration test covering an end-to-end interaction**

Create `src/App.test.ts`:

```ts
import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import App from './App.vue'
import { usePagesStore } from '@/stores/pages'

beforeEach(() => {
  localStorage.clear()
  setActivePinia(createPinia())
})

describe('App integration', () => {
  it('adds a clock element when the clock tool is picked, and toggles orientation', async () => {
    const w = mount(App)
    const store = usePagesStore()

    await w.get('[data-tool="clock"]').trigger('click')
    expect(store.selectedPage?.elements.some((e) => e.type === 'clock')).toBe(true)

    expect(store.orientation).toBe('landscape')
    await w.get('[data-role="orientation"]').trigger('click')
    expect(store.orientation).toBe('portrait')
  })

  it('publish shows a toast', async () => {
    const w = mount(App)
    await w.get('[data-role="publish"]').trigger('click')
    expect(w.get('[data-role="toast"]').text()).toContain('Published')
  })
})
```

Note: `mount(App)` with `setActivePinia` already active means the components resolve the same store the test inspects. Don't pass a second `createPinia()` here — that would create a different active instance than the one the test reads.

- [ ] **Step 5: Run the full test suite**

Run: `npm test`
Expected: PASS — all suites (smoke, button, pages, toolOptions, elementFactory, useDraggableResizable, widgets, EditorCanvas, ToolRail, PageSidebar, Timeline, App).

- [ ] **Step 6: Verify the production build**

Run: `npm run build`
Expected: build succeeds with no type errors.

- [ ] **Step 7: Manual smoke check (optional but recommended)**

Run: `npm run dev`, open the served URL. Confirm: tools add elements; elements drag/resize; sidebar adds/selects pages; dragging a thumbnail to the timeline adds an entry; delay input edits; orientation toggle flips the canvas; Publish shows a toast.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: assemble app shell with topbar, publish stub, and integration tests"
```

---

## Post-Implementation: update the mex scaffold (GROW)

After Task 11 lands, run the GROW step from `.mex/ROUTER.md`:

- [ ] **Ground:** The web UI now exists (Vite + Vue + shadcn-vue editor, frontend-only) with the Pinia document shape as the draft page-state contract.
- [ ] **Record:** Update `.mex/ROUTER.md` "Current Project State" — move the web UI from "Not yet built" to "Working" (note: frontend-only, no device wiring). Update `.mex/context/setup.md` with the real `npm` commands (`npm install`, `npm run dev`, `npm test`, `npm run build`) and remove the `[VERIFY AFTER FIRST IMPLEMENTATION]` markers that now hold. Update `.mex/context/stack.md` and `conventions.md` to replace `[TO BE DETERMINED]` slots that the implementation settled (Pinia, Vitest, file/naming conventions, drag/resize hand-rolled).
- [ ] **Orient:** Verify `.mex/patterns/add-page-component.md` against the real component-adding flow now that it exists; update its `[VERIFY AFTER FIRST IMPLEMENTATION]` steps.
- [ ] **Write:** Bump `last_updated` on every scaffold file changed. Run `mex log --type decision` if a notable decision was settled (e.g. "hand-rolled drag/resize, no library").

---

## Self-Review

**1. Spec coverage:**
- Stack (Vite/Vue/TS, Tailwind+shadcn-vue, Pinia, lucide, hand-rolled drag/resize) → Tasks 1, 2, 3, 5.
- Target device constraints (800×480 / 480×800, 6-colour palette, white bg, letterbox scaling) → Task 3 `pageSize`, Task 7 `DrawOptions` palette, Task 6 white widget bg, Task 7 EditorCanvas letterbox.
- Layout (TopBar, Pages sidebar, Tool rail, Editor canvas, Timeline) → Tasks 11, 9, 8, 7, 10.
- Tool options as popouts (select+open, persisted to localStorage, icon reflects choice) → Tasks 4, 8.
- Components list → all covered across Tasks 5–11.
- State shape (DocState + separate persisted options slice) → Tasks 3, 4.
- Interactions (select page, add element centred via current options, drag/resize, popout editing, timeline drag/reorder/delay, orientation toggle, publish toast) → Tasks 7, 8, 9, 10, 11.
- Out of scope (no device wiring, drawing engine placeholder, image stub, no JSON contract, no auth) → respected: DrawingLayer/ImageOptions placeholders, Publish stub, no network code.
- Testing (store sanity tests + App smoke render) → Tasks 3, 4, 11; plus per-component tests.

**2. Placeholder scan:** No "TBD/TODO/handle edge cases" left as work items; the only "placeholder/stub" mentions describe deliberately out-of-scope features (drawing engine, image upload, publish network) per the spec.

**3. Type consistency:** Types defined once in Task 3 `src/stores/types.ts` and imported everywhere. Store action names (`addPage`, `addElement`, `selectElement`, `updateElement`, `addToTimeline`, `reorderTimeline`, `setTimelineDelay`, `removeFromTimeline`, `toggleOrientation`, `setActiveTool`, `selectPage`) are used consistently in Tasks 7–11. `makeElement` signature is consistent between Task 4 (definition) and Tasks 7/8 (use). `useDraggableResizable` opts (`getRect`/`onUpdate`/`scale`) consistent between Task 5 definition and MovableElement use. `ensureToolOptionsPersistence` consistent between Task 4 and Task 8.
