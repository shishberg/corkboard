export const SAMPLE_TODAY = '2026-06-27' // fixed ISO date so editor + device agree

export interface SampleEvent { time: string; title: string }

export const SAMPLE_TODAY_EVENTS: SampleEvent[] = [
  { time: '09:00', title: 'Standup' },
  { time: '12:30', title: 'Lunch' },
  { time: '15:00', title: 'School pickup' },
]

export interface SampleDay { day: string; events: SampleEvent[] }

export const SAMPLE_WEEK: SampleDay[] = [
  { day: 'Mon', events: [{ time: '09:00', title: 'Standup' }] },
  { day: 'Tue', events: [{ time: '18:00', title: 'Soccer' }] },
  { day: 'Wed', events: [] },
  { day: 'Thu', events: [{ time: '12:30', title: 'Lunch' }] },
  { day: 'Fri', events: [{ time: '15:00', title: 'Pickup' }] },
  { day: 'Sat', events: [] },
  { day: 'Sun', events: [{ time: '10:00', title: 'Market' }] },
]

const DAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday']
const MONTHS = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December',
]

// Tomohiko Sakamoto's algorithm — returns 0=Sun, 1=Mon, …, 6=Sat
// Does NOT use new Date() so there are no local-timezone parsing surprises.
function weekday(year: number, month: number, day: number): number {
  const t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4]
  const y = month < 3 ? year - 1 : year
  return (y + Math.floor(y / 4) - Math.floor(y / 100) + Math.floor(y / 400) + t[month - 1] + day) % 7
}

/** Format an ISO date string (YYYY-MM-DD) as e.g. "Saturday 27 June". */
export function formatSampleDate(iso = SAMPLE_TODAY): string {
  const [yearStr, monthStr, dayStr] = iso.split('-')
  const year = parseInt(yearStr, 10)
  const month = parseInt(monthStr, 10)
  const day = parseInt(dayStr, 10)
  const dow = weekday(year, month, day)
  return `${DAYS[dow]} ${day} ${MONTHS[month - 1]}`
}

// ── Agenda view ──────────────────────────────────────────────────────────────
// Mirrors the device renderer (render.rs draw_agenda + sample.rs SAMPLE_AGENDA)
// so the editor preview matches the panel. Keep the two in sync.

/** "08:15" → "8:15am", "18:00" → "6:00pm". Empty string passes through. */
export function format12h(time: string): string {
  const [h, m] = time.split(':')
  const hour = parseInt(h, 10)
  if (time === '' || Number.isNaN(hour)) return time
  const suffix = hour < 12 ? 'am' : 'pm'
  const h12 = hour % 12 === 0 ? 12 : hour % 12
  return `${h12}:${m}${suffix}`
}

export interface AgendaEvent {
  time: string // 'HH:MM' or '' for all-day
  title: string
}
export interface AgendaDay {
  heading: string // 'Today', 'Tomorrow', or a full weekday name
  events: AgendaEvent[]
}

// Sample agenda events as day-offsets from SAMPLE_TODAY (a Saturday). Matches
// SAMPLE_AGENDA in the device's sample.rs.
const SAMPLE_AGENDA: { offset: number; time: string; title: string }[] = [
  { offset: 0, time: '', title: 'Last day of term' },
  { offset: 0, time: '08:15', title: 'Choir' },
  { offset: 0, time: '18:00', title: 'Ballet' },
  { offset: 1, time: '09:00', title: 'Markets' },
  { offset: 2, time: '15:00', title: 'School pickup' },
  { offset: 3, time: '18:00', title: 'Soccer' },
  { offset: 5, time: '12:30', title: 'Lunch' },
  { offset: 6, time: '19:30', title: 'Recorder' },
]

/** Day heading for agenda slot `i`: Today, Tomorrow, then the weekday name. */
function agendaHeading(slot: number, baseDow: number): string {
  if (slot === 0) return 'Today'
  if (slot === 1) return 'Tomorrow'
  return DAYS[(baseDow + slot) % 7]
}

/** The 7-day sample agenda (today + next 6 days), each with sorted events. */
export function sampleAgenda(): AgendaDay[] {
  const [y, mo, d] = SAMPLE_TODAY.split('-').map((s) => parseInt(s, 10))
  const baseDow = weekday(y, mo, d)
  return Array.from({ length: 7 }, (_, slot) => {
    const events = SAMPLE_AGENDA.filter((e) => e.offset === slot)
      .map((e) => ({ time: e.time, title: e.title }))
      // All-day ('') first, then ascending by time — same order as the device.
      .sort((a, b) => a.time.localeCompare(b.time))
    return { heading: agendaHeading(slot, baseDow), events }
  })
}
