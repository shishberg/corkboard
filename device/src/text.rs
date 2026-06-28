//! Text layout and 1-bit rendering, backed by FreeType.
//!
//! The panel is 6-colour with no greys, so text must be pure black/white — no
//! anti-aliasing (intermediate greys would quantise to green and read poorly on
//! e-paper). FreeType's monochrome mode (`TARGET_MONO`) is purpose-built for
//! this: it hints the outline onto the whole-pixel grid before rasterising, so
//! stems land on consistent pixel columns and small text stays crisp instead of
//! the jagged, broken look an unhinted rasteriser gives at these sizes.

use std::collections::HashMap;

use freetype::face::LoadFlag;
use freetype::Library;
pub use freetype::Face;

use crate::fonts::Fonts;

pub enum Align {
    Left,
    Center,
}

/// Line height as a multiple of the pixel size. Shared by wrapping, fitting and
/// drawing so they all agree.
pub const LINE_HEIGHT: f32 = 1.25;

/// FreeType faces for one render pass, built from the shared `Fonts` bytes.
///
/// FreeType faces are `!Send`/`!Sync` (they hold a refcounted library handle),
/// so they can't live in the shared `Fonts` state — we build them locally per
/// render. Each face independently keeps the library alive, so this owns the
/// faces with no lifetime tie back to the `Library`.
pub struct Faces {
    faces: HashMap<String, Face>,
    default_id: String,
}

impl Faces {
    /// Build a face for every font in `fonts`. Fonts whose bytes FreeType
    /// rejects are skipped; the default is guaranteed present (it falls back to
    /// the first face that loaded).
    pub fn build(lib: &Library, fonts: &Fonts) -> Self {
        let mut faces = HashMap::new();
        for (id, bytes) in fonts.entries() {
            if let Ok(face) = lib.new_memory_face(bytes.clone(), 0) {
                faces.insert(id.clone(), face);
            }
        }
        let default_id = if faces.contains_key(fonts.default_id()) {
            fonts.default_id().to_string()
        } else {
            faces
                .keys()
                .next()
                .cloned()
                .expect("at least one font must load")
        };
        Faces { faces, default_id }
    }

    /// Face for `id`, falling back to the default, then any loaded face.
    pub fn get(&self, id: &str) -> &Face {
        self.faces
            .get(id)
            .or_else(|| self.faces.get(&self.default_id))
            .or_else(|| self.faces.values().next())
            .expect("faces map is never empty")
    }

    pub fn default(&self) -> &Face {
        self.get(&self.default_id)
    }
}

/// Set the face to an integer pixel size and return it. Mono rendering wants a
/// whole-pixel em so hinting can grid-fit cleanly.
fn set_px(face: &Face, px: f32) -> u32 {
    let p = px.round().max(1.0) as u32;
    let _ = face.set_pixel_sizes(0, p);
    p
}

/// Hinted horizontal advance of one char, in whole pixels, at the face's current
/// size. Loads with the mono target so the advance matches what gets drawn.
fn advance_px(face: &Face, ch: char) -> f32 {
    if face.load_char(ch as usize, LoadFlag::TARGET_MONO).is_ok() {
        (face.glyph().advance().x >> 6) as f32
    } else {
        0.0
    }
}

/// Measure the pixel width of a single line of text at scale `px`.
pub fn measure_line(face: &Face, text: &str, px: f32) -> f32 {
    set_px(face, px);
    text.chars().map(|ch| advance_px(face, ch)).sum()
}

/// Word-wrap `text` to width `w` at scale `px`, breaking on whitespace. A single
/// word wider than `w` is left on its own (over)long line rather than split.
pub fn wrap_lines(face: &Face, text: &str, px: f32, w: f32) -> Vec<String> {
    set_px(face, px);
    let space_w = advance_px(face, ' ');

    let mut lines: Vec<String> = Vec::new();
    let mut cur_line = String::new();
    let mut cur_w = 0.0f32;

    for word in text.split_whitespace() {
        let word_w: f32 = word.chars().map(|ch| advance_px(face, ch)).sum();
        let lead = if cur_line.is_empty() { 0.0 } else { space_w };

        if !cur_line.is_empty() && cur_w + lead + word_w > w {
            lines.push(std::mem::take(&mut cur_line));
            cur_line.push_str(word);
            cur_w = word_w;
        } else {
            if !cur_line.is_empty() {
                cur_line.push(' ');
                cur_w += space_w;
            }
            cur_line.push_str(word);
            cur_w += word_w;
        }
    }
    if !cur_line.is_empty() {
        lines.push(cur_line);
    }
    lines
}

