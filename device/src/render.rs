use std::io::Cursor;

use image::{codecs::png::PngEncoder, ImageEncoder, RgbImage};

use crate::{
    config::Config,
    document::{Document, Element},
};

pub fn render(doc: &Document, _cfg: &Config) -> anyhow::Result<Vec<u8>> {
    let (w, h) = doc.orientation_size();
    let mut img = RgbImage::new(w, h);

    // Fill white background
    for pixel in img.pixels_mut() {
        *pixel = image::Rgb([255u8, 255u8, 255u8]);
    }

    if let Some(page) = doc.live_page() {
        for el in &page.elements {
            let (x, y, ew, eh, colour) = match el {
                Element::Calendar(e) => (e.x, e.y, e.w, e.h, e.colour.rgb()),
                Element::Image(e) => (e.x, e.y, e.w, e.h, e.colour.rgb()),
                Element::Drawing(e) => (e.x, e.y, e.w, e.h, e.colour.rgb()),
                Element::Text(e) => (e.x, e.y, e.w, e.h, e.colour.rgb()),
            };

            let x0 = x as u32;
            let y0 = y as u32;
            let x1 = ((x + ew) as u32).min(w);
            let y1 = ((y + eh) as u32).min(h);

            for py in y0..y1 {
                for px in x0..x1 {
                    img.put_pixel(px, py, image::Rgb(colour));
                }
            }
        }
    }

    let mut buf = Vec::new();
    let encoder = PngEncoder::new(Cursor::new(&mut buf));
    encoder.write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        image::ColorType::Rgb8.into(),
    )?;

    Ok(buf)
}
