---
name: decisions
description: Key architectural and technical decisions with reasoning. Load when making design choices or understanding why something is built a certain way.
triggers:
  - "why do we"
  - "why is it"
  - "decision"
  - "alternative"
  - "we chose"
edges:
  - target: context/architecture.md
    condition: when a decision relates to system structure
  - target: context/stack.md
    condition: when a decision relates to technology choice
last_updated: 2026-06-28
---

# Decisions

<!-- When a decision changes: DO NOT delete the old entry. Mark it superseded and add the
     new entry above it. The history is the event clock. -->

## Decision Log

### Device text rendering: FreeType monochrome (hinted) — SUPERSEDES `ab_glyph`
**Date:** 2026-06-28
**Status:** Active
**Decision:** The device renders all text with **FreeType in monochrome mode** (`FT_LOAD_TARGET_MONO`) via `freetype-rs` (`bundled` feature, compiled from source). Replaces `ab_glyph`. `Fonts` (shared state) stores font *bytes*; the renderer builds `!Send` faces locally per render pass (`text::Faces`). Default font is unchanged (Atkinson Hyperlegible).
**Reasoning:** The panel is 1-bit-per-channel (6 colours, no greys), so text can't be anti-aliased. `ab_glyph` has no hinting — it rasterises raw outlines and thresholds coverage, so small 1-bit text came out jagged and broken (mangled zeros, patchy stems). Best practice for un-antialiased small text is a bitmap font *or* a hinting rasteriser; FreeType's MONO mode is the reference implementation, grid-fitting stems to whole pixels. Pixel fonts through `ab_glyph` weren't actually crisp (it renders their outlines, not bitmap strikes) and don't fill auto-sized boxes. Result: crisp text, and editor↔device parity IoU rose 0.46→0.65 (hinted ink density now matches the browser).
**Alternatives considered:** `swash`/`skrifa` (pure-Rust, hinting — but tuned for grayscale AA; hint-then-threshold is worse than a dedicated mono path); `font-kit` (delegates to the platform rasteriser → non-deterministic Mac vs Pi); `rusttype` (no hinting, `ab_glyph`'s predecessor); a bitmap/pixel font (crisp but fixed-size/retro, all-caps in the ones tried). Keeping `ab_glyph` (rejected — the root cause of "rubbish" text).
**Consequences:** Build needs a C compiler (no system FreeType or `cmake` — `bundled` builds it; the Pi has `libfreetype` trivially anyway). Faces are rebuilt each render (cheap; render is occasional). `text.rs` keeps the same public API (`measure_line`/`wrap_lines`/`fit_font_size`/`draw_text`) so `render.rs` and the agenda layout were untouched. The editor preview is still browser-AA (parity is layout-based, not pixel-exact).

### Target display: Waveshare 7.3" E6 (Spectra 6), 800×480, 6-colour
**Date:** 2026-06-27 (recorded; the panel was chosen in the 2026-06-23 web-UI design round but never made it into `.mex/`)
**Status:** Active
**Decision:** The noticeboard runs on a Waveshare 7.3" E6 (Spectra 6) colour e-paper panel — 800×480, 6-colour palette (black/white/red/yellow/blue/green), full-refresh only. Specs and constraints live in `context/hardware.md`.
**Reasoning:** Chosen in round one as the target; it's the origin of the `EpaperColour` model and the 800×480 canvas. Persisting it here because it had been buried in `docs/specs/2026-06-23-web-ui-editor-design.md` (an event doc) instead of the deduplicated `context/` state read each session.
**Alternatives considered:** Not re-evaluated — this was settled in round one.
**Consequences:** Renderer must quantise to the 6 colours; canvas is fixed at 800×480 (portrait swaps axes); slow full-refresh means redraw on a trigger, not continuously. Refresh timing and SPI interface are marked [UNVERIFIED] in `context/hardware.md` pending the real datasheet/hardware.

### Device server in Rust, two-renderer split (approach A) — SUPERSEDES the Python decision
**Date:** 2026-06-27
**Status:** Active
**Decision:** The device server + renderer is written in **Rust** (web framework **axum**). The editor and the device run **two separate renderers** ("approach A"): the editor's Vue render is a rough *design surface* (approximate, allowed to drift); the device's Rust renderer is *authoritative*. The contract is "no glaring differences," not pixel-parity.
**Reasoning:** Rust is a deliberate learn-it/have-fun choice and is well-suited to driving the panel. Two renderers are fine because we don't need pixel-parity — only "no glaring differences." We rejected a single shared renderer (Rust→WASM in the editor): it solved a parity problem we don't have, at the cost of rebuilding the editor around a canvas and inheriting tile-vs-page dithering problems.
**Alternatives considered:** Python (superseded — the original tentative plan); Node-on-device (rejected — heavy for a credit-card board); shared Rust→WASM renderer / "approach C" (rejected — self-inflicted complexity for unneeded parity).
**Consequences:** Editor and device keep separate render code; a render bug is fixed in two small places. Pixel-comparison tests become a generous-threshold guardrail against glaring divergence, not a tautology. See `context/protocol.md`, `context/architecture.md`, and `docs/specs/2026-06-27-device-server-design.md`.

### Device topology: device is the source of truth; plain-file storage; served from the device
**Date:** 2026-06-27
**Status:** Active
**Decision:** The device stores the **whole document** (all draft pages + `livePageId` + uploaded images) and is the single source of truth. Storage is **plain files**: `document.json`, an `images/` dir (GC unreferenced on publish), and `config.json` for device config + secrets. The device serves the built editor + the API + the preview on **one origin**, reachable at `http://<hostname>.local/` where `hostname` is a config value (default `corkboard`, not hardcoded); dev uses a Vite proxy. Endpoints + shape are in `context/protocol.md`.
**Reasoning:** Shared household board — saved layouts are shared state, so any browser should see them; data is tiny. One box, one origin keeps deployment trivial and avoids CORS. Plain files are enough; no database needed.
**Alternatives considered:** Live-page-only storage (rejected — drafts would be trapped per-browser); separate editor host (deferred — owner has future decoupling ideas, not now); a DB (unnecessary).
**Consequences:** Publish replaces the whole document. A `config.json` holds the first device secrets (feed URLs) — kept out of the document and out of git.

### Refresh-driven product simplification
**Date:** 2026-06-27
**Status:** Active
**Decision:** After watching real Spectra 6 refresh speed: **drop the clock** (keep the date as a calendar variant), **drop the timeline/auto-loop**, and keep **multiple draft pages with exactly one live** (`livePageId`). Element types become calendar / image / drawing.
**Reasoning:** The panel's full refresh is too slow/flashy to cycle pages or run a per-minute clock. Removing both removes all fast-refresh pressure; the device only re-renders on publish or real content change.
**Alternatives considered:** Single page only (rejected — owner wants several saved layouts); keeping the loop (rejected — refresh too slow).
**Consequences:** `DocState` drops `timeline`/`TimelineEntry` and `ClockEl`, gains `livePageId`. Editor surgery: remove Timeline, ClockWidget/ClockOptions, clock tool/thumbnail, timeline reorder; add a date calendar variant and a "make live" affordance.

### Calendar events from a Google secret iCal feed; poll + semantic change-detection
**Date:** 2026-06-27
**Status:** Active
**Decision:** Calendar events come from a **Google Calendar secret iCal (ICS) URL**, fetched by the device over HTTPS and resolved **at render time** (not stored in the document). `CalendarEl` references a `feedId` + variant; feeds live in `config.json` as `{id,name,secretUrl}`. **Refresh model:** poll the feed every `pollIntervalMinutes` (default 60); track the displayed **calendar content** (resolved events + date) and re-render + push to the panel **only when that content changes**; publish and `POST /api/refresh` (a "Refresh now" button) always re-render. Change-detection is **semantic**, never pixel-based.
**Reasoning:** A secret iCal URL needs no OAuth — just an authenticated GET. Decoupling cheap polling from the slow panel refresh protects the panel from needless flashy refreshes and wear.
**Alternatives considered:** Manual event entry (rejected — goes stale, stops being a calendar); full Google OAuth (rejected — token storage/refresh too heavy for a household board); render-and-hash change-detection (withdrawn — compare content, not pixels).
**Consequences:** First device→internet dependency and first secret. Secret URLs never enter the document, an API response, or git.

### Text tool + shared bundled fonts
**Date:** 2026-06-27
**Status:** Active
**Decision:** Add a **text element** (`TextEl`: `text`, `font`, `align` + shared `colour`/geometry) and a `text` tool — free text in a chosen font. **Fonts** are bundled, self-hosted open files (NOT the Google Fonts CDN): the device embeds/loads them for the Rust renderer and serves them to the editor as `@font-face` from the same origin, so both shape from the same bytes and it works offline. A bundled **font manifest** (name → file) is the shared list of what's available — separate from `config.json` (fonts aren't secret/per-device). Seed set: Atkinson Hyperlegible (default), Inter, Caveat. `TextEl.font` is document data; per-element font choice is the point of the text tool. Text content is edited **on the canvas** (click into the element to edit in place); the `TextOptions` panel holds font / alignment / colour.
**Reasoning:** "Free text" was always intended (project description). It's the main reason fonts matter, so it flips the earlier "one font, no per-element choice" lean. CDN fonts don't work for a Rust renderer or an offline LAN board; one bundled source of bytes is the only way to keep editor and device text matching.
**Alternatives considered:** Google Fonts CDN at runtime (rejected — device can't use it, breaks offline); fonts in `config.json` / drop-in fonts (deferred — more complexity than needed now); panel-only text entry (rejected — content is edited on the canvas in place).
**Consequences:** New `TextEl` + `text` tool, `TextWidget`/`TextOptions` in the editor; the Rust renderer's text-shaping path is shared by text and calendar; `assets/fonts/` + a manifest get bundled into both the web app and the device.
**Update (2026-06-28):** The actual bundled set is **Atkinson Hyperlegible (default), DejaVu Sans, Carlito, Gelasio** — Inter/Caveat were dropped (google/fonts only ships them variable; static TTFs needed for parity). Each family now ships **Regular (400) + Bold (700)**; the agenda's day headings use the bold face.

### Use Vue + Vite for the web UI
**Date:** 2026-06-23
**Status:** Active
**Decision:** Build the page-editing web UI with Vue and Vite (TypeScript).
**Reasoning:** Familiarity and past success with Vue/Vite.
**Alternatives considered:** React (rejected — less familiar here, no advantage for this project).
**Consequences:** Web UI follows Vue idioms; component-library choices target Vue.

### Use shadcn-vue for UI components
**Date:** 2026-06-23
**Status:** Active
**Decision:** Use shadcn-vue's component collection for the web UI.
**Reasoning:** Cohesive widgets that fit together with little effort, and they keep the AI agent from reinventing the wheel into something ugly.
**Alternatives considered:** Hand-rolled components (rejected — slower, inconsistent look); other Vue UI kits (not evaluated in depth).
**Consequences:** Prefer shadcn-vue components over custom UI; look there first when adding interface elements.

### Write the device-side code in Python (tentative)
**Date:** 2026-06-23
**Status:** SUPERSEDED 2026-06-27 by "Device server in Rust, two-renderer split (approach A)" above. Kept for history.
**Decision:** Plan to write the device server in Python.
**Reasoning:** Expect most Orange Pi Zero 2W examples and libraries to be in Python.
**Alternatives considered:** Node or other runtimes (not chosen — would diverge from the likely-available examples).
**Consequences:** The device server is a separate codebase from the TS web UI; they talk over HTTP/JSON. Not committed — revisit before significant device work.

### No authentication for now
**Date:** 2026-06-23
**Status:** Active
**Decision:** No user accounts or auth; assume the device sits on a trusted private network where anyone can GET/POST.
**Reasoning:** Simplicity for a shared household-style noticeboard.
**Alternatives considered:** Accounts/auth (rejected for now — unnecessary complexity on a private network).
**Consequences:** Endpoints are open. Do not add auth-dependent logic. Revisit if the device is ever exposed beyond a trusted network.

## Pending Decisions
These are not yet decided — record them as proper entries above once made:
- (none open — the round-two device design resolved the previous three: hosting, persistence, and the schema/endpoints. See the 2026-06-27 entries above and `context/protocol.md`.)
- Renderer crate choices (raster, text shaping, ICS parse, image decode) — to be recorded when the renderer is implemented.
- OS for the Orange Pi Zero 2W — some lightweight Linux, not yet chosen (`context/hardware.md`).
