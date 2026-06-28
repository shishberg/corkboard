use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::{
    calendar::{self, CalendarData},
    config::Config,
    display::{Display, WebPreview},
    document::{Document, Element},
    fonts::Fonts,
    render,
    storage::Storage,
};

pub struct AppState {
    pub storage: Storage,
    pub config: Mutex<Config>,
    pub document: Mutex<Document>,
    pub display: Arc<dyn Display>,
    pub web_preview: Arc<WebPreview>,
    pub fonts: Arc<Fonts>,
    /// Current resolved calendar data — init to `CalendarData::empty()`.
    pub calendar: Mutex<CalendarData>,
    /// Semantic fingerprint of the last rendered calendar content, for change-detection.
    pub displayed_signature: Mutex<Option<String>>,
}

impl AppState {
    /// Re-render the live page with the currently stored calendar data and push to the display.
    pub fn render_and_show(&self) -> anyhow::Result<()> {
        let doc = self.document.lock().unwrap().clone();
        let cfg = self.config.lock().unwrap().clone();
        let cal = self.calendar.lock().unwrap().clone();
        let png = render::render(&doc, &cfg, &self.fonts, &self.storage, &cal)?;
        self.display.show(&png)?;
        Ok(())
    }

    /// Resolve all calendar feeds referenced by the live page.
    ///
    /// For each `feedId` found in a Calendar element on the live page that is
    /// also configured in `config.feeds`, this fetches and resolves the ICS.
    /// Feed fetch failures are logged (by feed id only — never by URL) and that
    /// feed falls back to sample in the renderer.
    ///
    /// Returns a fresh `CalendarData`; does not mutate `self`.
    pub async fn resolve_calendar(&self) -> CalendarData {
        use chrono::Datelike;

        // Real today from the system clock for the I/O path.
        let today = {
            let d = chrono::Local::now().date_naive();
            (d.year(), d.month(), d.day())
        };

        // Collect the distinct feedIds that the live page actually references.
        // Drop the document lock before any await.
        let feed_ids: Vec<String> = {
            let doc = self.document.lock().unwrap();
            if let Some(page) = doc.live_page() {
                let mut ids: Vec<String> = page
                    .elements
                    .iter()
                    .filter_map(|el| {
                        if let Element::Calendar(c) = el {
                            if !c.feed_id.is_empty() {
                                Some(c.feed_id.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                ids.sort();
                ids.dedup();
                ids
            } else {
                vec![]
            }
        };

        // Build a map of feedId → secretUrl for feeds that are both referenced
        // and configured.  Drop the config lock before any await.
        let feed_secrets: std::collections::HashMap<String, String> = {
            let cfg = self.config.lock().unwrap();
            cfg.feeds
                .iter()
                .filter(|f| feed_ids.contains(&f.id))
                .map(|f| (f.id.clone(), f.secret_url.clone()))
                .collect()
        };

        // Fetch and resolve each feed.  Failures are swallowed — the calendar
        // element will fall back to sample data.
        let mut feeds = BTreeMap::new();
        for feed_id in &feed_ids {
            if let Some(secret_url) = feed_secrets.get(feed_id) {
                // Never log the secret URL — only the feed id.
                tracing::info!("fetching calendar feed '{}'", feed_id);
                match calendar::fetch_and_resolve(secret_url, today).await {
                    Ok(resolved) => {
                        let week_count: usize = resolved.week.iter().map(|d| d.len()).sum();
                        tracing::info!(
                            "calendar feed '{}' resolved: {} event(s) today, {} this week",
                            feed_id,
                            resolved.today.len(),
                            week_count
                        );
                        feeds.insert(feed_id.clone(), resolved);
                    }
                    Err(_) => {
                        // Never log the secret URL.  Log only the feed id so
                        // operators can identify which feed failed.
                        tracing::warn!(
                            "calendar feed '{}' could not be fetched; falling back to sample",
                            feed_id
                        );
                    }
                }
            }
            // Feed not configured → leave it out → renderer uses sample fallback.
        }

        CalendarData { today, feeds }
    }

    /// Force-resolve all feeds, store the result, update the signature, and re-render.
    ///
    /// Called by `POST /api/refresh` and `PUT /api/document`.
    pub async fn refresh_and_render(&self) -> anyhow::Result<()> {
        tracing::info!("refreshing: re-resolving calendar feeds and re-rendering");
        let data = self.resolve_calendar().await;
        let sig = calendar::signature(data.today, &data.feeds);

        *self.calendar.lock().unwrap() = data;
        *self.displayed_signature.lock().unwrap() = Some(sig);

        self.render_and_show()
    }

    /// Resolve feeds and re-render ONLY when the semantic content has changed.
    ///
    /// Returns `true` if the display was updated, `false` if the content was
    /// identical to the last render.  Errors from `render_and_show` are
    /// propagated; feed-fetch errors are already swallowed inside
    /// `resolve_calendar`.
    pub async fn poll_once(&self) -> anyhow::Result<bool> {
        let data = self.resolve_calendar().await;
        let new_sig = calendar::signature(data.today, &data.feeds);

        let changed = {
            let guard = self.displayed_signature.lock().unwrap();
            match guard.as_deref() {
                Some(old_sig) => old_sig != new_sig,
                None => true, // first poll — always render
            }
        };

        if changed {
            *self.calendar.lock().unwrap() = data;
            *self.displayed_signature.lock().unwrap() = Some(new_sig);
            self.render_and_show()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
