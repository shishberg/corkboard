mod api;
mod calendar;
mod config;
mod display;
mod document;
mod fonts;
mod render;
mod sample;
mod state;
mod storage;
mod text;

use std::sync::{Arc, Mutex};

use axum::{extract::Request, middleware::Next, response::Response};
use calendar::CalendarData;
use display::WebPreview;
use fonts::Fonts;
use state::AppState;
use storage::Storage;
use tower_http::services::{ServeDir, ServeFile};

/// Log one line per HTTP request: method, path, response status, and how long
/// it took. Applied to the whole app so it covers the API and static files.
async fn log_request(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    let start = std::time::Instant::now();
    let response = next.run(req).await;
    tracing::info!(
        "{} {} -> {} ({} ms)",
        method,
        path,
        response.status().as_u16(),
        start.elapsed().as_millis()
    );
    response
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let data_path = std::env::var("CORKBOARD_DATA").unwrap_or_else(|_| "./data".to_string());
    let dist_path = std::env::var("CORKBOARD_DIST").unwrap_or_else(|_| "../dist".to_string());
    let port = std::env::var("CORKBOARD_PORT").unwrap_or_else(|_| "8080".to_string());

    let storage = Storage::new(&data_path);
    let config = storage.load_config();
    let document = storage
        .load_document()
        .unwrap_or_else(|_| document::Document::default());

    let preview = Arc::new(WebPreview::new());
    let fonts = Arc::new(Fonts::load());

    let state = Arc::new(AppState {
        storage,
        config: Mutex::new(config),
        document: Mutex::new(document),
        display: preview.clone(),
        web_preview: preview,
        fonts,
        calendar: Mutex::new(CalendarData::empty()),
        displayed_signature: Mutex::new(None),
    });

    // Initial render using sample fallback (no feeds resolved yet).
    if let Err(e) = state.render_and_show() {
        tracing::warn!("initial render failed: {}", e);
    }

    // Poll loop: re-resolve feeds on the configured interval and re-render only
    // when the resolved content has changed (semantic change-detection).
    {
        let poll_state = state.clone();
        tokio::spawn(async move {
            loop {
                // Resolve first, then sleep — so real calendar data appears on
                // startup instead of only after the first interval elapses.
                match poll_state.poll_once().await {
                    Ok(true) => tracing::info!("calendar poll: content changed, display updated"),
                    Ok(false) => tracing::debug!("calendar poll: no change"),
                    Err(e) => tracing::warn!("calendar poll error: {}", e),
                }
                let interval_secs = {
                    let cfg = poll_state.config.lock().unwrap();
                    cfg.poll_interval_minutes.max(1) * 60
                };
                tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;
            }
        });
    }

    let api_router = api::router();

    let serve_dir = ServeDir::new(&dist_path)
        .not_found_service(ServeFile::new(format!("{}/index.html", dist_path)));

    let app = api_router
        .with_state(state)
        .fallback_service(serve_dir)
        .layer(axum::middleware::from_fn(log_request));

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
