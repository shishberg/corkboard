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
    preview: { updatedAtMs: 0, renderCount: 0, connectedListeners: 0, lastPollAtMs: null },
    display: { kind: 'web-preview', os: 'macos', arch: 'aarch64' },
    fonts: { defaultId: 'atkinson-hyperlegible', ids: ['atkinson-hyperlegible'], boldIds: ['atkinson-hyperlegible'] },
    env: { dataDir: './data', distDir: '../dist', fontsDir: '../public/fonts', port: '8080' },
    logs: [],
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
})
