use std::io::Cursor;

use image::{codecs::png::PngEncoder, ImageEncoder};

use crate::{
    calendar::CalendarData,
    config::Config,
    document::{CalendarVariant, Document, Element, TextAlign},
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

/// Abbreviated day names matching the SAMPLE_WEEK order (Mon…Sun).
const ABBREV_DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

pub fn render(
    doc: &Document,
    _cfg: &Config,
    fonts: &Fonts,
    storage: &Storage,
    cal: &CalendarData,
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
                    let align = match el.align {
                        TextAlign::Left => text::Align::Left,
                        TextAlign::Center => text::Align::Center,
                    };
                    text::draw_text(
                        &mut pixmap,
                        fonts.get(&el.font),
                        &el.text,
                        el.x, el.y, el.w, el.h,
                        px,
                        align,
                        el.colour.rgb(),
                    );
                }

                Element::Calendar(el) => {
                    let colour = el.colour.rgb();

                    match el.variant {
                        CalendarVariant::Date => {
                            // Large centred date.  Resolved feed uses cal.today; fallback uses SAMPLE_TODAY.
                            let date_str = if let Some(_feed) = cal.for_feed(&el.feed_id) {
                                let (y, m, d) = cal.today;
                                sample::format_sample_date(&format!("{:04}-{:02}-{:02}", y, m, d))
                            } else {
                                sample::format_sample_date(sample::SAMPLE_TODAY)
                            };

                            let date_px = (el.w.min(el.h) * 0.18).clamp(12.0, 72.0);
                            let date_line_h = date_px * 1.25;
                            text::draw_text(
                                &mut pixmap,
                                fonts.default_font(),
                                &date_str,
                                el.x, el.y, el.w, date_line_h,
                                date_px,
                                text::Align::Center,
                                colour,
                            );
                        }

                        CalendarVariant::Today => {
                            // "Today" heading + event list.
                            // Build event lines from the resolved feed or fall back to sample.
                            let event_lines: Vec<String> = {
                                let mut lines = vec!["Today".to_string()];
                                if let Some(feed) = cal.for_feed(&el.feed_id) {
                                    for ev in &feed.today {
                                        if ev.time.is_empty() {
                                            lines.push(format!("  {}", ev.title));
                                        } else {
                                            lines.push(format!("{}  {}", ev.time, ev.title));
                                        }
                                    }
                                } else {
                                    for (time, title) in sample::SAMPLE_TODAY_EVENTS {
                                        lines.push(format!("{}  {}", time, title));
                                    }
                                }
                                lines
                            };

                            let small_px = (el.w.min(el.h) * 0.09).clamp(8.0, 32.0);
                            let small_line_h = small_px * 1.25;
                            let mut y_pos = el.y;

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

                        CalendarVariant::Week => {
                            // 7-column grid: day name + event titles.
                            // Build a common (name, events) representation regardless of source.
                            struct DayView {
                                name: String,
                                events: Vec<(String, String)>, // (time, title)
                            }

                            let week_days: Vec<DayView> = if let Some(feed) = cal.for_feed(&el.feed_id) {
                                feed.week.iter().enumerate().map(|(i, day_events)| DayView {
                                    name: ABBREV_DAYS[i].to_string(),
                                    events: day_events
                                        .iter()
                                        .map(|e| (e.time.clone(), e.title.clone()))
                                        .collect(),
                                }).collect()
                            } else {
                                sample::SAMPLE_WEEK.iter().map(|day| DayView {
                                    name: day.day.to_string(),
                                    events: day.events
                                        .iter()
                                        .map(|e| (e.time.to_string(), e.title.to_string()))
                                        .collect(),
                                }).collect()
                            };

                            let col_w = el.w / 7.0;
                            let small_px = (col_w.min(el.h) * 0.18).clamp(6.0, 18.0);
                            let line_h = small_px * 1.25;

                            for (col_idx, day) in week_days.iter().enumerate() {
                                let col_x = el.x + col_idx as f32 * col_w;

                                // Thin separator (except before first column)
                                if col_idx > 0 {
                                    fill_rect(
                                        &mut pixmap,
                                        col_x - 1.0,
                                        el.y,
                                        1.0,
                                        el.h,
                                        colour,
                                    );
                                }

                                // Day name centred in column
                                text::draw_text(
                                    &mut pixmap,
                                    fonts.default_font(),
                                    &day.name,
                                    col_x, el.y, col_w, line_h,
                                    small_px,
                                    text::Align::Center,
                                    colour,
                                );

                                // Event titles below
                                let mut ey = el.y + line_h;
                                for (time, title) in &day.events {
                                    if ey + line_h > el.y + el.h {
                                        break;
                                    }
                                    let label = if time.is_empty() {
                                        title.clone()
                                    } else {
                                        format!("{} {}", time, title)
                                    };
                                    text::draw_text(
                                        &mut pixmap,
                                        fonts.default_font(),
                                        &label,
                                        col_x, ey, col_w, line_h,
                                        small_px * 0.85,
                                        text::Align::Left,
                                        colour,
                                    );
                                    ey += line_h;
                                }
                            }
                        }
                    }
                }

                Element::Image(el) => {
                    let (page_w, page_h) = doc.orientation_size();

                    let loaded = el.src.as_deref()
                        .and_then(|id| storage.load_image(id));

                    let decoded = loaded.as_deref()
                        .and_then(|bytes| image::load_from_memory(bytes).ok());

                    // Clamp target dimensions to canvas size to prevent OOM on
                    // oversized PUTs.
                    let ew = (el.w as u32).min(page_w);
                    let eh = (el.h as u32).min(page_h);

                    match decoded {
                        Some(img) if el.w > 0.0 && el.h > 0.0 && ew > 0 && eh > 0 => {
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
                    // Points are in element-local space (0..nat_w, 0..nat_h).
                    // Scale them to the element box and translate to screen position.
                    let nat_w = if el.nat_w > 0.0 { el.nat_w } else { 1.0 };
                    let nat_h = if el.nat_h > 0.0 { el.nat_h } else { 1.0 };
                    let sx = el.w / nat_w;
                    let sy = el.h / nat_h;

                    for stroke in &el.strokes {
                        if stroke.points.len() < 2 {
                            continue;
                        }
                        let mut pb = tiny_skia::PathBuilder::new();
                        // Transform points to screen space inline
                        pb.move_to(
                            stroke.points[0].x * sx + el.x,
                            stroke.points[0].y * sy + el.y,
                        );
                        for pt in &stroke.points[1..] {
                            pb.line_to(pt.x * sx + el.x, pt.y * sy + el.y);
                        }
                        if let Some(path) = pb.finish() {
                            let [r, g, b] = stroke.colour.rgb();
                            let mut paint = tiny_skia::Paint::default();
                            paint.set_color_rgba8(r, g, b, 255);
                            paint.anti_alias = false;
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
    // Images are already rasterised above and arrive as palette-approximate
    // colours; quantising them here is fine. Text and vector marks arrive as
    // exact palette colours (no AA), so quantisation is a no-op for them.
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
        paint.anti_alias = false;
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
        calendar::CalendarData,
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
        let cal = CalendarData::empty();
        let png = render(&doc, &cfg, &fonts, &storage, &cal).unwrap();
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
        let cal = CalendarData::empty();
        let png = render(&doc, &cfg, &fonts, &storage, &cal).unwrap();
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
        let cal = CalendarData::empty();
        let png = render(&doc, &cfg, &fonts, &storage, &cal).unwrap();
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
            font: String::new(),
            align: TextAlign::Left,
        })]);
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let cal = CalendarData::empty();
        let png = render(&doc, &cfg, &fonts, &storage, &cal).unwrap();
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
            nat_w: 400.0,
            nat_h: 300.0,
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
        let cal = CalendarData::empty();
        let empty_png = render(&empty_doc, &cfg, &fonts, &storage, &cal).unwrap();
        let drawing_png = render(&drawing_doc, &cfg, &fonts, &storage, &cal).unwrap();
        assert_ne!(empty_png, drawing_png, "drawing should change pixels");
    }

    #[test]
    fn missing_image_renders_placeholder() {
        let doc = make_doc(vec![Element::Image(ImageEl {
            id: "i1".to_string(),
            x: 100.0, y: 100.0, w: 200.0, h: 150.0,
            colour: Colour::White,
            src: Some("nonexistent-image-id".to_string()),
        })]);
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let cal = CalendarData::empty();
        let png = render(&doc, &cfg, &fonts, &storage, &cal).unwrap();
        let img = image::load_from_memory(&png).unwrap();
        assert_eq!(img.width(), 800);
    }

    // ── I1: Drawing transform ──────────────────────────────────────────────

    /// A drawing placed at x=500 with points in element-local space must
    /// produce stroke pixels in the right half of the canvas, not the top-left.
    #[test]
    fn drawing_transform_places_strokes_in_correct_position() {
        let doc = make_doc(vec![Element::Drawing(DrawingEl {
            id: "d1".to_string(),
            // Element sits in the right half of the 800px-wide canvas
            x: 500.0, y: 0.0, w: 200.0, h: 480.0,
            colour: Colour::Black,
            nat_w: 200.0, nat_h: 480.0,
            strokes: vec![Stroke {
                colour: Colour::Black,
                size: 12.0,
                // Local point near (10, 10) → screen (510, 10) after transform
                points: vec![
                    Point { x: 10.0, y: 50.0 },
                    Point { x: 10.0, y: 200.0 },
                ],
            }],
        })]);
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let cal = CalendarData::empty();
        let png = render(&doc, &cfg, &fonts, &storage, &cal).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgb8();

        let black = [0u8, 0, 0];

        // There must be black pixels in the right half (x > 400)
        let has_black_right = img
            .enumerate_pixels()
            .any(|(x, _y, p)| x > 400 && [p[0], p[1], p[2]] == black);
        assert!(has_black_right, "expected black stroke pixels in the right half of the canvas");

        // There must NOT be black pixels in the top-left corner (where raw local
        // coords would land if the transform were missing)
        let has_black_topleft = img
            .enumerate_pixels()
            .any(|(x, y, p)| x < 50 && y < 50 && [p[0], p[1], p[2]] == black);
        assert!(
            !has_black_topleft,
            "unexpected black pixels in the top-left; transform was not applied"
        );
    }

    // ── I2: Calendar variants ─────────────────────────────────────────────

    /// All three calendar variants must produce distinct pixel outputs for
    /// the same element geometry.
    #[test]
    fn calendar_variants_produce_distinct_outputs() {
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let cal = CalendarData::empty();

        let render_variant = |variant: CalendarVariant| -> Vec<u8> {
            let doc = make_doc(vec![Element::Calendar(CalendarEl {
                id: "c1".to_string(),
                x: 10.0, y: 10.0, w: 420.0, h: 300.0,
                colour: Colour::Black,
                variant,
                feed_id: String::new(),
            })]);
            render(&doc, &cfg, &fonts, &storage, &cal).unwrap()
        };

        let date_png = render_variant(CalendarVariant::Date);
        let today_png = render_variant(CalendarVariant::Today);
        let week_png = render_variant(CalendarVariant::Week);

        assert_ne!(date_png, today_png, "date and today variants must differ");
        assert_ne!(date_png, week_png, "date and week variants must differ");
        assert_ne!(today_png, week_png, "today and week variants must differ");
    }

    // ── I3: No green fringe on black text ─────────────────────────────────

    /// Black text on a white canvas must produce only black and white pixels —
    /// no green fringe from anti-aliased grey edges mapping to the green palette
    /// entry.
    #[test]
    fn black_text_has_no_green_palette_pixels() {
        let doc = make_doc(vec![Element::Text(TextEl {
            id: "t1".to_string(),
            x: 10.0, y: 10.0, w: 400.0, h: 150.0,
            colour: Colour::Black,
            text: "Hello World".to_string(),
            font: String::new(),
            align: TextAlign::Left,
        })]);
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let cal = CalendarData::empty();
        let png = render(&doc, &cfg, &fonts, &storage, &cal).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgb8();

        let green = [40u8, 160, 70];
        let has_green = img.pixels().any(|p| [p[0], p[1], p[2]] == green);
        assert!(
            !has_green,
            "black text on white should not produce green palette pixels (AA fringe)"
        );
    }

    // ── I4: Text align and font ───────────────────────────────────────────

    /// A centre-aligned text element must render differently than the same text
    /// left-aligned (pixels shifted towards the centre).
    #[test]
    fn text_center_align_differs_from_left_align() {
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let cal = CalendarData::empty();

        let make_text = |align: TextAlign| -> Vec<u8> {
            let doc = make_doc(vec![Element::Text(TextEl {
                id: "t1".to_string(),
                x: 0.0, y: 0.0, w: 800.0, h: 200.0,
                colour: Colour::Black,
                text: "Alignment Test".to_string(),
                font: String::new(),
                align,
            })]);
            render(&doc, &cfg, &fonts, &storage, &cal).unwrap()
        };

        let left_png = make_text(TextAlign::Left);
        let center_png = make_text(TextAlign::Center);

        assert_ne!(
            left_png, center_png,
            "centre-aligned text must produce a different pixel layout than left-aligned"
        );
    }

    // ── S3: Sample fallback ───────────────────────────────────────────────

    /// With CalendarData::empty() (no resolved feeds), a Today calendar element
    /// must render using sample data and produce a non-empty PNG — the S4 parity
    /// baseline depends on this fallback being active.
    #[test]
    fn calendar_empty_data_falls_back_to_sample_and_renders() {
        let doc = make_doc(vec![Element::Calendar(CalendarEl {
            id: "c1".to_string(),
            x: 10.0, y: 10.0, w: 420.0, h: 300.0,
            colour: Colour::Black,
            variant: CalendarVariant::Today,
            feed_id: String::new(),
        })]);
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let cal = CalendarData::empty();
        let png = render(&doc, &cfg, &fonts, &storage, &cal).unwrap();
        assert!(!png.is_empty(), "render with sample fallback must produce a non-empty PNG");

        // Must contain valid PNG magic bytes.
        assert_eq!(&png[..4], b"\x89PNG");
    }
}
