//! The glyph atlas: rasterized glyphs packed into one texture (ADR-027).
//!
//! Glyphs are rasterized on the CPU (ab_glyph) the first time they are drawn
//! at a given pixel size, and packed into rows ("shelves") of one RGBA image:
//! white, with the glyph's coverage in alpha, so a sprite-style tint colours
//! it. The renderer uploads the rows that changed before drawing. When the
//! atlas is full it is cleared and refilled with what the current frame needs.

use std::collections::HashMap;
use std::ops::Range;

use ab_glyph::{Font as _, GlyphId, PxScale};

use super::font::{FontData, FontId};

/// Width and height of the atlas texture in texels. 1024² RGBA is 4 MiB and
/// fits every GPU (the downlevel limit is 2048).
pub(crate) const ATLAS_SIZE: u32 = 1024;

/// Transparent texels kept around every glyph, so filtering never reads a neighbour.
const PADDING: u32 = 1;

/// An empty texel: transparent white, so linear filtering at glyph edges
/// blends towards white (the tint colour), never towards black.
const CLEAR_TEXEL: [u8; 4] = [255, 255, 255, 0];

/// A glyph at one size: font, glyph index, and the em size in pixels
/// (quantized to 1/64 px so nearly equal sizes share a rasterization).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    font: FontId,
    glyph: u16,
    em_64ths: u32,
}

/// Where a rasterized glyph is, relative to its pen position and in the atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GlyphImage {
    /// Top-left corner of the bitmap relative to the glyph's pen position on
    /// the baseline, in pixels, +Y down.
    pub(crate) offset: (i32, i32),
    /// Bitmap width and height in pixels (= texels).
    pub(crate) size: (u32, u32),
    /// Top-left texel of the bitmap in the atlas.
    pub(crate) texel: (u32, u32),
}

/// The atlas has no room left for a glyph. The caller clears it and retries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AtlasFull;

/// One row of the shelf packer.
#[derive(Debug, Clone, Copy)]
struct Shelf {
    y: u32,
    height: u32,
    /// First free column.
    x: u32,
}

/// CPU side of the glyph atlas. The GPU copy is kept in sync by the sprite
/// pipeline from [`take_dirty_rows`](Self::take_dirty_rows).
pub(crate) struct GlyphAtlas {
    size: u32,
    /// RGBA8, rows top to bottom.
    pixels: Vec<u8>,
    /// `None` for glyphs with nothing to draw (spaces) or too big to store.
    cache: HashMap<GlyphKey, Option<GlyphImage>>,
    shelves: Vec<Shelf>,
    /// Top of the unused area below the last shelf.
    next_shelf_y: u32,
    /// Rows changed since the last upload.
    dirty: Option<Range<u32>>,
    /// Times the atlas was cleared because it was full.
    resets: u32,
}

impl Default for GlyphAtlas {
    fn default() -> Self {
        Self::new(ATLAS_SIZE)
    }
}

impl GlyphAtlas {
    /// An empty `size`×`size` atlas. All rows start dirty, so the first upload
    /// initializes the whole GPU texture.
    pub(crate) fn new(size: u32) -> Self {
        Self {
            size,
            pixels: CLEAR_TEXEL.repeat((size * size) as usize),
            cache: HashMap::new(),
            shelves: Vec::new(),
            next_shelf_y: 0,
            dirty: Some(0..size),
            resets: 0,
        }
    }

    /// Width and height in texels.
    pub(crate) fn size(&self) -> u32 {
        self.size
    }

    /// Empties the atlas (all cached glyphs are forgotten).
    pub(crate) fn reset(&mut self) {
        // `pixels` is RGBA8, so its length is a multiple of 4 (no remainder).
        self.pixels.as_chunks_mut::<4>().0.fill(CLEAR_TEXEL);
        self.cache.clear();
        self.shelves.clear();
        self.next_shelf_y = 0;
        self.dirty = Some(0..self.size);
        self.resets = self.resets.saturating_add(1);
    }

    /// How many times the atlas has been cleared because it was full.
    pub(crate) fn resets(&self) -> u32 {
        self.resets
    }

    /// The glyph `glyph` of `font` at an em size of `em_px` pixels,
    /// rasterizing and packing it on first use. `Ok(None)`: nothing to draw.
    pub(crate) fn glyph(
        &mut self,
        font_id: FontId,
        font: &FontData,
        glyph: GlyphId,
        em_px: f32,
    ) -> Result<Option<GlyphImage>, AtlasFull> {
        let key = GlyphKey {
            font: font_id,
            glyph: glyph.0,
            em_64ths: quantize(em_px),
        };
        if let Some(&cached) = self.cache.get(&key) {
            return Ok(cached);
        }
        let image = self.rasterize(font, glyph, key.em_64ths as f32 / 64.0)?;
        self.cache.insert(key, image);
        Ok(image)
    }

