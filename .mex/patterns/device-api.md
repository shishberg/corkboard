---
name: device-api
description: Load and publish page state between the web UI and the device server (GET state, POST publish, POST image upload).
triggers:
  - "publish"
  - "load pages"
  - "GET state"
  - "POST"
  - "device api"
  - "image upload"
edges:
  - target: context/protocol.md
    condition: for the canonical endpoints, document shape, config/secrets, and refresh semantics
  - target: context/architecture.md
    condition: to understand the full editor-to-device flow
  - target: patterns/add-page-component.md
    condition: when a new page type needs to round-trip through the page state
last_updated: 2026-06-27
---

# Call the Device API (Load / Publish / Preview / Refresh)

## Context
The **canonical contract is `context/protocol.md`** — read it first. Summary: the editor GETs
the whole document on load → user edits draft pages and marks one live → **Publish** PUTs the
whole document → the device re-renders and pushes to the panel only if the result changed.
Images upload separately and are referenced by id. The device serves a `preview.png` (the
real render) and exposes `POST /api/refresh` for a "Refresh now" button. No auth (LAN).

## Steps
1. On load: `GET /api/document` and hydrate the editor.
2. On Publish: `PUT /api/document` with the full document JSON (not a diff).
3. Images: `POST /api/images` per image → store the returned `id` in `ImageEl.src` and
   reference it from the page. Never inline image bytes in the document.
4. Calendar feeds: managed via `GET`/`PUT /api/feeds` (names only; secret URLs are write-only).
5. "Refresh now": `POST /api/refresh` to force an immediate feed fetch + re-render.
6. Handle the device being unreachable — a single physical device that may be powered off.

## Gotchas
- Publish sends the **whole** document, not a diff. Don't drop existing pages when publishing one edit.
- No auth headers — do not add them; endpoints are open by design on a private network.
- Secret feed URLs must never appear in the document, an API response, or git — they live only in the device's `config.json`.
- Calendar events are NOT in the document — the device resolves them from the feed at render time. The editor previews with sample events.

## Verify
- [ ] Load hydrates the full editor state from the device.
- [ ] Publish round-trips: what you publish comes back on reload.
- [ ] Image uploads are referenced by id and render correctly.
- [ ] `preview.png` reflects the published live page.
- [ ] "Refresh now" re-fetches the feed and updates the preview when events changed.
- [ ] The unreachable-device case is handled gracefully.

## Debug
- Edits not showing on the device: confirm the `PUT` succeeded and the live page actually changed (the panel only refreshes on change).
- State lost on reload: check the GET/serialize path against what Publish wrote.

## Update Scaffold
- [ ] Keep `context/protocol.md` in sync if endpoints, the document shape, or refresh semantics change.
- [ ] Update `.mex/ROUTER.md` "Current Project State" when load/publish works end to end.
