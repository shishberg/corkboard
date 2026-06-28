---
name: protocol
description: The wire contract between the editor and the device — endpoints, document shape, image handling, config/secrets, and refresh semantics. Load when implementing or changing the load/publish/preview/feed API.
triggers:
  - "endpoint"
  - "api"
  - "contract"
  - "wire"
  - "document json"
  - "publish"
  - "preview"
  - "feed"
edges:
  - target: context/architecture.md
    condition: for the high-level component overview
  - target: context/hardware.md
    condition: for the panel the renderer targets
  - target: patterns/device-api.md
    condition: for the how-to runbook of the load/publish flow
last_updated: 2026-06-28
---

# Editor ⇄ Device protocol

The canonical contract. The device is the single source of truth; it stores the whole
document and serves the editor, the API, and the preview on one origin (LAN, no auth).

## Endpoints
| Method + path | Purpose |
|---|---|
| `GET /` | The built Vue editor (static). |
| `GET /api/document` | The full document JSON (hydrates the editor). |
| `PUT /api/document` | Replace the whole document (publish). Triggers render + conditional panel push. Not a diff. |
| `POST /api/images` | Upload one image; returns `{ id }`. |
| `GET /api/images/:id` | Image bytes. |
| `GET /api/feeds` | List configured calendar feeds — `{ id, name }` only. **Secret URLs are never returned.** |
| `PUT /api/feeds` | Add/update/remove feeds; secret URLs are write-only. |
| `POST /api/refresh` | Force an immediate feed fetch + re-render now (the "Refresh now" button). |
| `GET /preview.png` | The current rendered 800×480 image — authoritative "what it looks like", and the stand-in until hardware exists. |

## Document shape (lives in `src/stores/types.ts`)
The editor's `DocState` IS the wire format. Round-two changes from the round-one shape:
- **Removed:** `ClockEl`; `timeline` / `TimelineEntry`.
- **Added:** `livePageId: string | null` on `DocState` (which page is displayed).
- **Added:** `Page.background?: EpaperColour` (per-page background; absent = white). The device
  fills the surface with it before drawing elements (`#[serde(default)]`, so older docs parse).
- **Changed:** `CalendarEl` drops the frozen `events: CalEvent[]` and instead holds
  `feedId: string` + `variant: 'date' | 'agenda'` + `font: string` + `align: 'left' | 'center'`
  + `daysAhead: number` (agenda horizon, 1..=7, default 7). Events are resolved on the device
  at render time, never stored in the document. (`agenda` was renamed from `week`, and the old
  single-day `today` variant was folded into `agenda`; the device accepts both `week` and `today`
  as serde aliases and the editor migrates them on load, so older docs still parse. `align` and
  `daysAhead` are `#[serde(default)]` — missing `align` defaults to centre, `daysAhead` to 7.)

- **Added:** `TextEl` (`type: 'text'`) — `text: string`, `font: string` (a name from the font
  manifest), `align: 'left' | 'center'`, plus the shared `colour` / `x` / `y` / `w` / `h`.

Element types are a discriminated union on `type`: `calendar | text | image | drawing`.

Fonts: text is shaped from bundled font files (default Atkinson Hyperlegible), served by the
device to the editor as `@font-face` and loaded directly by the Rust renderer. The available
set is a bundled **font manifest** (name → file), shared by both — NOT in `config.json`. Each
font ships a **regular (400) and bold (700)** face; the agenda's day headings render in the bold
face on both sides. `TextEl.font` / `CalendarEl.font` pick from that set. See the spec's Fonts section.

## Images
Uploaded separately (`POST /api/images`), referenced from the document by id (`ImageEl.src`
holds the id, not bytes). On publish, the device GCs any stored image not referenced by any
page.

## Config / secrets (`config.json`, NOT in the document, NOT in git)
- `feeds`: `[{ id, name, secretUrl }]` — Google Calendar secret iCal (ICS) URLs. Secret URLs
  never leave the device (the API exposes names only).
- `pollIntervalMinutes`: how often to poll the feed (default `60`).
- `hostname`: the mDNS name the device advertises (default `corkboard`), so it's reachable at
  `http://<hostname>.local/`. Not hardcoded anywhere.

## Refresh semantics
- Poll the feed every `pollIntervalMinutes`.
- The device tracks the currently-displayed **calendar content** (resolved events + date for
  the live page).
- On each poll: re-resolve and compare that content. Changed → re-render + push to panel.
  Unchanged → do nothing (protects the slow panel from needless refreshes).
- `PUT /api/document` (publish) and `POST /api/refresh` always re-render + push.
- Change-detection is **semantic** (compare resolved content), never pixel-based.

## Invariants
- No auth headers — endpoints are open on a trusted LAN. Do not add auth.
- Publish sends the **whole** document; never drop existing pages when publishing one edit.
- The editor must tolerate the device being unreachable (single physical device, may be off).
- Secret feed URLs never appear in `document.json`, in any API response, or in git.
