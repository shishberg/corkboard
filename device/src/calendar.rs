/// Calendar feed: ICS fetch/parse, event resolution, and semantic signature.
///
/// Pure functions (`parse_ics`, `resolve`, `signature`) take explicit inputs and are fully tested.
/// `fetch_ics` / `fetch_and_resolve` are thin I/O wrappers — network-dependent, not unit-tested.

use std::collections::BTreeMap;

// ── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct VEvent {
    pub summary: String,
    /// (year, month, day) — the first occurrence (DTSTART).
    pub date: (i32, u32, u32),
    /// "HH:MM" — None for all-day events.
    pub time: Option<String>,
    /// Recurrence rule, if this is a repeating event (RRULE). None = single.
    pub rrule: Option<RRule>,
    /// Dates excluded from the recurrence (EXDATE), as (year, month, day).
    pub exdates: Vec<(i32, u32, u32)>,
}

/// Recurrence frequency (the RRULE `FREQ`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Freq {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// A parsed RRULE. Only the fields common in real calendars are kept; `WKST`
/// and other parameters are accepted and ignored.
#[derive(Debug, Clone)]
pub struct RRule {
    pub freq: Freq,
    /// Every `interval` periods (default 1).
    pub interval: u32,
    /// Stop after this many occurrences (COUNT).
    pub count: Option<u32>,
    /// Last date an occurrence may fall on (UNTIL), as (year, month, day).
    pub until: Option<(i32, u32, u32)>,
    /// Weekdays an occurrence falls on (BYDAY); empty = use DTSTART's weekday.
    pub byday: Vec<chrono::Weekday>,
}

#[derive(Debug, Clone)]
pub struct ResolvedEvent {
    /// "HH:MM" or "" for all-day events.
    pub time: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedFeed {
    /// Events on today's date, sorted by time (all-day "" first, then HH:MM ascending).
    pub today: Vec<ResolvedEvent>,
    /// Events for the next 7 days: index 0 = today, 1 = tomorrow, … 6 = today+6.
    pub week: [Vec<ResolvedEvent>; 7],
    /// Abbreviated weekday name for each `week` slot (e.g. "Sat","Sun","Mon"…),
    /// since the window now floats with today rather than being a fixed Mon…Sun.
    pub week_labels: [String; 7],
}

/// Calendar data passed to the renderer.
#[derive(Debug, Clone)]
pub struct CalendarData {
    /// The "today" date used when resolving/rendering.
    pub today: (i32, u32, u32),
    /// Resolved feeds keyed by feedId. Empty when no feeds are configured.
    pub feeds: BTreeMap<String, ResolvedFeed>,
}

impl CalendarData {
    /// Empty state — today = SAMPLE_TODAY, no resolved feeds.
    /// With no feeds the renderer falls back to sample data, keeping S4 parity.
    pub fn empty() -> Self {
        use crate::sample::SAMPLE_TODAY;
        let mut parts = SAMPLE_TODAY.split('-');
        let y: i32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(2026);
        let m: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(6);
        let d: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(27);
        CalendarData {
            today: (y, m, d),
            feeds: BTreeMap::new(),
        }
    }

    /// Look up a resolved feed by id.  Returns None when:
    ///   - feed_id is empty (no feed configured on the element), or
    ///   - the feed was not resolved (fetch failed or not configured).
    /// Either case causes the renderer to use sample fallback.
    pub fn for_feed(&self, feed_id: &str) -> Option<&ResolvedFeed> {
        if feed_id.is_empty() {
            return None;
        }
        self.feeds.get(feed_id)
    }
}

// ── Pure functions ────────────────────────────────────────────────────────────

/// Unfold RFC 5545 line folding: a line beginning with SPACE or TAB is a
/// continuation of the previous line; the leading whitespace character is the
/// fold indicator (removed) and the remaining characters are appended.
fn unfold_ics(text: &str) -> String {
    let mut result = String::new();
    for line in text.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // Continuation: strip the leading fold indicator and append.
            result.push_str(&line[1..]);
        } else {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
        }
    }
    result
}

/// Parse the value portion of a DTSTART property (after the colon).
/// Returns `((year, month, day), time_str)`.
///
/// Handles:
/// - `20260627T090000Z`        → date (2026,6,27), time Some("09:00")
/// - `20260627T090000`         → date (2026,6,27), time Some("09:00")
/// - `20260627`                → date (2026,6,27), time None  (all-day)
fn parse_dtstart_value(val: &str) -> Option<((i32, u32, u32), Option<String>)> {
    if val.len() < 8 {
        return None;
    }
    let year: i32 = val.get(..4)?.parse().ok()?;
    let month: u32 = val.get(4..6)?.parse().ok()?;
    let day: u32 = val.get(6..8)?.parse().ok()?;

    // A "T" at position 8 signals a date-time value.
    let time = if val.len() >= 13 && val.as_bytes().get(8) == Some(&b'T') {
        let hh = val.get(9..11)?;
        let mm = val.get(11..13)?;
        Some(format!("{}:{}", hh, mm))
    } else {
        None
    };

    Some(((year, month, day), time))
}

