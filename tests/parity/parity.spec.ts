/**
 * Parity guardrail: renders the SAME document through both the Vue editor
 * (Chromium screenshot) and the Rust device (preview.png) and asserts that
 * the content regions broadly overlap.
 *
 * Prerequisites (built in beforeAll if absent):
 *   - dist/           (npm run build)
 *   - device/target/debug/corkboard-device  (cargo build in device/)
 *
 * The test document uses NO calendar feed so both renderers fall back to the
 * same deterministic sample data — making the output stable and comparable.
 */

import { test, expect, type Page } from '@playwright/test'
import { spawn, execSync, type ChildProcess } from 'child_process'
import * as fs from 'fs'
import * as net from 'net'
import * as os from 'os'
import * as path from 'path'
import { fileURLToPath } from 'url'
import { compareParity } from './compare'

// ── Paths ─────────────────────────────────────────────────────────────────────

// __dirname is not available in ES module scope; derive it from import.meta.url.
const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

const ROOT = path.resolve(__dirname, '../..')
const DIST = path.join(ROOT, 'dist')
const DEVICE_BIN = path.join(ROOT, 'device/target/debug/corkboard-device')
const FONTS = path.join(ROOT, 'public/fonts')

// ── Test document ─────────────────────────────────────────────────────────────
// One landscape page with text, calendar (agenda variant, no feed → sample data),
// and drawing.  Coordinates are well inside the 800×480 canvas.

const PAGE_ID = 'parity-page-1'

const TEST_DOC = {
  orientation: 'landscape',
  pages: [
    {
      id: PAGE_ID,
      name: 'Parity Test',
      elements: [
        // Text element — top-left quarter
        {
          type: 'text',
          id: 'parity-text-1',
          x: 20,
          y: 20,
          w: 400,
          h: 80,
          colour: 'blue',
          text: 'Hello Corkboard',
          font: 'atkinson-hyperlegible',
          align: 'left',
        },
        // Calendar element — agenda variant, mid-left
        {
          type: 'calendar',
          id: 'parity-cal-1',
          x: 20,
          y: 120,
          w: 380,
          h: 340,
          colour: 'black',
          variant: 'agenda',
          feedId: '',
        },
        // Drawing element — right side, diagonal cross pattern
        {
          type: 'drawing',
          id: 'parity-draw-1',
          x: 450,
          y: 20,
          w: 320,
          h: 440,
          colour: 'black',
          natW: 320,
          natH: 440,
          strokes: [
            {
              colour: 'black',
              size: 8,
              points: [
                { x: 20, y: 20 },
                { x: 160, y: 220 },
                { x: 300, y: 420 },
              ],
            },
            {
              colour: 'black',
              size: 8,
              points: [
                { x: 300, y: 20 },
                { x: 160, y: 220 },
                { x: 20, y: 420 },
              ],
            },
          ],
        },
      ],
    },
  ],
  livePageId: PAGE_ID,
  // Editor-only fields (ignored by device, used by editor for UI state)
  selectedPageId: PAGE_ID,
  selectedElId: null,
  activeTool: 'select',
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async function getFreePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const srv = net.createServer()
    srv.listen(0, '127.0.0.1', () => {
      const addr = srv.address() as net.AddressInfo
      const port = addr.port
      srv.close(() => resolve(port))
    })
    srv.on('error', reject)
  })
}

async function waitForDevice(port: number, timeoutMs = 30_000): Promise<void> {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    try {
      const res = await fetch(`http://localhost:${port}/preview.png`)
      if (res.ok) return
    } catch {
      // not ready yet
    }
    await new Promise((r) => setTimeout(r, 300))
  }
  throw new Error(`Device server on port ${port} did not respond within ${timeoutMs}ms`)
}

// ── Suite state ───────────────────────────────────────────────────────────────

let deviceProc: ChildProcess | null = null
let devicePort: number = 0
let tmpDir: string = ''

// ── beforeAll: build prerequisites and start device ──────────────────────────

