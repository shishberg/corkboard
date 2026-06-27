# Editor Surgery 1a — Data-model migration + remove clock & timeline (Implementation Plan)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate the editor's document model to the round-two shape — drop the clock element and the timeline, add `livePageId` — leaving a working editor with calendar / image / drawing elements.

**Architecture:** The editor is a Vite + Vue 3 + Pinia app. `usePagesStore` (`src/stores/pages.ts`) holds `DocState`; element types are a discriminated union on `type` in `src/stores/types.ts`. This plan is subtractive plus one new field: it removes `ClockEl`, the `clock` tool, the `timeline`, and their components, and adds `livePageId` (which page the device displays). No network, no Rust — pure frontend.

**Tech Stack:** TypeScript, Vue 3 (`<script setup>`), Pinia, Vitest + @vue/test-utils + jsdom. Build/type-check via `vue-tsc -b` (`npm run build`).

## Global Constraints

- Shared types come only from `@/stores/types` — no local redefinition.
- Tests are colocated `*.test.ts` and ARE type-checked; keep them compiling.
- jsdom runs at `scale=1` (`getBoundingClientRect` returns zeros); do not assert real-scale coordinates in jsdom.
- Stage only the files each task changed — never `git add -A`. End commit messages with `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.
- `npm test` and `npm run build` must both be green at the end of every task.
- This is round two: the device displays one **live** page (`livePageId`); there is no page loop. Element types after this plan: `calendar | image | drawing`.

---

### Task 1: Migrate `types.ts` and the store (drop `timeline`/`ClockEl`, add `livePageId`)

**Files:**
- Modify: `src/stores/types.ts`
- Modify: `src/stores/pages.ts`
- Test: `src/stores/pages.test.ts`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `DocState` without `timeline`; with `livePageId: string | null`.
  - `ToolId = 'select' | 'calendar' | 'draw' | 'image'` (no `'clock'`).
  - No `ClockEl`; `El = CalendarEl | ImageEl | DrawingEl`.
  - Store actions: `setLivePage(id: string)`; getter `livePage: Page | null`. Removed: `addToTimeline`, `reorderTimeline`, `setTimelineDelay`, `removeFromTimeline`.

- [ ] **Step 1: Update `src/stores/types.ts`**

Remove the `ClockEl` interface and the `'clock'` arm of `ToolId`, drop `ClockEl` from `El`, and change `DocState`. Resulting relevant lines:

```typescript
export type ToolId = 'select' | 'calendar' | 'draw' | 'image'
// (ClockEl interface deleted)
export type El = CalendarEl | ImageEl | DrawingEl

export interface DocState {
  orientation: Orientation
  pages: Page[]
  livePageId: string | null
  selectedPageId: string | null
  selectedElId: string | null
  activeTool: ToolId
}
```

Leave `CalendarEl`, `ImageEl`, `DrawingEl`, `TimelineEntry` definitions untouched for now except: delete `TimelineEntry` (it's only used by the timeline). Keep `CalEvent` (still referenced by `CalendarEl` until plan 1b).

- [ ] **Step 2: Rewrite the timeline tests in `src/stores/pages.test.ts` as `livePageId` tests**

The `clockEl` helper and all `*Timeline*` tests reference removed API. Replace the helper with an image element, and swap the timeline test block for `livePageId` tests. Change the import line and helper:

```typescript
import type { ImageEl, DrawingEl } from './types'

