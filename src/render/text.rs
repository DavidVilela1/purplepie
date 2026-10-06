//! Text drawn from ECS data (ADR-027).
//!
//! `Text` is the public component. Layout here is crate-private: it turns a
//! string into glyph bitmaps from the atlas, positioned in pixels relative
//! to the text's origin. `draw.rs` turns those into instances.

use ab_glyph::{Font as _, ScaleFont as _};

use super::Color;
use super::atlas::{AtlasFull, GlyphAtlas, GlyphImage, px_scale};
use super::font::{FontData, FontId};

/// A line (or several lines) of text, drawn at the entity's
/// [`Transform2D`](crate::math::Transform2D).
///
/// The transform's position is the **left end of the first line's
/// baseline**: capital letters rise above it, descenders (g, p, y) hang below.
/// Each `'\n'` starts a new line below the previous one; other control
/// characters are skipped. Characters the font has no glyph for are drawn as
/// the font's "missing glyph" box. There is no shaping: each character is one
/// glyph, so scripts that need ligatures or reordering are not supported yet.
///
/// Glyphs are rasterized for the size they appear on screen (after camera
/// zoom, DPI scale and the transform's scale), so text stays sharp when
/// zoomed. Rotated text is drawn smoothly but is not pixel-aligned. Draw
/// order comes from the entity's [`Layer`](super::Layer); within a layer, text
/// is drawn after quads and sprites. An entity needs both `Transform2D` and
/// `Text` to be drawn.
///
/// ```no_run
/// use purplepie::math::{Transform2D, Vec2};
/// use purplepie::render::{Color, Text};
/// # fn init(ctx: &mut purplepie::Context<'_>) -> purplepie::Result<()> {
/// let font = ctx.load_font("fonts/Poppins-Regular.ttf")?;
/// ctx.world_mut().spawn((
///     Transform2D::from_position(Vec2::new(-300.0, 200.0)),
///     Text::new("Score: 0", font, 32.0).with_color(Color::hex(0xF1FAEE)),
/// ));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    /// The characters to draw.
    pub content: String,
    /// The font, from [`Context::load_font`](crate::Context::load_font).
    pub font: FontId,
    /// Font size (em size) in world units, like CSS `font-size`: with the
    /// default view, `32.0` is 32 logical pixels. Capital letters are roughly
    /// 0.7 × size tall. Text larger than 512 physical pixels is not drawn.
    pub size: f32,
    /// Text colour (sRGB, straight alpha; ADR-015).
    pub color: Color,
}

impl Text {
    /// `content` in `font` at `size` world units, white.
    pub fn new(content: impl Into<String>, font: FontId, size: f32) -> Self {
        Self {
            content: content.into(),
            font,
            size,
            color: Color::WHITE,
        }
    }

    /// The same text in `color`.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

/// Largest em size, in physical pixels, that is rasterized. Larger text is
/// skipped: its glyphs would crowd the atlas out.
pub(crate) const MAX_EM_PIXELS: f32 = 512.0;

/// One glyph to draw: its bitmap in the atlas and where it goes, in whole
/// pixels relative to the text origin (+Y down).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlacedGlyph {
    /// Top-left corner relative to the origin (left end of the first baseline).
    pub(crate) position: (i32, i32),
    pub(crate) image: GlyphImage,
}

