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
last_updated: 2026-06-28
---

# Setup

Two buildable parts: the **Vue editor** (`src/`) and the **Rust device server** (`device/`).
The device server is the single source of truth — it serves the built editor + JSON API +
`preview.png` on one LAN origin, and renders the live page to a 6-colour PNG. The only unbuilt
piece is the `Panel` SPI driver (needs hardware); a `WebPreview` display stands in for the panel.

## Prerequisites
- Node.js — Node 25 is in use here. Note: Node 25 ships a broken built-in `localStorage` (its `clear`/`setItem` are undefined without `--localstorage-file`); tests work around this with a jsdom shim in `src/test-setup.ts`.
- Device toolchain: a **Rust** toolchain (for the device server + renderer; runs on the Orange Pi Zero 2W, develops on any machine via the `WebPreview` display). Not needed for the web UI. [Specific targets/cross-compilation TBD when the device work starts.]

## First-time Setup
- `npm install`
- Editor only: `npm run dev` — start the Vite dev server.
- Full stack: `npm run build` first (so `dist/` exists), then
  `cd device && CORKBOARD_DIST=../dist CORKBOARD_FONTS=../public/fonts cargo run`.
  Open the device origin (default `http://localhost:8080`); `/preview.png` is the rendered panel.

## Environment Variables
Device server only (the editor itself needs none — it talks to the device on the same origin):
- `CORKBOARD_DATA` — document, images, config (default `device/data`).
- `CORKBOARD_PORT` — HTTP port (default `8080`).
- `CORKBOARD_DIST` — path to the built editor (`../dist`).
- `CORKBOARD_FONTS` — path to bundled fonts (`../public/fonts`).

## Common Commands
- `npm run dev` — Vite dev server.
- `npm run build` — type-check (`vue-tsc -b`) + production build.
- `npm test` — run the Vitest suite once.
- `npm run test:watch` — Vitest in watch mode.
- `npm run test:parity` — Playwright editor↔preview parity tests.
- `cargo test` (in `device/`) — device server tests.

## Common Issues
- **`@vueuse/core` build warnings** (`#__PURE__` / `INVALID_ANNOTATION`) — upstream noise from a `reka-ui` transitive dep, not our code; harmless.
- **`baseUrl` deprecation warning** from `vue-tsc` — non-fatal (TS 7.0 deprecation); `tsconfig.app.json` already sets `ignoreDeprecations: "6.0"`.
- **Test files must stay type-checked** — they're covered by `tsconfig.vitest.json` (referenced from `tsconfig.json`), not `tsconfig.app.json` (which excludes tests). Don't drop tests from the type-check.
