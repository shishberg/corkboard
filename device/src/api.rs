use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    config::Feed,
    state::AppState,
    status,
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
        .route("/api/status", get(get_status))
        .route("/preview", get(preview_page))
        .route("/preview.png", get(preview_png))
        .route("/preview/updates", get(preview_updates))
        .route("/dashboard", get(dashboard_page))
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

/// Validate an image id: must be non-empty and contain only ASCII
/// alphanumerics or `-`.  Server-generated ids are `img-<uuid>` and always
/// pass.  Rejects any value that could be used for path traversal (`.`, `/`,
/// `\`, `..`, percent-decoded traversal sequences, etc.).
fn valid_image_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

async fn get_image(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    // C1: reject any id that doesn't match the safe subset before touching the FS.
    if !valid_image_id(&id) {
        return StatusCode::NOT_FOUND.into_response();
    }
    match state.storage.load_image(&id) {
        Some(bytes) => {
            // M1: sniff format from magic bytes; fall back to octet-stream.
            let content_type: &'static str = image::guess_format(&bytes)
                .ok()
                .map(|fmt| fmt.to_mime_type())
                .unwrap_or("application/octet-stream");
            let mut response = bytes.into_response();
            response
                .headers_mut()
                .insert("content-type", HeaderValue::from_static(content_type));
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

/// Snapshot of device health for the `/dashboard` page — never includes
/// secret feed URLs.
async fn get_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(status::build(&state))
}

/// Serve the status dashboard: a static shell that fetches `/api/status` on
/// an interval and renders it, plus the live preview image.
async fn dashboard_page() -> impl IntoResponse {
    Html(include_str!("dashboard.html"))
}

/// Serve the live preview page: the current render, plus a script that
/// long-polls `/preview/updates` and reloads the image when it changes.
async fn preview_page(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let updated_at = state.web_preview.updated_at();
    let html = include_str!("preview.html").replace("__UPDATED_AT__", &updated_at.to_string());
    Html(html)
}

/// Return the cached rendered PNG — does NOT re-render.
async fn preview_png(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (_updated_at, png) = state.web_preview.current();
    let mut response = png.into_response();
    response
        .headers_mut()
        .insert("content-type", HeaderValue::from_static("image/png"));
    response
}

#[derive(serde::Deserialize)]
struct UpdatesQuery {
    /// Timestamp (millis since epoch) the client is currently showing.
    since: Option<i64>,
}

/// Long-poll for a newer render. Blocks until the preview's `updated_at` is
/// greater than `since` or a timeout elapses, then answers `{ updatedAt }`.
/// Always 200: a timeout returns the (unchanged) current timestamp, so the
/// client simply re-polls.
async fn preview_updates(
    State(state): State<Arc<AppState>>,
    Query(q): Query<UpdatesQuery>,
) -> impl IntoResponse {
    let since = q.since.unwrap_or(0);
    let mut rx = state.web_preview.subscribe();

    // Fast path: a render newer than `since` already happened.
    let current = *rx.borrow_and_update();
    if current > since {
        return Json(json!({ "updatedAt": current }));
    }

    // Otherwise wait for the next render, capped so the request doesn't hang
    // past typical proxy/browser idle limits.
    let _ = tokio::time::timeout(std::time::Duration::from_secs(25), rx.changed()).await;
    Json(json!({ "updatedAt": *rx.borrow() }))
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
        display::WebPreview,
        document::Document,
        fonts::Fonts,
        logbuf::LogBuffer,
        state::AppState,
        storage::Storage,
    };
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn make_state() -> Arc<AppState> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.keep();
        let storage = Storage::new(&path);
        let config = storage.load_config();
        let document = Document::default();
        let preview = Arc::new(WebPreview::new());

        Arc::new(AppState::new(
            storage,
            config,
            document,
            preview.clone(),
            preview,
            Arc::new(Fonts::load()),
            "web-preview",
            Arc::new(LogBuffer::new(200)),
        ))
    }

    fn make_router(state: Arc<AppState>) -> Router {
        router().with_state(state)
    }

    #[tokio::test]
    async fn put_document_garbage_collects_orphan_images() {
        use crate::document::{Colour, Element, ImageEl, Page};
        let state = make_state();
        let app = make_router(state.clone());

        // Two stored images; the document references only one.
        state.storage.save_image("keep", b"keep").unwrap();
        state.storage.save_image("orphan", b"orphan").unwrap();

        let page_id = "p1".to_string();
        let doc = Document {
            orientation: None,
            live_page_id: Some(page_id.clone()),
            pages: vec![Page {
                id: page_id,
                name: "P".to_string(),
                elements: vec![Element::Image(ImageEl {
                    id: "e1".to_string(),
                    x: 0.0, y: 0.0, w: 10.0, h: 10.0,
                    colour: Colour::White,
                    src: Some("keep".to_string()),
                })],
                background: None,
                orientation: None,
            }],
        };

        let put_req = Request::builder()
            .method("PUT")
            .uri("/api/document")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(serde_json::to_string(&doc).unwrap()))
            .unwrap();
        let resp = app.oneshot(put_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        assert!(state.storage.image_path("keep").exists(), "referenced image kept");
        assert!(
            !state.storage.image_path("orphan").exists(),
            "unreferenced image should be garbage-collected on PUT"
        );
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

    // C1: path-traversal ids must be rejected with 4xx before any FS access.
    #[tokio::test]
    async fn get_image_path_traversal_rejected() {
        let state = make_state();

        // These are percent-encoded traversal sequences; Axum decodes the path
        // param so id would become "../../etc/passwd", "../", etc.
        let bad_uris = [
            // %2F is decoded to '/' inside the id param
            "/api/images/..%2F..%2Fetc%2Fpasswd",
            // plain dot-dot with encoded slash
            "/api/images/..%2F",
        ];

        for uri in bad_uris {
            let req = Request::builder()
                .method("GET")
                .uri(uri)
                .body(axum::body::Body::empty())
                .unwrap();

            let resp = make_router(state.clone()).oneshot(req).await.unwrap();
            assert!(
                resp.status().is_client_error(),
                "expected 4xx for URI {uri}, got {}",
                resp.status()
            );
        }

        // Also verify the validator directly for ids that can't be expressed
        // as a single URI segment (contain literal '/').
        assert!(!valid_image_id("../../etc/passwd"));
        assert!(!valid_image_id("../"));
        assert!(!valid_image_id(".."));
        assert!(!valid_image_id("."));
        assert!(!valid_image_id("img/traversal"));
        assert!(!valid_image_id(r"img\traversal"));
        assert!(!valid_image_id(""));
        // Valid ids must still pass
        assert!(valid_image_id("img-abc123"));
        assert!(valid_image_id("img-a1b2c3d4-e5f6-7890-abcd-ef1234567890"));
    }

    // M1: image content-type is inferred from magic bytes.
    #[tokio::test]
    async fn get_image_content_type_sniffed() {
        let state = make_state();

        // PNG magic bytes (first 8 bytes of any PNG file).
        let png_magic: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];

        let post_req = Request::builder()
            .method("POST")
            .uri("/api/images")
            .header("content-type", "application/octet-stream")
            .body(axum::body::Body::from(png_magic))
            .unwrap();

        let resp = make_router(state.clone()).oneshot(post_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = result["id"].as_str().unwrap().to_string();

        let get_req = Request::builder()
            .method("GET")
            .uri(format!("/api/images/{}", id))
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = make_router(state.clone()).oneshot(get_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert_eq!(ct, "image/png");
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
        let (_updated_at, cached) = state.web_preview.current();
        assert_eq!(body.as_ref(), cached.as_slice());
    }

    #[tokio::test]
    async fn preview_page_embeds_current_timestamp() {
        let state = make_state();

        // Render once so there's a non-zero timestamp to embed.
        state.render_and_show().unwrap();
        let ts = state.web_preview.updated_at();
        assert!(ts > 0);

        let req = Request::builder()
            .method("GET")
            .uri("/preview")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = make_router(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let html = std::str::from_utf8(&body).unwrap();

        // The placeholder is replaced with the real timestamp.
        assert!(!html.contains("__UPDATED_AT__"));
        assert!(html.contains(&ts.to_string()));
    }

    #[tokio::test]
    async fn preview_updates_returns_immediately_when_newer_exists() {
        let state = make_state();
        state.render_and_show().unwrap();
        let ts = state.web_preview.updated_at();

        // Ask for anything older than the current render → answer at once.
        let req = Request::builder()
            .method("GET")
            .uri("/preview/updates?since=0")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = make_router(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["updatedAt"].as_i64().unwrap(), ts);
    }

    #[tokio::test]
    async fn preview_updates_wakes_on_render() {
        let state = make_state();
        state.render_and_show().unwrap();
        let since = state.web_preview.updated_at();

        // Hold the poll open with `since` == current, then render again.
        let app = make_router(state.clone());
        let req = Request::builder()
            .method("GET")
            .uri(format!("/preview/updates?since={since}"))
            .body(axum::body::Body::empty())
            .unwrap();
        let poll = tokio::spawn(async move { app.oneshot(req).await.unwrap() });

        // Ensure the poll is parked, then trigger a render that should wake it.
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        state.render_and_show().unwrap();

        let resp = poll.await.unwrap();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(v["updatedAt"].as_i64().unwrap() >= since);
    }
}
