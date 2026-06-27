---
name: architecture
description: How the major pieces of this project connect and flow. Load when working on system design, integrations, or understanding how components interact.
triggers:
  - "architecture"
  - "system design"
  - "how does X connect to Y"
  - "integration"
  - "flow"
edges:
  - target: context/stack.md
    condition: when specific technology details are needed
  - target: context/decisions.md
    condition: when understanding why the architecture is structured this way
  - target: patterns/device-api.md
    condition: when implementing or debugging the load/publish flow between web UI and device
last_updated: 2026-06-27
---

# Architecture

## System Overview
The device is the single source of truth. It runs everything on one origin (LAN, no auth):
the built Vue editor, the JSON API, and a preview image. The editor is a *design surface*;
the device's renderer is *authoritative* (see the two-renderer decision in
`context/decisions.md`).

Flow: the editor GETs the whole document (JSON) from the device → user edits one of several
draft pages and marks one **live** → **Publish** PUTs the whole document back → the device
re-renders the live page and pushes it to the panel only if the result changed. Images
upload as separate POSTs and are referenced by id. Calendar events are not stored in the
document — the device fetches them from a Google secret iCal feed at render time.

**The exact wire contract (endpoints, document shape, config/secrets, refresh semantics)
lives in `context/protocol.md`.** Hardware constraints live in `context/hardware.md`.

## Key Components
- **Editor (Vue)** — the design surface. Page sidebar of draft thumbnails, a tool rail, and
  per-element options. Renders a *rough* preview (approximate; allowed to drift from the
  device). Talks to the device over HTTP; tolerates the device being unreachable.
- **Device server (Rust + axum)** — serves the editor, the API, and `preview.png`. Stores the
  whole document + images + config as plain files. Holds the **renderer** and the calendar
  **feed poller**.
- **Renderer (Rust)** — `page-state → 800×480 6-colour buffer`, behind a `Display` trait with
  two impls: `WebPreview` (HTTP image; the no-hardware path) and `Panel` (SPI, built last).
- **Document** — the JSON the editor and device share; the editor's `DocState` *is* the wire
  format. Shape + round-two changes are in `context/protocol.md`.

## External Dependencies
- **The noticeboard device** — Orange Pi Zero 2W driving a Waveshare 7.3" E6 panel (800×480,
  6-colour). Full specs: `context/hardware.md`.
- **On-device storage** — plain files: `document.json`, an `images/` dir (GC unreferenced on
  publish), and `config.json` for device config + secrets. No database.
- **Google Calendar (secret iCal feed)** — the first device→internet dependency. The device
  fetches an ICS URL over HTTPS and resolves events at render time. The secret URL lives in
  `config.json`, never in the document or git.

## What Does NOT Exist Here
- No user accounts and no authentication — trusted private network; anyone on the LAN can GET/POST.
- No cloud service — the device is the source of truth; no central backend in between. (Owner has future editor/display decoupling ideas, deferred.)
- No clock and no page loop/timeline — dropped because the panel's full refresh is too slow; the date is a calendar variant.
- The editor does **not** own authoritative rendering — it shows a rough design preview; the device renders the real thing. (This is a change from round one, when the editor did no display rendering at all.)
