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
last_updated: 2026-06-27
---

# Session Bootstrap

If you haven't already read `AGENTS.md`, read it now — it contains the project identity, non-negotiables, and commands.

Then read this file fully before doing anything else in this session.

## Current Project State

The **frontend-only web UI editor is built and working** (no device wiring yet).

**Working:**
- Web UI editor (Vite + Vue 3 + TS + Pinia + Tailwind v4 + shadcn-vue), frontend-only. App shell in `src/App.vue`: TopBar, PageSidebar, ToolRail, EditorCanvas, Timeline.
- Document state in `usePagesStore` (`src/stores/pages.ts`) — pages, elements, timeline, orientation, selection; this Pinia shape is the draft page-state contract.
- Persisted tool options (`src/stores/toolOptions.ts`, localStorage), element factory, hand-rolled drag/resize (`useDraggableResizable` + `MovableElement`), widgets (Clock/Calendar/Image with size-scaled fonts; pen via DrawingLayer/DrawingWidget).
- Element creation is live draw-to-place: picking clock/calendar/image only sets the active tool; pressing on the canvas creates the real element immediately and the drag sizes it (a click drops a default size), then the tool auto-switches back to select. Creation tools always create — elements are pointer-events:none unless the select tool is active, so only select selects. Backspace/Delete (or the ToolRail trash button) deletes the selected element.
- Colour is one global current colour shown as a swatch panel below the tools in `ToolRail` (`colour` in toolOptions; `colour` on every `BaseEl`). It drives clock/calendar text and pen ink; selecting an element reflects its colour in the panel and clicking a swatch recolours it (`store.setElementColour`).
- Pen uses **perfect-freehand** (`src/lib/freehand.ts` `strokeToPath`): strokes render as filled SVG ink paths (live preview + committed + thumbnail). Raw input points are stored element-local with `natW`/`natH`, so resizing scales the stroke and a tap leaves a dot. Drawings show in page thumbnails.
- 81 tests pass (`npm test`); `npm run build` clean.

**Designed but not yet built — the device server (round two, 2026-06-27).**
The full design is `docs/specs/2026-06-27-device-server-design.md`; the durable decisions are
in `context/decisions.md`, the wire contract in `context/protocol.md`, the panel in
`context/hardware.md`. In short: a **Rust + axum** device server is the single source of
truth; it serves the editor, the API, and a `preview.png`; renders the live page to the
panel (behind a `Display` trait, with a `WebPreview` stand-in until hardware arrives); two
renderers (editor = rough design surface, device = authoritative, "no glaring differences").
Not started in code.

Build order: (1) editor surgery → (2) server skeleton + API → (3) Rust renderer →
(4) calendar feed + refresh → (5) parity guardrail → (6) Panel SPI driver (when hardware lands).

**Editor surgery this design forces (NOT done yet — the code below still has the old shape):**
- Remove `Timeline`/`TimelineItem`, `ClockWidget`/`ClockOptions`, the clock tool, clock in thumbnails, timeline reorder.
- `DocState`: drop `timeline`/`TimelineEntry` and `ClockEl`; add `livePageId`.
- `CalendarEl`: drop frozen `events[]`; add `feedId` + a date-only variant; add a feed picker + "Refresh now" button + a "make live" affordance.
- Image upload still a stub (`ImageOptions`); now becomes `POST /api/images` referenced by id.

**Known issues:**
- None yet.

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
