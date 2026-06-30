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

    /// Re-render the live page with the current calendar data immediately,
    /// then re-resolve all feeds in the background and re-render again only
    /// when the semantic content has changed.
    ///
    /// Called by `POST /api/refresh` and `PUT /api/document`. The render-first
    /// order means /preview.png reflects the new document (e.g. a flipped
    /// orientation) without waiting for the calendar feed to be fetched.
    pub async fn refresh_and_render(&self) -> anyhow::Result<()> {
        tracing::info!("refresh: rendering first, then re-resolving feeds");
        // Render the new document first with whatever calendar data is currently
        // cached, so the user sees the change (e.g. flipped orientation) on
        // /preview.png immediately rather than waiting for the feed fetch.
        self.render_and_show()?;

        // Then re-resolve feeds and re-render only if the semantic content
        // actually changed.
        let data = self.resolve_calendar().await;
        let new_sig = calendar::signature(data.today, &data.feeds);

        let changed = {
            let guard = self.displayed_signature.lock().unwrap();
            match guard.as_deref() {
                Some(old_sig) => old_sig != new_sig,
                None => true,
            }
        };

        if changed {
            *self.calendar.lock().unwrap() = data;
            *self.displayed_signature.lock().unwrap() = Some(new_sig);
            self.render_and_show()?;
            tracing::info!("refresh: feed content changed, re-rendered");
        } else {
            tracing::info!("refresh: feed content unchanged, skipped re-render");
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::Feed,
        display::WebPreview,
        document::{CalendarEl, CalendarVariant, Colour, Document, Element, TextAlign},
        fonts::Fonts,
        storage::Storage,
    };
    use std::sync::{Arc, Mutex};
    use tokio::io::AsyncReadExt;

    fn make_state() -> Arc<AppState> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.keep();
        let storage = Storage::new(&path);
        let config = storage.load_config();
        let document = Document::default();
        let preview = Arc::new(WebPreview::new());

        Arc::new(AppState {
            storage,
            config: Mutex::new(config),
            document: Mutex::new(document),
            display: preview.clone(),
            web_preview: preview,
            fonts: Arc::new(Fonts::load()),
            calendar: Mutex::new(CalendarData::empty()),
            displayed_signature: Mutex::new(None),
        })
    }

    // `refresh_and_render` is called on every PUT and on /api/refresh. The user
    // expects the new document to be visible in /preview.png immediately, even
    // before calendar feeds have been re-fetched. The fix renders first (with
    // whatever calendar data is cached) and only then re-resolves feeds in the
    // background.
    #[tokio::test]
    async fn refresh_and_render_updates_preview_before_awaiting_calendar() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let accept_task = tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf).await;
                std::future::pending::<()>().await;
            }
        });

        let state = make_state();
        {
            let mut cfg = state.config.lock().unwrap();
            cfg.feeds = vec![Feed {
                id: "hung-feed".to_string(),
                name: "Hung".to_string(),
                secret_url: format!("http://127.0.0.1:{port}/path"),
            }];
        }

        {
            let mut doc = Document::default();
            let page_id = doc.pages[0].id.clone();
            doc.pages[0].elements.push(Element::Calendar(CalendarEl {
                id: "el-cal-1".to_string(),
                x: 10.0,
                y: 10.0,
                w: 300.0,
                h: 200.0,
                colour: Colour::Black,
                variant: CalendarVariant::Agenda,
                feed_id: "hung-feed".to_string(),
                font: String::new(),
                align: TextAlign::Center,
                days_ahead: 7,
                outline: None,
            }));
            doc.live_page_id = Some(page_id);
            state.storage.save_document(&doc).unwrap();
            *state.document.lock().unwrap() = doc;
        }

        assert_eq!(state.web_preview.updated_at(), 0);

        let state_for_task = state.clone();
        let render_task = tokio::spawn(async move {
            state_for_task.refresh_and_render().await
        });

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        assert!(
            state.web_preview.updated_at() > 0,
            "preview should be updated before the calendar fetch completes"
        );

        render_task.abort();
        accept_task.abort();
    }

    // The conditional re-render in `refresh_and_render` must NOT re-render
    // when the resolved calendar signature is unchanged. We pre-populate the
    // signature with the value that a no-feeds `resolve_calendar` will return,
    // then call `refresh_and_render` and assert the render count advanced by
    // exactly 1 (the initial render; the conditional re-render is skipped).
    #[tokio::test]
    async fn refresh_and_render_skips_second_render_when_signature_unchanged() {
        let state = make_state();
        // Warm up: produce the "no feeds" signature the test will match.
        let baseline = state.resolve_calendar().await;
        let sig = crate::calendar::signature(baseline.today, &baseline.feeds);
        *state.displayed_signature.lock().unwrap() = Some(sig);

        let before = state.web_preview.render_count();
        state.refresh_and_render().await.unwrap();
        let after = state.web_preview.render_count();

        assert_eq!(
            after - before,
            1,
            "expected exactly one render (the initial one); the conditional re-render should be skipped when the signature is unchanged"
        );
    }

    // When the signature DOES change between calls (e.g. a feed returned new
    // events), `refresh_and_render` should perform both renders: the initial
    // one for the new document, and the conditional one for the new content.
    #[tokio::test]
    async fn refresh_and_render_renders_twice_when_signature_changes() {
        let state = make_state();
        // Seed with a signature that will NOT match a fresh `resolve_calendar`.
        *state.displayed_signature.lock().unwrap() = Some("stale-signature".to_string());

        let before = state.web_preview.render_count();
        state.refresh_and_render().await.unwrap();
        let after = state.web_preview.render_count();

        assert_eq!(
            after - before,
            2,
            "expected the initial render plus a conditional re-render when the signature changed"
        );
    }
}
