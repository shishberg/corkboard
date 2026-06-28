---
name: router
description: Session bootstrap and navigation hub. Read at the start of every session before any task. Contains project state, routing table, and behavioural contract.
edges:
  - target: context/architecture.md
    condition: when working on system design, integrations, or understanding how components connect
  - target: context/protocol.md
    condition: when implementing or changing the editor-to-device API (endpoints, document shape, config/secrets, refresh)
  - target: context/stack.md
    condition: when working with specific technologies, libraries, or making tech decisions
  - target: context/hardware.md
    condition: when designing rendering, the device server, or anything that touches the display panel
  - target: context/conventions.md
    condition: when writing new code, reviewing code, or unsure about project patterns
  - target: context/decisions.md
    condition: when making architectural choices or understanding why something is built a certain way
  - target: context/setup.md
    condition: when setting up the dev environment or running the project for the first time
  - target: patterns/INDEX.md
    condition: when starting a task — check the pattern index for a matching pattern file
last_updated: 2026-06-28
---

# Session Bootstrap

If you haven't already read `AGENTS.md`, read it now — it contains the project identity, non-negotiables, and commands.

Then read this file fully before doing anything else in this session.

## Current Project State

**Round two is COMPLETE except hardware deploy.** Both halves are built and working end-to-end: the Vue **editor** (fully round-two: clock/timeline gone; `livePageId` + make-live + `deletePage`; calendar feed-reference; text + bundled fonts; wired to the device for load/publish) AND the Rust **device server** at `device/` (storage + API + `preview.png` + real 6-colour renderer + ICS calendar resolve/poll). Element types: `calendar | image | drawing | text`. The only unbuilt piece is the `Panel` SPI driver (needs hardware). Editor: 180 Vitest + 6 Playwright parity tests. Device: 64 cargo tests. All green; `npm run build` clean.

