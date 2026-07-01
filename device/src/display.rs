use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::sync::watch;

pub trait Display: Send + Sync {
    fn show(&self, png: &[u8]) -> anyhow::Result<()>;
}

/// Fans a render out to multiple `Display`s. Used so the physical panel and
/// the browser-facing `WebPreview` both get every render — `/preview.png`
/// stays useful for checking what should be on the wall even when the panel
/// is the "real" display.
///
/// Order matters: `main.rs` puts `WebPreview` before `Panel`, so a failed
/// panel render still leaves `/preview.png` showing the frame that *should*
/// be on the wall, even though it isn't there yet — useful for spotting
/// "the render pipeline is fine, the panel isn't" while debugging. The
/// accepted tradeoff is that `/preview.png` can briefly show a frame the
/// physical panel doesn't have yet if the panel call then fails or hangs
/// (the panel's own 120s BUSY timeout, see `panel.rs`). Reversing the order
/// would fix that but block preview updates behind a possible panel hang,
/// which is worse for the common case (working panel).
pub struct Fanout(pub Vec<Arc<dyn Display>>);

impl Display for Fanout {
    fn show(&self, png: &[u8]) -> anyhow::Result<()> {
        for d in &self.0 {
            d.show(png)?;
        }
        Ok(())
    }
}

struct Frame {
    /// Millis since the Unix epoch of the last render; 0 means "never rendered".
    updated_at: i64,
    bytes: Vec<u8>,
}

pub struct WebPreview {
    frame: std::sync::Mutex<Frame>,
    /// Notifies long-pollers of a new render. The watched value is the latest
    /// `updated_at`.
    tx: watch::Sender<i64>,
    /// How many times `show` has been called. Tests use this to count renders
    /// without parsing timestamps.
    render_count: AtomicUsize,
}

impl WebPreview {
    pub fn new() -> Self {
        let (tx, _rx) = watch::channel(0);
        WebPreview {
            frame: std::sync::Mutex::new(Frame {
                updated_at: 0,
                bytes: vec![],
            }),
            tx,
            render_count: AtomicUsize::new(0),
        }
    }

    /// The current PNG paired with its render timestamp (millis since epoch; 0
    /// if nothing has been rendered yet, in which case the PNG is a blank).
    pub fn current(&self) -> (i64, Vec<u8>) {
        let frame = self.frame.lock().unwrap();
        if frame.bytes.is_empty() {
            (frame.updated_at, blank_white_png())
        } else {
            (frame.updated_at, frame.bytes.clone())
        }
    }

    /// Timestamp of the last render (millis since epoch; 0 if never).
    pub fn updated_at(&self) -> i64 {
        self.frame.lock().unwrap().updated_at
    }

    /// Number of times `show` has been called. Useful for tests asserting that
    /// a code path performed exactly N renders, not just "at least one".
    #[allow(dead_code)]
    pub fn render_count(&self) -> usize {
        self.render_count.load(Ordering::Relaxed)
    }

    /// Subscribe to render notifications. The receiver's value is the latest
    /// `updated_at`; `changed()` resolves on the next render.
    pub fn subscribe(&self) -> watch::Receiver<i64> {
        self.tx.subscribe()
    }
}

impl Display for WebPreview {
    fn show(&self, png: &[u8]) -> anyhow::Result<()> {
        self.render_count.fetch_add(1, Ordering::Relaxed);
        let now = chrono::Utc::now().timestamp_millis();
        {
            let mut frame = self.frame.lock().unwrap();
            frame.updated_at = now;
            frame.bytes = png.to_vec();
        }
        // Store the new timestamp and wake any waiting long-pollers.
        // `send_replace` updates the value even when no receiver is currently
        // subscribed (unlike `send`, which no-ops with zero receivers).
        self.tx.send_replace(now);
        Ok(())
    }
}

fn blank_white_png() -> Vec<u8> {
    use image::{codecs::png::PngEncoder, ImageEncoder, RgbImage};
    let img = RgbImage::from_pixel(1, 1, image::Rgb([255u8, 255u8, 255u8]));
    let mut buf = Vec::new();
    let encoder = PngEncoder::new(Cursor::new(&mut buf));
    encoder
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            image::ColorType::Rgb8.into(),
        )
        .expect("encode blank PNG");
    buf
}
