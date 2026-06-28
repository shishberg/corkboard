use std::io::Cursor;

use image::{codecs::png::PngEncoder, ImageEncoder};

use crate::{
    calendar::{CalendarData, ResolvedFeed},
    config::Config,
    document::{CalendarVariant, Colour, Document, Element, TextAlign},
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
    cal: &CalendarData,
) -> anyhow::Result<Vec<u8>> {
    let (w, h) = doc.orientation_size();

    let mut pixmap = tiny_skia::Pixmap::new(w, h)
        .ok_or_else(|| anyhow::anyhow!("failed to create pixmap {}×{}", w, h))?;

    // Paint the page background (white when unset), then draw elements on top.
    let bg = doc
        .live_page()
        .and_then(|p| p.background.clone())
        .unwrap_or(Colour::White)
        .rgb();
    pixmap.fill(tiny_skia::Color::from_rgba8(bg[0], bg[1], bg[2], 255));

    if let Some(page) = doc.live_page() {
        for el in &page.elements {
            match el {
                Element::Text(el) => {
                    // Auto-size the text to fill its box: the largest size at
                    // which the wrapped text still fits width and height.
                    let font = fonts.get(&el.font);
                    let px = text::fit_font_size(font, &el.text, el.w, el.h, 10.0, 240.0);
                    let align = match el.align {
                        TextAlign::Left => text::Align::Left,
                        TextAlign::Center => text::Align::Center,
                    };
                    text::draw_text(
                        &mut pixmap,
                        font,
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

                            // Floor at 11px: below ~10px the unhinted outline
                            // font can't render legibly as 1-bit on the panel.
                            let small_px = (el.w.min(el.h) * 0.09).clamp(11.0, 32.0);
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

                        CalendarVariant::Agenda => {
                            // 7-day agenda: a day heading (Today, Tomorrow, then
                            // weekday names) with that day's events beneath it.
                            // Uses the resolved feed, or the sample feed when no
                            // feed is configured. The editor mirrors this layout.
                            let sample_owned;
                            let feed = match cal.for_feed(&el.feed_id) {
                                Some(f) => f,
                                None => {
                                    sample_owned = sample::sample_feed();
                                    &sample_owned
                                }
                            };
                            draw_agenda(
                                &mut pixmap,
                                fonts.default_font(),
                                feed,
                                el.x, el.y, el.w, el.h,
                                colour,
                            );
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
                                image::imageops::FilterType::Lanczos3,
                            );
                            // Keep the alpha channel so transparent PNG regions
                            // let whatever is underneath show through.
                            let rgba = resized.to_rgba8();

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
                                    let pix = rgba.get_pixel(dx as u32, dy as u32);
                                    let sa = pix[3] as f32 / 255.0;
                                    if sa <= 0.0 {
                                        // Fully transparent: leave the background.
                                        continue;
                                    }
                                    let idx = py * pw + px;
                                    // Composite src over dest. Everything already
                                    // painted is opaque (alpha 255), so the
                                    // destination's premultiplied channels equal
                                    // its straight RGB.
                                    let dst = pixels[idx];
                                    let blend = |s: u8, d: u8| -> u8 {
                                        (s as f32 * sa + d as f32 * (1.0 - sa))
                                            .round()
                                            .clamp(0.0, 255.0) as u8
                                    };
                                    pixels[idx] = tiny_skia::ColorU8::from_rgba(
                                        blend(pix[0], dst.red()),
                                        blend(pix[1], dst.green()),
                                        blend(pix[2], dst.blue()),
                                        255,
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

    // Floyd–Steinberg dither every pixel to the nearest palette colour, then
    // encode as PNG. Images arrive as full-colour pixels and need dithering so
    // photos look right on the 6-colour panel. Text, vector marks and the
    // background arrive as exact palette colours (no AA), so they have zero
    // quantisation error and nothing diffuses — they stay crisp.
    let wsz = w as usize;
    let hsz = h as usize;

    // Un-premultiplied working buffer, one [f32; 3] per pixel.
    let mut buf: Vec<[f32; 3]> = pixmap
        .pixels()
        .iter()
        .map(|pixel| {
            let a = pixel.alpha() as f32;
            if a > 0.0 {
                [
                    pixel.red() as f32 * 255.0 / a,
                    pixel.green() as f32 * 255.0 / a,
                    pixel.blue() as f32 * 255.0 / a,
                ]
            } else {
                [255.0, 255.0, 255.0]
            }
        })
        .collect();

    let mut rgb_img = image::RgbImage::new(w, h);
    for y in 0..hsz {
        for x in 0..wsz {
            let old = buf[y * wsz + x];
            let new = nearest_palette(old);
            rgb_img.put_pixel(x as u32, y as u32, image::Rgb(new));

            let err = [
                old[0] - new[0] as f32,
                old[1] - new[1] as f32,
                old[2] - new[2] as f32,
            ];

            // Standard Floyd–Steinberg weights: right 7/16, below-left 3/16,
            // below 5/16, below-right 1/16.
            let mut add = |xx: usize, yy: usize, f: f32| {
                let j = yy * wsz + xx;
                for c in 0..3 {
                    buf[j][c] += err[c] * f;
                }
            };
            if x + 1 < wsz {
                add(x + 1, y, 7.0 / 16.0);
            }
            if y + 1 < hsz {
                if x > 0 {
                    add(x - 1, y + 1, 3.0 / 16.0);
                }
                add(x, y + 1, 5.0 / 16.0);
                if x + 1 < wsz {
                    add(x + 1, y + 1, 1.0 / 16.0);
                }
            }
        }
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

/// Nearest palette colour to an arbitrary (possibly out-of-range, due to
/// accumulated dither error) RGB value, by squared Euclidean distance.
fn nearest_palette(rgb: [f32; 3]) -> [u8; 3] {
    PALETTE
        .iter()
        .min_by(|&&a, &&b| {
            let da = palette_dist2(rgb, a);
            let db = palette_dist2(rgb, b);
            da.total_cmp(&db)
        })
        .copied()
        .unwrap_or([255, 255, 255])
}

fn palette_dist2(c: [f32; 3], p: [u8; 3]) -> f32 {
    let dr = c[0] - p[0] as f32;
    let dg = c[1] - p[1] as f32;
    let db = c[2] - p[2] as f32;
    dr * dr + dg * dg + db * db
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

// ── Agenda calendar layout ───────────────────────────────────────────────────
// These constants and helpers are mirrored by the editor's CalendarWidget.vue so
// the on-screen preview matches the panel. Keep the two in sync.
const AGENDA_LINE_HEIGHT: f32 = 1.3;
const AGENDA_MIN_PX: f32 = 11.0;
const AGENDA_MAX_PX: f32 = 22.0;
const AGENDA_INSET: f32 = 4.0;

/// "08:15" → "8:15am", "18:00" → "6:00pm". Returns the input unchanged if it
/// isn't an "HH:MM" string.
fn to_12h(hhmm: &str) -> String {
    let (h, m) = match hhmm.split_once(':') {
        Some(parts) => parts,
        None => return hhmm.to_string(),
    };
    let hour: i32 = match h.parse() {
        Ok(n) => n,
        Err(_) => return hhmm.to_string(),
    };
    let suffix = if hour < 12 { "am" } else { "pm" };
    let h12 = match hour % 12 {
        0 => 12,
        n => n,
    };
    format!("{}:{}{}", h12, m, suffix)
}

/// Day heading for agenda slot `i`: "Today", "Tomorrow", then the weekday name.
fn agenda_heading(slot: usize, weekday_name: &str) -> String {
    match slot {
        0 => "Today".to_string(),
        1 => "Tomorrow".to_string(),
        _ => weekday_name.to_string(),
    }
}

/// One event line: "8:15am Choir", or just the title for an all-day event.
fn agenda_event_line(time: &str, title: &str) -> String {
    if time.is_empty() {
        title.to_string()
    } else {
        format!("{} {}", to_12h(time), title)
    }
}

/// Render the 7-day agenda for `feed` into the box (x, y, w, h). A single font
/// size is chosen so all day headings and events fit the height; overflow is
/// clipped. Each line is drawn single-line (clipped to width, no wrap).
fn draw_agenda(
    pixmap: &mut tiny_skia::Pixmap,
    font: &ab_glyph::FontVec,
    feed: &ResolvedFeed,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    colour: [u8; 3],
) {
    // Only days that have events are shown — empty days are skipped entirely.
    let shown: Vec<usize> = (0..7).filter(|&i| !feed.week[i].is_empty()).collect();
    if shown.is_empty() {
        return;
    }
    let total_events: usize = shown.iter().map(|&i| feed.week[i].len()).sum();
    // One heading per shown day + all events, plus a half-line gap before each
    // heading after the first. Solve for the px that fits the height.
    let lines_equiv = (shown.len() + total_events) as f32 + 0.5 * (shown.len() as f32 - 1.0);
    let avail_h = (h - 2.0 * AGENDA_INSET).max(1.0);
    let px = (avail_h / (AGENDA_LINE_HEIGHT * lines_equiv))
        .floor()
        .clamp(AGENDA_MIN_PX, AGENDA_MAX_PX);
    let line_h = px * AGENDA_LINE_HEIGHT;
    let gap = line_h * 0.5;
    let indent = px * 0.9;

    let x0 = x + AGENDA_INSET;
    let inner_w = w - 2.0 * AGENDA_INSET;
    let bottom = y + h;
    let mut y_pos = y + AGENDA_INSET;

    for (n, &slot) in shown.iter().enumerate() {
        if n > 0 {
            y_pos += gap;
        }
        if y_pos + line_h > bottom {
            break;
        }
        let heading = agenda_heading(slot, &feed.week_labels[slot]);
        text::draw_text(
            pixmap, font, &heading,
            x0, y_pos, inner_w, line_h,
            px, text::Align::Left, colour,
        );
        y_pos += line_h;

        for ev in &feed.week[slot] {
            if y_pos + line_h > bottom {
                break;
            }
            let line = agenda_event_line(&ev.time, &ev.title);
            text::draw_text(
                pixmap, font, &line,
                x0 + indent, y_pos, inner_w - indent, line_h,
                px, text::Align::Left, colour,
            );
            y_pos += line_h;
        }
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
            orientation: None,
            live_page_id: Some(page_id.clone()),
            pages: vec![Page {
                id: page_id,
                name: "Test".to_string(),
                elements,
                background: None,
                orientation: Some(Orientation::Landscape),
            }],
        }
    }

    fn make_doc_with_background(bg: Colour) -> Document {
        let mut doc = make_doc(vec![]);
        doc.pages[0].background = Some(bg);
        doc
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
        doc.pages[0].orientation = Some(Orientation::Portrait);
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
    fn page_background_fills_the_surface() {
        // With no elements, a blue page background should paint every pixel blue.
        let doc = make_doc_with_background(Colour::Blue);
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let cal = CalendarData::empty();
        let png = render(&doc, &cfg, &fonts, &storage, &cal).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgb8();
        let blue = Colour::Blue.rgb();
        for pixel in img.pixels() {
            assert_eq!([pixel[0], pixel[1], pixel[2]], blue, "non-blue background pixel");
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

    /// A flat mid-grey image (no palette colour matches it) must be dithered:
    /// its region should contain more than one palette colour, mixing black and
    /// white rather than collapsing to a single flat fill.
    #[test]
    fn flat_grey_image_is_dithered() {
        // Encode a uniform 64×64 mid-grey PNG.
        let grey = image::RgbImage::from_pixel(64, 64, image::Rgb([128, 128, 128]));
        let mut png_bytes = Vec::new();
        image::DynamicImage::ImageRgb8(grey)
            .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .unwrap();

        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        storage.save_image("grey", &png_bytes).unwrap();

        let doc = make_doc(vec![Element::Image(ImageEl {
            id: "i1".to_string(),
            x: 100.0, y: 100.0, w: 200.0, h: 200.0,
            colour: Colour::White,
            src: Some("grey".to_string()),
        })]);
        let cfg = Config::default();
        let fonts = Fonts::load();
        let cal = CalendarData::empty();
        let png = render(&doc, &cfg, &fonts, &storage, &cal).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgb8();

        // Collect distinct palette colours inside the image region.
        let mut seen = std::collections::HashSet::new();
        for y in 100..300u32 {
            for x in 100..300u32 {
                let p = img.get_pixel(x, y);
                seen.insert([p[0], p[1], p[2]]);
            }
        }
        assert!(
            seen.len() > 1,
            "flat grey image should dither to multiple palette colours, got {:?}",
            seen
        );
        for c in &seen {
            assert!(PALETTE.contains(c), "dithered pixel {:?} not in palette", c);
        }
    }

    /// A PNG with a fully transparent half must let the page background show
    /// through there, while the opaque half paints its own colour.
    #[test]
    fn transparent_png_shows_background_through() {
        // Left half opaque red, right half fully transparent.
        let mut rgba = image::RgbaImage::new(64, 64);
        for (x, _y, p) in rgba.enumerate_pixels_mut() {
            *p = if x < 32 {
                image::Rgba([255, 0, 0, 255])
            } else {
                image::Rgba([0, 0, 0, 0])
            };
        }
        let mut png_bytes = Vec::new();
        image::DynamicImage::ImageRgba8(rgba)
            .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .unwrap();

        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        storage.save_image("half", &png_bytes).unwrap();

        // Blue page background so transparent pixels must come out blue.
        let mut doc = make_doc(vec![Element::Image(ImageEl {
            id: "i1".to_string(),
            x: 0.0, y: 0.0, w: 200.0, h: 200.0,
            colour: Colour::White,
            src: Some("half".to_string()),
        })]);
        doc.pages[0].background = Some(Colour::Blue);

        let cfg = Config::default();
        let fonts = Fonts::load();
        let cal = CalendarData::empty();
        let png = render(&doc, &cfg, &fonts, &storage, &cal).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgb8();

        let blue = Colour::Blue.rgb();
        // Right side of the image (transparent) must be the blue background.
        assert_eq!(
            [img.get_pixel(150, 100)[0], img.get_pixel(150, 100)[1], img.get_pixel(150, 100)[2]],
            blue,
            "transparent region should show the blue background"
        );
        // Left side (opaque red) must NOT be blue.
        let left = img.get_pixel(50, 100);
        assert_ne!([left[0], left[1], left[2]], blue, "opaque region should not be background");
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
        let agenda_png = render_variant(CalendarVariant::Agenda);

        assert_ne!(date_png, today_png, "date and today variants must differ");
        assert_ne!(date_png, agenda_png, "date and agenda variants must differ");
        assert_ne!(today_png, agenda_png, "today and agenda variants must differ");
    }

    #[test]
    fn to_12h_formats_times() {
        assert_eq!(to_12h("08:15"), "8:15am");
        assert_eq!(to_12h("18:00"), "6:00pm");
        assert_eq!(to_12h("00:30"), "12:30am");
        assert_eq!(to_12h("12:00"), "12:00pm");
        assert_eq!(to_12h(""), ""); // all-day passthrough
    }

    #[test]
    fn agenda_heading_uses_today_tomorrow_then_weekday() {
        assert_eq!(agenda_heading(0, "Saturday"), "Today");
        assert_eq!(agenda_heading(1, "Sunday"), "Tomorrow");
        assert_eq!(agenda_heading(2, "Monday"), "Monday");
    }

    #[test]
    fn agenda_variant_renders_without_panic() {
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let cal = CalendarData::empty();
        let doc = make_doc(vec![Element::Calendar(CalendarEl {
            id: "c1".to_string(),
            x: 10.0, y: 10.0, w: 300.0, h: 400.0,
            colour: Colour::Black,
            variant: CalendarVariant::Agenda,
            feed_id: String::new(),
        })]);
        let png = render(&doc, &cfg, &fonts, &storage, &cal).unwrap();
        // Non-trivial output (the sample agenda drew some ink).
        assert!(png.len() > 1000);
    }

    #[test]
    fn agenda_with_no_events_does_not_panic() {
        let cfg = Config::default();
        let fonts = Fonts::load();
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        // A feed resolved from no events → every day empty → nothing to show.
        let mut cal = CalendarData::empty();
        cal.feeds
            .insert("f".to_string(), crate::calendar::resolve(&[], (2026, 6, 27)));
        let doc = make_doc(vec![Element::Calendar(CalendarEl {
            id: "c1".to_string(),
            x: 10.0, y: 10.0, w: 300.0, h: 300.0,
            colour: Colour::Black,
            variant: CalendarVariant::Agenda,
            feed_id: "f".to_string(),
        })]);
        let png = render(&doc, &cfg, &fonts, &storage, &cal).unwrap();
        assert!(!png.is_empty());
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
