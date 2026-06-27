use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    config::Feed,
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/document", get(get_document))
        .route("/api/document", put(put_document))
        .route("/api/images", post(post_image))
        .route("/api/images/{id}", get(get_image))
        .route("/api/feeds", get(get_feeds))
        .route("/api/feeds", put(put_feeds))
        .route("/api/refresh", post(refresh))
        .route("/preview.png", get(preview_png))
}

async fn get_document(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let doc = state.document.lock().unwrap().clone();
    Json(doc)
}

async fn put_document(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let doc: crate::document::Document = serde_json::from_slice(&body)
        .map_err(|e| anyhow::anyhow!("invalid document JSON: {}", e))?;

    state.storage.save_document(&doc)?;
    state.storage.gc_images(&doc)?;

    {
        let mut guard = state.document.lock().unwrap();
        *guard = doc;
    }

    // Re-resolve feeds because the new document may reference different feedIds.
    state.refresh_and_render().await?;

    Ok(Json(json!({"ok": true})))
}

async fn post_image(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let id = format!("img-{}", Uuid::new_v4());
    state.storage.save_image(&id, &body)?;
    Ok(Json(json!({"id": id})))
}

async fn get_image(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    match state.storage.load_image(&id) {
        Some(bytes) => {
            let mut response = bytes.into_response();
            response
                .headers_mut()
                .insert("content-type", HeaderValue::from_static("application/octet-stream"));
            response
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_feeds(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let cfg = state.config.lock().unwrap();
    Json(cfg.public_feeds())
}

async fn put_feeds(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let feeds: Vec<Feed> = serde_json::from_slice(&body)
        .map_err(|e| anyhow::anyhow!("invalid feeds JSON: {}", e))?;

    {
        let mut cfg = state.config.lock().unwrap();
        cfg.feeds = feeds;
        state.storage.save_config(&cfg)?;
    }

    Ok(Json(json!({"ok": true})))
}

async fn refresh(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    state.refresh_and_render().await?;
    Ok(Json(json!({"ok": true})))
}

/// Return the cached rendered PNG — does NOT re-render.
async fn preview_png(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let png = state.web_preview.current();
    let mut response = png.into_response();
    response
        .headers_mut()
        .insert("content-type", HeaderValue::from_static("image/png"));
    response
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {}", self.0),
        )
            .into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError(e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        calendar::CalendarData,
        display::WebPreview,
        document::Document,
        fonts::Fonts,
        state::AppState,
        storage::Storage,
    };
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

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

    fn make_router(state: Arc<AppState>) -> Router {
        router().with_state(state)
    }

    #[tokio::test]
    async fn put_then_get_document_and_preview() {
        let state = make_state();
        let app = make_router(state.clone());

        // Build a simple document
        let doc = Document::default();
        let doc_json = serde_json::to_string(&doc).unwrap();

        // PUT /api/document
        let put_req = Request::builder()
            .method("PUT")
            .uri("/api/document")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(doc_json))
            .unwrap();

        let resp = app.clone().oneshot(put_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // GET /api/document
        let get_req = Request::builder()
            .method("GET")
            .uri("/api/document")
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = make_router(state.clone()).oneshot(get_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let returned: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(returned.get("pages").is_some());

        // GET /preview.png
        let preview_req = Request::builder()
            .method("GET")
            .uri("/preview.png")
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = make_router(state.clone()).oneshot(preview_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        // PNG magic bytes
        assert_eq!(&body[0..4], b"\x89PNG");
    }

    #[tokio::test]
    async fn feeds_never_expose_secret_url() {
        let state = make_state();
        let app = make_router(state.clone());

        let feeds_json = serde_json::json!([{
            "id": "feed-1",
            "name": "My Bulletin",
            "secretUrl": "https://secret.example.com/very-secret-token"
        }]);

        // PUT /api/feeds
        let put_req = Request::builder()
            .method("PUT")
            .uri("/api/feeds")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(feeds_json.to_string()))
            .unwrap();

        let resp = app.oneshot(put_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // GET /api/feeds
        let get_req = Request::builder()
            .method("GET")
            .uri("/api/feeds")
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = make_router(state.clone()).oneshot(get_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let body_str = std::str::from_utf8(&body).unwrap();

        assert!(!body_str.contains("very-secret-token"));
        assert!(!body_str.contains("secretUrl"));
        assert!(body_str.contains("My Bulletin"));
    }

    #[tokio::test]
    async fn post_image_and_retrieve() {
        let state = make_state();
        let image_bytes = b"fake-png-data-\x89PNG\r\n";

        // POST /api/images
        let post_req = Request::builder()
            .method("POST")
            .uri("/api/images")
            .header("content-type", "image/png")
            .body(axum::body::Body::from(image_bytes.as_ref()))
            .unwrap();

        let resp = make_router(state.clone()).oneshot(post_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = result["id"].as_str().unwrap().to_string();
        assert!(id.starts_with("img-"));

        // GET /api/images/:id
        let get_req = Request::builder()
            .method("GET")
            .uri(format!("/api/images/{}", id))
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = make_router(state.clone()).oneshot(get_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.as_ref(), image_bytes);
    }

    #[tokio::test]
    async fn get_image_not_found() {
        let state = make_state();

        let get_req = Request::builder()
            .method("GET")
            .uri("/api/images/nonexistent-id")
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = make_router(state).oneshot(get_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn preview_png_serves_cached_bytes() {
        let state = make_state();
        let app = make_router(state.clone());

        // PUT a document to trigger render
        let doc = Document::default();
        let doc_json = serde_json::to_string(&doc).unwrap();
        let put_req = Request::builder()
            .method("PUT")
            .uri("/api/document")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(doc_json))
            .unwrap();
        let resp = app.clone().oneshot(put_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // GET /preview.png
        let preview_req = Request::builder()
            .method("GET")
            .uri("/preview.png")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = make_router(state.clone()).oneshot(preview_req).await.unwrap();
        let body = resp.into_body().collect().await.unwrap().to_bytes();

        // Must match what's cached in web_preview
        let cached = state.web_preview.current();
        assert_eq!(body.as_ref(), cached.as_slice());
    }
}