    fn rasterize(
        &mut self,
        font: &FontData,
        glyph: GlyphId,
        em_px: f32,
    ) -> Result<Option<GlyphImage>, AtlasFull> {
        let Some(outline) = font
            .font
            .outline_glyph(glyph.with_scale(px_scale(font, em_px)))
        else {
            return Ok(None); // no outline: a space or an empty glyph
        };
        let bounds = outline.px_bounds();
        let (width, height) = (bounds.width() as u32, bounds.height() as u32);
        if width == 0 || height == 0 {
            return Ok(None);
        }
        if width + 2 * PADDING > self.size || height + 2 * PADDING > self.size {
            log::warn!(
                "glyph {} of {} at {em_px} px is too large for the glyph atlas; skipped",
                glyph.0,
                font.source.display()
            );
            return Ok(None);
        }
        let (x, y) = self.allocate(width + 2 * PADDING, height + 2 * PADDING)?;
        let (x, y) = (x + PADDING, y + PADDING);
        let stride = self.size as usize * 4;
        let pixels = &mut self.pixels;
        outline.draw(|gx, gy, coverage| {
            let alpha = (coverage.clamp(0.0, 1.0) * 255.0).round() as u8;
            let i = (y + gy) as usize * stride + (x + gx) as usize * 4;
            pixels[i + 3] = alpha;
        });
        self.mark_dirty(y..y + height);
        Ok(Some(GlyphImage {
            offset: (bounds.min.x as i32, bounds.min.y as i32),
            size: (width, height),
            texel: (x, y),
        }))
    }

    /// Finds room for a `width`×`height` block: the first shelf that is tall
    /// enough (but not wastefully tall) with space left, else a new shelf.
    fn allocate(&mut self, width: u32, height: u32) -> Result<(u32, u32), AtlasFull> {
        let size = self.size;
        if let Some(shelf) = self.shelves.iter_mut().find(|s| {
            s.height >= height && s.height <= height + height / 4 + 2 && size - s.x >= width
        }) {
            let at = (shelf.x, shelf.y);
            shelf.x += width;
            return Ok(at);
        }
        if size - self.next_shelf_y < height {
            return Err(AtlasFull);
        }
        let y = self.next_shelf_y;
        self.shelves.push(Shelf {
            y,
            height,
            x: width,
        });
        self.next_shelf_y += height;
        Ok((0, y))
    }

    fn mark_dirty(&mut self, rows: Range<u32>) {
        self.dirty = Some(match self.dirty.take() {
            Some(d) => d.start.min(rows.start)..d.end.max(rows.end),
            None => rows,
        });
    }

    /// The rows changed since the last call and their RGBA bytes (full width),
    /// or `None` if nothing changed.
    pub(crate) fn take_dirty_rows(&mut self) -> Option<(Range<u32>, &[u8])> {
        let rows = self.dirty.take()?;
        let stride = self.size as usize * 4;
        let bytes = &self.pixels[rows.start as usize * stride..rows.end as usize * stride];
        Some((rows, bytes))
    }

    /// The RGBA texel at (`x`, `y`).
    #[cfg(test)]
    pub(crate) fn texel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = (y * self.size + x) as usize * 4;
        [
            self.pixels[i],
            self.pixels[i + 1],
            self.pixels[i + 2],
            self.pixels[i + 3],
        ]
    }
}

/// The em size in 1/64 px, the precision glyphs are cached at.
fn quantize(em_px: f32) -> u32 {
    (em_px * 64.0).round().max(1.0) as u32
}

/// ab_glyph scales by the font's height (ascent − descent); PurplePie sizes
/// are em sizes, like CSS `font-size`. Converts one into the other.
pub(crate) fn px_scale(font: &FontData, em_px: f32) -> PxScale {
    let units_per_em = font.font.units_per_em().unwrap_or(1000.0);
    PxScale::from(em_px * font.font.height_unscaled() / units_per_em)
}

#[cfg(test)]
mod tests {
    use super::super::font::tests::poppins;
    use super::*;

    fn glyph_of(c: char) -> GlyphId {
        let (fonts, id) = poppins();
        fonts.get(id).expect("font").font.glyph_id(c)
    }

