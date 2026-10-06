//! Text drawn from ECS data (ADR-027).
//!
//! `Text`, `TextAnchor` and `TextMetrics` are public. Layout here is
//! crate-private: it turns a string into glyph bitmaps from the atlas,
//! positioned in pixels relative to the text's origin. `draw.rs` turns those
//! into instances; `Context::measure_text` uses [`measure`].

use ab_glyph::{Font as _, FontArc, GlyphId, PxScaleFont, ScaleFont as _};

use super::Color;
use super::atlas::{AtlasFull, GlyphAtlas, GlyphImage, px_scale};
use super::font::{FontData, FontId};
use crate::math::Vec2;

/// A line (or several lines) of text, drawn at the entity's
/// [`Transform2D`](crate::math::Transform2D).
///
/// By default the transform's position is the **left end of the first line's
/// baseline**: capital letters rise above it, descenders (g, p, y) hang below.
/// [`anchor`](Self::anchor) picks another point of the text block instead
/// (for example its centre), and also aligns the lines of multi-line text.
/// [`Context::measure_text`](crate::Context::measure_text) returns its size.
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
/// use purplepie::render::{Color, Text, TextAnchor};
/// # fn init(ctx: &mut purplepie::Context<'_>) -> purplepie::Result<()> {
/// let font = ctx.load_font("fonts/Poppins-Regular.ttf")?;
/// ctx.world_mut().spawn((
///     Transform2D::from_position(Vec2::new(-300.0, 200.0)),
///     Text::new("Score: 0", font, 32.0).with_color(Color::hex(0xF1FAEE)),
/// ));
/// // Centred on the world origin, both lines centred under each other.
/// ctx.world_mut().spawn((
///     Transform2D::default(),
///     Text::new("GAME OVER\nSpace to play again", font, 40.0).with_anchor(TextAnchor::CENTER),
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
    /// Which point of the text block is at the transform's position, and how
    /// lines are aligned. Default: [`TextAnchor::BASELINE_LEFT`].
    pub anchor: TextAnchor,
}

impl Text {
    /// `content` in `font` at `size` world units, white, anchored at the left
    /// end of the first baseline.
    pub fn new(content: impl Into<String>, font: FontId, size: f32) -> Self {
        Self {
            content: content.into(),
            font,
            size,
            color: Color::WHITE,
            anchor: TextAnchor::BASELINE_LEFT,
        }
    }

    /// The same text in `color`.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// The same text with another anchor.
    pub fn with_anchor(mut self, anchor: TextAnchor) -> Self {
        self.anchor = anchor;
        self
    }
}

/// Horizontal part of a [`TextAnchor`]: which edge of every line is at the
/// position. `Center` and `Right` also centre or right-align multi-line text.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum HorizontalAnchor {
    /// Lines start at the position (left-aligned). The default.
    #[default]
    Left,
    /// Each line is centred on the position.
    Center,
    /// Lines end at the position (right-aligned).
    Right,
}

/// Vertical part of a [`TextAnchor`]: which height of the text block is at
/// the position. Measured from the font's ascent and descent lines (the same
/// for every string), so text does not jump when its characters change.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum VerticalAnchor {
    /// The first line's baseline. The default.
    #[default]
    Baseline,
    /// The top of the block: the first line's ascent line.
    Top,
    /// Halfway between `Top` and `Bottom`.
    Middle,
    /// The bottom of the block: the last line's descent line.
    Bottom,
}

/// Which point of a [`Text`] block sits at the entity's position (ADR-027).
///
/// The block is [`TextMetrics::width`] wide (the widest line) and
/// [`TextMetrics::height`] tall. Use the constants for the common cases.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TextAnchor {
    /// Left, centre or right (also aligns the lines).
    pub horizontal: HorizontalAnchor,
    /// Baseline, top, middle or bottom.
    pub vertical: VerticalAnchor,
}

