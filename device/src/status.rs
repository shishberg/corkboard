/// Assembles the JSON payload behind `GET /api/status`, the data source for
/// the `/dashboard` page. Read-only: gathers a snapshot of `AppState` plus a
/// handful of process-level facts (OS/arch, the env vars that configured
/// this run). Never includes secret feed URLs.
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStatus {
    pub now_ms: i64,
    pub started_at_ms: i64,
    pub uptime_secs: i64,
    pub hostname: String,
    pub poll_interval_minutes: u64,
    pub feeds: Vec<FeedInfo>,
    pub document: DocumentInfo,
    pub preview: PreviewInfo,
    pub display: DisplayInfo,
    pub fonts: FontsInfo,
    pub env: EnvInfo,
    pub logs: Vec<crate::logbuf::LogEntry>,
    pub system: crate::sysstat::SystemInfo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedInfo {
    pub id: String,
    pub name: String,
    /// `None` if this feed has never been fetched this run (e.g. not
    /// referenced by the live page yet).
    pub last_attempt_ms: Option<i64>,
    pub ok: Option<bool>,
    pub today_event_count: Option<usize>,
    pub error: Option<String>,
    /// The next 7 days (index 0 = today) as currently shown on the display —
    /// from the last resolve that actually changed content, not necessarily
    /// the most recent fetch attempt (a poll that finds no change never
    /// updates `AppState::calendar`). Empty if the feed has never resolved.
    pub week: Vec<DayEvents>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayEvents {
    /// "Today", "Tomorrow", then the weekday name — matches the renderer's
    /// own agenda headings (`render::agenda_heading`).
    pub label: String,
    pub events: Vec<EventInfo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventInfo {
    /// "HH:MM", or "" for an all-day event.
    pub time: String,
    pub title: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInfo {
    pub page_count: usize,
    pub live_page_id: Option<String>,
    pub live_page_name: Option<String>,
    /// mtime of `document.json` — when it was last published.
    pub saved_at_ms: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewInfo {
    /// millis since epoch of the last render pushed to the display; 0 if none yet.
    pub updated_at_ms: i64,
    pub render_count: usize,
    /// Long-poll clients (`GET /preview/updates`) currently connected.
    pub connected_listeners: usize,
    pub last_poll_at_ms: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    /// "panel" or "web-preview" — which `Display` backend is active.
    pub kind: String,
    pub os: String,
    pub arch: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FontsInfo {
    pub default_id: String,
    pub ids: Vec<String>,
    /// Subset of `ids` that also ship a bold (700) face.
    pub bold_ids: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvInfo {
    pub data_dir: String,
    pub dist_dir: String,
    pub fonts_dir: String,
    pub port: String,
}

fn env_var_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

pub fn build(state: &AppState) -> DashboardStatus {
    let now_ms = chrono::Utc::now().timestamp_millis();
    let cfg = state.config.lock().unwrap().clone();
    let doc = state.document.lock().unwrap().clone();
    let feed_status = state.feed_status.lock().unwrap().clone();
    let calendar = state.calendar.lock().unwrap();

    let feeds = cfg
        .feeds
        .iter()
        .map(|f| {
            let s = feed_status.get(&f.id);
            let week = calendar
                .for_feed(&f.id)
                .map(|rf| {
                    rf.week
                        .iter()
                        .zip(rf.week_labels.iter())
                        .enumerate()
                        .map(|(slot, (events, label))| DayEvents {
                            label: crate::render::agenda_heading(slot, label),
                            events: events
                                .iter()
                                .map(|e| EventInfo {
                                    time: e.time.clone(),
                                    title: e.title.clone(),
                                })
                                .collect(),
                        })
                        .collect()
                })
                .unwrap_or_default();
            FeedInfo {
                id: f.id.clone(),
                name: f.name.clone(),
                last_attempt_ms: s.map(|s| s.last_attempt_ms),
                ok: s.map(|s| s.ok),
                today_event_count: s.and_then(|s| s.today_event_count),
                error: s.and_then(|s| s.error.clone()),
                week,
            }
        })
        .collect();
    drop(calendar);

    let live_page = doc.live_page();
    let document = DocumentInfo {
        page_count: doc.pages.len(),
        live_page_id: doc.live_page_id.clone(),
        live_page_name: live_page.map(|p| p.name.clone()),
        saved_at_ms: state.storage.document_saved_at_ms(),
    };

    let (updated_at_ms, _) = state.web_preview.current();
    let preview = PreviewInfo {
        updated_at_ms,
        render_count: state.web_preview.render_count(),
        connected_listeners: state.web_preview.subscriber_count(),
        last_poll_at_ms: *state.last_poll_at_ms.lock().unwrap(),
    };

    let display = DisplayInfo {
        kind: state.display_kind.clone(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    };

    let ids = state.fonts.ids();
    let bold_ids = ids
        .iter()
        .filter(|id| state.fonts.has_bold(id))
        .cloned()
        .collect();
    let fonts = FontsInfo {
        default_id: state.fonts.default_id().to_string(),
        ids,
        bold_ids,
    };

    let env = EnvInfo {
        data_dir: env_var_or("CORKBOARD_DATA", "./data"),
        dist_dir: env_var_or("CORKBOARD_DIST", "../dist"),
        fonts_dir: env_var_or("CORKBOARD_FONTS", "../public/fonts"),
        port: env_var_or("CORKBOARD_PORT", "8080"),
    };

    DashboardStatus {
        now_ms,
        started_at_ms: state.started_at_ms,
        uptime_secs: (now_ms - state.started_at_ms) / 1000,
        hostname: cfg.hostname,
        poll_interval_minutes: cfg.poll_interval_minutes,
        feeds,
        document,
        preview,
        display,
        fonts,
        env,
        logs: state.logs.snapshot(),
        system: crate::sysstat::snapshot(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        document::Document, display::WebPreview, fonts::Fonts, logbuf::LogBuffer,
        storage::Storage,
    };
    use std::sync::Arc;

    fn make_state() -> AppState {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.keep());
        let config = storage.load_config();
        let document = Document::default();
        let preview = Arc::new(WebPreview::new());

        AppState::new(
            storage,
            config,
            document,
            preview.clone(),
            preview,
            Arc::new(Fonts::load()),
            "web-preview",
            Arc::new(LogBuffer::new(200)),
        )
    }

    #[test]
    fn build_reports_no_feeds_and_default_display_kind() {
        let state = make_state();
        let status = build(&state);
        assert!(status.feeds.is_empty());
        assert_eq!(status.display.kind, "web-preview");
        assert_eq!(status.document.page_count, 1);
        assert!(status.fonts.ids.contains(&status.fonts.default_id));
    }

    #[test]
    fn build_surfaces_feed_status_by_id() {
        use crate::{config::Feed, state::FeedStatus};

        let state = make_state();
        {
            let mut cfg = state.config.lock().unwrap();
            cfg.feeds = vec![Feed {
                id: "family".to_string(),
                name: "Family".to_string(),
                secret_url: "https://secret.example.com/token".to_string(),
            }];
        }
        state.feed_status.lock().unwrap().insert(
            "family".to_string(),
            FeedStatus {
                last_attempt_ms: 123,
                ok: false,
                today_event_count: None,
                error: Some("HTTP 404".to_string()),
            },
        );

        let status = build(&state);
        assert_eq!(status.feeds.len(), 1);
        assert_eq!(status.feeds[0].id, "family");
        assert_eq!(status.feeds[0].ok, Some(false));
        assert_eq!(status.feeds[0].error.as_deref(), Some("HTTP 404"));
        assert!(status.feeds[0].week.is_empty());

        // The secret URL must never appear anywhere in the serialized status.
        let json = serde_json::to_string(&status).unwrap();
        assert!(!json.contains("secret.example.com"));
        assert!(!json.contains("token"));
    }

    #[test]
    fn build_surfaces_the_week_ahead_from_the_currently_displayed_calendar() {
        use crate::calendar::{CalendarData, ResolvedEvent, ResolvedFeed};
        use crate::config::Feed;
        use std::collections::BTreeMap;

        let state = make_state();
        {
            let mut cfg = state.config.lock().unwrap();
            cfg.feeds = vec![Feed {
                id: "family".to_string(),
                name: "Family".to_string(),
                secret_url: "https://secret.example.com/token".to_string(),
            }];
        }

        let mut week: [Vec<ResolvedEvent>; 7] = Default::default();
        week[0] = vec![ResolvedEvent {
            time: "09:00".to_string(),
            title: "Standup".to_string(),
        }];
        week[2] = vec![ResolvedEvent {
            time: "".to_string(),
            title: "Bin day".to_string(),
        }];
        let mut week_labels: [String; 7] = Default::default();
        week_labels[2] = "Friday".to_string();

        let mut feeds = BTreeMap::new();
        feeds.insert(
            "family".to_string(),
            ResolvedFeed {
                today: week[0].clone(),
                week,
                week_labels,
            },
        );
        *state.calendar.lock().unwrap() = CalendarData {
            today: (2026, 7, 1),
            feeds,
        };

        let status = build(&state);
        let days = &status.feeds[0].week;
        assert_eq!(days.len(), 7);
        assert_eq!(days[0].label, "Today");
        assert_eq!(days[0].events[0].time, "09:00");
        assert_eq!(days[0].events[0].title, "Standup");
        assert_eq!(days[1].label, "Tomorrow");
        assert!(days[1].events.is_empty());
        assert_eq!(days[2].label, "Friday");
        assert_eq!(days[2].events[0].title, "Bin day");
    }
}
