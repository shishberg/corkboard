use std::io::Cursor;

use image::{codecs::png::PngEncoder, ImageEncoder};

use crate::{
    config::Config,
    document::{Document, Element},
    fonts::Fonts,
    sample,
    storage::Storage,
    text,
};

/// Six-colour e-ink palette (RGB).
const PALETTE: [[u8; 3]; 6] = [
    [0, 0, 0],       // Black
    [255, 255, 255], // White
    [220, 40, 40],   // Red
    [240, 200, 30],  // Yellow
    [40, 80, 200],   // Blue
    [40, 160, 70],   // Green
];

pub fn render(
    doc: &Document,
    _cfg: &Config,
    fonts: &Fonts,
    storage: &Storage,
) -> anyhow::Result<Vec<u8>> {
    let (w, h) = doc.orientation_size();

    let mut pixmap = tiny_skia::Pixmap::new(w, h)
        .ok_or_else(|| anyhow::anyhow!("failed to create pixmap {}×{}", w, h))?;
    pixmap.fill(tiny_skia::Color::WHITE);

    if let Some(page) = doc.live_page() {
        for el in &page.elements {
            match el {
                Element::Text(el) => {
                    let px = (el.w.min(el.h) * 0.25).clamp(10.0, 96.0);
                    text::draw_text(
                        &mut pixmap,
                        fonts.default_font(),
                        &el.text,
                        el.x, el.y, el.w, el.h,
                        px,
                        text::Align::Left,
                        el.colour.rgb(),
                    );
                }

                Element::Calendar(el) => {
                    let colour = el.colour.rgb();

                    // Line 1: big date, centred
                    let date_px = (el.w.min(el.h) * 0.18).clamp(12.0, 72.0);
                    let date_line_h = date_px * 1.25;
                    let date_str = sample::format_sample_date(sample::SAMPLE_TODAY);
                    text::draw_text(
                        &mut pixmap,
                        fonts.default_font(),
                        &date_str,
                        el.x, el.y, el.w, date_line_h,
                        date_px,
                        text::Align::Center,
                        colour,
                    );

                    // Lines 2+: "Today" header + events (smaller)
                    let small_px = (el.w.min(el.h) * 0.09).clamp(8.0, 32.0);
                    let small_line_h = small_px * 1.25;
                    let mut y_pos = el.y + date_line_h;

                    let mut event_lines: Vec<String> = vec!["Today".to_string()];
                    for (time, title) in sample::SAMPLE_TODAY_EVENTS {
                        event_lines.push(format!("{}  {}", time, title));
                    }

                    for line in &event_lines {
                        if y_pos + small_line_h > el.y + el.h {
                            break;
                        }
                        text::draw_text(
                            &mut pixmap,
                            fonts.default_font(),
                            line,
                            el.x, y_pos, el.w, small_line_h,
                            small_px,
                            text::Align::Left,
                            colour,
                        );
                        y_pos += small_line_h;
                    }
                }

                Element::Image(el) => {
                    let loaded = el.image_id.as_deref()
                        .and_then(|id| storage.load_image(id));

                    let decoded = loaded.as_deref()
                        .and_then(|bytes| image::load_from_memory(bytes).ok());

                    match decoded {
                        Some(img) if el.w > 0.0 && el.h > 0.0 => {
                            let ew = el.w as u32;
                            let eh = el.h as u32;
                            let resized = img.resize_exact(
                                ew, eh,
                                image::imageops::FilterType::Nearest,
                            );
                            let rgb = resized.to_rgb8();

                            let ex = el.x as usize;
                            let ey = el.y as usize;
                            let pw = w as usize;
                            let ph = h as usize;
                            let pixels = pixmap.pixels_mut();

                            for dy in 0..eh as usize {
                                let py = ey + dy;
                                if py >= ph {
                                    break;
                                }
                                for dx in 0..ew as usize {
                                    let px = ex + dx;
                                    if px >= pw {
                                        break;
                                    }
                                    let pix = rgb.get_pixel(dx as u32, dy as u32);
                                    let idx = py * pw + px;
                                    pixels[idx] = tiny_skia::ColorU8::from_rgba(
                                        pix[0], pix[1], pix[2], 255,
                                    )
                                    .premultiply();
                                }
                            }
                        }
                        _ => {
                            // Grey placeholder for missing / invalid images
                            fill_rect(&mut pixmap, el.x, el.y, el.w, el.h, [204, 204, 204]);
                        }
                    }
                }

                Element::Drawing(el) => {
                    for stroke in &el.strokes {
                        if stroke.points.len() < 2 {
                            continue;
                        }
                        let mut pb = tiny_skia::PathBuilder::new();
                        pb.move_to(stroke.points[0].x, stroke.points[0].y);
                        for pt in &stroke.points[1..] {
                            pb.line_to(pt.x, pt.y);
                        }
                        if let Some(path) = pb.finish() {
                            let [r, g, b] = stroke.colour.rgb();
                            let mut paint = tiny_skia::Paint::default();
                            paint.set_color_rgba8(r, g, b, 255);
                            let stroke_style = tiny_skia::Stroke {
                                width: stroke.size,
                                line_cap: tiny_skia::LineCap::Round,
                                line_join: tiny_skia::LineJoin::Round,
                                ..Default::default()
                            };
                            pixmap.stroke_path(
                                &path,
                                &paint,
                                &stroke_style,
                                tiny_skia::Transform::identity(),
                                None,
                            );
                        }
                    }
                }
            }
        }
    }

    // Quantise every pixel to the nearest palette colour, then encode as PNG.
    let mut rgb_img = image::RgbImage::new(w, h);
    for (i, pixel) in pixmap.pixels().iter().enumerate() {
        let a = pixel.alpha() as f32;
        let (r, g, b) = if a > 0.0 {
            (
                (pixel.red() as f32 * 255.0 / a).round().clamp(0.0, 255.0) as u8,
                (pixel.green() as f32 * 255.0 / a).round().clamp(0.0, 255.0) as u8,
                (pixel.blue() as f32 * 255.0 / a).round().clamp(0.0, 255.0) as u8,
            )
        } else {
            (255, 255, 255)
        };

        let nearest = PALETTE
            .iter()
            .min_by_key(|&&p| {
                let dr = r as i32 - p[0] as i32;
                let dg = g as i32 - p[1] as i32;
                let db = b as i32 - p[2] as i32;
                dr * dr + dg * dg + db * db
            })
            .copied()
            .unwrap_or([255, 255, 255]);

        let px_x = (i % w as usize) as u32;
        let px_y = (i / w as usize) as u32;
        rgb_img.put_pixel(px_x, px_y, image::Rgb(nearest));
    }

    let mut buf = Vec::new();
    let encoder = PngEncoder::new(Cursor::new(&mut buf));
    encoder.write_image(
        rgb_img.as_raw(),
        rgb_img.width(),
        rgb_img.height(),
        image::ColorType::Rgb8.into(),
    )?;

    Ok(buf)
}