/// Largest integer pixel size in `[min_px, max_px]` at which `text` word-wraps
/// to fit inside `(w, h)` — i.e. every wrapped line fits the width and all lines
/// stack within the height (using LINE_HEIGHT). This auto-sizes text to fill its
/// box: short text grows large, long text shrinks to fit. Falls back to `min_px`.
pub fn fit_font_size(
    face: &Face,
    text: &str,
    w: f32,
    h: f32,
    min_px: f32,
    max_px: f32,
) -> f32 {
    let min_px = min_px.round().max(1.0);
    let max_px = max_px.round().max(min_px);
    if text.trim().is_empty() || w <= 0.0 || h <= 0.0 {
        return min_px;
    }

    // Fitting is monotonic in px (bigger size never fits better), so walk up from
    // the minimum and stop at the first size that no longer fits.
    let mut best = min_px;
    let mut px = min_px;
    while px <= max_px {
        let lines = wrap_lines(face, text, px, w);
        let total_h = lines.len() as f32 * px * LINE_HEIGHT;
        let widest = lines
            .iter()
            .map(|l| measure_line(face, l, px))
            .fold(0.0f32, f32::max);
        if total_h <= h && widest <= w {
            best = px;
            px += 1.0;
        } else {
            break;
        }
    }
    best
}

/// Render `text` into `pixmap` within the box (x, y, w, h).
///
/// - Word-wraps at width `w`; clips lines whose bottom would exceed y+h.
/// - Baseline for line i = y + ascent + i * line_height  (line_height = px * 1.25).
/// - Each glyph is a FreeType 1-bit (mono) bitmap; set pixels are painted the
///   given colour, unset pixels leave the background untouched.
pub fn draw_text(
    pixmap: &mut tiny_skia::Pixmap,
    face: &Face,
    text: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    px: f32,
    align: Align,
    colour: [u8; 3],
) {
    let px = set_px(face, px);
    let line_height = px as f32 * LINE_HEIGHT;
    let ascent = face
        .size_metrics()
        .map(|m| (m.ascender >> 6) as f32)
        .unwrap_or(px as f32 * 0.8);

    let lines = wrap_lines(face, text, px as f32, w);

    for (i, line) in lines.iter().enumerate() {
        let line_top = y + i as f32 * line_height;
        if line_top + line_height > y + h {
            break;
        }
        // Snap the baseline to a whole pixel row so horizontal stems don't
        // straddle two rows.
        let baseline = (line_top + ascent).round() as i32;

        let line_w = measure_line(face, line, px as f32);
        let x_start = match align {
            Align::Left => x,
            Align::Center => x + (w - line_w) / 2.0,
        };

        let mut pen_x = x_start;
        for ch in line.chars() {
            if face
                .load_char(ch as usize, LoadFlag::RENDER | LoadFlag::TARGET_MONO)
                .is_err()
            {
                continue;
            }
            let slot = face.glyph();
            let left = slot.bitmap_left();
            let top = slot.bitmap_top();
            let advance = (slot.advance().x >> 6) as f32;
            let bitmap = slot.bitmap();
            blit_mono(
                pixmap,
                &bitmap,
                pen_x.round() as i32 + left,
                baseline - top,
                colour,
            );
            pen_x += advance;
        }
    }
}

