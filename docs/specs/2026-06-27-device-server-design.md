# Corkboard device server — design (2026-06-27)

Round-two design: the on-device server + renderer for the e-paper noticeboard, plus the
editor changes it forces. Round one built the frontend editor.

**This doc is the round-two record, not the source of truth.** The durable decisions live
in `.mex/` — the decision log (`mex timeline` / `.mex/events/decisions.jsonl`), the
decision entries in `.mex/context/decisions.md`, and the contract in
`.mex/context/protocol.md`. If this doc and `.mex/` ever disagree, `.mex/` wins.

## Locked context (see `.mex/` for detail)
- **Hardware** (`context/hardware.md`): Waveshare 7.3" E6, 800×480, 6-colour Spectra,
  full-refresh only; host Orange Pi Zero 2W.
- **Round-one flow**: editor GETs the full state on load, POSTs the full state to publish
  (not a diff), images upload separately, no auth (private LAN), editor tolerates an
  unreachable device.

## 1. Two renderers (approach A)
Two separate renderers, deliberately **not** pixel-identical:
- **Editor (Vue)** — a *rough design surface*. Fast, interactive, allowed to be approximate.
- **Device (Rust)** — the *authoritative* renderer. Its output is the truth.

The contract between them is **"no glaring differences,"** not pixel-parity. We rejected a
single shared renderer (Rust→WASM in the editor): it solved a parity problem we don't
actually have, at the cost of rebuilding the editor around a canvas. Rust on the device is
also the deliberate "learn Rust / have fun" choice.

## 2. Product shape
A board is a set of saved **page layouts**; exactly one is **live** on the display at a time.
A page holds freely-placed **elements**:
- **Calendar** — the date, today's events, or the week ahead, pulled from a Google Calendar
  feed (`variant: 'date' | 'today' | 'week'`).
- **Text** — free text in a chosen font.
- **Image** — an uploaded picture.
- **Drawing** — freehand ink.

The display changes only when you publish a different layout or when the live page's calendar
events change. It's a slow, static noticeboard, not a live dashboard.

*Rationale aside:* earlier thinking included a clock and an auto-cycling timeline of pages;
both were dropped because the Spectra 6's slow, flashy full refresh makes a per-minute clock
and page-looping impractical, and removing them removes all fast-refresh pressure. See the
product-simplification entry in `.mex/context/decisions.md`.

## 3. Data-model changes (`src/stores/types.ts`)
- Remove `ClockEl` and the `clock` tool.
- Remove `timeline` / `TimelineEntry` from `DocState`.
- Add `livePageId: string | null` to `DocState` (which page is displayed).
- `CalendarEl`: replace the frozen `events: CalEvent[]` with a **feed reference** —
  `feedId: string` plus `variant: 'date' | 'today' | 'week'`. Events are resolved on the
  **device at render time**, never stored in the document. The editor renders **sample**
  events for preview.
- Add `TextEl` (`type: 'text'`) and a `text` tool: `text: string`, `font: string` (a name
  from the font manifest, default Atkinson Hyperlegible), `align: 'left' | 'center'`, plus
  the shared `colour` / `x` / `y` / `w` / `h`. Text wraps and scales within its box.

## 4. Device server (Rust + axum)
One process, one origin: serves the built editor (static), the JSON API, and the preview.

### Endpoints (canonical list in `context/protocol.md`)
- `GET /` → the editor (static build)
- `GET /api/document` → the full document JSON
- `PUT /api/document` → replace the document (publish); triggers render + conditional push
- `POST /api/images` → upload one image, returns `{ id }`
- `GET /api/images/:id` → image bytes
- `GET /api/feeds` / `PUT /api/feeds` → manage calendar feeds (names only; secret URLs are write-only, never returned)
- `POST /api/refresh` → force an immediate feed fetch + re-render now
- `GET /preview.png` → the current rendered 800×480 image (authoritative "what it looks like"; the stand-in until hardware arrives)

### Storage (plain files)
- `document.json` — the whole document (all pages + `livePageId`)
- `images/<id>.<ext>` — uploaded images; GC any unreferenced on publish
- `config.json` — device config + **secrets** (feed secret URLs, `pollIntervalMinutes`,
  `hostname`). Separate from the document, never in git, never returned by the API.

## 5. Display abstraction (the hardware-free path)
A Rust `Display` trait. Two impls:
- **`WebPreview`** — serves `preview.png`. The dev / no-hardware path, and the authoritative
  "what it looks like" view once hardware exists.
- **`Panel`** — the real SPI driver. Built last, slots in behind the trait.

The renderer always produces an 800×480 6-colour buffer; the buffer goes to whatever
display(s) are active.

## 6. Renderer
`page-state → 800×480 6-colour buffer`, per element:
- **text / calendar / date** — text, laid out with a real font (measured, no glyph-width fudge).
- **image** — decode + quantise/dither to the 6 colours.
- **drawing** — perfect-freehand-equivalent strokes from the stored points.

Specific crates (raster, text shaping, ICS parse, image decode) are chosen at implementation
time and recorded then.

### Fonts
Both renderers must shape text from the **same font bytes**, or text won't match.
- **Bundled, self-hosted — not the Google Fonts CDN.** Google Fonts is fine as a *source* to
  download open `.ttf`/`.otf` files from, but the Rust renderer needs the actual files and the
  board must work offline. Font files live in the repo (e.g. `assets/fonts/`).
- **One source of bytes for both sides.** The device embeds/loads the files for its renderer
  *and* serves them to the editor as `@font-face` from the same origin → identical glyphs,
  offline-safe.
