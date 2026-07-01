---
name: hardware
description: The target device and display panel — physical constraints the renderer and device server must respect. Load when designing rendering, the device server, or anything that touches the panel.
triggers:
  - "panel"
  - "display"
  - "resolution"
  - "e-paper"
  - "epaper"
  - "refresh"
  - "waveshare"
  - "orange pi"
edges:
  - target: context/architecture.md
    condition: when relating the hardware to the system structure
  - target: context/decisions.md
    condition: when understanding why this hardware was chosen
last_updated: 2026-06-27
---

# Hardware

The single canonical record of the target device and display. Other docs link here
rather than restating these numbers. When the real hardware arrives, verify the
items marked **[UNVERIFIED]** against the datasheet and update this file.

## Display panel
- **Model:** Waveshare 7.3" E6 (Spectra 6) colour e-paper.
- **Resolution:** 800×480, native landscape. Portrait (480×800) is a logical rotation in the editor, not a different panel.
- **Colour model:** 6-colour palette — `black`, `white`, `red`, `yellow`, `blue`, `green`. Every pixel is quantised to exactly one of these six. This is the source of the `EpaperColour` union in `src/stores/types.ts`. There is no greyscale and no blending — rendering must dither/quantise to these six.
- **Background:** white (widgets render on white today).
- **Refresh:** full-panel refresh only, no partial refresh; a full refresh takes on the order of tens of seconds. **[UNVERIFIED — from Claude training knowledge of this product; confirm exact timing against the datasheet.]** Consequence: redraw rarely. The clock's useful granularity is minutes, not seconds; avoid any design that wants frequent updates.
- **Interface:** SPI (bus 0, device 0, 4MHz, mode 0) + 3-4 GPIO lines (RST, DC, BUSY, and PWR for the PhotoPainter carrier's power switch). Driver implemented in `device/src/panel.rs`, ported from Waveshare's own `epd7in3e` demo. **[UNVERIFIED — the actual GPIO chip/line numbers on the Orange Pi are not yet known; see `context/decisions.md`'s "Panel driver" entry.]**
- **Colour codes on the wire:** BLACK=0x0, WHITE=0x1, YELLOW=0x2, RED=0x3, BLUE=0x5, GREEN=0x6 (0x4 unused by the panel), packed two pixels per byte.
- **Carrier:** Waveshare PhotoPainter kit's carrier board (not a bare HAT wired directly to the header) — this is why the driver has a PWR line to enable panel power.

## Host device
- **Board:** Orange Pi Zero 2W (Allwinner H618, quad-core Cortex-A53). Runs the device server and drives the panel over its GPIO/SPI header.
- **OS:** Armbian (Debian flavour), headless — SSH only, no keyboard/mouse/GUI. See `context/decisions.md` for reasoning vs Orange Pi OS (Arch).
- **Hardware:** as of 2026-07-01 the Orange Pi is in hand; the panel is not yet delivered. Early device-server work (including the panel driver) is being built and tested via the web preview (`WebPreview`) standing in for the physical panel.

## Why this matters to rendering
- Output must be quantised to the 6-colour palette — the editor's on-screen colours map 1:1 to panel colours, so "what you see is what prints," subject to quantisation.
- 800×480 is the fixed canvas. Logical portrait swaps the axes.
- Slow full-refresh means the device renders on a timer/trigger, not continuously.
