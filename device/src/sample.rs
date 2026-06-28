/// Sample calendar data for the device preview.
/// Mirrors sampleCalendar.ts in the Vue app.

const DAYS: [&str; 7] = [
    "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday",
];
const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

/// Sakamoto's algorithm: returns 0=Sunday … 6=Saturday.
fn weekday(year: i32, month: i32, day: i32) -> usize {
    let t: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    let w = (y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + day) % 7;
    w as usize
}

/// Format a "YYYY-MM-DD" date string as "Weekday D Month".
/// Example: "2026-06-27" → "Saturday 27 June"
pub fn format_sample_date(date: &str) -> String {
    let parts: Vec<i32> = date.split('-')
        .filter_map(|s| s.parse().ok())
        .collect();
    if parts.len() != 3 {
        return date.to_string();
    }
    let (year, month, day) = (parts[0], parts[1], parts[2]);
    let wd = weekday(year, month, day);
    format!("{} {} {}", DAYS[wd], day, MONTHS[(month - 1) as usize])
}

pub const SAMPLE_TODAY: &str = "2026-06-27";

/// (time, title) pairs for the sample day.
pub const SAMPLE_TODAY_EVENTS: &[(&str, &str)] = &[
    ("09:00", "Standup"),
    ("12:30", "Lunch"),
    ("15:00", "School pickup"),
];

/// Sample events for the agenda view, used when no calendar feed is configured.
/// Anchored to SAMPLE_TODAY (2026-06-27, a Saturday) and mirrored exactly by
/// sampleCalendar.ts in the editor so the preview matches the panel.
/// `(year, month, day, "HH:MM" or "" for all-day, title)`.
const SAMPLE_AGENDA: &[(i32, u32, u32, &str, &str)] = &[
    (2026, 6, 27, "", "Last day of term"),
    (2026, 6, 27, "08:15", "Choir"),
    (2026, 6, 27, "18:00", "Ballet"),
    (2026, 6, 28, "09:00", "Markets"),
    (2026, 6, 29, "15:00", "School pickup"),
    (2026, 6, 30, "18:00", "Soccer"),
    (2026, 7, 2, "12:30", "Lunch"),
    (2026, 7, 3, "19:30", "Recorder"),
];

/// Build a resolved feed from the sample agenda events, as if it had been
/// fetched and resolved at SAMPLE_TODAY. Lets the agenda renderer use one code
/// path for both real feeds and the no-feed sample fallback.
pub fn sample_feed() -> crate::calendar::ResolvedFeed {
    use crate::calendar::{resolve, VEvent};
    let events: Vec<VEvent> = SAMPLE_AGENDA
        .iter()
        .map(|&(y, m, d, time, title)| VEvent {
            summary: title.to_string(),
            date: (y, m, d),
            time: if time.is_empty() { None } else { Some(time.to_string()) },
            ..Default::default()
        })
        .collect();
    resolve(&events, (2026, 6, 27))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_sample_date_saturday() {
        assert_eq!(format_sample_date("2026-06-27"), "Saturday 27 June");
    }

    #[test]
    fn sample_feed_today_and_tomorrow_have_expected_events() {
        let feed = sample_feed();
        // Slot 0 = today (Sat 27): all-day "Last day of term" sorts first.
        assert_eq!(feed.week[0][0].title, "Last day of term");
        assert!(feed.week[0][0].time.is_empty());
        assert!(feed.week[0].iter().any(|e| e.title == "Choir" && e.time == "08:15"));
        // Slot 1 = tomorrow (Sun 28).
        assert_eq!(feed.week[1].len(), 1);
        assert_eq!(feed.week[1][0].title, "Markets");
        // Labels are full weekday names; slot 2 = Monday.
        assert_eq!(feed.week_labels[2], "Monday");
    }
}
