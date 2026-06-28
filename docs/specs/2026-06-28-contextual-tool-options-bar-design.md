# Contextual tool-options bar

Date: 2026-06-28

## Goal

Move all tool settings out of the vertical `ToolRail` popovers into a single
horizontal options bar above the canvas. The bar shows only the settings
relevant to the current tool or the currently selected element. Drop the inline
labels, turn text-align into icons, and make the font apply to calendars (and any
future element type that contains text).

This is one instance of a general principle: **tools work consistently.** A
setting that edits the tool default also edits the selected element, and the same
relevance wiring decides visibility for every panel.

## Layout

The options bar sits over the canvas column only — to the right of the page
sidebar and tool rail, below the global `TopBar`, above `EditorCanvas`.

```
┌───────────────────────────────────────┐
│ Corkboard        orient  Preview  Pub  │  TopBar (unchanged)
├─────┬─────┬───────────────────────────┤
│pages│tool │ [settings]   ● ● ● ● ● ●   │  ToolOptionsBar (new)
│     │rail ├───────────────────────────┤
│     │     │         canvas            │
└─────┴─────┴───────────────────────────┘
```

The vertical `ToolRail` keeps the tool buttons (select / calendar / draw / image
/ text / background) plus the z-order and delete actions. It loses its setting
popovers and its colour panel.

Panels render left → right: tool-specific panels first, colour swatches last.
When the select tool is active with nothing selected, the bar is empty.

## Panel registry (the wiring)

`ToolOptionsBar` holds a declarative list of panels. Each entry is a component
plus its relevance:

```ts
interface PanelDef {
  component: Component
  tools: ToolId[]            // active tools that show this panel
  elementTypes: El['type'][] // selected element types that show this panel
}
```

A panel is visible when `activeTool ∈ tools` **OR**
`selectedEl?.type ∈ elementTypes`. A single `visiblePanels` computed filters the
list; the template renders the visible components in order.

| Panel            | tools                                  | elementTypes                  |
|------------------|----------------------------------------|-------------------------------|
| Calendar         | `calendar`                             | `calendar`                    |
| Font             | `calendar`, `text`                     | `calendar`, `text`            |
| Align            | `text`                                 | `text`                        |
| Pen size         | `draw`                                 | — (per-stroke; no retro-edit) |
| Colour swatches  | `draw`, `calendar`, `text`, `background` | `calendar`, `text`, `drawing` |

Colour is just the first panel that uses this wiring; everything else reuses it.

## Per-panel behaviour

Every panel edits the **selected element** when one is selected, otherwise the
tool default in `toolOptions`. Font and align already do this; calendar
variant/feed are extended to match.

- **Calendar** — variant buttons (Date / Today / Agenda) + feed `<select>`. The
  "Feed" label is removed. Writes to the selected calendar via new
  `setElementVariant` / `setElementFeed` mutators, else to
  `opts.calendarVariant` / `opts.feedId`.
- **Font** — font `<select>`, no "Font" label. Writes the selected text *or*
  calendar element via `setElementFont` (guard widened to `text | calendar`),
  else `opts.font`.
- **Align** — two icon buttons, `AlignLeft` / `AlignCenter` (lucide), replacing
  the "Left" / "Center" text buttons and the "Align" label. Text only.
- **Pen size** — unchanged dot buttons. Tool default only (a placed drawing has
  per-stroke sizes; no single value to edit).
- **Colour swatches** — the existing swatch panel and its target/value logic move
  out of `ToolRail` into `ColourSwatches.vue`. Behaviour unchanged: background
  tool → page background; else recolour the selected element and set the pen
  colour. The active swatch reflects the same source.

## Font on calendars

Calendars gain a `font`, rendered in both the editor preview and on the device.

**Editor**
- `CalendarEl` gains `font: string` (`src/stores/types.ts`).
- `elementFactory` sets `font: opts.font` when creating a calendar.
- `setElementFont` guard widens from `text` to `text | calendar`.
- `CalendarWidget` applies `fontFamily` to its root, falling back to the default
  font id when `font` is empty (so migrated/older calendars still render).

**Device (Rust)**
- `CalendarEl` in `device/src/document.rs` gains
  `#[serde(default)] pub font: String`.
- `render.rs` uses `faces.get(&el.font)` instead of `faces.default()` for the
  three calendar variants (Date / Today / Agenda). `Faces::get` already falls
  back to the default face on an empty/unknown id, so existing documents render
  exactly as before.

No migration code is needed: the new fields default to empty and both renderers
fall back to the default font.

## Files

**New**
- `src/components/ToolOptionsBar.vue` — registry + visibility filter.
- `src/components/ToolOptions/ColourSwatches.vue` — swatches + colour logic.
- `src/components/ToolOptions/FontOptions.vue`, `AlignOptions.vue` — split from
  `TextOptions.vue` (which is removed).

**Changed**
- `src/App.vue` — place `ToolOptionsBar` above `EditorCanvas`.
- `src/components/ToolRail.vue` — remove popovers + colour panel; keep tool
  buttons, z-order, delete.
- `src/components/ToolOptions/CalendarOptions.vue` — selected-element editing,
  drop "Feed" label.
- `src/components/widgets/CalendarWidget.vue` — apply font.
- `src/stores/types.ts` — `CalendarEl.font`.
- `src/stores/elementFactory.ts` — set calendar font.
- `src/stores/pages.ts` — widen `setElementFont`; add `setElementVariant`,
  `setElementFeed`.
- `device/src/document.rs` — `CalendarEl.font`.
- `device/src/render.rs` — calendar text uses the element's font.

## Tests

- `ToolRail.test` — assert popovers/colour gone; tool buttons + z-order + delete
  remain.
- New `ToolOptionsBar` test — panel visibility for each tool and each selected
  element type (the registry is the core of this change).
- `CalendarOptions` / new `FontOptions` / `AlignOptions` / `DrawOptions` tests —
  edit selected element vs tool default; align icons; no labels.
- `widgets.test` — calendar renders with its font.
- `pages` store tests — `setElementFont` on a calendar; `setElementVariant`,
  `setElementFeed`.
- Device: `document` round-trip with calendar font; `render` test that a calendar
  font change alters output; a parity test mirroring it.
