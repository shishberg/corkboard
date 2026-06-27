---
name: stack
description: Technology stack, library choices, and the reasoning behind them. Load when working with specific technologies or making decisions about libraries and tools.
triggers:
  - "library"
  - "package"
  - "dependency"
  - "which tool"
  - "technology"
edges:
  - target: context/decisions.md
    condition: when the reasoning behind a tech choice is needed
  - target: context/conventions.md
    condition: when understanding how to use a technology in this codebase
  - target: patterns/add-page-component.md
    condition: when adding web UI with shadcn-vue components
last_updated: 2026-06-27
---

# Stack

## Core Technologies
- **TypeScript** — language for the web UI.
- **Vite + Vue 3** (`<script setup>`) — build tool and framework for the web UI.
- **Pinia** — state management. The document state (`usePagesStore`) is the single source of truth and the draft page-state contract.
- **Vitest + @vue/test-utils + jsdom** — testing.
- **Rust + axum (device server + renderer)** — the device-side language and web framework (decided 2026-06-27; supersedes the earlier tentative Python plan). Separate codebase from the TS web UI; not started yet. Renderer crates TBD at implementation time.

## Key Libraries
- **Tailwind CSS v4** — via the `@tailwindcss/vite` plugin, CSS-first config (no `tailwind.config.js` / `postcss.config.js`). Theme tokens live in `src/style.css`; base color `zinc`.
- **shadcn-vue** (on `reka-ui`) — base components live in `src/components/ui/` (Button, Popover, Tooltip). Prefer these over hand-rolling UI. The `cn()` helper is in `src/lib/utils.ts`. (Note: ToolRail currently hand-styles native `<button>`s for the icon rail — a known consistency nit, not shadcn `Button`.)
- **@lucide/vue** — icons. NOT `lucide-vue-next`, which is deprecated; `@lucide/vue` is its drop-in successor (same PascalCase exports).
- **Hand-rolled drag/resize** — `useDraggableResizable` composable + `MovableElement`, no third-party DnD library. Pointer deltas are divided by the canvas `scale()` so dragging tracks the pointer when letterboxed.
- **HTTP client (editor ↔ device)** — a `fetch`-based API client for the endpoints in `context/protocol.md` (GET/PUT document, image upload, feeds, refresh, preview). Not built yet.

## What We Deliberately Do NOT Use
- No auth/session libraries — no accounts for now (private-network assumption).
- No hand-rolled UI widgets where shadcn-vue already has one.
- No third-party drag-and-drop library — drag/resize and timeline DnD are hand-rolled.
- No `lucide-vue-next` (deprecated) — use `@lucide/vue`.

## Version Constraints
- Tailwind **v4** (not v3) — the CSS-first / Vite-plugin path, not PostCSS.
- shadcn-vue v4 base colors no longer include `slate`; we use `zinc`.
