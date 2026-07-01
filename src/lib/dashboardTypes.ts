// Mirrors device/src/status.rs's `DashboardStatus` (camelCase over the wire,
// same as every other endpoint).

export interface FeedInfo {
  id: string
  name: string
  lastAttemptMs: number | null
  ok: boolean | null
  todayEventCount: number | null
  error: string | null
}

export interface DocumentInfo {
  pageCount: number
  livePageId: string | null
  livePageName: string | null
  savedAtMs: number | null
}

export interface PreviewInfo {
  updatedAtMs: number
  renderCount: number
  connectedListeners: number
  lastPollAtMs: number | null
}

export interface DisplayInfo {
  kind: string
  os: string
  arch: string
}

export interface FontsInfo {
  defaultId: string
  ids: string[]
  boldIds: string[]
}

export interface EnvInfo {
  dataDir: string
  distDir: string
  fontsDir: string
  port: string
}

export interface LogEntry {
  timeMs: number
  level: string
  target: string
  message: string
}

export interface DashboardStatus {
  nowMs: number
  startedAtMs: number
  uptimeSecs: number
  hostname: string
  pollIntervalMinutes: number
  feeds: FeedInfo[]
  document: DocumentInfo
  preview: PreviewInfo
  display: DisplayInfo
  fonts: FontsInfo
  env: EnvInfo
  logs: LogEntry[]
}