- **A bundled font manifest** (name → file) lists what's available; both the editor and the
  device read it. It lives with the bundled assets **in git, not in `config.json`**, because:
  it's *build-scoped* (changes only when a developer adds/removes a bundled font, and must
  match the shipped files); it has *no secrets*; and the *editor* reads it too. `config.json`
  is the opposite on all three — *deploy-scoped* per-device runtime state, holds *secrets*
  (feed URLs), kept *out of git*, and never read by the editor. Different lifecycle, different
  trust boundary → different file.
- **Seed set:** Atkinson Hyperlegible (default — built for glanceable legibility), Inter
  (clean/neutral), Caveat (handwritten). Adding more = drop the file in + a manifest line.
- Font *choice* is **document data** (`TextEl.font`), not config. Calendar text uses the
  default font.

**Manifest format** — `assets/fonts/manifest.json` (committed; served at `/fonts/manifest.json`):

```json
{
  "fonts": [
    {
      "id": "atkinson-hyperlegible",
      "name": "Atkinson Hyperlegible",
      "default": true,
      "faces": [
        { "weight": 400, "style": "normal", "file": "atkinson-hyperlegible/Regular.ttf" },
        { "weight": 700, "style": "normal", "file": "atkinson-hyperlegible/Bold.ttf" }
      ]
    }
  ]
}
```

- `id` — stable key stored in `TextEl.font` (the display `name` can change without breaking documents).
- `name` — label shown in the font picker.
- `default` — the fallback font (calendar text, or any element with a missing/unknown font).
- `faces` — `weight` + `style` → `file` (path relative to the fonts dir). **Ship a separate
  font file per weight (a "static" font file), not a "variable font" file** — so neither the
  browser nor the Rust shaper interpolates the weight axis, and both draw the identical
  outline. (A property of the font *files*, not of this manifest.)

Both sides read this one file: the editor generates `@font-face` rules (`src: /fonts/<file>`);
the Rust renderer loads each file and registers it under the family + weight/style. v1 may ship
just the Regular face per family; the `faces` array lets Bold/Italic be added later as drop-ins.

## 7. Refresh model
Decouple *checking the feed* (cheap) from *refreshing the panel* (slow, flashy, wears it):
- **Poll** the feed on an interval — default **60 min**, set in `config.json`
  (`pollIntervalMinutes`).
- The device tracks the **currently-displayed calendar content** (resolved events + date for
  the live page).
- On each poll: re-resolve from the feed and **compare the calendar content**. If it
  **changed** → re-render + push to the panel. If unchanged → do nothing.
- **Publish** and **`POST /api/refresh`** always re-render + push.
- Date rollover is caught by the poll (the date is part of the compared content; it flips
  within one interval). No dedicated midnight tick unless prompt flip is wanted later.

Change-detection is **semantic** (compare the resolved content), not pixel-based.

## 8. Calendar events
Google Calendar **secret iCal (ICS) URL**. The device fetches over HTTPS, parses, and
resolves events for the live page's calendar element(s) by `feedId`. Feeds live in device
config as `{ id, name, secretUrl }`; the document carries only `feedId`; secret URLs never
enter the document or git. This is the first **config/secret** and the first device→internet
dependency. Rejected full OAuth as too heavy for a household board.

## 9. Hosting / dev workflow
The device serves editor + API + preview on one origin, LAN only, no internet needed for the
editor. Its mDNS hostname is a **config value** (`hostname`, default `corkboard`), so the
board is reachable at `http://<hostname>.local/` — `http://corkboard.local/` by default, but
nothing hardcodes it. Dev: the Vite dev server proxies API/preview calls to a running
device-server instance. Future editor/display decoupling: **deferred** — keep it simple now.

## 10. Testing — the parity guardrail
Render the same page-state through the editor (Playwright screenshot) and the device
renderer (`preview.png`), diff on a **generous** threshold — fail only on *glaring*
divergence (missing/misplaced element, wrong colour), not minor text/layout drift. The
calendar uses fixed **sample** events on both sides for determinism.

## 11. Editor changes this forces
- Remove `Timeline.vue` / `TimelineItem.vue`, `ClockWidget` / `ClockOptions`, the clock tool
  in `ToolRail`, clock in thumbnails, the timeline-reorder logic.
- Add: the date calendar variant; a feed picker in `CalendarOptions`; a **"Refresh now"**
  button (calls `POST /api/refresh`); `livePageId` + a "make this page live" affordance.
- Add a **text tool**: `TextWidget` + `TextOptions`. Text content is edited **on the canvas**
  (click into the element to edit it in place); `TextOptions` holds font (picker over the
  manifest), alignment, and colour. On-canvas editing has to account for the canvas
  scale/letterbox and coexist with select/drag — confirm in a real browser (jsdom is scale=1).

## Build order (becomes the implementation plan)
1. **Editor surgery** — remove clock + timeline, add `livePageId`, calendar feed-ref + date
   variant, "Refresh now" button.
2. **Server skeleton** — `Display` trait + `WebPreview` + axum + file storage + the API
   (real `GET`/`PUT`/images/feeds/refresh/preview).
3. **Renderer** — calendar/date, image quantise/dither, drawing strokes.
4. **Calendar** — ICS fetch/parse + the refresh model + semantic change-detection.
5. **Parity guardrail** — the editor↔device pixel-comparison test.
6. **(Later, when hardware lands)** the `Panel` SPI driver behind the trait.

## Out of scope / deferred
- The `Panel` SPI driver (until the board arrives).
- Editor/display decoupling.
- OAuth calendars.
- A dedicated midnight tick (the poll subsumes it).
