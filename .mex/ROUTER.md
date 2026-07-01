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
last_updated: 2026-07-01
---

# Session Bootstrap

If you haven't already read `AGENTS.md`, read it now — it contains the project identity, non-negotiables, and commands.

Then read this file fully before doing anything else in this session.

## Current Project State

**Calendar variants are now `date | agenda` only** (the single-day `today` variant was removed and folded into `agenda`; both `today` and `week` remain serde/load aliases → `agenda`). `CalendarEl` carries `align: 'left'|'center'` (the text alignment tool now applies to calendars too) and `daysAhead: number` (agenda horizon 1..=7, default 7, set via a number input in `CalendarOptions`). The **Date** variant auto-fits its font to fill the box like a text field (`textFit`/`fit_font_size`), top-aligned with `align` horizontally. The **Agenda** variant draws **bold day headings** (using each font's real 700 face — all four bundled fonts now ship a Bold.ttf), indented events, and a 1px divider (text colour) under each event; it fits a font size then truncates at a line boundary when content overflows. Editor `CalendarWidget` and device `draw_agenda` share the exact layout algorithm (WYSIWYG; parity test green).

**Round two is COMPLETE except hardware deploy.** Both halves are built and working end-to-end: the Vue **editor** (fully round-two: clock/timeline gone; `livePageId` + make-live + `deletePage`; calendar feed-reference; text + bundled fonts; wired to the device for load/publish) AND the Rust **device server** at `device/` (storage + API + `preview.png` + real 6-colour renderer + ICS calendar resolve/poll). Element types: `calendar | image | drawing | text`. The `Panel` SPI driver is written (`device/src/panel.rs`) and reviewed across six codex + Opus passes for hardware-safety and readability. It drives the panel as a **full-host one-shot per refresh**: every `show()` opens a fresh session (assert PWR → open SPI → claim RST/DC/BUSY), runs `reset → init → frame + refresh → deep_sleep`, then releases *everything* — mirroring Waveshare's own `module_init`/`module_exit` bracketing (register-identical against the canonical `waveshareteam/e-Paper` repo). Nothing is held between refreshes; the panel is unpowered between them. Teardown is RAII-guaranteed (`Session`/`OpeningGuard` `Drop`): RST/DC low → close SPI → drop PWR on every exit path, plus a best-effort emergency POWER_OFF on a *failed* refresh. A missing PWR var is a hard error unless `CORKBOARD_PANEL_NO_PWR=1` opts out. Unit-tested with fakes behind `HostOpener`/`SpiBus`/`GpioLine` traits (no real hardware or Linux host needed; only the real backend is `target_os = "linux"`-gated), but still unverified against the real panel, which isn't in hand yet. (This is the second, clean-room implementation; it replaced an earlier partial one-shot that held peripherals open and left a CS-idle-on-failure residual.) Editor: 216 Vitest + 6 Playwright parity tests. Device: 119 cargo tests. All green; `npm run build` clean.

**Working — editor:**
- Web UI editor (Vite + Vue 3 + TS + Pinia + Tailwind v4 + shadcn-vue). App shell in `src/App.vue`: TopBar, then a row of PageSidebar, ToolRail, and a canvas column (ToolOptionsBar above EditorCanvas). (Timeline removed in 1a.) Hydrates its document from the device on startup; Publish does `PUT /api/document`.
- Document state in `usePagesStore` (`src/stores/pages.ts`) — pages, elements, per-page `orientation` (portrait/landscape is a property of each `Page`, not the document; `pageSize`/`toggleOrientation` follow the selected page; `hydrate` migrates a legacy document-level orientation onto pages), selection, and `livePageId` (the one page the device displays; defaults to the first page; `setLivePage(id)` guards unknown ids; `livePage` getter is null-safe; `deletePage(id)` reassigns `livePageId`/`selectedPageId` and is a no-op on the last page). This Pinia shape is the draft page-state contract (= the wire `DocState`).
- Persisted tool options (`src/stores/toolOptions.ts`, localStorage; `load()` whitelists known keys so stale persisted keys are dropped), element factory (`calendar | image | text`), hand-rolled drag/resize (`useDraggableResizable` + `MovableElement`), widgets (Calendar/Image/Text; Text auto-fits its box via `src/lib/textFit.ts` which measures the real DOM, mirroring the device's `fit_font_size`; pen via DrawingLayer/DrawingWidget; `DrawOptions` pen sizes are shown as different-sized circles, not numbers).
- Element creation is live draw-to-place for **calendar/text**: picking the tool only sets it active; pressing on the canvas creates the real element immediately and the drag sizes it (a click drops a default size), then the tool auto-switches back to select. Creation tools always create — elements are pointer-events:none unless the select tool is active, so only select selects. **Image is NOT draw-to-place** — it's a one-click action (see Images). Backspace/Delete (or the ToolRail trash button) deletes the selected element.
- **Tool settings live in `ToolOptionsBar.vue`** (horizontal bar above the canvas), not in the `ToolRail` (which now holds only tool buttons + z-order + delete). The bar has a declarative panel registry: each panel (`ToolOptions/CalendarOptions`, `FontOptions`, `AlignOptions`, `DrawOptions`, `ColourSwatches`) declares the tools and selected-element types it applies to, and shows when `activeTool` matches OR the selected element's type matches — so settings follow the current tool or selection. Panels edit the selected element when one is selected, else the tool default in `toolOptions` (`setElementVariant`/`setElementFeed`/`setElementFont`/`setElementAlign`/`setElementColour`). Align is icon buttons (`AlignLeft`/`AlignCenter`); no inline labels.
- Colour is one global current colour shown by `ColourSwatches` in the bar (`colour` in toolOptions; `colour` on every `BaseEl`). It drives calendar/text glyphs and pen ink; selecting an element reflects its colour and clicking a swatch recolours it (`store.setElementColour`). A **background tool** (`activeTool === 'background'`, paint-bucket icon) reroutes the same swatches to set the current page's background (`Page.background?: EpaperColour`, default white; `store.setPageBackground`); the editor surface and the device renderer both paint it.
- **Font applies to text and calendars.** `CalendarEl` carries `font` (alongside `TextEl.font`); `CalendarWidget` renders it (falling back to the default font when empty) and the device renderer uses `faces.get(&el.font)` for calendar text. Empty font → default face on both sides, so older documents are unaffected.
- **Z-order:** `ToolRail` has bring-to-front / send-to-back buttons (next to the trash) acting on the selected element via `store.bringToFront(id?)` / `store.sendToBack(id?)` (move it to the end / start of the page's `elements`; later = drawn on top).
- **Images:** the ToolRail image button is a one-click action — it opens the file dialog directly, then `src/lib/imageTool.ts` `addImageFromFile` uploads via `deviceApi.uploadImage` (`POST /api/images` → `{id}`), reads the natural size, and drops a **centred, aspect-correct** element (`imagePlacement` in `elementFactory`, scaled to fit half the page), then reactivates select. `ImageWidget` shows the image from `deviceApi.imageUrl(id)` = `/api/images/{id}` and reports its natural size on load so `EditorCanvas` can **lock resize to the image's aspect ratio** (`useDraggableResizable` `aspect` option) and snap the box if needed. (the old ImageOptions component and the `toolOptions.imageId` field were removed.) `store.setElementSrc(id, src)` still sets the src.
- **TopBar:** Publish promotes the currently-selected page to live (`setLivePage`) then `PUT`s the document (the old per-page "Make live" button in `PageSidebar` is gone; the green Live badge stays). A **Preview** link (`<a href="/preview.png" target="_blank">`) opens the device's rendered PNG in a new tab.
- Pen uses **perfect-freehand** (`src/lib/freehand.ts` `strokeToPath`): strokes render as filled SVG ink paths (live preview + committed + thumbnail). Raw input points are stored element-local with `natW`/`natH`, so resizing scales the stroke and a tap leaves a dot. Drawings show in page thumbnails.
- 216 Vitest tests pass (`npm test`); `npm run build` clean. (Browser parity tests: `npm run test:parity`.)

**BUILT — the device server (Rust + axum) at `device/`.** Single source of truth: stores the
whole document + images + config as plain files; serves the built editor + the JSON API +
`preview.png` on one origin (LAN, no auth); renders the live page to an 800×480 six-colour PNG
behind a `Display` trait (`WebPreview` impl serves `preview.png`; `Panel` drives the real
e-paper panel over SPI/GPIO — written, but unverified against real hardware since the panel
isn't in hand). Resolves calendar events from a Google secret-iCal URL at
render time, polls the feed, and re-renders only on semantic content change. **119 cargo tests.**
- Modules (`device/src/`): `main` (bootstrap + poll task — the poll task resolves once on startup
  then sleeps the interval, so real calendar data shows immediately, not after the first interval;
  also a `log_request` middleware logs every HTTP request as `METHOD path -> status (ms)` at INFO,
  and storage/state log saves, image GC, feed fetches, and refreshes — secret feed URLs are never
  logged, only feed ids/counts),
  `config`, `document` (serde mirror of
  the editor `DocState`), `storage` (files + image GC; `gc_images` runs on every `PUT /api/document`
  so images unreferenced by the saved document are deleted), `display` (trait + `WebPreview`), `render`
  (tiny-skia + ab_glyph; calendar/text/image/drawing → 6-colour palette via **Floyd–Steinberg
  dithering** of the final buffer — text/strokes/background are exact palette colours so only images
  dither; images are Lanczos3-resized and **alpha-composited** so transparent PNGs show the background;
  text **auto-fits** its box via `text::fit_font_size`, glyphs are grid-fit + stem-darkened for crisp
  small 1-bit text, calendar text floored at 11px), `text`, `sample`
  (deterministic calendar data, ported from `src/lib/sampleCalendar.ts` — keep the two in sync),
  `fonts` (loads `public/fonts` + embedded Atkinson fallback), `calendar` (hand-rolled ICS parse +
  resolve + semantic `signature`; `parse_ics` reads `RRULE`/`EXDATE` and `resolve` expands recurrence
  — FREQ DAILY/WEEKLY/MONTHLY/YEARLY with INTERVAL/COUNT/UNTIL/BYDAY/EXDATE, WKST ignored — so
  recurring Google-calendar events actually appear. The agenda view is the **next 7 days starting
  today** (slot 0 = today … slot 6 = today+6), NOT a fixed Mon–Sun ISO week — `ResolvedFeed` carries
  `week_labels` (full weekday names) for each slot. `render`'s `draw_agenda` lays it out as a list:
  **bold** day headings (Today, Tomorrow, then weekday names) with events indented beneath, 12-hour
  times, one shared font size, and a 1px divider (text colour) under each event; it shows the first
  `daysAhead` days (1..=7) and truncates at a line boundary when content overflows — mirrored exactly
  by `CalendarWidget.vue` for WYSIWYG. Feedless
  fallback resolves a shared sample (`sample::sample_feed` == `sampleCalendar.ts` `sampleAgenda`)),
  `api`, `state`.
- Run it: `cd device && CORKBOARD_DIST=../dist CORKBOARD_FONTS=../public/fonts cargo run`
  (needs `npm run build` first so `../dist` exists). Env: `CORKBOARD_DATA` (default `device/data`),
  `CORKBOARD_PORT` (8080), `CORKBOARD_DIST`, `CORKBOARD_FONTS`.
- Editor↔device sync (1e): the editor hydrates from `GET /api/document` on startup and Publish
  does `PUT /api/document`; `src/lib/deviceApi.ts` has the tolerant client (`getDocument`,
  `putDocument`, `fetchFeeds`, `refreshNow`).
- Parity guardrail (S4): `npm run test:parity` (Playwright) compares the editor surface screenshot
  vs `preview.png` on a coarse content-mask IoU (≥0.35; feedless doc → both use sample data).

The full design is `docs/specs/2026-06-27-device-server-design.md`; durable decisions in
`context/decisions.md`, the wire contract in `context/protocol.md`, the panel in `context/hardware.md`.

**The `Panel` SPI driver is now written** (`device/src/panel.rs`, Linux-only, `CORKBOARD_DISPLAY=panel`)
— ported from Waveshare's own `epd7in3e` demo code, see `context/decisions.md`'s "Panel driver" entry.
Untested against real hardware: the panel isn't in hand yet, and the Orange Pi's real GPIO chip/line
numbers are unverified (`gpioinfo` step in `patterns/deploy-to-orange-pi.md`). Plus a small set
of low-risk review Minors (see the device handoff / `.superpowers/sdd/review-device-final.md`).

**Editor surgery — DONE (plans 1a–1d, all implemented; 137 tests green, build clean):**
- **1a:** removed clock + timeline (components + store API); `DocState` gained `livePageId`; factory/tool-options dropped `clockVariant`.
- **1b:** `CalendarEl` now `{ variant: 'date'|'agenda'; feedId: string; font; align; daysAhead }` (NO embedded events; `agenda` was renamed from the old `week` and absorbed the removed `today` variant — both load as aliases). `align`/`daysAhead` added later (see the calendar bullet at the top). `useFeedsStore` (stub `[{id:'family',name:'Family'}]`, tolerant `loadFeeds()`); tolerant `src/lib/deviceApi.ts` (`fetchFeeds`/`refreshNow`, relative URLs, swallows errors — editor tolerates an offline device); deterministic `src/lib/sampleCalendar.ts` for preview; CalendarOptions feed picker + date variant (the picker hides the "(none)" option when any feed is available and auto-selects the first feed when the current `feedId` isn't one of them). (TopBar "Refresh now" removed 2026-06-28: Publish always re-`PUT`s the document and the device's `PUT /api/document` re-resolves feeds + re-renders, so a no-op Publish already doubles as refresh. `deviceApi.refreshNow` + `POST /api/refresh` still exist.)
- **1c:** `TextEl` (`text`/`font`/`align`) + `text` tool; `TextWidget` on-canvas editing via an **uncontrolled contenteditable** (imperative `textContent`, focus-guarded watch — do NOT reactively interpolate into a contenteditable, it resets the caret). Edit mode is **double-click to edit** (2026-06-28): `EditorCanvas` owns a local `editingElId`; TextWidget/MovableElement take an `editing` prop. While `editing`, MovableElement skips its drag-`preventDefault` so the caret can land, and TextWidget is `contenteditable` + auto-focused; blur emits `stop-editing`. New text boxes auto-enter edit mode. `EditorCanvas`'s Backspace/Delete handler ignores keys from any `contenteditable`/input (uses `closest`, since jsdom's `isContentEditable` is unreliable) so editing removes characters, not the element. font picker + align (now `ToolOptions/FontOptions` + `AlignOptions` in the `ToolOptionsBar`); `src/lib/fonts.ts` manifest loader + `injectFontFaces`; bundled fonts at `public/fonts/` (served at `/fonts/`), manifest `public/fonts/manifest.json`. **Four bundled families** — Atkinson Hyperlegible (default), DejaVu Sans, Carlito, Gelasio — each shipping a **Regular (400) + Bold (700)** static TTF; the device loads the bold face per font and the agenda's day headings render in it.
- **1d:** make-live UI in `PageSidebar` (Make-live button + Live badge, ring=selected vs border=live so both show) + `deletePage` that reassigns `livePageId`/`selectedPageId` when the live/selected page is removed (no-op on last page).
- Image upload is wired end-to-end: `ImageOptions` uploads to `POST /api/images` and stores the returned id as the element `src` (resolved for display via `/api/images/{id}`).

**Known issues:**
- The device `Panel` SPI driver is written but unverified against real hardware (panel not in hand; GPIO chip/line numbers unknown for the Orange Pi). A few low-risk device review Minors remain (see `.superpowers/sdd/review-device-final.md`): `/api/bogus` returns index.html not 404; ICS `signature` delimiter aliasing; 2MB upload limit; image content-type now sniffed.

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

**The mex scaffold describes current reality, not history.** Every statement in `ROUTER.md` and `context/` must be true *right now*. When state changes, edit the claim in place — never leave a sentence that has become false, and never add a "this used to be X" aside unless it's load-bearing (a migration alias, a superseded decision in `decisions.md`). If you notice a stale fact while doing unrelated work, fix it. Plan-phase changelogs (e.g. the "Editor surgery 1a–1d" notes) are kept only as long as every fact in them still holds; correct them like anything else.

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
