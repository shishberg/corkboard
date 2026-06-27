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