    #[test]
    fn rasterized_glyphs_are_white_with_coverage_in_alpha() {
        let (fonts, id) = poppins();
        let font = fonts.get(id).expect("font");
        let mut atlas = GlyphAtlas::new(256);
        let image = atlas
            .glyph(id, font, glyph_of('H'), 32.0)
            .expect("room")
            .expect("H has an outline");
        // 'H' at 32 px em: about 22 px tall (cap height ~0.7 em), sitting on the baseline.
        assert!((20..=24).contains(&image.size.1), "{image:?}");
        assert!(image.offset.1 < 0 && image.offset.1 + image.size.1 as i32 <= 1);
        let mut covered = 0;
        for y in 0..image.size.1 {
            for x in 0..image.size.0 {
                let [r, g, b, a] = atlas.texel(image.texel.0 + x, image.texel.1 + y);
                assert_eq!([r, g, b], [255, 255, 255]);
                covered += u32::from(a == 255);
            }
        }
        assert!(covered > 50, "the stems are fully covered: {covered}");
        // The padding around the glyph stays transparent.
        let (x, y) = image.texel;
        assert_eq!(atlas.texel(x - 1, y - 1), CLEAR_TEXEL);
        assert_eq!(atlas.texel(x + image.size.0, y), CLEAR_TEXEL);
    }

    #[test]
    fn glyphs_are_cached_per_size_and_spaces_are_blank() {
        let (fonts, id) = poppins();
        let font = fonts.get(id).expect("font");
        let mut atlas = GlyphAtlas::new(256);
        let a = atlas.glyph(id, font, glyph_of('a'), 20.0).expect("room");
        let again = atlas.glyph(id, font, glyph_of('a'), 20.0 + 1.0 / 256.0);
        assert_eq!(Ok(a), again, "sizes within 1/128 px share a rasterization");
        let bigger = atlas.glyph(id, font, glyph_of('a'), 40.0).expect("room");
        assert_ne!(a.map(|i| i.texel), bigger.map(|i| i.texel));
        assert!(bigger.expect("outline").size.1 > a.expect("outline").size.1);
        assert_eq!(atlas.glyph(id, font, glyph_of(' '), 20.0), Ok(None));
    }

    #[test]
    fn packed_glyphs_never_overlap() {
        let (fonts, id) = poppins();
        let font = fonts.get(id).expect("font");
        let mut atlas = GlyphAtlas::new(512);
        let mut rects = Vec::new();
        for size in [12.0, 17.0, 24.0, 33.0] {
            for c in '!'..='~' {
                if let Some(image) = atlas.glyph(id, font, glyph_of(c), size).expect("fits") {
                    rects.push(image);
                }
            }
        }
        for (i, a) in rects.iter().enumerate() {
            assert!(a.texel.0 + a.size.0 < 512 && a.texel.1 + a.size.1 < 512);
            for b in &rects[i + 1..] {
                let apart_x = a.texel.0 + a.size.0 + PADDING <= b.texel.0
                    || b.texel.0 + b.size.0 + PADDING <= a.texel.0;
                let apart_y = a.texel.1 + a.size.1 + PADDING <= b.texel.1
                    || b.texel.1 + b.size.1 + PADDING <= a.texel.1;
                assert!(apart_x || apart_y, "{a:?} overlaps {b:?}");
            }
        }
    }

    #[test]
    fn a_full_atlas_reports_it_and_reset_makes_room() {
        let (fonts, id) = poppins();
        let font = fonts.get(id).expect("font");
        let mut atlas = GlyphAtlas::new(64);
        let mut full = false;
        for c in 'A'..='Z' {
            if atlas.glyph(id, font, glyph_of(c), 30.0) == Err(AtlasFull) {
                full = true;
                break;
            }
        }
        assert!(full, "26 capitals at 30 px cannot fit in 64×64");
        atlas.reset();
        assert_eq!(atlas.resets(), 1);
        assert!(
            atlas
                .glyph(id, font, glyph_of('Z'), 30.0)
                .expect("room")
                .is_some()
        );
        // A glyph bigger than the whole atlas is skipped, not an endless reset.
        assert_eq!(atlas.glyph(id, font, glyph_of('W'), 500.0), Ok(None));
    }

    #[test]
    fn dirty_rows_cover_new_glyphs_and_are_taken_once() {
        let (fonts, id) = poppins();
        let font = fonts.get(id).expect("font");
        let mut atlas = GlyphAtlas::new(128);
        let (rows, bytes) = atlas.take_dirty_rows().expect("a new atlas is all dirty");
        assert_eq!((rows, bytes.len()), (0..128, 128 * 128 * 4));
        assert!(atlas.take_dirty_rows().is_none());
        let image = atlas
            .glyph(id, font, glyph_of('g'), 16.0)
            .expect("room")
            .expect("outline");
        let (rows, bytes) = atlas.take_dirty_rows().expect("glyph rows are dirty");
        assert_eq!(rows, image.texel.1..image.texel.1 + image.size.1);
        assert_eq!(bytes.len(), rows.len() * 128 * 4);
        // Cached glyphs do not dirty anything.
        atlas.glyph(id, font, glyph_of('g'), 16.0).expect("room");
        assert!(atlas.take_dirty_rows().is_none());
    }
}
