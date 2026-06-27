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
- **Interface:** SPI, driven from the host's GPIO header (typical for Waveshare e-paper). **[UNVERIFIED — confirm pinout/driver against the panel + host when hardware lands.]**

## Host device
- **Board:** Orange Pi Zero 2W (Allwinner H618, quad-core Cortex-A53). Runs the device server and drives the panel over its GPIO/SPI header.
- **OS:** not yet chosen — some lightweight Linux. **[TO BE DETERMINED]**
- **Hardware not yet in hand:** as of 2026-06-27 the board has not been delivered. Early device-server work is being built to run on a normal dev machine with a web preview standing in for the panel, so it can be developed before the hardware exists.

## Why this matters to rendering
- Output must be quantised to the 6-colour palette — the editor's on-screen colours map 1:1 to panel colours, so "what you see is what prints," subject to quantisation.
- 800×480 is the fixed canvas. Logical portrait swaps the axes.
- Slow full-refresh means the device renders on a timer/trigger, not continuously.