/// Parse an ICS text into a list of `VEvent`s.
///
/// - Unfolds RFC 5545 line folding first.
/// - Extracts only `BEGIN:VEVENT … END:VEVENT` blocks.
/// - Reads `SUMMARY:` and `DTSTART` (handles `;TZID=…`, `;VALUE=DATE`).
/// - Skips VEVENTs with no SUMMARY or no parseable DTSTART.
pub fn parse_ics(text: &str) -> Vec<VEvent> {
    let unfolded = unfold_ics(text);

    let mut events = Vec::new();
    let mut in_vevent = false;
    let mut cur_summary: Option<String> = None;
    let mut cur_dtstart: Option<String> = None;
    let mut cur_rrule: Option<RRule> = None;
    let mut cur_exdates: Vec<(i32, u32, u32)> = Vec::new();

    for line in unfolded.lines() {
        match line {
            "BEGIN:VEVENT" => {
                in_vevent = true;
                cur_summary = None;
                cur_dtstart = None;
                cur_rrule = None;
                cur_exdates = Vec::new();
            }
            "END:VEVENT" => {
                in_vevent = false;
                if let (Some(summary), Some(dtstart_val)) =
                    (cur_summary.take(), cur_dtstart.take())
                {
                    if let Some((date, time)) = parse_dtstart_value(&dtstart_val) {
                        events.push(VEvent {
                            summary,
                            date,
                            time,
                            rrule: cur_rrule.take(),
                            exdates: std::mem::take(&mut cur_exdates),
                        });
                    }
                }
            }
            _ if in_vevent => {
                if let Some(val) = line.strip_prefix("SUMMARY:") {
                    cur_summary = Some(val.to_string());
                } else if line.starts_with("DTSTART") {
                    // Property may have parameters: DTSTART;TZID=…:VALUE or DTSTART:VALUE
                    // Use the first colon as the separator.
                    if let Some(colon) = line.find(':') {
                        cur_dtstart = Some(line[colon + 1..].to_string());
                    }
                } else if line.starts_with("RRULE") {
                    if let Some(colon) = line.find(':') {
                        cur_rrule = parse_rrule(&line[colon + 1..]);
                    }
                } else if line.starts_with("EXDATE") {
                    if let Some(colon) = line.find(':') {
                        // The value may be a comma-separated list of date-times.
                        for v in line[colon + 1..].split(',') {
                            if let Some((date, _)) = parse_dtstart_value(v) {
                                cur_exdates.push(date);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    events
}

/// Parse the value of an RRULE property (the part after `RRULE:`), e.g.
/// `FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE;UNTIL=20261231T000000Z`.
/// Returns None if there is no recognisable `FREQ`.
fn parse_rrule(val: &str) -> Option<RRule> {
    let mut freq: Option<Freq> = None;
    let mut interval: u32 = 1;
    let mut count: Option<u32> = None;
    let mut until: Option<(i32, u32, u32)> = None;
    let mut byday: Vec<chrono::Weekday> = Vec::new();

    for part in val.split(';') {
        let (key, value) = match part.split_once('=') {
            Some(kv) => kv,
            None => continue,
        };
        match key.to_ascii_uppercase().as_str() {
            "FREQ" => {
                freq = match value.to_ascii_uppercase().as_str() {
                    "DAILY" => Some(Freq::Daily),
                    "WEEKLY" => Some(Freq::Weekly),
                    "MONTHLY" => Some(Freq::Monthly),
                    "YEARLY" => Some(Freq::Yearly),
                    _ => None,
                };
            }
            "INTERVAL" => {
                if let Ok(n) = value.parse::<u32>() {
                    interval = n.max(1);
                }
            }
            "COUNT" => count = value.parse::<u32>().ok(),
            "UNTIL" => until = parse_dtstart_value(value).map(|(d, _)| d),
            "BYDAY" => {
                byday = value
                    .split(',')
                    .filter_map(parse_weekday)
                    .collect();
            }
            _ => {} // WKST and anything else: ignored
        }
    }

    freq.map(|freq| RRule { freq, interval, count, until, byday })
}

/// Parse a two-letter ICS weekday code (`MO`,`TU`,…). A `BYDAY` token may carry
/// an ordinal prefix (e.g. `2MO`); the trailing two letters are what matter.
fn parse_weekday(token: &str) -> Option<chrono::Weekday> {
    use chrono::Weekday::*;
    let code = token.trim();
    let code = &code[code.len().saturating_sub(2)..];
    match code.to_ascii_uppercase().as_str() {
        "MO" => Some(Mon),
        "TU" => Some(Tue),
        "WE" => Some(Wed),
        "TH" => Some(Thu),
        "FR" => Some(Fri),
        "SA" => Some(Sat),
        "SU" => Some(Sun),
        _ => None,
    }
}

/// Resolve a list of parsed events against an explicit `today` date into a
/// `ResolvedFeed`.
///
/// today-list: events whose date == today, sorted by time (all-day "" first,
/// then ascending by "HH:MM").
///
/// week-grid: the next 7 days starting today (slot 0 = today … slot 6 =
/// today+6), each slot carrying that date's events sorted the same way, plus a
/// `week_labels` entry naming each slot's weekday.
///
/// Uses `chrono::NaiveDate` for date arithmetic. The function is pure and
/// deterministic because `today` is an explicit input — no system clock.
pub fn resolve(events: &[VEvent], today: (i32, u32, u32)) -> ResolvedFeed {
    use chrono::{Days, NaiveDate};

    let (y, m, d) = today;
    // Fallback to epoch if date is invalid (should not happen in practice).
    let today_nd =
        NaiveDate::from_ymd_opt(y, m, d).unwrap_or_else(|| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());

    // Collect the events occurring on a given calendar date (expanding
    // recurrence), sorted all-day first then by time.
    let events_on = |slot_nd: NaiveDate| -> Vec<ResolvedEvent> {
        let mut day_events: Vec<ResolvedEvent> = events
            .iter()
            .filter(|e| occurs_on(e, slot_nd))
            .map(|e| ResolvedEvent {
                time: e.time.clone().unwrap_or_default(),
                title: e.summary.clone(),
            })
            .collect();
        day_events.sort_by(|a, b| a.time.cmp(&b.time));
        day_events
    };

    // The week window is the next 7 days starting today: slot i = today + i.
    let week: [Vec<ResolvedEvent>; 7] =
        std::array::from_fn(|i| events_on(today_nd + Days::new(i as u64)));
    let week_labels: [String; 7] =
        std::array::from_fn(|i| weekday_abbrev(today_nd + Days::new(i as u64)));

    ResolvedFeed {
        today: events_on(today_nd),
        week,
        week_labels,
    }
}

/// Three-letter weekday abbreviation ("Mon"…"Sun") for a date.
fn weekday_abbrev(date: chrono::NaiveDate) -> String {
    use chrono::Datelike;
    const NAMES: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    NAMES[date.weekday().num_days_from_monday() as usize].to_string()
}

/// Anchor a date to the Monday of its week.
fn monday_of(date: chrono::NaiveDate) -> chrono::NaiveDate {
    use chrono::{Datelike, Days};
    date - Days::new(date.weekday().num_days_from_monday() as u64)
}

/// Does `event` occur on `date`? Non-recurring events occur only on their
/// DTSTART; recurring events are expanded per their RRULE, honouring INTERVAL,
/// COUNT, UNTIL, BYDAY (weekly), and EXDATE.
fn occurs_on(event: &VEvent, date: chrono::NaiveDate) -> bool {
    use chrono::{Datelike, NaiveDate};

    let (sy, sm, sd) = event.date;
    let start = match NaiveDate::from_ymd_opt(sy, sm, sd) {
        Some(s) => s,
        None => return false,
    };

    let rule = match &event.rrule {
        None => return date == start,
        Some(r) => r,
    };

    if date < start {
        return false;
    }
    // Explicitly excluded instance.
    if event
        .exdates
        .iter()
        .filter_map(|&(y, m, d)| NaiveDate::from_ymd_opt(y, m, d))
        .any(|x| x == date)
    {
        return false;
    }
    // Past the recurrence end.
    if let Some((uy, um, ud)) = rule.until {
        if let Some(until) = NaiveDate::from_ymd_opt(uy, um, ud) {
            if date > until {
                return false;
            }
        }
    }

    let interval = rule.interval.max(1) as i64;

    // `index` is the 0-based ordinal of this occurrence within the series, used
    // only for the COUNT cap. None means `date` is not on the cadence at all.
    let index: Option<i64> = match rule.freq {
        Freq::Daily => {
            let diff = (date - start).num_days();
            (diff % interval == 0).then_some(diff / interval)
        }
        Freq::Weekly => {
            if rule.byday.is_empty() {
                if date.weekday() != start.weekday() {
                    return false;
                }
                let weeks = (date - start).num_days() / 7;
                (weeks % interval == 0).then_some(weeks / interval)
            } else {
                if !rule.byday.contains(&date.weekday()) {
                    return false;
                }
                let weeks = (monday_of(date) - monday_of(start)).num_days() / 7;
                // Per-week index (good enough for the COUNT cap in practice).
                (weeks % interval == 0).then_some(weeks / interval)
            }
        }
        Freq::Monthly => {
            if date.day() != start.day() {
                return false;
            }
            let months =
                (date.year() - start.year()) as i64 * 12 + date.month() as i64 - start.month() as i64;
            (months >= 0 && months % interval == 0).then_some(months / interval)
        }
        Freq::Yearly => {
            if date.month() != start.month() || date.day() != start.day() {
                return false;
            }
            let years = (date.year() - start.year()) as i64;
            (years >= 0 && years % interval == 0).then_some(years / interval)
        }
    };

    match index {
        None => false,
        Some(idx) => match rule.count {
            Some(c) => (idx as u64) < c as u64,
            None => true,
        },
    }
}

/// Deterministic semantic fingerprint of the resolved calendar data.
///
/// Incorporates both `today` and the resolved feeds map so a date rollover
/// (same event bytes, different day) produces a different signature.
///
/// Same content → same string.  Any event add/remove/edit, different feeds
/// present, or different `today` → different string.  Uses BTreeMap iteration
/// order (sorted by key) so the output is reproducible.
pub fn signature(today: (i32, u32, u32), feeds: &BTreeMap<String, ResolvedFeed>) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("today:{:?}|", today));
    for (feed_id, feed) in feeds {
        parts.push(format!("feed:{}", feed_id));
        parts.push("today:".to_string());
        for ev in &feed.today {
            parts.push(format!("  {}|{}", ev.time, ev.title));
        }
        for (i, day_events) in feed.week.iter().enumerate() {
            parts.push(format!("week{}:", i));
            for ev in day_events {
                parts.push(format!("  {}|{}", ev.time, ev.title));
            }
        }
    }
    parts.join("\n")
}

// ── I/O wrappers (thin; not unit-tested) ─────────────────────────────────────

/// Fetch an ICS document from a URL and return the body text.
/// Returns an error on any HTTP non-2xx status or network failure.
/// The URL is intentionally NOT included in the error message to prevent
/// accidental logging of secret feed URLs.
pub async fn fetch_ics(url: &str) -> anyhow::Result<String> {
    let response = reqwest::get(url)
        .await
        .map_err(|_| anyhow::anyhow!("HTTP request failed"))?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        return Err(anyhow::anyhow!("HTTP {}", status));
    }

    let text = response
        .text()
        .await
        .map_err(|_| anyhow::anyhow!("failed to read response body"))?;

    Ok(text)
}

/// Fetch, parse, and resolve a calendar feed in one step.
pub async fn fetch_and_resolve(
    secret_url: &str,
    today: (i32, u32, u32),
) -> anyhow::Result<ResolvedFeed> {
    let text = fetch_ics(secret_url).await?;
    let events = parse_ics(&text);
    Ok(resolve(&events, today))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // A representative ICS document used across several parse tests.
    // Contains:
    //   1. An all-day event on 2026-06-27 using VALUE=DATE form.
    //   2. A date-time event on 2026-06-28 at 09:30 UTC.
    //   3. A date-time event with TZID parameter on 2026-06-29T15:00.
    //   4. RFC 5545 line folding on the SUMMARY of event 1 ("Holiday break").
    //   5. Junk / non-VEVENT lines (ignored).
    //   6. A VEVENT with no SUMMARY (skipped).
    //   7. A VEVENT with no DTSTART (skipped).
    //
    // NOTE: use concat! rather than a backslash-continued string literal because
    // Rust strips leading whitespace from the continuation line, which would
    // swallow the fold indicator space we need to test.
    const TEST_ICS: &str = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Test//Test//EN\r\n",
        "X-WR-CALNAME:My Test Cal\r\n",
        "BEGIN:VEVENT\r\n",
        "DTSTART;VALUE=DATE:20260627\r\n",
        // SUMMARY is folded: "Holiday " ends the first segment; the
        // continuation line starts with a space (fold indicator, stripped on
        // unfold), leaving "break" appended → "Holiday break".
        "SUMMARY:Holiday \r\n",
        " break\r\n",
        "END:VEVENT\r\n",
        "BEGIN:VEVENT\r\n",
        "DTSTART:20260628T093000Z\r\n",
        "SUMMARY:Team meeting\r\n",
        "END:VEVENT\r\n",
        "BEGIN:VEVENT\r\n",
        "DTSTART;TZID=America/New_York:20260629T150000\r\n",
        "SUMMARY:Afternoon call\r\n",
        "END:VEVENT\r\n",
        "JUNK:this line is outside a VEVENT and should be ignored\r\n",
        // VEVENT with DTSTART but no SUMMARY — must be skipped.
        "BEGIN:VEVENT\r\n",
        "DTSTART:20260630T120000Z\r\n",
        "END:VEVENT\r\n",
        // VEVENT with SUMMARY but no DTSTART — must be skipped.
        "BEGIN:VEVENT\r\n",
        "SUMMARY:No DTSTART must be skipped\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR"
    );

    // ── parse_ics ─────────────────────────────────────────────────────────────

    #[test]
    fn parse_ics_returns_correct_event_count() {
        let events = parse_ics(TEST_ICS);
        // 3 valid events; the VEVENT with no SUMMARY and the one with no DTSTART are skipped.
        assert_eq!(events.len(), 3, "expected 3 valid events, got {:?}", events.len());
    }

    #[test]
    fn parse_ics_all_day_event_has_no_time() {
        let events = parse_ics(TEST_ICS);
        let ev = events.iter().find(|e| e.date == (2026, 6, 27))
            .expect("should find 2026-06-27 event");
        assert_eq!(ev.time, None, "all-day event should have no time");
    }

    #[test]
    fn parse_ics_line_folding_joins_summary() {
        let events = parse_ics(TEST_ICS);
        let ev = events.iter().find(|e| e.date == (2026, 6, 27))
            .expect("should find 2026-06-27 event");
        // "Holiday \r\n break" unfolds to "Holiday break" (trailing space is content,
        // leading space on continuation is the fold indicator and is stripped).
        assert_eq!(ev.summary, "Holiday break");
    }

    #[test]
    fn parse_ics_datetime_event_has_time() {
        let events = parse_ics(TEST_ICS);
        let ev = events.iter().find(|e| e.date == (2026, 6, 28))
            .expect("should find 2026-06-28 event");
        assert_eq!(ev.time, Some("09:30".to_string()));
        assert_eq!(ev.summary, "Team meeting");
    }

    #[test]
    fn parse_ics_tzid_form_extracts_date_and_time() {
        let events = parse_ics(TEST_ICS);
        let ev = events.iter().find(|e| e.date == (2026, 6, 29))
            .expect("should find 2026-06-29 event");
        assert_eq!(ev.time, Some("15:00".to_string()));
        assert_eq!(ev.summary, "Afternoon call");
    }

    #[test]
    fn parse_ics_skips_vevent_with_no_summary() {
        // The event on 2026-06-30 has a DTSTART but no SUMMARY — must be skipped.
        let events = parse_ics(TEST_ICS);
        assert!(!events.iter().any(|e| e.date == (2026, 6, 30)));
    }

    #[test]
    fn parse_ics_skips_vevent_with_no_dtstart() {
        let events = parse_ics(TEST_ICS);
        assert!(!events.iter().any(|e| e.summary == "No DTSTART must be skipped"));
    }

    #[test]
    fn parse_ics_empty_string_returns_empty() {
        assert!(parse_ics("").is_empty());
    }

    // ── resolve ───────────────────────────────────────────────────────────────

    fn fixture_events() -> Vec<VEvent> {
        vec![
            // Wednesday 2026-06-24 (in the Mon 22 … Sun 28 ISO week)
            VEvent { summary: "Standup".to_string(), date: (2026, 6, 24), time: Some("09:00".to_string()), ..Default::default() },
            // Saturday 2026-06-27 (today in our fixture)
            VEvent { summary: "Market".to_string(), date: (2026, 6, 27), time: Some("10:00".to_string()), ..Default::default() },
            // Saturday 2026-06-27 — all-day event (should sort before "10:00")
            VEvent { summary: "Birthday".to_string(), date: (2026, 6, 27), time: None, ..Default::default() },
            // Tuesday 2026-06-23
            VEvent { summary: "Soccer".to_string(), date: (2026, 6, 23), time: Some("18:00".to_string()), ..Default::default() },
            // Next week — must NOT appear in this week's grid
            VEvent { summary: "Next week".to_string(), date: (2026, 7, 1), time: Some("09:00".to_string()), ..Default::default() },
        ]
    }

    // today = Saturday 2026-06-27. The week window is the next 7 days:
    // slot 0 Sat 06-27, 1 Sun 06-28, 2 Mon 06-29, 3 Tue 06-30, 4 Wed 07-01,
    // 5 Thu 07-02, 6 Fri 07-03.
    const TODAY: (i32, u32, u32) = (2026, 6, 27);

    #[test]
    fn resolve_today_list_contains_same_day_events() {
        let feed = resolve(&fixture_events(), TODAY);
        let titles: Vec<&str> = feed.today.iter().map(|e| e.title.as_str()).collect();
        assert!(titles.contains(&"Market"), "Market should be in today list");
        assert!(titles.contains(&"Birthday"), "Birthday should be in today list");
        assert!(!titles.contains(&"Standup"), "Standup is on a different day");
        assert!(!titles.contains(&"Next week"), "Next week event must not be in today");
    }

    #[test]
    fn resolve_today_list_sorted_all_day_first() {
        let feed = resolve(&fixture_events(), TODAY);
        // "" < "10:00" so Birthday (all-day) should come before Market (10:00)
        assert_eq!(feed.today[0].title, "Birthday");
        assert_eq!(feed.today[1].title, "Market");
    }

    #[test]
    fn resolve_week_groups_events_into_correct_slots() {
        let feed = resolve(&fixture_events(), TODAY);

        // Today (slot 0 = Sat 06-27) has Birthday (all-day, first) and Market.
        let today = &feed.week[0];
        assert_eq!(today.len(), 2);
        assert_eq!(today[0].title, "Birthday");
        assert_eq!(today[1].title, "Market");

        // "Next week" (07-01) is now within the next-7-days window: slot 4 (Wed).
        let wed = &feed.week[4];
        assert_eq!(wed.len(), 1);
        assert_eq!(wed[0].title, "Next week");

        // Standup (06-24) and Soccer (06-23) are in the past — outside the window.
        let all: Vec<&str> = feed.week.iter().flatten().map(|e| e.title.as_str()).collect();
        assert!(!all.contains(&"Standup"));
        assert!(!all.contains(&"Soccer"));
    }

    #[test]
    fn resolve_week_labels_name_each_days_weekday() {
        let feed = resolve(&fixture_events(), TODAY);
        assert_eq!(feed.week_labels, ["Sat", "Sun", "Mon", "Tue", "Wed", "Thu", "Fri"]);
    }

    #[test]
    fn resolve_past_event_not_in_week_grid() {
        // An event before today must not appear in the forward-looking window.
        let feed = resolve(&fixture_events(), TODAY);
        let any_soccer = feed.week.iter().flatten().any(|e| e.title == "Soccer");
        assert!(!any_soccer, "a past event must not appear in the next-7-days grid");
    }

    #[test]
    fn resolve_with_no_events_returns_empty_feed() {
        let feed = resolve(&[], TODAY);
        assert!(feed.today.is_empty());
        assert!(feed.week.iter().all(|d| d.is_empty()));
    }

    // ── recurrence (RRULE / EXDATE) ─────────────────────────────────────────────
    // TODAY = Sat 2026-06-27; the window is the next 7 days:
    // slot 0 Sat 06-27, 1 Sun 06-28, 2 Mon 06-29, 3 Tue 06-30,
    // 4 Wed 07-01, 5 Thu 07-02, 6 Fri 07-03.

    fn rrule(freq: Freq) -> RRule {
        RRule { freq, interval: 1, count: None, until: None, byday: vec![] }
    }

    fn recurring(summary: &str, date: (i32, u32, u32), rule: RRule, exdates: Vec<(i32, u32, u32)>) -> VEvent {
        VEvent { summary: summary.to_string(), date, time: Some("08:30".to_string()), rrule: Some(rule), exdates }
    }

    fn week_titles(feed: &ResolvedFeed) -> Vec<&str> {
        feed.week.iter().flatten().map(|e| e.title.as_str()).collect()
    }

    #[test]
    fn parse_ics_extracts_rrule_and_exdate() {
        let ics = concat!(
            "BEGIN:VCALENDAR\r\n",
            "BEGIN:VEVENT\r\n",
            "DTSTART;TZID=Australia/Sydney:20260219T083000\r\n",
            "RRULE:FREQ=WEEKLY;WKST=TU\r\n",
            "EXDATE;TZID=Australia/Sydney:20260305T083000\r\n",
            "SUMMARY:Recorder\r\n",
            "END:VEVENT\r\n",
            "END:VCALENDAR\r\n",
        );
        let events = parse_ics(ics);
        assert_eq!(events.len(), 1);
        let rule = events[0].rrule.as_ref().expect("should have an rrule");
        assert_eq!(rule.freq, Freq::Weekly);
        assert_eq!(events[0].exdates, vec![(2026, 3, 5)]);
    }

    #[test]
    fn resolve_weekly_event_recurs_into_the_current_week() {
        // Starts Thursday 2026-06-04, weekly → the Thursday in the window is
        // 2026-07-02 (slot 5).
        let ev = recurring("Recorder", (2026, 6, 4), rrule(Freq::Weekly), vec![]);
        let feed = resolve(&[ev], TODAY);
        assert_eq!(feed.week[5].len(), 1);
        assert_eq!(feed.week[5][0].title, "Recorder");
    }

    #[test]
    fn resolve_weekly_exdate_excludes_that_occurrence() {
        // Exclude the in-window Thursday (07-02).
        let ev = recurring("Recorder", (2026, 6, 4), rrule(Freq::Weekly), vec![(2026, 7, 2)]);
        let feed = resolve(&[ev], TODAY);
        assert!(feed.week[5].is_empty(), "the excluded Thursday must be empty");
    }

    #[test]
    fn resolve_weekly_interval_two_skips_the_off_week() {
        // Start Thu 2026-06-11; the in-window Thursday 07-02 is 3 weeks later —
        // odd, so INTERVAL=2 skips it.
        let mut rule = rrule(Freq::Weekly);
        rule.interval = 2;
        let ev = recurring("Fortnightly", (2026, 6, 11), rule, vec![]);
        let feed = resolve(&[ev], TODAY);
        assert!(feed.week[5].is_empty());
    }

    #[test]
    fn resolve_weekly_until_stops_recurrence() {
        let mut rule = rrule(Freq::Weekly);
        rule.until = Some((2026, 6, 20));
        let ev = recurring("Ended", (2026, 6, 4), rule, vec![]);
        let feed = resolve(&[ev], TODAY);
        assert!(week_titles(&feed).is_empty());
    }

    #[test]
    fn resolve_weekly_count_limits_recurrence() {
        // Count 2 → only 06-04 and 06-11; Thu 06-25 (index 3) is past the cap.
        let mut rule = rrule(Freq::Weekly);
        rule.count = Some(2);
        let ev = recurring("Twice", (2026, 6, 4), rule, vec![]);
        let feed = resolve(&[ev], TODAY);
        assert!(week_titles(&feed).is_empty());
    }

    #[test]
    fn resolve_weekly_byday_lands_on_each_listed_weekday() {
        use chrono::Weekday::*;
        // Starts Mon 2026-06-01, BYDAY=MO,WE. In the window the Monday is 06-29
        // (slot 2) and the Wednesday is 07-01 (slot 4).
        let mut rule = rrule(Freq::Weekly);
        rule.byday = vec![Mon, Wed];
        let ev = recurring("Class", (2026, 6, 1), rule, vec![]);
        let feed = resolve(&[ev], TODAY);
        assert_eq!(feed.week[2].len(), 1, "Monday 06-29");
        assert!(feed.week[3].is_empty(), "Tuesday 06-30");
        assert_eq!(feed.week[4].len(), 1, "Wednesday 07-01");
    }

    #[test]
    fn resolve_daily_event_fills_every_day() {
        let ev = recurring("Standup", (2026, 6, 1), rrule(Freq::Daily), vec![]);
        let feed = resolve(&[ev], TODAY);
        assert!(feed.week.iter().all(|d| d.len() == 1), "every day should have it");
    }

    #[test]
    fn resolve_all_day_single_event_lands_on_its_weekday() {
        // "End of term 2" — all-day, single (non-recurring) on Fri 2026-07-03.
        let mk = || VEvent {
            summary: "End of term 2".to_string(),
            date: (2026, 7, 3),
            time: None,
            ..Default::default()
        };
        // today = Wed 2026-07-01 → window 07-01… → Jul 3 is slot 2.
        let feed = resolve(&[mk()], (2026, 7, 1));
        assert_eq!(feed.week[2].len(), 1, "the event's day should hold it");
        assert_eq!(feed.week[2][0].title, "End of term 2");
        assert!(feed.week[2][0].time.is_empty(), "all-day event has empty time");
        // The forward-looking window also surfaces it from Sun 2026-06-28
        // (window 06-28…07-04): Jul 3 is slot 5. This is the whole point — the
        // old ISO-week view (Jun 22…28) would have hidden it.
        let from_sunday = resolve(&[mk()], (2026, 6, 28));
        assert_eq!(from_sunday.week[5].len(), 1);
        assert_eq!(from_sunday.week[5][0].title, "End of term 2");
    }

    #[test]
    fn resolve_monthly_event_matches_same_day_of_month() {
        // Start 2026-05-30 monthly → recurs on the 30th → Tue 2026-06-30 (slot 3).
        let ev = recurring("Rent", (2026, 5, 30), rrule(Freq::Monthly), vec![]);
        let feed = resolve(&[ev], TODAY);
        assert_eq!(feed.week[3].len(), 1);
        assert_eq!(feed.week[3][0].title, "Rent");
    }

    // ── signature ─────────────────────────────────────────────────────────────

    fn make_feed_map(feed_id: &str, events: &[VEvent]) -> BTreeMap<String, ResolvedFeed> {
        let mut map = BTreeMap::new();
        map.insert(feed_id.to_string(), resolve(events, TODAY));
        map
    }

    #[test]
    fn signature_same_content_produces_equal_strings() {
        let map1 = make_feed_map("f1", &fixture_events());
        let map2 = make_feed_map("f1", &fixture_events());
        assert_eq!(signature(TODAY, &map1), signature(TODAY, &map2));
    }

    #[test]
    fn signature_changed_title_produces_different_string() {
        let mut events = fixture_events();
        let map1 = make_feed_map("f1", &events);

        // events[1] is "Market" (06-27 = today), which is inside the window.
        events[1].summary = "CHANGED TITLE".to_string();
        let map2 = make_feed_map("f1", &events);

        assert_ne!(signature(TODAY, &map1), signature(TODAY, &map2));
    }

    #[test]
    fn signature_added_event_produces_different_string() {
        let mut events = fixture_events();
        let map1 = make_feed_map("f1", &events);

        events.push(VEvent {
            summary: "New event".to_string(),
            date: (2026, 6, 27),
            time: Some("14:00".to_string()),
            ..Default::default()
        });
        let map2 = make_feed_map("f1", &events);

        assert_ne!(signature(TODAY, &map1), signature(TODAY, &map2));
    }

    #[test]
    fn signature_empty_map_is_stable() {
        let empty: BTreeMap<String, ResolvedFeed> = BTreeMap::new();
        assert_eq!(signature(TODAY, &empty), signature(TODAY, &empty));
    }

    // ── I1: UTF-8-safe DTSTART parsing ───────────────────────────────────────

    #[test]
    fn parse_dtstart_value_non_ascii_returns_none_not_panic() {
        // "€" is 3 UTF-8 bytes (e2 82 ac).  "2026062€xx" byte-length is > 8
        // but slicing val[4..6] would straddle the euro-sign — must not panic.
        assert!(parse_dtstart_value("2026062\u{20AC}xx").is_none());
        // Also verify on a plain multi-byte char in the year field
        assert!(parse_dtstart_value("\u{20AC}2060628").is_none());
        // Short values still rejected
        assert!(parse_dtstart_value("202606").is_none());
        assert!(parse_dtstart_value("").is_none());
    }

    #[test]
    fn parse_ics_with_non_ascii_dtstart_skips_event_without_panic() {
        // Build an ICS with a DTSTART containing a multi-byte UTF-8 char.
        let ics = concat!(
            "BEGIN:VCALENDAR\r\n",
            "BEGIN:VEVENT\r\n",
            "DTSTART:2026062\u{20AC}xx\r\n",
            "SUMMARY:Bad date event\r\n",
            "END:VEVENT\r\n",
            "END:VCALENDAR\r\n"
        );
        // Must not panic; the event is skipped (unparseable DTSTART).
        let events = parse_ics(ics);
        assert!(events.is_empty());
    }

    // ── I2: today folded into signature ──────────────────────────────────────

    #[test]
    fn signature_different_today_produces_different_string() {
        let map = make_feed_map("f1", &fixture_events());
        let today1: (i32, u32, u32) = (2026, 6, 27);
        let today2: (i32, u32, u32) = (2026, 6, 28);
        assert_ne!(
            signature(today1, &map),
            signature(today2, &map),
            "different today must yield different signature even with identical feeds"
        );
    }

    #[test]
    fn signature_same_today_and_feeds_produces_equal_string() {
        let map1 = make_feed_map("f1", &fixture_events());
        let map2 = make_feed_map("f1", &fixture_events());
        assert_eq!(signature(TODAY, &map1), signature(TODAY, &map2));
    }

    // ── CalendarData ──────────────────────────────────────────────────────────

    #[test]
    fn calendar_data_empty_has_no_feeds() {
        let cal = CalendarData::empty();
        assert!(cal.feeds.is_empty());
    }

    #[test]
    fn calendar_data_empty_today_matches_sample_today() {
        use crate::sample::SAMPLE_TODAY;
        let cal = CalendarData::empty();
        let (y, m, d) = cal.today;
        let formatted = format!("{:04}-{:02}-{:02}", y, m, d);
        assert_eq!(formatted, SAMPLE_TODAY);
    }

    #[test]
    fn for_feed_returns_none_for_empty_id() {
        let cal = CalendarData::empty();
        assert!(cal.for_feed("").is_none());
    }

    #[test]
    fn for_feed_returns_none_when_not_resolved() {
        let cal = CalendarData::empty();
        assert!(cal.for_feed("some-feed-id").is_none());
    }

    #[test]
    fn for_feed_returns_some_when_present() {
        let mut cal = CalendarData::empty();
        let feed = resolve(&[], TODAY);
        cal.feeds.insert("my-feed".to_string(), feed);
        assert!(cal.for_feed("my-feed").is_some());
    }
}
