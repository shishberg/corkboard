import { describe, it, expect } from 'vitest'
import { formatSampleDate, SAMPLE_TODAY, SAMPLE_TODAY_EVENTS, SAMPLE_WEEK } from './sampleCalendar'

describe('formatSampleDate', () => {
  it('formats SAMPLE_TODAY as Saturday 27 June', () => {
    expect(formatSampleDate('2026-06-27')).toBe('Saturday 27 June')
  })

  it('uses SAMPLE_TODAY as default', () => {
    expect(formatSampleDate()).toBe(formatSampleDate(SAMPLE_TODAY))
  })

  it('formats another known date correctly — 2026-01-01 is Thursday', () => {
    expect(formatSampleDate('2026-01-01')).toBe('Thursday 1 January')
  })
})

describe('sample data shapes', () => {
  it('SAMPLE_TODAY_EVENTS has 3 entries', () => {
    expect(SAMPLE_TODAY_EVENTS).toHaveLength(3)
  })

  it('SAMPLE_WEEK has 7 entries', () => {
    expect(SAMPLE_WEEK).toHaveLength(7)
  })
})
