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
  - target: context/architecture.md
    condition: to understand the full web UI to device flow
  - target: patterns/add-page-component.md
    condition: when a new page type needs to round-trip through the page state
last_updated: 2026-06-23
---

# Call the Device API (Load / Publish)

## Context
Load `context/architecture.md`. The flow: the web UI GETs the current page state (JSON) from
the device on load → user edits → **Publish** POSTs the new state → the device saves it and
refreshes the display. Images upload as separate POST requests. No auth — private-network
assumption.

## Steps
1. On load: GET the current page state and hydrate the editor from it.
2. On Publish: POST the full new page state as JSON.
3. For images: POST each image in its own request, then reference it from the page state. [TO BE DETERMINED — how images are referenced/stored]
4. Handle the device being unreachable — it's a single physical device that may be powered off.

## Gotchas
- Publish sends the whole page state, not a diff (current assumption). Don't drop existing pages when publishing one edit.
- No auth headers — do not add them; endpoints are open by design on a private network.
- Endpoints, JSON schema, and image handling are [TO BE DETERMINED] until the device server exists. [VERIFY AFTER FIRST IMPLEMENTATION]

## Verify
- [ ] Load hydrates the full editor state from the device.
- [ ] Publish round-trips: what you publish comes back on reload.
- [ ] Image uploads are referenced correctly from the page state.
- [ ] The unreachable-device case is handled gracefully.

## Debug
- Edits not showing on the device: confirm the Publish POST succeeded and the device refreshed the display.
- State lost on reload: check the GET/serialize path against what Publish wrote.

## Update Scaffold
- [ ] Record the page-state JSON schema and endpoint paths in `.mex/context/architecture.md` once defined.
- [ ] Update `.mex/ROUTER.md` "Current Project State" when load/publish works end to end.
