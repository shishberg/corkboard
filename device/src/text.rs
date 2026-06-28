use ab_glyph::{Font, PxScale, ScaleFont};

pub enum Align {
    Left,
    Center,
}

/// Measure the pixel width of a single line of text at scale `px`.
pub fn measure_line(font: &ab_glyph::FontVec, text: &str, px: f32) -> f32 {
    let scaled = font.as_scaled(PxScale::from(px));
    text.chars()
        .map(|ch| scaled.h_advance(scaled.glyph_id(ch)))
        .sum()
}

/// Render `text` into `pixmap` within the box (x, y, w, h).
///
/// - Word-wraps at width `w`; clips lines whose bottom would exceed y+h.
/// - Baseline for line i = y + ascent + i * line_height  (line_height = px * 1.25).
/// - Alpha-blends each glyph pixel against whatever is already in the pixmap.
pub fn draw_text(
    pixmap: &mut tiny_skia::Pixmap,
    font: &ab_glyph::FontVec,
    text: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    px: f32,
    align: Align,
    colour: [u8; 3],
) {
    let pw = pixmap.width();
    let ph = pixmap.height();

    // Snap the pixel size to an integer. Outline fonts have no hinting here, so
    // a whole-pixel em makes stems land on consistent pixel counts instead of
    // smearing across fractional boundaries — the main lever for crisp 1-bit
    // text at small sizes.
    let px = px.round().max(1.0);

    // Collect glyph blits first so we can drop the scaled-font borrow before
    // calling pixmap.pixels_mut().
    let glyph_blits: Vec<(i32, i32, f32)> = {
        let scale = PxScale::from(px);
        let scaled = font.as_scaled(scale);
        let ascent = scaled.ascent();
        let line_height = px * 1.25;

        // Word-wrap
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut lines: Vec<String> = Vec::new();
        let mut cur_line = String::new();
        let mut cur_w = 0.0f32;

        for word in &words {
            let word_w = measure_line(font, word, px);
            let space_w = if cur_line.is_empty() {
                0.0
            } else {
                measure_line(font, " ", px)
            };

            if !cur_line.is_empty() && cur_w + space_w + word_w > w {
                lines.push(cur_line.clone());
                cur_line = word.to_string();
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

        let mut blits: Vec<(i32, i32, f32)> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let line_top = y + i as f32 * line_height;
            let line_bottom = line_top + line_height;
            if line_bottom > y + h {
                break;
            }
            // Snap the baseline to a whole pixel row so horizontal stems and the
            // x-height line don't straddle two rows.
            let baseline_y = (line_top + ascent).round();

            let line_w = measure_line(font, line, px);
            let x_start = match align {
                Align::Left => x,
                Align::Center => x + (w - line_w) / 2.0,
            };

            let mut gx = x_start;
            for ch in line.chars() {
                let glyph_id = scaled.glyph_id(ch);
                let advance = scaled.h_advance(glyph_id);
                // Accumulate advances at full precision (keeps spacing even) but
                // grid-fit each glyph's origin to a whole pixel column so vertical
                // stems stay sharp.
                let glyph = glyph_id
                    .with_scale_and_position(scale, ab_glyph::point(gx.round(), baseline_y));

                if let Some(outline) = font.outline_glyph(glyph) {
                    let bounds = outline.px_bounds();
                    outline.draw(|dx, dy, cov| {
                        let bx = bounds.min.x as i32 + dx as i32;
                        let by = bounds.min.y as i32 + dy as i32;
                        blits.push((bx, by, cov));
                    });
                }

                gx += advance;
            }
        }

        blits
    };

    // Blit into the pixmap as 1-bit text: a pixel turns full colour when the
    // glyph covers enough of it, otherwise it's left untouched. We never write
    // intermediate greys — anti-aliased edges would quantise to green on the
    // 6-colour palette, and grey text is unreadable on e-paper anyway.
    //
    // The threshold is below 0.5 ("stem darkening"): at small sizes thin stems
    // cover well under half a pixel and would drop out at 0.5, so a lower cutoff
    // keeps them present and the text a touch heavier, which reads better on the
    // panel. FreeType applies the same idea for small monochrome text.
    const COVERAGE_ON: f32 = 0.4;
    let [fr, fg, fb] = colour;

    let pixels = pixmap.pixels_mut();
    for (bx, by, cov) in glyph_blits {
        if bx < 0 || by < 0 || bx >= pw as i32 || by >= ph as i32 {
            continue;
        }
        if cov >= COVERAGE_ON {
            let idx = by as usize * pw as usize + bx as usize;
            pixels[idx] =
                tiny_skia::ColorU8::from_rgba(fr, fg, fb, 255).premultiply();
        }
        // below the threshold: leave the background pixel unchanged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::Pixmap;

    #[test]
    fn draw_text_leaves_red_pixels() {
        let mut pixmap = Pixmap::new(200, 50).unwrap();
        pixmap.fill(tiny_skia::Color::WHITE);

        let fonts = crate::fonts::Fonts::load();
        let font = fonts.get("atkinson-hyperlegible");

        draw_text(
            &mut pixmap,
            font,
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
}
