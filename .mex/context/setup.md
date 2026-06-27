---
name: setup
description: Dev environment setup and commands. Load when setting up the project for the first time or when environment issues arise.
triggers:
  - "setup"
  - "install"
  - "environment"
  - "getting started"
  - "how do I run"
  - "local development"
edges:
  - target: context/stack.md
    condition: when specific technology versions or library details are needed
  - target: context/architecture.md
    condition: when understanding how components connect during setup
last_updated: 2026-06-27
---

# Setup

The frontend web UI is the only buildable thing so far (frontend-only; no device server yet).

## Prerequisites
- Node.js — Node 25 is in use here. Note: Node 25 ships a broken built-in `localStorage` (its `clear`/`setItem` are undefined without `--localstorage-file`); tests work around this with a jsdom shim in `src/test-setup.ts`.
- Device toolchain: a **Rust** toolchain (for the device server + renderer; runs on the Orange Pi Zero 2W, develops on any machine via the `WebPreview` display). Not needed for the web UI. [Specific targets/cross-compilation TBD when the device work starts.]

## First-time Setup
- `npm install`
- `npm run dev` — start the Vite dev server.

## Environment Variables
None. The web UI is frontend-only; there's no device host to point at yet. (A device address will likely appear once the page-state contract exists.)

## Common Commands
- `npm run dev` — Vite dev server.
- `npm run build` — type-check (`vue-tsc -b`) + production build.
- `npm test` — run the Vitest suite once.
- `npm run test:watch` — Vitest in watch mode.

## Common Issues
- **`@vueuse/core` build warnings** (`#__PURE__` / `INVALID_ANNOTATION`) — upstream noise from a `reka-ui` transitive dep, not our code; harmless.
- **`baseUrl` deprecation warning** from `vue-tsc` — non-fatal (TS 7.0 deprecation); `tsconfig.app.json` already sets `ignoreDeprecations: "6.0"`.
- **Test files must stay type-checked** — they're covered by `tsconfig.vitest.json` (referenced from `tsconfig.json`), not `tsconfig.app.json` (which excludes tests). Don't drop tests from the type-check.
