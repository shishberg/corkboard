/**
 * Coarse content-mask comparison for editor↔device render parity.
 *
 * Both images are downscaled to a GW×GH grid.  A cell is "content" when its
 * average luminance is below the threshold OR its average colour saturation is
 * clearly non-grey.  Content-IoU measures the overlap of these masks.
 *
 * The grid is deliberately coarse so font/anti-alias/subpixel drift between
 * the browser renderer and the Rust renderer does NOT fail the test.  The
 * metric catches a missing or badly misplaced element, not minor rendering
 * differences.
 */

import { PNG } from 'pngjs'

// Grid dimensions (cells).  800÷40=20 px/cell, 480÷24=20 px/cell at source size.
export const GW = 40
export const GH = 24

// A cell with average luminance below this OR saturation above SATURATION_THRESH
// is considered "content" (non-background).
const LUMINANCE_THRESH = 0.85
const SATURATION_THRESH = 0.15

interface CellStats {
  lum: number
  sat: number
}

function cellStats(
  data: Uint8Array | Buffer,
  imgW: number,
  imgH: number,
  cx: number,
  cy: number,
): CellStats {
  const x0 = Math.floor((cx * imgW) / GW)
  const x1 = Math.min(imgW, Math.floor(((cx + 1) * imgW) / GW))
  const y0 = Math.floor((cy * imgH) / GH)
  const y1 = Math.min(imgH, Math.floor(((cy + 1) * imgH) / GH))

  let sumLum = 0
  let sumSat = 0
  let count = 0

  for (let py = y0; py < y1; py++) {
    for (let px = x0; px < x1; px++) {
      const i = (py * imgW + px) * 4 // RGBA
      const r = data[i] / 255
      const g = data[i + 1] / 255
      const b = data[i + 2] / 255

      // Weighted luminance (Rec. 601)
      const lum = r * 0.299 + g * 0.587 + b * 0.114
      sumLum += lum

      // HSL saturation
      const max = Math.max(r, g, b)
      const min = Math.min(r, g, b)
      const l = (max + min) / 2
      const sat =
        max === min
          ? 0
          : l < 0.5
            ? (max - min) / (max + min)
            : (max - min) / (2 - max - min)
      sumSat += sat
      count++
    }
  }

  if (count === 0) return { lum: 1, sat: 0 }
  return { lum: sumLum / count, sat: sumSat / count }
}

function buildMask(buf: Buffer): boolean[][] {
  const png = PNG.sync.read(buf)
  const { width, height, data } = png

  const mask: boolean[][] = []
  for (let cy = 0; cy < GH; cy++) {
    mask[cy] = []
    for (let cx = 0; cx < GW; cx++) {
      const { lum, sat } = cellStats(data as Buffer, width, height, cx, cy)
      mask[cy][cx] = lum < LUMINANCE_THRESH || sat > SATURATION_THRESH
    }
  }
  return mask
}

export interface ParityResult {
  contentIoU: number
  editorDensity: number
  deviceDensity: number
}

/**
 * Compare two PNG buffers (editor screenshot vs device preview.png).
 *
 * Returns:
 *   contentIoU      — Jaccard index of content-cell masks (0–1; 1 if both blank)
 *   editorDensity   — fraction of cells that are "content" in the editor image
 *   deviceDensity   — fraction of cells that are "content" in the device image
 */
export function compareParity(editorPng: Buffer, devicePng: Buffer): ParityResult {
  const maskE = buildMask(editorPng)
  const maskD = buildMask(devicePng)

  let intersection = 0
  let union = 0
  let editorContent = 0
  let deviceContent = 0

  for (let cy = 0; cy < GH; cy++) {
    for (let cx = 0; cx < GW; cx++) {
      const e = maskE[cy][cx]
      const d = maskD[cy][cx]
      if (e) editorContent++
      if (d) deviceContent++
      if (e || d) union++
      if (e && d) intersection++
    }
  }

  const total = GW * GH
  return {
    contentIoU: union === 0 ? 1.0 : intersection / union,
    editorDensity: editorContent / total,
    deviceDensity: deviceContent / total,
  }
}
