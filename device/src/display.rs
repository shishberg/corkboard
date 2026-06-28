use std::io::Cursor;

use tokio::sync::watch;

pub trait Display: Send + Sync {
    fn show(&self, png: &[u8]) -> anyhow::Result<()>;
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

    /// Subscribe to render notifications. The receiver's value is the latest
    /// `updated_at`; `changed()` resolves on the next render.
    pub fn subscribe(&self) -> watch::Receiver<i64> {
        self.tx.subscribe()
    }
}

impl Display for WebPreview {
    fn show(&self, png: &[u8]) -> anyhow::Result<()> {
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
