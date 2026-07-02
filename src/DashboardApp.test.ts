import { describe, it, expect, vi, afterEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import DashboardApp from './DashboardApp.vue'
import * as deviceApi from '@/lib/deviceApi'
import type { DashboardStatus } from '@/lib/dashboardTypes'

afterEach(() => {
  vi.restoreAllMocks()
})

function sampleStatus(): DashboardStatus {
  return {
    nowMs: 1000,
    startedAtMs: 0,
    uptimeSecs: 1,
    hostname: 'corkboard.local',
    pollIntervalMinutes: 60,
    feeds: [],
    document: { pageCount: 1, livePageId: 'p1', livePageName: 'Page 1', savedAtMs: null },
    preview: { updatedAtMs: 0, renderCount: 0, lastPollAtMs: null },
    display: { kind: 'web-preview', os: 'macos', arch: 'aarch64' },
    fonts: { defaultId: 'atkinson-hyperlegible', ids: ['atkinson-hyperlegible'], boldIds: ['atkinson-hyperlegible'] },
    env: { dataDir: './data', distDir: '../dist', fontsDir: '../public/fonts', port: '8080' },
    logs: [],
    system: {
      cpuTempC: null,
      loadAvg1: null,
      loadAvg5: null,
      loadAvg15: null,
      cpuFreqMhz: null,
      memTotalKb: null,
      memAvailableKb: null,
    },
  }
}

describe('DashboardApp', () => {
  it('shows loading, then renders the fetched status', async () => {
    vi.spyOn(deviceApi, 'fetchStatus').mockResolvedValue(sampleStatus())
    const w = mount(DashboardApp)
    expect(w.text()).toContain('loading')
    await flushPromises()
    expect(w.text()).toContain('corkboard.local')
    expect(w.text()).toContain('Page 1')
  })

  it('shows an unreachable message when the device cannot be reached', async () => {
    vi.spyOn(deviceApi, 'fetchStatus').mockResolvedValue(null)
    const w = mount(DashboardApp)
    await flushPromises()
    expect(w.text()).toContain('unreachable')
  })

  it('has an Editor link back to the editor', async () => {
    vi.spyOn(deviceApi, 'fetchStatus').mockResolvedValue(sampleStatus())
    const w = mount(DashboardApp)
    await flushPromises()
    expect(w.get('[data-role="editor"]').attributes('href')).toBe('/')
  })

  it('shows a feed’s week ahead and errors', async () => {
    const status = sampleStatus()
    status.feeds = [
      {
        id: 'family',
        name: 'Family',
        lastAttemptMs: 500,
        ok: true,
        todayEventCount: 1,
        error: null,
        week: [
          { label: 'Today', events: [{ time: '09:00', title: 'Standup' }] },
          { label: 'Tomorrow', events: [] },
          { label: 'Friday', events: [{ time: '', title: 'Bin day' }] },
        ],
      },
      {
        id: 'work',
        name: 'Work',
        lastAttemptMs: 500,
        ok: false,
        todayEventCount: null,
        error: 'HTTP 404',
        week: [],
      },
    ]
    vi.spyOn(deviceApi, 'fetchStatus').mockResolvedValue(status)
    const w = mount(DashboardApp)
    await flushPromises()
    expect(w.text()).toContain('Standup')
    expect(w.text()).toContain('Bin day')
    expect(w.text()).toContain('HTTP 404')
  })

  it('fetches once on load and only again when Refresh is clicked', async () => {
    const spy = vi.spyOn(deviceApi, 'fetchStatus').mockResolvedValue(sampleStatus())
    const w = mount(DashboardApp)
    await flushPromises()
    expect(spy).toHaveBeenCalledTimes(1)

    vi.useFakeTimers()
    await vi.advanceTimersByTimeAsync(20000)
    vi.useRealTimers()
    expect(spy).toHaveBeenCalledTimes(1)

    await w.get('[data-role="refresh"]').trigger('click')
    await flushPromises()
    expect(spy).toHaveBeenCalledTimes(2)
  })

  it('shows system stats when available', async () => {
    const status = sampleStatus()
    status.system = {
      cpuTempC: 42.8,
      loadAvg1: 0.52,
      loadAvg5: 0.58,
      loadAvg15: 0.59,
      cpuFreqMhz: 1008,
      memTotalKb: 1998960,
      memAvailableKb: 512340,
    }
    vi.spyOn(deviceApi, 'fetchStatus').mockResolvedValue(status)
    const w = mount(DashboardApp)
    await flushPromises()
    expect(w.text()).toContain('42.8°C')
    expect(w.text()).toContain('1008 MHz')
    // Load average is a mini bar graph now; the numbers surface in its tooltip.
    expect(w.html()).toContain('15m: 0.59 · 5m: 0.58 · 1m: 0.52')
  })

  it('orders load-avg bars 15m/5m/1m, tallest bar tallest value', async () => {
    const status = sampleStatus()
    status.system = {
      cpuTempC: null,
      loadAvg1: 2.0,
      loadAvg5: 1.0,
      loadAvg15: 0.5,
      cpuFreqMhz: null,
      memTotalKb: null,
      memAvailableKb: null,
    }
    vi.spyOn(deviceApi, 'fetchStatus').mockResolvedValue(status)
    const w = mount(DashboardApp)
    await flushPromises()
    const bars = w.findAll('[data-role="load-avg-bar"]')
    expect(bars).toHaveLength(3)
    const heights = bars.map((b) => parseFloat(/height:\s*([\d.]+)/.exec(b.attributes('style') ?? '')?.[1] ?? '0'))
    // 15m (0.5) shortest, 1m (2.0) tallest, in that left-to-right order.
    expect(heights[0]).toBeLessThan(heights[1])
    expect(heights[1]).toBeLessThan(heights[2])
  })

  it('shows a dash for load avg when no data is available', async () => {
    vi.spyOn(deviceApi, 'fetchStatus').mockResolvedValue(sampleStatus())
    const w = mount(DashboardApp)
    await flushPromises()
    expect(w.findAll('[data-role="load-avg-bar"]')).toHaveLength(0)
  })
})