/// Blit a FreeType 1-bit mono bitmap into the pixmap at (dst_x, dst_y).
/// Bits are MSB-first within each byte; `pitch` bytes per row.
fn blit_mono(
    pixmap: &mut tiny_skia::Pixmap,
    bitmap: &freetype::Bitmap,
    dst_x: i32,
    dst_y: i32,
    colour: [u8; 3],
) {
    let width = bitmap.width();
    let rows = bitmap.rows();
    let pitch = bitmap.pitch().unsigned_abs() as usize;
    let buffer = bitmap.buffer();
    if width <= 0 || rows <= 0 || buffer.is_empty() {
        return;
    }

    let pw = pixmap.width() as i32;
    let ph = pixmap.height() as i32;
    let [r, g, b] = colour;
    let on = tiny_skia::ColorU8::from_rgba(r, g, b, 255).premultiply();
    let pixels = pixmap.pixels_mut();

    for row in 0..rows {
        let row_off = row as usize * pitch;
        for col in 0..width {
            let byte = buffer[row_off + (col / 8) as usize];
            let bit = (byte >> (7 - (col % 8))) & 1;
            if bit == 0 {
                continue;
            }
            let x = dst_x + col;
            let y = dst_y + row;
            if x < 0 || y < 0 || x >= pw || y >= ph {
                continue;
            }
            pixels[(y * pw + x) as usize] = on;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Pixmap;

    fn test_faces() -> (Library, Faces) {
        let lib = Library::init().unwrap();
        let fonts = crate::fonts::Fonts::load();
        let faces = Faces::build(&lib, &fonts);
        (lib, faces)
    }

    #[test]
    fn faces_unknown_id_falls_back_to_default() {
        let (_lib, faces) = test_faces();
        // An unknown id must not panic and must yield a usable face (the default).
        let _ = faces.get("nonexistent-id-xyz");
        let _ = faces.default();
    }

    #[test]
    fn draw_text_leaves_coloured_pixels() {
        let (_lib, faces) = test_faces();
        let mut pixmap = Pixmap::new(200, 50).unwrap();
        pixmap.fill(tiny_skia::Color::WHITE);

        draw_text(
            &mut pixmap,
            faces.default(),
            "Hello",
            0.0, 0.0, 200.0, 50.0,
            20.0,
            Align::Left,
            [220, 40, 40],
        );

        let has_reddish = pixmap.pixels().iter().any(|p| {
            let a = p.alpha() as u32;
            if a == 0 {
                return false;
            }
            let r = (p.red() as u32 * 255 / a) as u8;
            let g = (p.green() as u32 * 255 / a) as u8;
            r > 150 && g < 100
        });

        assert!(has_reddish, "expected at least one reddish pixel after drawing red text");
    }

    #[test]
    fn draw_text_is_one_bit_no_grey() {
        // Every painted pixel must be exactly the text colour — no anti-aliased
        // greys (they'd quantise badly on the 6-colour panel).
        let (_lib, faces) = test_faces();
        let mut pixmap = Pixmap::new(120, 40).unwrap();
        pixmap.fill(tiny_skia::Color::WHITE);
        draw_text(&mut pixmap, faces.default(), "Agenda 0", 0.0, 0.0, 120.0, 40.0, 16.0, Align::Left, [0, 0, 0]);
        for p in pixmap.pixels() {
            let (r, g, b) = (p.red(), p.green(), p.blue());
            let is_black = r == 0 && g == 0 && b == 0;
            let is_white = r == 255 && g == 255 && b == 255;
            assert!(is_black || is_white, "found a non-1-bit pixel: ({r},{g},{b})");
        }
    }

    #[test]
    fn fit_grows_for_short_text_in_a_bigger_box() {
        let (_lib, faces) = test_faces();
        let font = faces.default();
        let small = fit_font_size(font, "Hi", 100.0, 40.0, 10.0, 240.0);
        let big = fit_font_size(font, "Hi", 400.0, 200.0, 10.0, 240.0);
        assert!(big > small, "bigger box should give bigger font ({big} vs {small})");
    }

    #[test]
    fn fit_shrinks_for_more_text() {
        let (_lib, faces) = test_faces();
        let font = faces.default();
        let short = fit_font_size(font, "Hi", 220.0, 90.0, 10.0, 240.0);
        let long = fit_font_size(
            font,
            "The quick brown fox jumps over the lazy dog again and again",
            220.0,
            90.0,
            10.0,
            240.0,
        );
        assert!(long < short, "more text should shrink the font ({long} vs {short})");
    }

    #[test]
    fn fit_result_actually_fits_and_grows_past_the_floor() {
        let (_lib, faces) = test_faces();
        let font = faces.default();
        let (w, h) = (240.0, 80.0);
        let px = fit_font_size(font, "Hello World", w, h, 10.0, 240.0);
        let lines = wrap_lines(font, "Hello World", px, w);
        let total_h = lines.len() as f32 * px * LINE_HEIGHT;
        let widest = lines.iter().map(|l| measure_line(font, l, px)).fold(0.0, f32::max);
        assert!(total_h <= h, "fitted text must not overflow height");
        assert!(widest <= w, "fitted text must not overflow width");
        assert!(px > 10.0, "short text should grow well above the floor, got {px}");
    }

    #[test]
    fn fit_empty_text_returns_the_floor() {
        let (_lib, faces) = test_faces();
        assert_eq!(fit_font_size(faces.default(), "   ", 200.0, 100.0, 10.0, 240.0), 10.0);
    }
}
