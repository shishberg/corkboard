# Corkboard

A WYSIWYG editor and device server for a wall-mounted **e-paper noticeboard**.

You lay out pages in the browser — text, images, drawings, and a calendar fed from
an iCal URL — and the device renders the live page to a Waveshare 7.3" 6-colour
e-paper panel (800×480). What you see in the editor is what the panel shows.

## Parts

- **Editor** (`src/`) — Vite + Vue 3 + TypeScript + Pinia + Tailwind v4 + shadcn-vue.
  Draw-to-place elements, per-page orientation, a global 6-colour palette, bundled
  fonts, and a Preview link to the device's rendered PNG.
- **Device server** (`device/`) — Rust + axum. The single source of truth: stores the
  document, images, and config as plain files; serves the built editor and JSON API on
  one LAN origin; renders the live page to a 6-colour PNG (tiny-skia + ab_glyph, with
  Floyd–Steinberg dithering); resolves calendar events from an iCal feed.

The editor hydrates from `GET /api/document` on startup; Publish does `PUT /api/document`,
which re-resolves feeds and re-renders.

## Status

Round two is complete end-to-end **except the hardware deploy**. Both halves work against
a web preview standing in for the panel. The `Panel` SPI driver (behind the `Display`
trait, `device/src/panel.rs`, Linux-only) is now written — ported from Waveshare's own
`epd7in3e` demo — but untested against real hardware: the panel isn't in hand yet, and the
GPIO chip/line numbers are unverified (see `.mex/context/decisions.md`).

Tests: 216 Vitest + 6 Playwright parity (editor), 119 cargo tests (device). All green.

## Requirements

- **Node.js** (Node 25 in use). Node 25 ships a broken built-in `localStorage`; tests
  shim around it in `src/test-setup.ts`.
- **Rust toolchain** — for the device server and renderer. Not needed for the editor alone.

## Running it

Editor only (dev):

```sh
npm install
npm run dev
```

Full stack — build the editor, then run the device server (it serves `../dist`):

```sh
npm run build
cd device && CORKBOARD_DIST=../dist CORKBOARD_FONTS=../public/fonts cargo run
```

Then open the device origin (default `http://localhost:8080`). `/preview.png` is the
rendered panel image.

### Device env vars

| Var               | Default          | What                         |
|-------------------|------------------|------------------------------|
| `CORKBOARD_DATA`  | `device/data`    | Document, images, config     |
| `CORKBOARD_PORT`  | `8080`           | HTTP port                    |
| `CORKBOARD_DIST`  | —                | Path to the built editor     |
| `CORKBOARD_FONTS` | —                | Path to bundled fonts        |
| `CORKBOARD_DISPLAY` | (web preview)  | Set to `panel` to drive the real e-paper panel (Linux only) instead of just the web preview |
| `CORKBOARD_PANEL_SPI` | `/dev/spidev0.0` | SPI device for the panel |
| `CORKBOARD_PANEL_GPIOCHIP` | —        | gpiochip device for RST/DC/BUSY/PWR (required when `CORKBOARD_DISPLAY=panel`) |
| `CORKBOARD_PANEL_RST_LINE` / `_DC_LINE` / `_BUSY_LINE` | — | GPIO line offsets (required when `CORKBOARD_DISPLAY=panel`) |
| `CORKBOARD_PANEL_PWR_LINE` | —        | Power-enable line for the PhotoPainter carrier board. Required unless `CORKBOARD_PANEL_NO_PWR=1` — a missing value is a hard error, never a silent "no gate" |
| `CORKBOARD_PANEL_NO_PWR` | —          | Set to `1` to explicitly opt out of a power gate (bare HAT with no PWR line to switch) |

## Commands

| Command                       | What                                   |
|-------------------------------|----------------------------------------|
| `npm run dev`                 | Vite dev server                        |
| `npm run build`               | Type-check (`vue-tsc -b`) + build      |
| `npm test`                    | Vitest suite once                      |
| `npm run test:watch`          | Vitest in watch mode                   |
| `npm run test:parity`         | Playwright editor↔preview parity tests |
| `cargo test` (in `device/`)   | Device server tests                    |
| `cargo check --target aarch64-unknown-linux-gnu` (in `device/`) | Type-check the Linux-only panel driver — `src/panel.rs` is `#[cfg(target_os = "linux")]`-gated, so a dev-Mac `cargo check`/`test` never compiles it. Needs the aarch64 cross C toolchain (`aarch64-linux-gnu-gcc`) for the bundled C deps (freetype, ring); runs clean on the target/CI. |

## Target hardware

- **Panel:** Waveshare 7.3" E6 (Spectra 6) — 800×480, 6 colours (black, white, red,
  yellow, blue, green), full-panel refresh only (tens of seconds; redraw rarely).
- **Host:** Orange Pi Zero 2W, driving the panel over SPI.

## Layout

- `src/` — Vue editor
- `device/` — Rust device server and renderer
- `public/fonts/` — bundled font families (Atkinson Hyperlegible, DejaVu Sans, Carlito, Gelasio), each Regular + Bold
- `docs/specs/` — design docs
- `.mex/` — project context and patterns (start at `.mex/ROUTER.md`)