/// Lays out `content` at an em size of `em_px` pixels and calls `emit` for
/// every visible glyph, rasterizing glyphs into `atlas` as needed.
///
/// Pen positions are rounded to whole pixels, so every glyph bitmap lands
/// exactly on the pixel grid. Kerning from the font's `kern` table is applied.
/// With `skip_when_full`, glyphs that do not fit are dropped instead of
/// returning [`AtlasFull`].
pub(crate) fn layout(
    content: &str,
    font_id: FontId,
    font: &FontData,
    em_px: f32,
    atlas: &mut GlyphAtlas,
    skip_when_full: bool,
    mut emit: impl FnMut(PlacedGlyph),
) -> Result<(), AtlasFull> {
    let scaled = font.font.as_scaled(px_scale(font, em_px));
    let line_advance = (scaled.height() + scaled.line_gap()).round() as i32;
    let mut pen_x = 0.0_f32;
    let mut baseline = 0_i32;
    let mut previous = None;
    for c in content.chars() {
        if c == '\n' {
            pen_x = 0.0;
            baseline += line_advance;
            previous = None;
            continue;
        }
        if c.is_control() {
            continue;
        }
        let glyph = font.font.glyph_id(c);
        if let Some(previous) = previous {
            pen_x += scaled.kern(previous, glyph);
        }
        previous = Some(glyph);
        let image = match atlas.glyph(font_id, font, glyph, em_px) {
            Ok(image) => image,
            Err(AtlasFull) if skip_when_full => None,
            Err(full) => return Err(full),
        };
        if let Some(image) = image {
            let x = pen_x.round() as i32 + image.offset.0;
            let y = baseline + image.offset.1;
            emit(PlacedGlyph {
                position: (x, y),
                image,
            });
        }
        pen_x += scaled.h_advance(glyph);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::font::tests::poppins;
    use super::*;

    fn placed(content: &str, em_px: f32) -> Vec<PlacedGlyph> {
        let (fonts, id) = poppins();
        let mut atlas = GlyphAtlas::new(512);
        let mut out = Vec::new();
        layout(
            content,
            id,
            fonts.get(id).expect("font"),
            em_px,
            &mut atlas,
            false,
            |g| out.push(g),
        )
        .expect("fits");
        out
    }

    #[test]
    fn text_defaults_to_white() {
        let (_, font) = poppins();
        let text = Text::new("hi", font, 12.0);
        assert_eq!(text.color, Color::WHITE);
        assert_eq!(text.content, "hi");
        assert_eq!(text.with_color(Color::BLACK).color, Color::BLACK);
    }

    #[test]
    fn glyphs_advance_left_to_right_on_the_baseline() {
        let glyphs = placed("HH", 40.0);
        assert_eq!(glyphs.len(), 2);
        let (a, b) = (glyphs[0], glyphs[1]);
        // Same glyph, same bitmap, moved right by the advance (whole pixels).
        assert_eq!(a.image, b.image);
        assert_eq!(a.position.1, b.position.1);
        let advance = b.position.0 - a.position.0;
        // Poppins 'H' advances 692/1000 em: 27.68 px at 40 px, rounded to 28.
        assert_eq!(advance, 28);
        // Capitals sit on the baseline (y = 0): the bitmap ends at or just below it.
        let bottom = a.position.1 + a.image.size.1 as i32;
        assert!((0..=1).contains(&bottom), "{bottom}");
        assert!(a.position.1 < -20, "cap height ≈ 0.7 em above the baseline");
    }

    #[test]
    fn spaces_advance_without_drawing_and_newlines_start_a_lower_line() {
        let glyphs = placed("A A\nA", 20.0);
        assert_eq!(glyphs.len(), 3, "the space draws nothing");
        assert!(glyphs[1].position.0 > glyphs[0].position.0 + 10);
        assert_eq!(
            glyphs[2].position.0, glyphs[0].position.0,
            "new line restarts at x = 0"
        );
        let line = glyphs[2].position.1 - glyphs[0].position.1;
        // Poppins: ascent 1.05 + descent 0.35 + gap 0.1 = 1.5 em = 30 px at 20 px.
        assert_eq!(line, 30);
    }

    #[test]
    fn control_characters_are_skipped_and_empty_text_draws_nothing() {
        assert!(placed("", 20.0).is_empty());
        assert_eq!(placed("\tA\r", 20.0), placed("A", 20.0));
    }

    #[test]
    fn a_full_atlas_is_reported_or_skipped_on_request() {
        let (fonts, id) = poppins();
        let font = fonts.get(id).expect("font");
        let mut atlas = GlyphAtlas::new(64);
        let alphabet: String = ('A'..='Z').collect();
        assert_eq!(
            layout(&alphabet, id, font, 30.0, &mut atlas, false, |_| {}),
            Err(AtlasFull)
        );
        let mut drawn = 0;
        layout(&alphabet, id, font, 30.0, &mut atlas, true, |_| drawn += 1).expect("skips");
        assert!(drawn > 0 && drawn < 26, "{drawn}");
    }
}