test.beforeAll(async () => {
  // 1. Ensure dist/ exists.
  if (!fs.existsSync(path.join(DIST, 'index.html'))) {
    console.log('[parity] Building editor (npm run build)…')
    execSync('npm run build', { cwd: ROOT, stdio: 'inherit' })
  }

  // 2. Ensure device binary exists.
  if (!fs.existsSync(DEVICE_BIN)) {
    console.log('[parity] Building device binary (cargo build)…')
    execSync('cargo build', { cwd: path.join(ROOT, 'device'), stdio: 'inherit' })
  }

  // 3. Find a free port and create a temp data dir.
  devicePort = await getFreePort()
  tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'corkboard-parity-'))

  // 4. Spawn the device binary.
  deviceProc = spawn(DEVICE_BIN, [], {
    env: {
      ...process.env,
      CORKBOARD_DATA: tmpDir,
      CORKBOARD_PORT: String(devicePort),
      CORKBOARD_DIST: DIST,
      CORKBOARD_FONTS: FONTS,
      RUST_LOG: 'warn',
    },
    stdio: ['ignore', 'pipe', 'pipe'],
  })

  deviceProc.on('error', (err) => {
    throw new Error(`Failed to spawn device binary: ${err.message}`)
  })

  // 5. Wait until the device is accepting requests.
  await waitForDevice(devicePort)

  // 6. PUT the fixed test document.
  const res = await fetch(`http://localhost:${devicePort}/api/document`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(TEST_DOC),
  })
  if (!res.ok) {
    throw new Error(`PUT /api/document failed: ${res.status} ${await res.text()}`)
  }
})

test.afterAll(async () => {
  deviceProc?.kill()
  deviceProc = null
  // Clean up the temp dir (ignore errors — it's a temp dir)
  try {
    fs.rmSync(tmpDir, { recursive: true, force: true })
  } catch {
    // best-effort
  }
})

// ── The parity test ───────────────────────────────────────────────────────────

test('editor render and device preview.png broadly match', async ({ page }: { page: Page }) => {
  const base = `http://localhost:${devicePort}`

  // Set viewport large enough that the 800×480 surface fits without scaling down too much.
  await page.setViewportSize({ width: 1280, height: 820 })
  await page.goto(base)

  // Wait for the surface to appear.
  await page.locator('[data-role="surface"]').waitFor({ state: 'visible', timeout: 15_000 })

  // Wait for fonts to be loaded by the browser.
  await page.evaluate(() => document.fonts.ready)

  // Wait for the editor to hydrate the document (text content must appear).
  await page.getByText('Hello Corkboard').first().waitFor({ state: 'visible', timeout: 15_000 })

  // Give the renderer a moment to finish painting (font swap, SVG paths, etc.).
  await page.waitForTimeout(500)

  // ── Capture editor render ─────────────────────────────────────────────────
  const box = await page.locator('[data-role="surface"]').boundingBox()
  if (!box) throw new Error('Could not locate [data-role="surface"] bounding box')

  const editorPng = await page.screenshot({
    clip: { x: box.x, y: box.y, width: box.width, height: box.height },
  })

  // ── Fetch device render ───────────────────────────────────────────────────
  const deviceRes = await fetch(`${base}/preview.png`)
  if (!deviceRes.ok) throw new Error(`GET /preview.png failed: ${deviceRes.status}`)
  const devicePngBuf = Buffer.from(await deviceRes.arrayBuffer())

  // ── Compare ───────────────────────────────────────────────────────────────
  const { contentIoU, editorDensity, deviceDensity } = compareParity(
    Buffer.from(editorPng),
    devicePngBuf,
  )

  console.log(
    `[parity] contentIoU=${contentIoU.toFixed(3)}  ` +
      `editorDensity=${editorDensity.toFixed(3)}  ` +
      `deviceDensity=${deviceDensity.toFixed(3)}`,
  )

  // Neither side should be a blank white image.
  expect(editorDensity, 'editor render appears blank').toBeGreaterThan(0.02)
  expect(deviceDensity, 'device preview.png appears blank').toBeGreaterThan(0.02)

  // Content regions must broadly overlap — catching a missing or misplaced element.
  // Threshold is 0.35 (generous to tolerate font/AA/scale differences).
  expect(contentIoU, 'content regions differ too much between editor and device').toBeGreaterThan(
    0.35,
  )
})