fn fill_rect(pixmap: &mut tiny_skia::Pixmap, x: f32, y: f32, w: f32, h: f32, colour: [u8; 3]) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    if let Some(rect) = tiny_skia::Rect::from_xywh(x, y, w, h) {
        let path = tiny_skia::PathBuilder::from_rect(rect);
        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(colour[0], colour[1], colour[2], 255);
        pixmap.fill_path(
            &path,
            &paint,
            tiny_skia::FillRule::Winding,
            tiny_skia::Transform::identity(),
            None,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::Config,
        document::*,
        fonts::Fonts,
        storage::Storage,
    };

    fn make_doc(elements: Vec<Element>) -> Document {
        let page_id = "page-1".to_string();
        Document {
            orientation: Orientation::Landscape,
            live_page_id: Some(page_id.clone()),
            pages: vec![Page {
                id: page_id,
                name: "Test".to_string(),
                elements,
            }],
        }
    }

    #[test]
    fn render_landscape_size() {
        let doc = make_doc(vec![]);
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let png = render(&doc, &cfg, &fonts, &storage).unwrap();
        let img = image::load_from_memory(&png).unwrap();
        assert_eq!(img.width(), 800);
        assert_eq!(img.height(), 480);
    }

    #[test]
    fn render_portrait_size() {
        let mut doc = make_doc(vec![]);
        doc.orientation = Orientation::Portrait;
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let png = render(&doc, &cfg, &fonts, &storage).unwrap();
        let img = image::load_from_memory(&png).unwrap();
        assert_eq!(img.width(), 480);
        assert_eq!(img.height(), 800);
    }

    #[test]
    fn all_pixels_are_palette_colours() {
        let doc = make_doc(vec![]);
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let png = render(&doc, &cfg, &fonts, &storage).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgb8();
        for pixel in img.pixels() {
            let rgb = [pixel[0], pixel[1], pixel[2]];
            assert!(
                PALETTE.contains(&rgb),
                "pixel {:?} not in palette",
                rgb
            );
        }
    }

    #[test]
    fn red_text_produces_red_pixels() {
        let doc = make_doc(vec![Element::Text(TextEl {
            id: "t1".to_string(),
            x: 10.0, y: 10.0, w: 300.0, h: 100.0,
            colour: Colour::Red,
            text: "Hello World".to_string(),
        })]);
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let png = render(&doc, &cfg, &fonts, &storage).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgb8();
        let red = [220u8, 40, 40];
        let has_red = img.pixels().any(|p| [p[0], p[1], p[2]] == red);
        assert!(has_red, "expected at least one red palette pixel");
    }

    #[test]
    fn drawing_element_changes_pixels() {
        let empty_doc = make_doc(vec![]);
        let drawing_doc = make_doc(vec![Element::Drawing(DrawingEl {
            id: "d1".to_string(),
            x: 0.0, y: 0.0, w: 400.0, h: 300.0,
            colour: Colour::Black,
            strokes: vec![Stroke {
                colour: Colour::Black,
                size: 5.0,
                points: vec![
                    Point { x: 10.0, y: 10.0 },
                    Point { x: 100.0, y: 100.0 },
                ],
            }],
        })]);
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let empty_png = render(&empty_doc, &cfg, &fonts, &storage).unwrap();
        let drawing_png = render(&drawing_doc, &cfg, &fonts, &storage).unwrap();
        assert_ne!(empty_png, drawing_png, "drawing should change pixels");
    }

    #[test]
    fn missing_image_renders_placeholder() {
        let doc = make_doc(vec![Element::Image(ImageEl {
            id: "i1".to_string(),
            x: 100.0, y: 100.0, w: 200.0, h: 150.0,
            colour: Colour::White,
            image_id: Some("nonexistent-image-id".to_string()),
        })]);
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let png = render(&doc, &cfg, &fonts, &storage).unwrap();
        let img = image::load_from_memory(&png).unwrap();
        assert_eq!(img.width(), 800);
    }
}