function imageEl(id: string): ImageEl {
  return { id, type: 'image', x: 0, y: 0, w: 200, h: 150, colour: 'black', src: '' }
}
```

Then replace every `clockEl(` call in the file with `imageEl(`, and in the "patch cannot change id or type" test change the expected `type` from `'clock'` to `'image'`. Delete these tests entirely (they call removed actions): `addToTimeline appends...`, `reorderTimeline moves an entry`, `setTimelineDelay updates one entry`, all three `reorderTimeline:` guard tests, `addToTimeline: ignores unknown pageId`, and `removeFromTimeline: ignores negative index`. Add this block in their place:

```typescript
  it('starts with the first page live', () => {
    const s = usePagesStore()
    expect(s.livePageId).toBe(s.pages[0].id)
    expect(s.livePage?.id).toBe(s.pages[0].id)
  })

  it('setLivePage changes which page is live', () => {
    const s = usePagesStore()
    const b = s.addPage()
    s.setLivePage(b)
    expect(s.livePageId).toBe(b)
  })

  it('setLivePage ignores an unknown id', () => {
    const s = usePagesStore()
    const before = s.livePageId
    s.setLivePage('ghost')
    expect(s.livePageId).toBe(before)
  })
```

- [ ] **Step 3: Run the store tests to verify they fail**

Run: `npx vitest run src/stores/pages.test.ts`
Expected: FAIL — `livePage`/`setLivePage` don't exist, and `s.timeline` is gone (compile/type errors are fine; the point is red).

- [ ] **Step 4: Update `src/stores/pages.ts`**

In `state`, replace `timeline: []` with `livePageId: first.id`. Add the `livePage` getter next to `selectedPage`:

```typescript
    livePage(state): Page | null {
      return state.pages.find((p) => p.id === state.livePageId) ?? null
    },
```

Delete the four timeline actions (`addToTimeline`, `reorderTimeline`, `setTimelineDelay`, `removeFromTimeline`). Add `setLivePage`:

```typescript
    setLivePage(id: string) {
      if (!this.pages.some((p) => p.id === id)) return
      this.livePageId = id
    },
```

Update the `import type` line to drop `DocState`-unused names if the editor flags them (keep `DocState, El, Page, ToolId, BaseEl, EpaperColour`).

- [ ] **Step 5: Run the store tests to verify they pass**

Run: `npx vitest run src/stores/pages.test.ts`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/stores/types.ts src/stores/pages.ts src/stores/pages.test.ts
git commit -m "refactor(store): drop timeline + ClockEl, add livePageId

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 2: Remove clock from the element factory and tool options

**Files:**
- Modify: `src/stores/elementFactory.ts`
- Modify: `src/stores/toolOptions.ts`
- Test: `src/stores/elementFactory.test.ts`

**Interfaces:**
- Consumes: `ToolId` from Task 1.
- Produces:
  - `makeElement(tool: 'calendar' | 'image', opts, pageSize, rect?)` and `defaultSize(tool: 'calendar' | 'image')`.
  - `FactoryOpts` without `clockVariant`.
  - `ToolOptionsState` without `clockVariant`.

- [ ] **Step 1: Update `src/stores/elementFactory.test.ts`**

Remove any test that constructs a `'clock'` element and any `clockVariant` in opts. Where a test calls `makeElement('clock', ...)`, delete that case. Ensure the remaining `FactoryOpts` literals drop `clockVariant`. (Open the file and remove clock references; the calendar/image/drawing assertions stay.)

- [ ] **Step 2: Run to verify red**

Run: `npx vitest run src/stores/elementFactory.test.ts`
Expected: FAIL to compile (references to removed `clock`).

- [ ] **Step 3: Update `src/stores/elementFactory.ts`**

Narrow the tool unions, drop `clockVariant` and `SIZES.clock` and the `'clock'` case:

```typescript
interface FactoryOpts {
  calendarVariant: 'today' | 'week'
  colour: EpaperColour
}

const SIZES = {
  calendar: { w: 300, h: 220 },
  image: { w: 200, h: 150 },
}

export function defaultSize(tool: 'calendar' | 'image'): { w: number; h: number } {
  return { ...SIZES[tool] }
}

export function makeElement(
  tool: 'calendar' | 'image',
  opts: FactoryOpts,
  pageSize: { w: number; h: number },
  rect?: Rect,
): El {
  const { w, h } = SIZES[tool]
  const geom: Rect = rect ?? { w, h, x: (pageSize.w - w) / 2, y: (pageSize.h - h) / 2 }
  const base = { id: uid(), ...geom, colour: opts.colour }
  switch (tool) {
    case 'calendar':
      return { ...base, type: 'calendar', variant: opts.calendarVariant, events: [] }
    case 'image':
      return { ...base, type: 'image', src: '' }
  }
}
```

- [ ] **Step 4: Update `src/stores/toolOptions.ts`**

Remove `clockVariant` from `ToolOptionsState` and from `defaults`:

```typescript
interface ToolOptionsState {
  calendarVariant: 'today' | 'week'
  colour: EpaperColour
  penSize: number
}

const defaults: ToolOptionsState = {
  calendarVariant: 'today',
  colour: 'black',
  penSize: 4,
}
```

- [ ] **Step 5: Run factory + toolOptions tests to verify green**

Run: `npx vitest run src/stores/elementFactory.test.ts src/stores/toolOptions.test.ts`
Expected: PASS. (If `toolOptions.test.ts` asserts `clockVariant`, remove that assertion.)

- [ ] **Step 6: Commit**

```bash
git add src/stores/elementFactory.ts src/stores/elementFactory.test.ts src/stores/toolOptions.ts src/stores/toolOptions.test.ts
git commit -m "refactor(factory): remove clock element + clockVariant option

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 3: Delete the clock UI and its references

**Files:**
- Delete: `src/components/widgets/ClockWidget.vue`, `src/components/ToolOptions/ClockOptions.vue`
- Modify: `src/components/EditorCanvas.vue`, `src/components/ToolRail.vue`
- Test: `src/components/widgets/widgets.test.ts`, `src/components/ToolRail.test.ts`, `src/components/EditorCanvas.test.ts`

**Interfaces:**
- Consumes: `makeElement` (Task 2), `ToolId` (Task 1).
- Produces: an editor whose tools are `select | calendar | draw | image` and whose element switch handles `calendar | image | drawing`.

- [ ] **Step 1: Update the component tests first**

In `src/components/widgets/widgets.test.ts`: remove the `ClockWidget` import and every test that mounts it. In `src/components/ToolRail.test.ts`: remove any assertion querying `[data-tool="clock"]` (e.g. a "renders a clock tool" test). In `src/components/EditorCanvas.test.ts`: change any test that sets `activeTool = 'clock'` or expects a created clock element to use `'calendar'` (and a calendar element) instead.

- [ ] **Step 2: Run to verify red**

Run: `npx vitest run src/components/widgets/widgets.test.ts src/components/ToolRail.test.ts src/components/EditorCanvas.test.ts`
Expected: FAIL (missing imports / removed `clock`).

- [ ] **Step 3: Delete the clock components**

```bash
git rm src/components/widgets/ClockWidget.vue src/components/ToolOptions/ClockOptions.vue
```

- [ ] **Step 4: Update `src/components/EditorCanvas.vue`**

Remove the `ClockWidget` import (line 7). In the script, change the create-tool typing and checks to drop clock:

```typescript
let createTool: 'calendar' | 'image' = 'calendar'
```

```typescript
function onSurfacePointerDown(e: PointerEvent) {
  const tool = store.activeTool
  if (tool === 'calendar' || tool === 'image') {
    createTool = tool
    createStart = surfaceLocal(e)
    createStartRaw = { x: e.clientX, y: e.clientY }
    const el = makeElement(
      tool,
      { calendarVariant: opts.calendarVariant, colour: opts.colour },
      size.value,
      { x: createStart.x, y: createStart.y, w: CREATE_MIN, h: CREATE_MIN },
    )
    store.addElement(el)
    creatingId = el.id
    window.addEventListener('pointermove', onCreateMove)
    window.addEventListener('pointerup', onCreateUp)
    window.addEventListener('pointercancel', onCreateUp)
  } else {
    clearSelection()
  }
}
```

In the template, delete the `<ClockWidget ... />` line so the switch starts at `CalendarWidget`:

```html
        <CalendarWidget v-if="el.type === 'calendar'" :el="el" />
        <ImageWidget v-else-if="el.type === 'image'" :el="el" />
        <DrawingWidget v-else-if="el.type === 'drawing'" :el="el" />
```

- [ ] **Step 5: Update `src/components/ToolRail.vue`**

Remove the `ClockOptions` import (line 11), remove `Clock` and `CalendarClock` from the `@lucide/vue` import, delete the `clockGlyph` computed (lines 20-22), and delete the entire `<!-- Clock -->` `<Popover>` block (lines 66-79).

- [ ] **Step 6: Run the full suite + build to verify green**

Run: `npm test`
Expected: PASS.
Run: `npm run build`
Expected: clean (ignore the known `@vueuse/core` `#__PURE__` warnings).

- [ ] **Step 7: Commit**

```bash
git add -u src/components/EditorCanvas.vue src/components/ToolRail.vue src/components/widgets/widgets.test.ts src/components/ToolRail.test.ts src/components/EditorCanvas.test.ts
git commit -m "feat(editor): remove the clock tool and widget

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 4: Delete the timeline UI

**Files:**
- Delete: `src/components/Timeline.vue`, `src/components/TimelineItem.vue`, `src/components/Timeline.test.ts`
- Modify: `src/App.vue`
- Test: `src/App.test.ts`

**Interfaces:**
- Consumes: the store from Task 1 (no timeline API).
- Produces: an app shell with no timeline strip.

- [ ] **Step 1: Update `src/App.test.ts`**

Remove any assertion that the app renders the `Timeline` component or a timeline region (e.g. a query for the timeline strip). If `App.test.ts` only checks that the shell mounts, no change is needed beyond removing timeline-specific expectations.

- [ ] **Step 2: Delete the timeline components and their test**

```bash
git rm src/components/Timeline.vue src/components/TimelineItem.vue src/components/Timeline.test.ts
```

- [ ] **Step 3: Update `src/App.vue`**

Remove the `Timeline` import (line 6) and the `<Timeline />` element (line 19):

```html
<script setup lang="ts">
import TopBar from './components/TopBar.vue'
import PageSidebar from './components/PageSidebar.vue'
import ToolRail from './components/ToolRail.vue'
import EditorCanvas from './components/EditorCanvas.vue'
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
  </div>
</template>
```

- [ ] **Step 4: Run the full suite + build**

Run: `npm test`
Expected: PASS.
Run: `npm run build`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add -u src/App.vue src/App.test.ts
git commit -m "feat(editor): remove the timeline

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

## After this plan

The editor now uses the round-two `DocState` (calendar / image / drawing, `livePageId`, no timeline/clock) and the suite + build are green. The remaining Phase 1 plans build on this shape:

- **1b — Calendar feed-ref + date variant:** `CalendarEl` drops `events[]` for `feedId` + a `date` variant; `CalendarOptions` gets a feed picker (stub feed list) and the date variant; `CalendarWidget` renders the date and sample events.
- **1c — Text tool + on-canvas editing:** new `TextEl` + `text` tool, `TextWidget`/`TextOptions`, click-into-edit on the canvas, font picker over the bundled font manifest.
- **1d — "Make page live" UI:** a control in `PageSidebar`/`PageThumbnail` to set `livePageId`, with the live page marked.

## Self-review notes

- Spec coverage: this plan implements the "remove clock", "remove timeline", and "add `livePageId`" items of the round-two design's §3 and §11; calendar feed-ref, text tool, and live-page UI are explicitly carried to plans 1b–1d.
- The `clockEl`→`imageEl` test-helper swap touches several existing store tests; Task 1 Step 2 lists exactly which tests change or are deleted.
- `git rm` is used for tracked-file deletions so the index and working tree stay consistent.
