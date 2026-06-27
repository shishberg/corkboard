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

#[allow(dead_code)]
pub struct WeekEvent {
    pub time: &'static str,
    pub title: &'static str,
}

#[allow(dead_code)]
pub struct WeekDay {
    pub day: &'static str,
    pub events: &'static [WeekEvent],
}

#[allow(dead_code)]
pub const SAMPLE_WEEK: &[WeekDay] = &[
    WeekDay { day: "Mon", events: &[WeekEvent { time: "09:00", title: "Standup" }] },
    WeekDay { day: "Tue", events: &[WeekEvent { time: "18:00", title: "Soccer" }] },
    WeekDay { day: "Wed", events: &[] },
    WeekDay { day: "Thu", events: &[WeekEvent { time: "12:30", title: "Lunch" }] },
    WeekDay { day: "Fri", events: &[WeekEvent { time: "15:00", title: "Pickup" }] },
    WeekDay { day: "Sat", events: &[] },
    WeekDay { day: "Sun", events: &[WeekEvent { time: "10:00", title: "Market" }] },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_sample_date_saturday() {
        assert_eq!(format_sample_date("2026-06-27"), "Saturday 27 June");
    }
}