**Working — editor:**
- Web UI editor (Vite + Vue 3 + TS + Pinia + Tailwind v4 + shadcn-vue). App shell in `src/App.vue`: TopBar, PageSidebar, ToolRail, EditorCanvas. (Timeline removed in 1a.) Hydrates its document from the device on startup; Publish does `PUT /api/document`.
- Document state in `usePagesStore` (`src/stores/pages.ts`) — pages, elements, orientation, selection, and `livePageId` (the one page the device displays; defaults to the first page; `setLivePage(id)` guards unknown ids; `livePage` getter is null-safe; `deletePage(id)` reassigns `livePageId`/`selectedPageId` and is a no-op on the last page). This Pinia shape is the draft page-state contract (= the wire `DocState`).
- Persisted tool options (`src/stores/toolOptions.ts`, localStorage; `load()` whitelists known keys so stale persisted keys are dropped), element factory (`calendar | image | text`), hand-rolled drag/resize (`useDraggableResizable` + `MovableElement`), widgets (Calendar/Image/Text with size-scaled fonts; pen via DrawingLayer/DrawingWidget; `DrawOptions` pen sizes are shown as different-sized circles, not numbers).
- Element creation is live draw-to-place: picking calendar/image/text only sets the active tool; pressing on the canvas creates the real element immediately and the drag sizes it (a click drops a default size), then the tool auto-switches back to select. Creation tools always create — elements are pointer-events:none unless the select tool is active, so only select selects. Backspace/Delete (or the ToolRail trash button) deletes the selected element.
- Colour is one global current colour shown as a swatch panel below the tools in `ToolRail` (`colour` in toolOptions; `colour` on every `BaseEl`). It drives calendar/text glyphs and pen ink; selecting an element reflects its colour in the panel and clicking a swatch recolours it (`store.setElementColour`). A **background tool** (`activeTool === 'background'`, paint-bucket icon) reroutes the same swatches to set the current page's background (`Page.background?: EpaperColour`, default white; `store.setPageBackground`); the editor surface and the device renderer both paint it.
- **Z-order:** `ToolRail` has bring-to-front / send-to-back buttons (next to the trash) acting on the selected element via `store.bringToFront(id?)` / `store.sendToBack(id?)` (move it to the end / start of the page's `elements`; later = drawn on top).
- **Images:** `ImageOptions` uploads a file via `deviceApi.uploadImage` (`POST /api/images` → `{id}`); the id is stored as the element `src`, kept as a pending `toolOptions.imageId` (session-only, not persisted) for the next placed image, and applied to the selected image element if one is selected. `ImageWidget` shows the image from `deviceApi.imageUrl(id)` = `/api/images/{id}` (so it only renders when served by the device). `store.setElementSrc(id, src)` sets it.
- **TopBar:** Publish promotes the currently-selected page to live (`setLivePage`) then `PUT`s the document (the old per-page "Make live" button in `PageSidebar` is gone; the green Live badge stays). A **Preview** link (`<a href="/preview.png" target="_blank">`) opens the device's rendered PNG in a new tab.
- Pen uses **perfect-freehand** (`src/lib/freehand.ts` `strokeToPath`): strokes render as filled SVG ink paths (live preview + committed + thumbnail). Raw input points are stored element-local with `natW`/`natH`, so resizing scales the stroke and a tap leaves a dot. Drawings show in page thumbnails.
- 154 Vitest tests pass (`npm test`); `npm run build` clean. (Browser parity tests: `npm run test:parity`.)

**BUILT — the device server (Rust + axum) at `device/`.** Single source of truth: stores the
whole document + images + config as plain files; serves the built editor + the JSON API +
`preview.png` on one origin (LAN, no auth); renders the live page to an 800×480 six-colour PNG
behind a `Display` trait (`WebPreview` impl serves `preview.png`; the `Panel` SPI driver is the
ONLY deferred piece — needs hardware). Resolves calendar events from a Google secret-iCal URL at
render time, polls the feed, and re-renders only on semantic content change. **63 cargo tests.**
- Modules (`device/src/`): `main` (bootstrap + poll task), `config`, `document` (serde mirror of
  the editor `DocState`), `storage` (files + image GC), `display` (trait + `WebPreview`), `render`
  (tiny-skia + ab_glyph; calendar/text/image/drawing → 6-colour quantise), `text`, `sample`
  (deterministic calendar data, ported from `src/lib/sampleCalendar.ts` — keep the two in sync),
  `fonts` (loads `public/fonts` + embedded Atkinson fallback), `calendar` (hand-rolled ICS parse +
  resolve + semantic `signature`), `api`, `state`.
- Run it: `cd device && CORKBOARD_DIST=../dist CORKBOARD_FONTS=../public/fonts cargo run`
  (needs `npm run build` first so `../dist` exists). Env: `CORKBOARD_DATA` (default `./data`),
  `CORKBOARD_PORT` (8080), `CORKBOARD_DIST`, `CORKBOARD_FONTS`.
- Editor↔device sync (1e): the editor hydrates from `GET /api/document` on startup and Publish
  does `PUT /api/document`; `src/lib/deviceApi.ts` has the tolerant client (`getDocument`,
  `putDocument`, `fetchFeeds`, `refreshNow`).
- Parity guardrail (S4): `npm run test:parity` (Playwright) compares the editor surface screenshot
  vs `preview.png` on a coarse content-mask IoU (≥0.35; feedless doc → both use sample data).

The full design is `docs/specs/2026-06-27-device-server-design.md`; durable decisions in
`context/decisions.md`, the wire contract in `context/protocol.md`, the panel in `context/hardware.md`.

**Only remaining (hardware): the `Panel` SPI driver** behind the `Display` trait, plus a small set
of low-risk review Minors (see the device handoff / `.superpowers/sdd/review-device-final.md`).

**Editor surgery — DONE (plans 1a–1d, all implemented; 137 tests green, build clean):**
- **1a:** removed clock + timeline (components + store API); `DocState` gained `livePageId`; factory/tool-options dropped `clockVariant`.
- **1b:** `CalendarEl` now `{ variant: 'date'|'today'|'week'; feedId: string }` (NO embedded events). `useFeedsStore` (stub `[{id:'family',name:'Family'}]`, tolerant `loadFeeds()`); tolerant `src/lib/deviceApi.ts` (`fetchFeeds`/`refreshNow`, relative URLs, swallows errors — editor tolerates an offline device); deterministic `src/lib/sampleCalendar.ts` for preview; CalendarOptions feed picker + date variant. (TopBar "Refresh now" removed 2026-06-28: Publish always re-`PUT`s the document and the device's `PUT /api/document` re-resolves feeds + re-renders, so a no-op Publish already doubles as refresh. `deviceApi.refreshNow` + `POST /api/refresh` still exist.)
- **1c:** `TextEl` (`text`/`font`/`align`) + `text` tool; `TextWidget` on-canvas editing via an **uncontrolled contenteditable** (imperative `textContent`, focus-guarded watch — do NOT reactively interpolate into a contenteditable, it resets the caret). Edit mode is **double-click to edit** (2026-06-28): `EditorCanvas` owns a local `editingElId`; TextWidget/MovableElement take an `editing` prop. While `editing`, MovableElement skips its drag-`preventDefault` so the caret can land, and TextWidget is `contenteditable` + auto-focused; blur emits `stop-editing`. New text boxes auto-enter edit mode. `EditorCanvas`'s Backspace/Delete handler ignores keys from any `contenteditable`/input (uses `closest`, since jsdom's `isContentEditable` is unreliable) so editing removes characters, not the element. `TextOptions` font picker + align; `src/lib/fonts.ts` manifest loader + `injectFontFaces`; bundled fonts at `public/fonts/` (served at `/fonts/`), manifest `public/fonts/manifest.json`. **Seed = Atkinson Hyperlegible only** (Regular+Bold static TTFs); Inter/Caveat deferred (google/fonts only ships them variable; design needs static for parity).
- **1d:** make-live UI in `PageSidebar` (Make-live button + Live badge, ring=selected vs border=live so both show) + `deletePage` that reassigns `livePageId`/`selectedPageId` when the live/selected page is removed (no-op on last page).
- Image upload is wired end-to-end: `ImageOptions` uploads to `POST /api/images` and stores the returned id as the element `src` (resolved for display via `/api/images/{id}`).

**Known issues:**
- The device `Panel` SPI driver is not built (needs hardware). A few low-risk device review Minors remain (see `.superpowers/sdd/review-device-final.md`): `/api/bogus` returns index.html not 404; ICS `signature` delimiter aliasing; 2MB upload limit; image content-type now sniffed. Bundled fonts ship Atkinson only (Inter/Caveat deferred — need static TTFs).

**Open decisions** (see `context/decisions.md`):
- Renderer crate choices (raster, text shaping, ICS parse, image decode) — decide at implementation time.
- OS for the Orange Pi Zero 2W (some lightweight Linux).
- Project non-negotiables (deferred by the owner until real constraints emerge).
- (Resolved this round: device language → Rust; hosting → from the device; persistence → plain files; page-state schema/endpoints → `context/protocol.md`.)

## Routing Table

Load the relevant file based on the current task. Always load `context/architecture.md` first if not already in context this session.

| Task type | Load |
|-----------|------|
| Understanding how the system works | `context/architecture.md` |
| Editor↔device API (endpoints, document shape, refresh) | `context/protocol.md` |
| Working with a specific technology | `context/stack.md` |
| Rendering, device server, or the display panel | `context/hardware.md` |
| Writing or reviewing code | `context/conventions.md` |
| Making a design decision | `context/decisions.md` |
| Setting up or running the project | `context/setup.md` |
| Any specific task | Check `patterns/INDEX.md` for a matching pattern |

## Behavioural Contract

For every task, follow this loop:

1. **CONTEXT** — Load the relevant context file(s) from the routing table above. Check `patterns/INDEX.md` for a matching pattern. If one exists, follow it. Narrate what you load: "Loading architecture context..."
2. **BUILD** — Do the work. If a pattern exists, follow its Steps. If you are about to deviate from an established pattern, say so before writing any code — state the deviation and why.
3. **VERIFY** — Load `context/conventions.md` and run the Verify Checklist item by item. State each item and whether the output passes. Do not summarise — enumerate explicitly.
4. **DEBUG** — If verification fails or something breaks, check `patterns/INDEX.md` for a debug pattern. Follow it. Fix the issue and re-run VERIFY.
5. **GROW** — After meaningful work, run this binary checklist:
   - **Ground:** What changed in reality? Name the changed behavior, system, command, dependency, or workflow.
   - **Record:** If project state changed, update the "Current Project State" section above. If documented facts changed, update the relevant `context/` file surgically.
   - **Orient:** If this task can recur and no pattern exists, create one in `patterns/` using `patterns/README.md`, then add it to `patterns/INDEX.md`. If a pattern exists but you learned a gotcha, update it.
   - **Write:** Bump `last_updated` in every scaffold file you changed. If the why matters, run `mex log --type decision "<what changed and why>"` or `mex log "<note>"`.
