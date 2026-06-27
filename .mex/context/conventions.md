---
name: conventions
description: How code is written in this project — naming, structure, patterns, and style. Load when writing new code or reviewing existing code.
triggers:
  - "convention"
  - "pattern"
  - "naming"
  - "style"
  - "how should I"
  - "what's the right way"
edges:
  - target: context/architecture.md
    condition: when a convention depends on understanding the system structure
  - target: context/stack.md
    condition: when a convention depends on the chosen libraries (e.g. shadcn-vue)
last_updated: 2026-06-27
---

# Conventions

Kept deliberately light — the owner chose to grow conventions from real work rather than
set them up front. Add specifics here as the first code lands (see the GROW step in ROUTER.md).

## Naming
- Components: PascalCase `.vue` files (`EditorCanvas.vue`, `MovableElement.vue`), grouped in subfolders by role: `src/components/widgets/`, `src/components/ToolOptions/`, `src/components/ui/` (shadcn-vue).
- Stores: camelCase `.ts` in `src/stores/` (`pages.ts`, `toolOptions.ts`); shared types in `src/stores/types.ts` — defined once, imported everywhere (don't redefine `El`/`Page`/etc.).
- Composables: `src/composables/`, `useXxx` naming.
- Imports use the `@/` alias for `src/` (e.g. `@/stores/pages`).

## Structure
- All web-UI code under `src/`. Tests are colocated as `*.test.ts` next to the code they cover.
- `<script setup lang="ts">` everywhere; typed `defineProps`/`defineEmits`.
- The store owns document state and guards its own mutations (validates ids/indices); components call store actions rather than mutating state directly.
- DOM test hooks use `data-role="..."` / `data-tool="..."` attributes, queried in tests.
- (Future) the device server (Rust + axum) will be a separate codebase talking HTTP via the document contract in `context/protocol.md` — not built yet.

## Patterns
- Prefer existing shadcn-vue components over custom UI (see `stack.md`).
- Element types are a discriminated union on `type`; narrow with `v-if="el.type === '...'"` before passing to a typed widget prop.
- Geometry updates go through `store.updateElement(id, {x,y,w,h})` (geometry-only by design).

## Verify Checklist
Before presenting any code:
- [ ] `npm test` passes (Vitest) and `npm run build` is clean (this also type-checks via `vue-tsc -b`).
- [ ] Test files stay type-checked (covered by `tsconfig.vitest.json`; don't exclude them everywhere).
- [ ] Shared types come from `@/stores/types` (no local redefinition).
- [ ] New UI prefers shadcn-vue components over hand-rolled equivalents.
- [ ] No auth/account logic was added (private-network assumption holds).
- [ ] Commits stage only the task's own files (never `git add -A`). Tracked: source, `.mex/`, and `docs/specs/` + `docs/plans/`. Stay untracked: `.serena/`, `.vscode/`, `CLAUDE.md`, `.superpowers/`.
