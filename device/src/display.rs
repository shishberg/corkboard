use std::io::Cursor;

pub trait Display: Send + Sync {
    fn show(&self, png: &[u8]) -> anyhow::Result<()>;
}

pub struct WebPreview {
    latest: std::sync::Mutex<Vec<u8>>,
}

impl WebPreview {
    pub fn new() -> Self {
        WebPreview {
            latest: std::sync::Mutex::new(vec![]),
        }
    }

    pub fn current(&self) -> Vec<u8> {
        let data = self.latest.lock().unwrap();
        if data.is_empty() {
            blank_white_png()
        } else {
            data.clone()
        }
    }
}

impl Display for WebPreview {
    fn show(&self, png: &[u8]) -> anyhow::Result<()> {
        let mut data = self.latest.lock().unwrap();
        *data = png.to_vec();
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
