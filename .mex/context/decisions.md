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
last_updated: 2026-06-23
---

# Decisions

<!-- When a decision changes: DO NOT delete the old entry. Mark it superseded and add the
     new entry above it. The history is the event clock. -->

## Decision Log

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
**Status:** Active (tentative — open to change)
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
- Where the web UI is hosted: served from the device, or hosted elsewhere.
- Persistence on the device: how page state and uploaded images are stored.
- The page-state JSON schema and the endpoint paths.
