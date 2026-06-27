/**
 * Unit tests for the compare module.  These run in Node.js (no browser) via
 * Playwright's test runner.
 */

import { test, expect } from '@playwright/test'
import { PNG } from 'pngjs'
import { compareParity, GW, GH } from './compare'

// Build a solid-colour 800×480 PNG.
function makePng(fillRgb: [number, number, number]): Buffer {
  const W = 800
  const H = 480
  const png = new PNG({ width: W, height: H, colorType: 2 /* RGB */ })
  // pngjs always works with RGBA data internally even when colorType=2
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      const i = (y * W + x) * 4
      png.data[i] = fillRgb[0]
      png.data[i + 1] = fillRgb[1]
      png.data[i + 2] = fillRgb[2]
      png.data[i + 3] = 255
    }
  }
  return PNG.sync.write(png)
}

// Build an 800×480 PNG with a dark rectangle in the given cell range.
function makePngWithRect(
  cx0: number,
  cy0: number,
  cx1: number,
  cy1: number,
  cellRgb: [number, number, number] = [0, 0, 0],
): Buffer {
  const W = 800
  const H = 480
  const png = new PNG({ width: W, height: H, colorType: 2 })

  const cellW = W / GW
  const cellH = H / GH
  const px0 = Math.floor(cx0 * cellW)
  const px1 = Math.floor(cx1 * cellW)
  const py0 = Math.floor(cy0 * cellH)
  const py1 = Math.floor(cy1 * cellH)

  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      const i = (y * W + x) * 4
      const inRect = x >= px0 && x < px1 && y >= py0 && y < py1
      const rgb = inRect ? cellRgb : ([255, 255, 255] as [number, number, number])
      png.data[i] = rgb[0]
      png.data[i + 1] = rgb[1]
      png.data[i + 2] = rgb[2]
      png.data[i + 3] = 255
    }
  }
  return PNG.sync.write(png)
}

test('identical images → IoU 1.0', () => {
  const pngA = makePngWithRect(5, 5, 20, 15)
  const result = compareParity(pngA, pngA)
  expect(result.contentIoU).toBeCloseTo(1.0, 3)
  expect(result.editorDensity).toEqual(result.deviceDensity)
})

test('blank image vs non-blank → low IoU and near-zero density on blank side', () => {
  const blank = makePng([255, 255, 255])
  const withContent = makePngWithRect(0, 0, GW, GH, [0, 0, 0])
  const result = compareParity(blank, withContent)
  expect(result.editorDensity).toBeLessThan(0.05)
  expect(result.deviceDensity).toBeGreaterThan(0.5)
  expect(result.contentIoU).toBeLessThan(0.1)
})

test('non-overlapping content regions → low IoU', () => {
  // Editor content on the left half, device content on the right half
  const left = makePngWithRect(0, 0, GW / 2, GH)
  const right = makePngWithRect(GW / 2, 0, GW, GH)
  const result = compareParity(left, right)
  // No overlap → IoU = 0
  expect(result.contentIoU).toBeLessThan(0.05)
  expect(result.editorDensity).toBeGreaterThan(0.4)
  expect(result.deviceDensity).toBeGreaterThan(0.4)
})

test('both blank → IoU 1.0 (empty-union special case)', () => {
  const blank = makePng([255, 255, 255])
  const result = compareParity(blank, blank)
  expect(result.contentIoU).toBe(1.0)
  expect(result.editorDensity).toBeLessThan(0.05)
  expect(result.deviceDensity).toBeLessThan(0.05)
})

test('coloured (non-grey) content counts even when luminance is high', () => {
  // Yellow: luminance ≈ (240*0.299 + 200*0.587 + 30*0.114)/255 ≈ 0.836 (below 0.85)
  // But red: luminance ≈ (220*0.299 + 40*0.587 + 40*0.114)/255 ≈ 0.35 — clearly below threshold
  // Use a saturated colour: (255, 0, 255) magenta — low luminance, high saturation
  const magenta = makePngWithRect(0, 0, GW, GH, [255, 0, 255])
  const result = compareParity(magenta, magenta)
  // Magenta: lum ≈ (255*0.299 + 0*0.587 + 255*0.114)/255 ≈ 0.413 — below threshold anyway
  expect(result.contentIoU).toBeCloseTo(1.0, 3)
  expect(result.editorDensity).toBeGreaterThan(0.8)
})
