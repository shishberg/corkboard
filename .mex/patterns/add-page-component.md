---
name: add-page-component
description: Add a new noticeboard page type or editing widget to the Vue web UI using shadcn-vue.
triggers:
  - "new page type"
  - "new widget"
  - "add component"
  - "editor tool"
  - "shadcn-vue"
edges:
  - target: context/stack.md
    condition: when you need to know which UI library/components are available
  - target: context/conventions.md
    condition: before writing UI code, to follow naming and structure conventions
  - target: patterns/device-api.md
    condition: when the new page type needs to round-trip through the page state
last_updated: 2026-06-24
---

# Add a Page Component / Editor Widget

## Context
Load `context/stack.md` (shadcn-vue + the element-type model) and `context/architecture.md`.
Components are plain Vue SFCs imported directly where used — there is no global registration.
Existing editor element types: `clock`, `calendar`, `image` (plus a `drawing` overlay placeholder).

## Steps (adding a new editor ELEMENT type — the common case)
1. Add the type to the discriminated union in `src/stores/types.ts` (e.g. `FooEl extends BaseEl { type: 'foo'; ... }`, add to the `El` union).
2. Extend `makeElement` in `src/stores/elementFactory.ts` with a default size + initial state for the new type.
3. Create the widget SFC in `src/components/widgets/FooWidget.vue` (prop `el: FooEl`, display-only).
4. Wire it into the `v-if` chains in `src/components/EditorCanvas.vue` AND `src/components/PageThumbnail.vue` (`v-else-if="el.type === 'foo'"`).
5. Add a tool button in `src/components/ToolRail.vue` (with `data-tool="foo"`) and, if it has options, a `ToolOptions/FooOptions.vue` popover editing its slice of `useToolOptionsStore`.
6. Add a colocated `*.test.ts`. Prefer shadcn-vue components for any new UI.

## Gotchas
- The display will eventually be rendered by the device, not the web UI — the web UI only edits state. (Device wiring is not built yet.)
- Persistence is frontend-only right now: tool options persist to localStorage; there is no page-state JSON contract or Publish serialization yet (Publish is a stub). A new type does NOT need to round-trip a contract this pass — that comes when the device API exists (see `patterns/device-api.md`).
- Forgetting to add the `v-else-if` branch in BOTH EditorCanvas and PageThumbnail leaves the element invisible in one place.

## Verify
- [ ] `npm test` and `npm run build` pass.
- [ ] New type is in the `El` union, `makeElement`, the EditorCanvas AND PageThumbnail render chains.
- [ ] Uses shadcn-vue components, not hand-rolled UI (where one exists).
- [ ] Appears on the canvas, in the sidebar thumbnail, and (once added to a page) is selectable/movable.

## Debug
- If the page type vanishes after Publish/reload, check the JSON serialize/deserialize path (see `patterns/device-api.md`).

## Update Scaffold
- [ ] Update `.mex/ROUTER.md` "Current Project State" if a new page type now works.
- [ ] Update `.mex/context/` files (e.g. record the page-state schema once defined).
- [ ] If a new recurring task emerged, add a pattern and update `INDEX.md`.
