mod api;
mod config;
mod display;
mod document;
mod render;
mod state;
mod storage;

use std::sync::{Arc, Mutex};

use display::WebPreview;
use state::AppState;
use storage::Storage;
use tower_http::services::{ServeDir, ServeFile};

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

    let state = Arc::new(AppState {
        storage,
        config: Mutex::new(config),
        document: Mutex::new(document),
        display: preview,
    });

    if let Err(e) = state.render_and_show() {
        tracing::warn!("initial render failed: {}", e);
    }

    let api_router = api::router();

    let serve_dir = ServeDir::new(&dist_path)
        .not_found_service(ServeFile::new(format!("{}/index.html", dist_path)));

    let app = api_router.with_state(state).fallback_service(serve_dir);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
