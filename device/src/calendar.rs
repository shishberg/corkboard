/// Calendar feed: ICS fetch/parse, event resolution, and semantic signature.
///
/// Pure functions (`parse_ics`, `resolve`, `signature`) take explicit inputs and are fully tested.
/// `fetch_ics` / `fetch_and_resolve` are thin I/O wrappers — network-dependent, not unit-tested.

use std::collections::BTreeMap;

// ── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VEvent {
    pub summary: String,
    /// (year, month, day)
    pub date: (i32, u32, u32),
    /// "HH:MM" — None for all-day events.
    pub time: Option<String>,
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
    /// Events grouped by ISO week day: index 0 = Monday … 6 = Sunday.
    pub week: [Vec<ResolvedEvent>; 7],
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
    let year: i32 = val[..4].parse().ok()?;
    let month: u32 = val[4..6].parse().ok()?;
    let day: u32 = val[6..8].parse().ok()?;

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

    for line in unfolded.lines() {
        match line {
            "BEGIN:VEVENT" => {
                in_vevent = true;
                cur_summary = None;
                cur_dtstart = None;
            }
            "END:VEVENT" => {
                in_vevent = false;
                if let (Some(summary), Some(dtstart_val)) =
                    (cur_summary.take(), cur_dtstart.take())
                {
                    if let Some((date, time)) = parse_dtstart_value(&dtstart_val) {
                        events.push(VEvent { summary, date, time });
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
                }
            }
            _ => {}
        }
    }

    events
}

/// Resolve a list of parsed events against an explicit `today` date into a
/// `ResolvedFeed`.
///
/// today-list: events whose date == today, sorted by time (all-day "" first,
/// then ascending by "HH:MM").
///
/// week-grid: the 7-day ISO week (Mon…Sun) containing `today`, each slot
/// carrying the events for that calendar date sorted the same way.
///
/// Uses `chrono::NaiveDate` for ISO week arithmetic. The function is pure and
/// deterministic because `today` is an explicit input — no system clock.
pub fn resolve(events: &[VEvent], today: (i32, u32, u32)) -> ResolvedFeed {
    use chrono::{Datelike, Days, NaiveDate};

    let (y, m, d) = today;
    // Fallback to epoch if date is invalid (should not happen in practice).
    let today_nd =
        NaiveDate::from_ymd_opt(y, m, d).unwrap_or_else(|| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());

    // num_days_from_monday(): 0 = Monday … 6 = Sunday.
    let days_from_monday = today_nd.weekday().num_days_from_monday() as u64;
    let monday_nd = today_nd - Days::new(days_from_monday);

    // Build the 7-slot week array (0 = Monday … 6 = Sunday).
    let week: [Vec<ResolvedEvent>; 7] = std::array::from_fn(|i| {
        let slot_nd = monday_nd + Days::new(i as u64);
        let slot_date = (slot_nd.year(), slot_nd.month(), slot_nd.day());
        let mut day_events: Vec<ResolvedEvent> = events
            .iter()
            .filter(|e| e.date == slot_date)
            .map(|e| ResolvedEvent {
                time: e.time.clone().unwrap_or_default(),
                title: e.summary.clone(),
            })
            .collect();
        day_events.sort_by(|a, b| a.time.cmp(&b.time));
        day_events
    });

    // Today's events.
    let mut today_events: Vec<ResolvedEvent> = events
        .iter()
        .filter(|e| e.date == today)
        .map(|e| ResolvedEvent {
            time: e.time.clone().unwrap_or_default(),
            title: e.summary.clone(),
        })
        .collect();
    today_events.sort_by(|a, b| a.time.cmp(&b.time));

    ResolvedFeed {
        today: today_events,
        week,
    }
}

/// Deterministic semantic fingerprint of the resolved feeds map.
///
/// Same content → same string.  Any event add/remove/edit, or different feeds
/// present → different string.  Uses BTreeMap iteration order (sorted by key)
/// so the output is reproducible.
pub fn signature(feeds: &BTreeMap<String, ResolvedFeed>) -> String {
    let mut parts: Vec<String> = Vec::new();
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
            VEvent { summary: "Standup".to_string(), date: (2026, 6, 24), time: Some("09:00".to_string()) },
            // Saturday 2026-06-27 (today in our fixture)
            VEvent { summary: "Market".to_string(), date: (2026, 6, 27), time: Some("10:00".to_string()) },
            // Saturday 2026-06-27 — all-day event (should sort before "10:00")
            VEvent { summary: "Birthday".to_string(), date: (2026, 6, 27), time: None },
            // Tuesday 2026-06-23
            VEvent { summary: "Soccer".to_string(), date: (2026, 6, 23), time: Some("18:00".to_string()) },
            // Next week — must NOT appear in this week's grid
            VEvent { summary: "Next week".to_string(), date: (2026, 7, 1), time: Some("09:00".to_string()) },
        ]
    }

    // today = Saturday 2026-06-27; ISO week is Mon 2026-06-22 … Sun 2026-06-28.
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
        // ISO week: Mon=22, Tue=23, Wed=24, Thu=25, Fri=26, Sat=27, Sun=28
        // index:     0       1       2       3       4       5       6

        // Tuesday (index 1) has Soccer
        let tue = &feed.week[1];
        assert_eq!(tue.len(), 1);
        assert_eq!(tue[0].title, "Soccer");

        // Wednesday (index 2) has Standup
        let wed = &feed.week[2];
        assert_eq!(wed.len(), 1);
        assert_eq!(wed[0].title, "Standup");

        // Saturday (index 5) has Birthday and Market
        let sat = &feed.week[5];
        assert_eq!(sat.len(), 2);
        assert_eq!(sat[0].title, "Birthday"); // all-day first
        assert_eq!(sat[1].title, "Market");

        // Sunday (index 6) has nothing in this week
        assert!(feed.week[6].is_empty());
    }

    #[test]
    fn resolve_next_week_event_not_in_week_grid() {
        let feed = resolve(&fixture_events(), TODAY);
        let any_next = feed.week.iter().flatten().any(|e| e.title == "Next week");
        assert!(!any_next, "next week's event must not appear in the current week grid");
    }

    #[test]
    fn resolve_with_no_events_returns_empty_feed() {
        let feed = resolve(&[], TODAY);
        assert!(feed.today.is_empty());
        assert!(feed.week.iter().all(|d| d.is_empty()));
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
        assert_eq!(signature(&map1), signature(&map2));
    }

    #[test]
    fn signature_changed_title_produces_different_string() {
        let mut events = fixture_events();
        let map1 = make_feed_map("f1", &events);

        events[0].summary = "CHANGED TITLE".to_string();
        let map2 = make_feed_map("f1", &events);

        assert_ne!(signature(&map1), signature(&map2));
    }

    #[test]
    fn signature_added_event_produces_different_string() {
        let mut events = fixture_events();
        let map1 = make_feed_map("f1", &events);

        events.push(VEvent {
            summary: "New event".to_string(),
            date: (2026, 6, 27),
            time: Some("14:00".to_string()),
        });
        let map2 = make_feed_map("f1", &events);

        assert_ne!(signature(&map1), signature(&map2));
    }

    #[test]
    fn signature_empty_map_is_stable() {
        let empty: BTreeMap<String, ResolvedFeed> = BTreeMap::new();
        assert_eq!(signature(&empty), signature(&empty));
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