impl TextAnchor {
    /// Left end of the first baseline (the default).
    pub const BASELINE_LEFT: Self = Self::new(HorizontalAnchor::Left, VerticalAnchor::Baseline);
    /// Middle of the first baseline; lines centred.
    pub const BASELINE_CENTER: Self = Self::new(HorizontalAnchor::Center, VerticalAnchor::Baseline);
    /// Right end of the first baseline; lines right-aligned.
    pub const BASELINE_RIGHT: Self = Self::new(HorizontalAnchor::Right, VerticalAnchor::Baseline);
    /// Top-left corner of the block.
    pub const TOP_LEFT: Self = Self::new(HorizontalAnchor::Left, VerticalAnchor::Top);
    /// Middle of the top edge; lines centred.
    pub const TOP_CENTER: Self = Self::new(HorizontalAnchor::Center, VerticalAnchor::Top);
    /// Top-right corner; lines right-aligned.
    pub const TOP_RIGHT: Self = Self::new(HorizontalAnchor::Right, VerticalAnchor::Top);
    /// Middle of the left edge.
    pub const CENTER_LEFT: Self = Self::new(HorizontalAnchor::Left, VerticalAnchor::Middle);
    /// Centre of the block; lines centred.
    pub const CENTER: Self = Self::new(HorizontalAnchor::Center, VerticalAnchor::Middle);
    /// Middle of the right edge; lines right-aligned.
    pub const CENTER_RIGHT: Self = Self::new(HorizontalAnchor::Right, VerticalAnchor::Middle);
    /// Bottom-left corner of the block.
    pub const BOTTOM_LEFT: Self = Self::new(HorizontalAnchor::Left, VerticalAnchor::Bottom);
    /// Middle of the bottom edge; lines centred.
    pub const BOTTOM_CENTER: Self = Self::new(HorizontalAnchor::Center, VerticalAnchor::Bottom);
    /// Bottom-right corner; lines right-aligned.
    pub const BOTTOM_RIGHT: Self = Self::new(HorizontalAnchor::Right, VerticalAnchor::Bottom);

    /// An anchor from its two parts.
    pub const fn new(horizontal: HorizontalAnchor, vertical: VerticalAnchor) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    /// 0 (left), 0.5 (centre) or 1 (right): the share of a line's width that
    /// lies left of the position.
    fn horizontal_share(self) -> f32 {
        match self.horizontal {
            HorizontalAnchor::Left => 0.0,
            HorizontalAnchor::Center => 0.5,
            HorizontalAnchor::Right => 1.0,
        }
    }

    /// How far the first baseline lies below the position (+Y down), for a
    /// block whose top is `ascent` above the first baseline and whose bottom
    /// is `below` under it.
    fn baseline_drop(self, ascent: f32, below: f32) -> f32 {
        match self.vertical {
            VerticalAnchor::Baseline => 0.0,
            VerticalAnchor::Top => ascent,
            VerticalAnchor::Middle => (ascent - below) / 2.0,
            VerticalAnchor::Bottom => -below,
        }
    }
}

/// The size of a [`Text`] in world units, from
/// [`Context::measure_text`](crate::Context::measure_text).
///
/// Widths are advance widths (where the next character would start), with
/// kerning; heights come from the font's ascent, descent and line spacing,
/// so they do not depend on which characters are used. Drawn text is placed
/// on whole pixels, so it can differ from these values by up to a pixel.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TextMetrics {
    /// Width of the widest line.
    pub width: f32,
    /// From the top of the first line (its ascent) to the bottom of the last
    /// line (its descent).
    pub height: f32,
    /// Height of the font's ascent line above a baseline.
    pub ascent: f32,
    /// Depth of the font's descent line below a baseline (positive).
    pub descent: f32,
    /// Distance from one baseline to the next.
    pub line_height: f32,
    /// Number of lines (`'\n'` count + 1; at least 1).
    pub lines: usize,
}

impl TextMetrics {
    /// The block's rectangle relative to the entity position (world units,
    /// +Y up) when drawn with `anchor`, as (min, max) corners: e.g. for a
    /// background panel or a click area.
    pub fn bounds(&self, anchor: TextAnchor) -> (Vec2, Vec2) {
        let left = -self.width * anchor.horizontal_share();
        let below = self.height - self.ascent;
        let drop = anchor.baseline_drop(self.ascent, below);
        // +Y down: top = drop − ascent, bottom = drop + below. Flip for +Y up.
        (
            Vec2::new(left, -(drop + below)),
            Vec2::new(left + self.width, self.ascent - drop),
        )
    }
}

