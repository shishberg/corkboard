---
name: agents
description: Always-loaded project anchor. Read this first. Contains project identity, non-negotiables, commands, and pointer to ROUTER.md for full context.
last_updated: 2026-06-23
---

# Corkboard

## What This Is
Manages pages pushed for display on a shared device that acts as a noticeboard — current time, upcoming events, images, and free text.

## Non-Negotiables
[TO BE DETERMINED] — hard rules not yet defined; deferred by the owner until real constraints emerge.
- Decide what must never happen to the published page state or the display (e.g. never blank the noticeboard, never lose published pages).
- Known boundary so far: no auth — the device sits on a trusted private network where anyone can GET/POST. Do not add account/auth logic without revisiting this.

## Commands
[TO BE DETERMINED] — nothing is built yet.
- Web UI (Vite + Vue) will likely be `npm run dev` / `npm run build` once scaffolded.
- Device server commands pending the Python decision.
- Fill in the real commands after the first scaffold lands.

## Scaffold Growth
After meaningful work, run GROW:
- Ground: what changed in reality?
- Record: update `ROUTER.md` and relevant `context/` files
- Orient: create or update a `patterns/` runbook if this can recur
- Write: bump `last_updated` on changed scaffold files and run `mex log` when rationale matters

The scaffold grows from real work, not just setup. See the GROW step in `ROUTER.md` for details.

## Navigation
At the start of every session, read `ROUTER.md` before doing anything else.
For full project context, patterns, and task guidance — everything is there.