/// Measures `content` in `font` at an em size of `size` (any unit; the result
/// is in the same unit).
pub(crate) fn measure(content: &str, font: &FontData, size: f32) -> TextMetrics {
    if !(size.is_finite() && size > 0.0) {
        return TextMetrics {
            lines: content.split('\n').count(),
            ..TextMetrics::default()
        };
    }
    let scaled = font.font.as_scaled(px_scale(font, size));
    let (mut width, mut lines) = (0.0_f32, 0_usize);
    let walked = walk(
        content,
        &scaled,
        |w| {
            width = width.max(w);
            lines += 1;
        },
        |_, _, _| Ok(()),
    );
    debug_assert!(walked.is_ok(), "measuring never touches the atlas");
    let ascent = scaled.ascent();
    let descent = -scaled.descent();
    let line_height = scaled.height() + scaled.line_gap();
    TextMetrics {
        width,
        height: ascent + descent + (lines - 1) as f32 * line_height,
        ascent,
        descent,
        line_height,
        lines,
    }
}

/// Walks the glyphs of `content` the way the layout places them: calls
/// `visit(line, glyph, pen_x)` for every drawable character and
/// `line_end(width)` at the end of every line (its advance width, kerning
/// included; always at least once).
fn walk(
    content: &str,
    scaled: &PxScaleFont<&FontArc>,
    mut line_end: impl FnMut(f32),
    mut visit: impl FnMut(usize, GlyphId, f32) -> Result<(), AtlasFull>,
) -> Result<(), AtlasFull> {
    let mut line = 0;
    let mut pen_x = 0.0_f32;
    let mut previous = None;
    for c in content.chars() {
        if c == '\n' {
            line_end(pen_x);
            line += 1;
            pen_x = 0.0;
            previous = None;
            continue;
        }
        if c.is_control() {
            continue;
        }
        let glyph = scaled.glyph_id(c);
        if let Some(previous) = previous {
            pen_x += scaled.kern(previous, glyph);
        }
        previous = Some(glyph);
        visit(line, glyph, pen_x)?;
        pen_x += scaled.h_advance(glyph);
    }
    line_end(pen_x);
    Ok(())
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

/// Lays out `content` at an em size of `em_px` pixels with `anchor` and calls
/// `emit` for every visible glyph, rasterizing glyphs into `atlas` as needed.
///
/// Pen positions, line offsets and the anchor shift are rounded to whole
/// pixels, so every glyph bitmap lands exactly on the pixel grid. Kerning from
/// the font's `kern` table is applied. With `skip_when_full`, glyphs that do
/// not fit are dropped instead of returning [`AtlasFull`]. `widths` is scratch
/// space (one entry per line), reused between calls to avoid allocating.
#[allow(clippy::too_many_arguments)]
pub(crate) fn layout(
    content: &str,
    font_id: FontId,
    font: &FontData,
    em_px: f32,
    anchor: TextAnchor,
    atlas: &mut GlyphAtlas,
    skip_when_full: bool,
    widths: &mut Vec<f32>,
    mut emit: impl FnMut(PlacedGlyph),
) -> Result<(), AtlasFull> {
    let scaled = font.font.as_scaled(px_scale(font, em_px));
    let line_advance = (scaled.height() + scaled.line_gap()).round() as i32;
    // First pass: each line's shift for the horizontal anchor, in whole pixels.
    let share = anchor.horizontal_share();
    widths.clear();
    walk(
        content,
        &scaled,
        |w| widths.push((-w * share).round()),
        |_, _, _| Ok(()),
    )?;
    let lines = widths.len() as i32;
    let below = (lines - 1) as f32 * line_advance as f32 - scaled.descent();
    let drop = anchor.baseline_drop(scaled.ascent(), below).round() as i32;
    let shifts: &[f32] = widths;
    walk(
        content,
        &scaled,
        |_| {},
        |line, glyph, pen_x| {
            let image = match atlas.glyph(font_id, font, glyph, em_px) {
                Ok(image) => image,
                Err(AtlasFull) if skip_when_full => None,
                Err(full) => return Err(full),
            };
            if let Some(image) = image {
                let x = shifts[line] as i32 + pen_x.round() as i32 + image.offset.0;
                let y = drop + line as i32 * line_advance + image.offset.1;
                emit(PlacedGlyph {
                    position: (x, y),
                    image,
                });
            }
            Ok(())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::super::font::tests::poppins;
    use super::*;

    fn placed(content: &str, em_px: f32) -> Vec<PlacedGlyph> {
        placed_with(content, em_px, TextAnchor::BASELINE_LEFT)
    }

    fn placed_with(content: &str, em_px: f32, anchor: TextAnchor) -> Vec<PlacedGlyph> {
        let (fonts, id) = poppins();
        let mut atlas = GlyphAtlas::new(512);
        let mut out = Vec::new();
        layout(
            content,
            id,
            fonts.get(id).expect("font"),
            em_px,
            anchor,
            &mut atlas,
            false,
            &mut Vec::new(),
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
            layout(
                &alphabet,
                id,
                font,
                30.0,
                crate::render::TextAnchor::BASELINE_LEFT,
                &mut atlas,
                false,
                &mut Vec::new(),
                |_| {}
            ),
            Err(AtlasFull)
        );
        let mut drawn = 0;
        layout(
            &alphabet,
            id,
            font,
            30.0,
            crate::render::TextAnchor::BASELINE_LEFT,
            &mut atlas,
            true,
            &mut Vec::new(),
            |_| drawn += 1,
        )
        .expect("skips");
        assert!(drawn > 0 && drawn < 26, "{drawn}");
    }

    fn metrics(content: &str, size: f32) -> TextMetrics {
        let (fonts, id) = poppins();
        measure(content, fonts.get(id).expect("font"), size)
    }

    #[test]
    fn text_defaults_to_the_baseline_left_anchor() {
        let (_, font) = poppins();
        let text = Text::new("hi", font, 12.0);
        assert_eq!(text.anchor, TextAnchor::BASELINE_LEFT);
        assert_eq!(TextAnchor::default(), TextAnchor::BASELINE_LEFT);
        assert_eq!(
            text.with_anchor(TextAnchor::CENTER).anchor,
            TextAnchor::CENTER
        );
    }

    #[test]
    fn measure_matches_the_font_metrics() {
        // Poppins (units per 1000): ascent 1050, descent 350, line gap 100, 'H' advance 692, ' ' 267.
        let m = metrics("H H", 10.0);
        assert!((m.width - (6.92 + 2.67 + 6.92)).abs() < 1e-4, "{m:?}");
        assert!((m.ascent - 10.5).abs() < 1e-4 && (m.descent - 3.5).abs() < 1e-4);
        assert!((m.line_height - 15.0).abs() < 1e-4 && (m.height - 14.0).abs() < 1e-4);
        let three = metrics("H\nHHH\n", 10.0);
        assert_eq!(three.lines, 3, "a trailing newline starts an (empty) line");
        assert!((three.width - 3.0 * 6.92).abs() < 1e-4);
        assert!((three.height - (14.0 + 2.0 * 15.0)).abs() < 1e-4);
        let empty = metrics("", 10.0);
        assert_eq!((empty.width, empty.lines), (0.0, 1));
        assert!((empty.height - 14.0).abs() < 1e-4);
        assert_eq!(metrics("ab\ncd", f32::NAN).width, 0.0);
        assert_eq!(metrics("ab\ncd", 0.0).lines, 2);
    }

    #[test]
    fn bounds_place_the_block_around_the_anchor() {
        let m = TextMetrics {
            width: 100.0,
            height: 40.0,
            ascent: 30.0,
            descent: 10.0,
            line_height: 45.0,
            lines: 1,
        };
        let cases = [
            (TextAnchor::BASELINE_LEFT, (0.0, -10.0), (100.0, 30.0)),
            (TextAnchor::TOP_LEFT, (0.0, -40.0), (100.0, 0.0)),
            (TextAnchor::CENTER, (-50.0, -20.0), (50.0, 20.0)),
            (TextAnchor::BOTTOM_RIGHT, (-100.0, 0.0), (0.0, 40.0)),
            (TextAnchor::TOP_CENTER, (-50.0, -40.0), (50.0, 0.0)),
            (TextAnchor::BASELINE_RIGHT, (-100.0, -10.0), (0.0, 30.0)),
        ];
        for (anchor, min, max) in cases {
            let (got_min, got_max) = m.bounds(anchor);
            assert_eq!(
                (got_min, got_max),
                (Vec2::from(min), Vec2::from(max)),
                "{anchor:?}"
            );
        }
        // Two lines: the block grows downwards from the first baseline.
        let two = TextMetrics {
            height: 85.0,
            lines: 2,
            ..m
        };
        assert_eq!(
            two.bounds(TextAnchor::TOP_LEFT),
            (Vec2::new(0.0, -85.0), Vec2::new(100.0, 0.0))
        );
        assert_eq!(
            two.bounds(TextAnchor::BASELINE_LEFT),
            (Vec2::new(0.0, -55.0), Vec2::new(100.0, 30.0))
        );
    }

    #[test]
    fn anchors_shift_the_layout_by_whole_pixels() {
        let base = placed("Hg", 40.0);
        let m = metrics("Hg", 40.0);
        let shift = |anchor| {
            let moved = placed_with("Hg", 40.0, anchor);
            assert_eq!(moved.len(), base.len());
            let d = (
                moved[0].position.0 - base[0].position.0,
                moved[0].position.1 - base[0].position.1,
            );
            for (a, b) in moved.iter().zip(&base) {
                assert_eq!(
                    (a.position.0 - b.position.0, a.position.1 - b.position.1),
                    d,
                    "rigid shift"
                );
            }
            d
        };
        assert_eq!(shift(TextAnchor::BASELINE_LEFT), (0, 0));
        assert_eq!(
            shift(TextAnchor::BASELINE_RIGHT).0,
            -(m.width.round() as i32)
        );
        assert_eq!(
            shift(TextAnchor::BASELINE_CENTER).0,
            -((m.width / 2.0).round() as i32)
        );
        // +Y down in pixels: anchoring at the top moves the baseline down by the ascent.
        assert_eq!(shift(TextAnchor::TOP_LEFT), (0, m.ascent.round() as i32));
        assert_eq!(
            shift(TextAnchor::BOTTOM_LEFT),
            (0, -(m.descent.round() as i32))
        );
        assert_eq!(
            shift(TextAnchor::CENTER_LEFT).1,
            ((m.ascent - m.descent) / 2.0).round() as i32
        );
    }

    #[test]
    fn centred_and_right_aligned_lines_align_individually() {
        // Line 1 is three times as wide as line 2.
        let left = placed("HHH\nH", 40.0);
        let centre = placed_with("HHH\nH", 40.0, TextAnchor::BASELINE_CENTER);
        let right = placed_with("HHH\nH", 40.0, TextAnchor::BASELINE_RIGHT);
        let w1 = metrics("HHH", 40.0).width;
        let w2 = metrics("H", 40.0).width;
        let dx = |a: &[PlacedGlyph], i: usize| a[i].position.0 - left[i].position.0;
        assert_eq!(dx(&centre, 0), -((w1 / 2.0).round() as i32));
        assert_eq!(
            dx(&centre, 3),
            -((w2 / 2.0).round() as i32),
            "second line centred on its own"
        );
        assert_eq!(dx(&right, 0), -(w1.round() as i32));
        assert_eq!(dx(&right, 3), -(w2.round() as i32));
        // Lines stay one line height apart; the anchor does not change that.
        assert_eq!(
            right[3].position.1 - right[0].position.1,
            left[3].position.1 - left[0].position.1
        );
    }
}
